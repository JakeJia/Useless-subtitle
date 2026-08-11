中文 | [English](../README.md)

# Useless Subtitle（字幕遮罩工具）

Useless Subtitle 是一款轻量桌面字幕遮罩工具，用于在观看视频时遮挡不需要的硬字幕。它支持多个独立置顶遮罩、自定义颜色与不透明度、完整鼠标穿透、托盘恢复以及安全的多显示器位置恢复。

## 核心功能

- **多遮罩：** 可创建、复制、配置、隐藏、找回和删除相互独立的遮罩。
- **精确编辑：** 拖动遮罩空白区域可移动窗口，拖动四边或四角可进行原生缩放。
- **安全鼠标穿透：** 锁定后隐藏编辑控件，并将点击和滚轮传递给下层应用；通过系统托盘恢复锁定遮罩。
- **外观和几何持久化：** 稳定标识与 Rust 集中状态服务可避免多窗口更新相互覆盖。
- **安全启动：** 恢复的遮罩始终以可见、可编辑状态启动；配置损坏或原显示器断开时会回退到可恢复状态。
- **DPI 感知恢复：** 位置使用显示器相对逻辑像素保存，并在恢复时限制到已连接显示器内。
- **托盘控制：** 可新建遮罩、明确显示或隐藏全部、全部解锁、逐个编辑以及找回屏幕外窗口。

## 支持平台

- **一级支持：** Windows 10/11 x64。
- **一级支持：** macOS 11 及以上，Intel 与 Apple Silicon。
- **实验性支持：** Linux x64。实际行为取决于桌面环境、系统托盘实现以及 X11/Wayland 合成器。

## 下载与安装

请从 [GitHub Releases 页面](https://github.com/JakeJia/Useless-subtitle/releases)下载最新版本。

- **Windows x64：** MSI、安装程序或便携 ZIP。
- **macOS Apple Silicon：** aarch64 DMG 或便携应用 ZIP。
- **macOS Intel：** x64 DMG 或便携应用 ZIP。
- **Linux x64：** AppImage、DEB 或 RPM。

当前社区构建可能尚未签名，因此 macOS Gatekeeper 或 Windows SmartScreen 可能显示警告。选择系统的手动打开选项前，请确认安装包来自本仓库。

## 使用方法

1. 首次启动时，主显示器中央会出现一个黑色、90% 不透明度的遮罩。
2. 拖动遮罩空白区域可移动窗口，拖动边缘或四角可调整尺寸。
3. 编辑状态下工具栏始终可见：
   - `⋯` 打开遮罩菜单。
   - `🔒` 锁定遮罩并启用完整鼠标穿透。
   - `×` 删除当前遮罩，但不退出应用。
4. 右键单击可编辑遮罩，会打开与 `⋯` 相同的菜单。
5. 锁定后，遮罩有意不保留可点击控件。请打开系统托盘，针对单个遮罩选择 **Show and Edit**，或选择 **Unlock All**。
6. 删除全部遮罩后，应用仍在托盘运行。可选择 **New Mask** 新建遮罩，或选择 **Quit** 退出。

## 开发与验证

环境要求：Node.js 22、Rust 1.97.1，以及 Tauri 2 所需的平台依赖。

```bash
npm ci
npm run check
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
npm run tauri build
```

GitHub Actions 会执行前端检查、Rust 格式/lint/测试，并构建 Windows、macOS Intel、macOS Apple Silicon 和 Linux 安装包。普通分支构建不会发布 Release；只有 `v*` tag 才能触发发布。

## 项目文档

- [软件需求规格说明书](./SRS_zh.md) — v0.1.5 使用的 v0.3 实施基线。
- [历史实施计划](./implement_plan.md) — 已归档的原四阶段实现说明。

## 当前路线图

- [x] 跨平台打包自动化。
- [x] 稳定的多遮罩状态与持久化。
- [x] 原生移动和八方向缩放。
- [x] 事务式鼠标穿透与托盘恢复。
- [x] 显示器/DPI 感知恢复和屏幕外找回。
- [ ] 平台代码签名与 macOS 公证。
- [ ] 可选配置档案与自动更新。

## 许可证

[MIT License](../LICENSE)
