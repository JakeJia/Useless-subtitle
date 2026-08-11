[中文](./SRS_zh.md) | English

# Software Requirements Specification (SRS)

> **Project Name**: Useless Subtitle
> **Document Version**: v0.2 (Draft)
> **Update Time**: 2026-08-07
> **Description**: This document is formulated based on the ISO/IEC/IEEE 29148 standard framework, aiming to comprehensively outline the tool's functions, interactions, non-functional requirements, and boundary scenarios.

---

## 1. Introduction

### 1.1 Purpose
This document clarifies the product requirements, system boundaries, interaction logic, and non-functional metrics of the "Useless Subtitle" mask tool, providing a standard basis for technology selection, architecture design, and automated testing.

### 1.2 Glossary
| Term | Alias | Description |
| :--- | :--- | :--- |
| **Always on Top** | Topmost | The window maintains the highest priority in the operating system's window hierarchy and is not covered by other normal windows. |
| **Click-through** | Pass-through | The mask window ignores all mouse click/scroll events, passing them directly to the application below the mask (such as a browser video player). |
| **Lock Mode** | Lock Mode | The mask enters a fixed state, disabling dragging and resizing, and enabling mouse click-through. |
| **System Tray** | Menu Bar | The small icon in the Windows taskbar notification area or the macOS top Menu Bar. |
| **High DPI Awareness** | DPI Awareness | Adapts to high-resolution screen scaling (e.g., 125%, 150%, 200%) to prevent blurry interfaces or coordinate offsets. |

### 1.3 Product Scope
This project is a lightweight desktop utility tool. It is primarily used when users watch online videos (like Bilibili, YouTube) or play local videos, overlaying one or more customizable always-on-top mask layers on the screen to cover unwanted subtitles in the video frame (such as Chinese subtitles in bilingual videos), providing a focused foreign language learning or distraction-free viewing environment.

---

## 2. Overall Description

### 2.1 Operating Environment
* **OS Support**:
  * [ ] Option A: **Windows First** (Windows 10 / 11 64-bit)
  * [ ] Option B: **macOS First** (macOS 11+ Intel & Apple Silicon)
  * [x] Option C: **Cross-platform** (Windows + macOS) *(Recommended: Easily achievable using Tauri / PyQt)*

### 2.2 User Demographics and Typical Scenarios
* **Target Users**: Foreign language learners, overseas film and television enthusiasts, video/content creators.
* **Core Scenarios**: 
  1. A user opens Bilibili in a browser to play a video with original English audio, but the video has hardcoded Chinese subtitles.
  2. The user launches this tool, drags the mask layer to precisely cover the Chinese subtitle area, and adjusts the appropriate opacity/color.
  3. The user presses a shortcut key or clicks lock, the mask enters click-through mode, and the user can normally click the video to play/pause or adjust the progress bar.

---

## 3. Functional Requirements

### 3.1 Mask Window & Appearance
* **3.1.1 Borderless Always on Top**: The mask window must not have any native OS title bars or borders, and must maintain the highest topmost level (`Topmost`).
* **3.1.2 Color and Opacity Adjustment**:
  * **Color**: Supports common color presets (black, white, gray, translucent gray) as well as custom HEX/RGB via a color palette.
  * **Opacity**: Supports 10% - 100% opacity, with 10% step intervals.
  * **Recommended Defaults**: Pure black (`#000000`), opacity `90%`.
* **3.1.3 Style Extensions (Optional)**:
  * Rounded corners for the four edges.

### 3.2 Drag, Resize & Edge Cases
* **3.2.1 Free Dragging and Resizing**: 
  * In the unlocked state, holding the left mouse button on the border allows one-way dragging to expand, and holding the four corners allows two-way dragging to expand.
  * When the mouse moves to the mask edges/corners, a resize cursor is displayed, allowing size adjustment.
* **3.2.2 Minimum Size and Anti-loss Protection**:
  * Set a minimum size limit (e.g., `50px x 20px`) to prevent it from being resized too small to be selected.
  * **Anti-loss Logic**: Upon startup, verify if the previously saved coordinates are within the valid area of the currently connected monitors. If out of bounds, automatically reset to the center of the primary monitor.

### 3.3 Click-through / Lock Mode **[Core Feature]**
* **3.3.1 State Switching Logic**:
  ```
  [Editable State]  <---(Shortcut / Tray Menu / Context Menu)--->  [Locked State]
  - Draggable/Resizable                                            - Cannot drag/resize
  - Intercepts mouse events                                        - Mouse clicks/scrolls pass directly through to the bottom
  - Shows dashed border/resize cursors                             - Borderless solid color/transparent
  ```
* **3.3.2 Unlock Mechanisms**:
  * Once in click-through mode, the mask no longer responds to right-clicks. Reliable unlock methods must be provided:
    * **Method 1**: To avoid conflicts with user application shortcuts, do not set global shortcuts.
    * **Method 2**: System tray icon (Right-click tray icon -> Click "Unlock All" or "Toggle Visibility").
    * **Method 3**: Equip the top right corner of the mask frame with an unlock/lock icon button, displaying a 🔒 and 🔓 state. When hovering, the mouse shows a clickable cursor. Both locked and unlocked states can be clicked, switching to the opposite state upon a single click, with the icon changing accordingly.
  * **Comprehensive Solution**: Lock/Unlock button + System tray icon.

### 3.4 Configuration Persistence
* **3.4.1 Auto-save Configuration**:
  * When exiting the software or modifying settings, automatically save the following configurations to a local JSON/config file:
    * Mask position (`x`, `y`), size (`width`, `height`)
    * Color (`color`), opacity (`opacity`)
    * Lock state (`is_locked`)
* **3.4.2 Memory Reload**: Automatically restore the previous mask state and position upon the next startup.

### 3.5 Multi-Mask Support
* **Requirement Description**: Is it allowed to create multiple mask layers simultaneously (e.g., covering top prompts and bottom subtitles at the same time)?
  * [ ] **Single Mask Mode**: Simple and lightweight, only one mask window globally.
  * [x] **Multi-Mask Mode**: The tray menu supports "New Mask", with each window managed independently.

---

## 4. UI / UX Control

### 4.1 Control Interaction Entry Design
Since the mask window itself is extremely minimalist with no UI, control operations need to be implemented through peripheral entries:
1. **System Tray**:
   - **Left Click**: Toggle visibility of all masks.
   - **Right-click Menu Items**:
     - Toggle lock state (`Unlock All`)
     - Mask settings (`Color...` / `Opacity 10%~100%`) - *Note: Handled via context menu in current implementation*
     - Reset position and size (`Reset Position`)
     - Launch on startup settings (`Launch on Startup`)
     - Exit software (`Quit`)

---

## 5. Boundary Conditions

### 5.1 Fullscreen Adaptation
* **Problem Description**: When the browser switches to HTML5 fullscreen or OS fullscreen (like F11 fullscreen), the video window may preempt the highest level, causing normal always-on-top windows to be covered.
* **Solution Requirements**:
  * On Windows, `WS_EX_TOPMOST` and the corresponding top-level window hierarchy must be used.
  * On macOS, the Window Level needs to be set to an Overlay at the `NSStatusWindowLevel` or `NSScreenSaverWindowLevel` level to ensure it remains on top even in fullscreen.

### 5.2 Multi-monitor and High DPI Scaling Adaptation
* **Problem Description**: When a user drags the mask between different DPI scalings (e.g., primary screen 4K 150%, secondary screen 1080p 100%), sudden size changes or coordinate offsets may occur.
* **Solution Requirements**: The program must declare `Per-Monitor DPI Aware`, using physical to logical pixel conversion to ensure smooth dragging.

### 5.3 OS Permissions
* **macOS Permission Requirements**: On macOS, listening to global shortcuts and implementing window event click-through may require the user to grant permissions in "System Settings -> Privacy & Security -> Accessibility". A friendly guide prompt should be provided upon the software's first launch.

---

## 6. Non-Functional Requirements

### 6.1 Performance
* **Memory Footprint**: Memory usage requirements when running resident in the background:
  * Windows / macOS: **< 30MB** (if using Tauri / C# / C++) or **< 80MB** (if using Python/PyQt).
* **CPU Usage**: CPU usage in background idle and click-through states should be close to **0%**.
* **Startup Speed**: Software cold start time should not exceed **1.0 second**.

### 6.2 Deployment
* **Portable and Lightweight**: Provide a single-file, installation-free portable version, ready to use upon extraction; do not force reliance on complex installation packages.

---

## 7. Tech Stack Assessment

| Tech Stack | Cross-platform | Memory | Package Size | Dev Efficiency | Recommendation | Comments |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **Tauri + Web** | Yes | ~20-30MB | ~10MB | High | ⭐⭐⭐⭐⭐ | Highly recommended. Native Rust window base + Web frontend UI, extremely lightweight and easy to implement topmost/click-through/tray. |
| **C# WPF / WinUI** | No (Win only) | ~15-30MB | ~15MB | High | ⭐⭐⭐⭐ | If only targeting Windows, WPF handles Win32 `WS_EX_TRANSPARENT` click-through extremely maturely and smoothly. |
| **Python + PyQt6** | Yes | ~60-90MB | ~30MB | Very High | ⭐⭐⭐ | Fastest development, but the packaged Python interpreter has a relatively large size and memory footprint. |
| **Electron** | Yes | ~100MB+ | ~80MB+ | High | ⭐⭐ | Memory and size are too large, violating the "extremely lightweight" original intention. |

---

## 8. Action Items

Please confirm the following items before starting coding:
1. [x] **Operating Platform**: Cross-platform (Windows/macOS)
2. [x] **Multi-Mask**: Support creating [multiple mask layers] simultaneously
3. [x] **Tech Stack**: Primary choice is **Tauri**, secondary choice is **Python/PyQt**