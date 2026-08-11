[中文](./docs/README_zh.md) | English

# Useless Subtitle

A minimalist, lightweight desktop always-on-top mask tool, designed specifically for watching videos with unwanted hardcoded subtitles (like built-in Chinese subtitles on Bilibili, YouTube, etc.). It helps you easily cover unwanted subtitle areas, giving you an immersive viewing and foreign language learning experience.

## ✨ Features

- **Always on Top**: Constantly floats above video players or browser web pages, remaining stably on top without being covered.
- **ToolWindow Mode**: A pure experience, forcibly hiding the taskbar icon and Alt-Tab switching, with only the system tray standing guard.
- **Lock and Click-through**: One-click lock mode. Once locked, all interactive elements within the mask disappear, and mouse events (clicks, scrolling, etc.) pass directly through to the underlying video, without affecting any of your operations.
- **Multi-Mask Management**: Supports creating multiple mask layers simultaneously. In edit mode, you can right-click any mask to independently adjust its color and opacity, or click the independent `❌` in the top right corner to destroy it.
- **Safe Boot Mechanism**: Automatically restores the previous position and size upon each startup, but intelligently unlocks all states, completely avoiding the "black screen panic" of a deadlocked startup.
- **Flexible Local + Global Control**:
  - **Local Operation**: Provides built-in floating `🔒` and `❌` buttons in the top right corner during edit mode.
  - **Global Tray**: One-click "Toggle Visibility" and "Unlock All" from the system tray, saying goodbye to shortcut conflicts.
- **Highly Customizable Appearance**: Supports stepless opacity adjustment from 10% to 100%, free setting of mask colors (pure black, translucent gray, etc.), and soft rounded corner transitions.

## 📚 Documentation

The functional boundaries and interaction logic of the software have been finalized through in-depth sandbox deduction. See details:
- [Software Requirements Specification (SRS)](./docs/SRS.md) - **v0.3 Final Confirmed**

## 🛠️ Tech Stack

- **Supported Platforms**: Cross-platform support (Windows 10/11 & macOS 11+)
- **Primary Framework**: **Tauri (Rust + Web)** - Extremely lightweight size (~10MB) and low memory footprint (~20MB)
- **Alternative Framework**: **Python / PyQt6**
- **Build Tools**: Vite / Cargo (Tauri)

## 📥 Download and Installation

**No programming knowledge required, download and use:**
1. Visit the [Releases page](../../releases) of this project to find the latest version.
2. Download the corresponding file for your operating system:
   - **Windows**: Download the `.msi` or `.exe` installer and double-click to install.
   - **macOS**: Download the `.dmg`, double-click to open, and drag it into the "Applications" folder.
   - **Linux**: Download the `.AppImage` or `.deb`.

## 💡 How to Use

1. After opening the software, a **translucent black mask** will appear on the desktop.
2. **Move and Resize**: Use the mouse to drag it over the unwanted subtitle area, and drag the edges to adjust the size.
3. **Right-click Settings**: **Right-click** on the mask to independently modify its color and opacity.
4. **One-click Click-through**: Click the `🔒` icon in the top right corner of the mask. The mask border will disappear, and **mouse clicks will completely pass through the mask** (without affecting clicks on the underlying web page and video controls).
5. **Global Tray Control**: If you need to disable click-through, create multiple masks, or completely exit the software, find the software icon in the **system tray (bottom right corner of the taskbar / Mac menu bar)** and right-click for global control.

## 🗺️ Roadmap

- [ ] **v0.1 MVP**: Implement a basic always-on-top borderless mask window, ensure it hides the taskbar icon as a ToolWindow, and support mouse dragging and corner/edge resizing.
- [ ] **v0.2 Independent Control and Local Persistence**: Introduce a right-click menu within the mask for independent color and opacity adjustment; implement real-time configuration saving to local JSON and safe reloading; tray creation/closing of multiple masks.
- [ ] **v0.3 Lock and Click-through Interaction**: Implement local floating `🔒`/`❌` buttons on the mask; connect underlying OS APIs to achieve pure mouse click-through, and implement global unlocking via the tray.
- [ ] **v1.0 Cross-platform Official Release**: Optimize the always-on-top experience in web/video full-screen mode and multi-monitor DPI scaling mapping, and release Windows and macOS installation/portable packages.

## 📄 License

[MIT License](LICENSE)