# 原生阴影修复 review

> **日期：** 2026-10-15（UTC）
>
> **基线：** v0.1.6 / `4c61451b7dd60c7222f5c182c7cd4b300e3fd65d`
>
> **软件版本：** 0.1.7；[PR #3](https://github.com/JakeJia/Useless-subtitle/pull/3) 合并后从主分支创建 `v0.1.7` tag，通过既有工作流发布
>
> **状态：** 最小修复已实现；本地自动检查通过，原生回归及四平台 CI 结果见下文；Windows/macOS 交互待验证。

## 问题与修复

Windows 用户反馈遮罩仍有灰色细边。v0.1.6 已移除 CSS 边框和内阴影，但 `build_mask_window` 未设置原生阴影，Tao 的 Windows 默认值为开启。[Tauri 2.11.5 的 `shadow` 文档](https://docs.rs/tauri/2.11.5/tauri/struct.WebviewWindowBuilder.html#method.shadow)明确指出，开启阴影会让无装饰 Windows 窗口出现 1px 白色边框，Windows 11 还会增加系统圆角。这与反馈的残留细边相符，最终显示效果仍需 Windows 实测确认。

生产代码仅在遮罩窗口 builder 中加入 `shadow(false)`，并注释原因；在窗口创建、尺寸设置和显示之前生效。新建、复制、恢复均复用该路径。设置窗口使用独立 builder，保留正常窗口样式。配置结构、外观数值、CSS 圆角和缩放命中区沿用原实现。

## 依赖及影响 review

审阅依据为锁定的 Tauri 2.11.5、tauri-runtime-wry 2.11.4、Tao 0.35.3、Wry 0.55.1 源码；本记录是开发者对实际代码及差异的自审。

| 检查项 | 结论及验证边界 |
|---|---|
| Windows 八方向缩放 | runtime 创建无装饰窗口时把阴影标志传给缩放处理器。关闭阴影走完整边缘/角落命中分支，保留 `resizable(true)`；前端八方向区域和原生缩放命令不变。源码支持该方案，Windows 实际交互仍待验。 |
| 锁定穿透与托盘恢复 | Tao 的穿透使用独立 `IGNORE_CURSOR_EVENT` 标志，对应 `WS_EX_TRANSPARENT`/`WS_EX_LAYERED`；本次不改锁定、显示和托盘处理。需在 Windows 检查包括边缘在内的实际穿透。 |
| 尺寸、位置与 DPI | 阴影开启时 Tao 为无装饰窗口加入隐藏框架留白；关闭后外框与客户区的映射会变化。现有配置保存外框尺寸、恢复设置客户区尺寸，因此升级后旧遮罩可见区域与此前可能有细微差异。不按未知系统边距改写旧数据或增加固定像素补偿；Windows 需检查旧配置恢复、反复重启和混合 DPI。 |
| 创建顺序 | 使用 builder 创建期设置，原生缩放处理器一开始即读取正确标志；不在已显示窗口上动态切换阴影，避免额外的框架刷新及几何事件。 |
| 圆角及控件 | 关闭 Windows 系统阴影也会去掉该选项附带的系统圆角；现有 CSS 10px 圆角保留。工具栏控件自身边框不属于遮罩描边。 |
| macOS | runtime 将该选项映射为 `with_has_shadow(false)`，符合无边缘阴影需求；不改变尺寸、工作区或穿透代码。原生显示及交互待验。 |
| Linux | 此 API 在 Linux 不生效；本地回归验证应用交互和数据流程，不能证明 Windows/macOS 的合成或原生边缘效果。 |

## 分阶段检查

| 阶段 | Review 与测试 |
|---|---|
| 一：方案 | 阅读原生阴影默认值、创建期参数传递、缩放命中、窗口尺寸及穿透实现；确认无需新增平台 API、依赖或配置迁移。 |
| 二：最小实现 | 审阅差异，生产代码仅增加一项 builder 设置及原因注释；设置窗口独立，所有遮罩创建入口均覆盖。 |
| 三：回归 | Rust fmt、clippy（`-D warnings`）、17 项 Rust 测试、前端类型检查、4 项测试、生产构建和 Linux 原生调试构建通过。原生交互及 CI 记录如下。 |

未添加仅断言源码包含 `shadow(false)` 的测试：该断言不能验证系统边缘、尺寸或命中行为。使用现有行为测试和真实原生交互进行回归，平台视觉效果单独验收。

## 原生回归与 CI

- Linux 环境：Debian 13 x64、Xvfb 1280 × 800、Openbox、X11、100% 缩放，无桌面合成器。使用真实应用、X11 指针事件和实际 D-Bus 托盘菜单；数据写入隔离的 `/tmp/useless-shadow-review/`。
- 已通过：移动、八方向缩放、设置颜色/不透明度、新建 100% 黑色、复制继承蓝色 40%、全部隐藏/显示、锁定输入区域及托盘解锁、96 × 32 最小尺寸删除、零遮罩会话及退出保存。Linux Tao 锁定保留 1 × 1 输入区域，未将其表述为全平台零输入。
- 退出重启通过：已初始化空集合补建一个默认遮罩，第二次重启保留同一标识；直接恢复 v0.1.6 的蓝色 40% / 白色 90% 配置，标识、外观和几何信息一致；重复重启无几何漂移，隐藏/锁定状态均恢复为可见、可编辑。原始检查结果见上述隔离目录的 `interaction-results.json`。
- 四平台 CI 使用既有质量检查及 Windows x64、macOS Intel、macOS Apple Silicon、Linux x64 构建矩阵，实时状态见[修复分支 Actions](https://github.com/JakeJia/Useless-subtitle/actions?query=branch%3Afix%2Fnative-mask-shadow)及对应 PR。构建通过不等同于 Windows/macOS 交互验收。

## Windows/macOS 待验

Windows 10/11 的直线边缘应在黑、白、彩色以及不同不透明度下均无额外框线；分别检查编辑和锁定、100%/125%/150%/200% 显示缩放、八方向缩放、最小尺寸工具栏、锁定后的边缘点击/滚轮穿透、托盘“显示并编辑”、旧配置恢复及多次重启的尺寸位置。混合 DPI/负坐标显示器也应覆盖。macOS 两架构检查阴影、圆角、移动缩放、穿透和托盘恢复。

当前云端没有 Windows/macOS 交互桌面。源码 review、本地回归和跨平台编译分别提供证据，不作为这些平台已通过交互验收或“零回归”的结论。

## 0.1.7 版本与发布流程

PR #3 将 npm、Cargo、Tauri 及锁文件中的应用版本统一为 0.1.7，原生阴影修复随该版本构建进入安装包。沿用产品名称、应用标识和配置结构；已安装的 0.1.6 不会因 PR 更新而自动改变，需运行包含修复的新构建。

版本差异 review 确认六处应用版本字段一致，第三方依赖版本及阴影修复代码未改变。0.1.7 的本地 fmt、clippy、17 项 Rust 测试、4 项前端测试、类型检查、生产资源构建及原生调试构建通过；原生启动、旧配置恢复、托盘“显示并编辑”和正常退出检查通过。

[0.1.7 测试构建 CI](https://github.com/JakeJia/Useless-subtitle/actions/runs/38678931317)已通过质量检查及四平台构建，测试提交为 `24b8bbbe635d6313efa8951c1e7a8ae7f089735a`。Windows 便携 EXE 的文件版本及产品版本已核对为 0.1.7.0，下载归档的 SHA-256 与 GitHub 产物记录一致。发布准备仅同步文档和 Release 正文，应用代码及依赖保持该已验证提交的内容。

完成本次文档差异 review 和版本一致性检查后合并 PR，确认主分支包含修复及全部 0.1.7 版本字段，再从该提交创建 `v0.1.7` tag。tag 工作流重新执行质量检查和 Windows x64、macOS Intel、macOS Apple Silicon、Linux x64 构建，全部通过后才运行发布 job。最终核对 tag 指向、正式 Release 及 10 个预期产物；不改写 v0.1.6 的 tag 或安装包。

发布结果以 [v0.1.7 Release](https://github.com/JakeJia/Useless-subtitle/releases/tag/v0.1.7) 和对应 Actions 为准。Windows/macOS 交互待验及未签名分发状态继续保留；构建成功不代替真机验收。
