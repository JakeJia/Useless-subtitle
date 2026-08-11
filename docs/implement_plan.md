# Useless Subtitle - 开发实施计划 (Implementation Plan)

> **目标架构**: Tauri v2 (Rust) + Vue 3 (TypeScript) + Tailwind CSS
> **需求依据**: 参照 `SRS.md (v0.3 Final Confirmed)`

本计划将整体开发拆解为 4 个敏捷冲刺阶段 (Sprints)。每个阶段完成即可获得一个可运行的版本，确保核心风险（如窗口穿透、多开管理）被尽早验证。

---

## 🟢 Sprint 1: 骨架搭建与基础拖拽 (MVP)

**目标**: 跑通前后端通信，得到一个可以鼠标拖拽、拉伸的透明无边框“纯净方块”。

* [ ] **1.1 项目初始化**
  * 使用 `pnpm create tauri-app` 生成项目骨架 (选择 Vue + TS 模板)。
  * 清理模板自带的冗余代码。
* [ ] **1.2 窗口基础配置 (ToolWindow)**
  * 修改 `tauri.conf.json`（或 `src-tauri/src/main.rs`），设置初始窗口属性：
    * `decorations: false` (无边框)
    * `transparent: true` (透明背景)
    * `alwaysOnTop: true` (置顶)
    * `skipTaskbar: true` (隐藏任务栏图标)
* [ ] **1.3 前端 UI 基础实现**
  * 在 `App.vue` 实现 100vw/100vh 的全屏 `div`，背景设为半透明黑色。
  * 加上 `data-tauri-drag-region` 属性，测试原生鼠标拖拽。
* [ ] **1.4 窗口边缘变形 (Resize) 处理**
  * 由于无边框窗口丢失了系统原生的拉伸判定，需在前端四个角和边缘放置隐形手柄 (HTML `div`)。
  * 绑定鼠标按下事件，通过 Tauri API (`appWindow.startResizing()`) 呼出系统级的边缘拉伸调整。

---

## 🟡 Sprint 2: 托盘系统与多遮罩实例 (Multi-Window)

**目标**: 摆脱依赖主界面的控制，完全接管系统托盘；实现动态生成和销毁多块遮罩。

* [ ] **2.1 系统托盘 (System Tray) 初始化**
  * 在 `src-tauri/src/tray.rs` (需新建) 中注册原生系统托盘图标。
  * 构建托盘右键菜单：`新建遮罩 (New)`、`显示/隐藏所有 (Toggle Visibility)`、`退出 (Quit)`。
* [ ] **2.2 多窗口动态生成**
  * 点击托盘的“新建遮罩”时，在 Rust 中通过 `tauri::WebviewWindowBuilder` 动态生成新的独立窗口。
  * 为新窗口生成唯一的 `label` (如 `mask_1`, `mask_2`) 以便独立追踪。
* [ ] **2.3 遮罩内部控制 (销毁自身)**
  * 前端 Vue 组件实现：鼠标悬浮时在右上角显示 `❌` 按钮。
  * 点击 `❌` 时，前端通过 Tauri IPC 调用 `appWindow.close()` 销毁当前窗口。

---

## 🟠 Sprint 3: 穿透引擎与独立调色 (The Core Magic)

**目标**: 攻克本项目最大的核心技术点：锁定状态下的“幽灵穿透”；并支持多遮罩各自拥有不同的颜色。

* [ ] **3.1 实现局部调色盘**
  * 前端编写右键上下文菜单 (`ContextMenu.vue`)。
  * 实现颜色选择和 10%~100% 透明度调节。
  * Vue 状态与 UI 背景色/透明度双向绑定。
* [ ] **3.2 穿透引擎 (Rust 核心 API)**
  * 前端右上角悬浮 `🔒` 按钮：点击后，UI 隐藏所有控制手柄和边框。
  * **前端向 Rust 发送锁定指令**：调用 `appWindow.setIgnoreCursorEvents(true)`，正式开启鼠标完全穿透。
* [ ] **3.3 托盘全局解锁 (唯一救命稻草)**
  * 在托盘菜单增加 `解锁所有遮罩 (Unlock All)` 选项。
  * 点击时，Rust 遍历获取当前所有的活跃窗口 (通过 `app.webview_windows()`)。
  * 对所有窗口强制执行 `set_ignore_cursor_events(false)`，恢复鼠标拦截。
  * 通过 Event 广播通知所有前端窗口：“你们已被解锁，请重新显示 UI 控制边框”。

---

## 🔴 Sprint 4: 状态持久化与安全启动打磨 (Polish & Safe Boot)

**目标**: 加上防崩溃的配置保存，完善防死锁体验，准备多平台发布。

* [ ] **4.1 本地配置持久化**
  * 引入 Tauri 官方插件 `@tauri-apps/plugin-store`。
  * 前端监听：当停止拖拽 (`onmouseup`) 或修改颜色后，立即向 `store.json` 保存当前窗口的 `label`、`x`、`y`、`width`、`height`、`color`、`opacity`。
* [ ] **4.2 安全重载与防丢逻辑**
  * Rust 启动时接管入口，不再使用默认窗口。
  * 从 `store.json` 读取历史窗口列表，循环创建并复原坐标尺寸。
  * **安全校验 A**：无论上次保存时是否锁定，本次启动**强制禁用** `ignoreCursorEvents`，确保所有方块可被编辑。
  * **安全校验 B**：校验坐标 `(x, y)` 是否在当前活动显示器范围内（避免拔掉副屏后找不到遮罩）。如果越界，坐标强制重置为 `(100, 100)`。
* [ ] **4.3 编译与打包测试**
  * （推荐切换至 Windows/macOS 原生环境测试）执行 `pnpm tauri build`。
  * 验证 `.exe` 在全屏游戏或 Bilibili 网页全屏下的置顶有效性。
  * 验证任务栏中是否干净（无图标残留）。