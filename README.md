[中文](./docs/README_zh.md) | English

# Useless Subtitle

Useless Subtitle is a lightweight desktop mask utility for covering unwanted hardcoded subtitles while watching videos. It supports multiple independent always-on-top masks, configurable color and opacity, full mouse click-through, tray-based recovery, and safe multi-monitor restoration.

## Features

- **Multiple masks:** Create, duplicate, configure, hide, recover, and delete independent masks.
- **Precise edit controls:** Drag the mask background to move it and use all four edges or corners for native resizing.
- **Safe click-through:** Locking removes edit controls and passes clicks and scrolling to the application underneath. Locked masks are recovered from the system tray.
- **Persistent appearance and geometry:** Stable identifiers and a centralized Rust state service prevent multi-window updates from overwriting each other.
- **Safe startup:** Restored masks always start visible and editable. Corrupt configuration and disconnected displays fall back to recoverable states.
- **DPI-aware recovery:** Geometry is stored in monitor-relative logical pixels and clamped to a connected display on restoration.
- **Tray control:** Create masks, explicitly show or hide all, unlock all, edit individual masks, and recover off-screen windows.

## Supported Platforms

- **Tier 1:** Windows 10/11 x64.
- **Tier 1:** macOS 11+, Intel and Apple Silicon.
- **Experimental:** Linux x64. Behavior depends on the desktop environment, system tray implementation, and X11/Wayland compositor.

## Download and Installation

Download the latest package from the [GitHub Releases page](https://github.com/JakeJia/Useless-subtitle/releases).

- **Windows x64:** MSI, setup executable, or portable ZIP.
- **macOS Apple Silicon:** aarch64 DMG or portable application ZIP.
- **macOS Intel:** x64 DMG or portable application ZIP.
- **Linux x64:** AppImage, DEB, or RPM.

Current community builds may be unsigned. macOS Gatekeeper or Windows SmartScreen can therefore display a warning. Verify that the package was downloaded from this repository before choosing the platform’s manual-open option.

## How to Use

1. On first launch, one black 90%-opaque mask appears in the center of the primary display.
2. Drag an empty part of the mask to move it. Drag an edge or corner to resize it.
3. While editing, use the always-visible toolbar:
   - `⋯` opens the mask menu.
   - `🔒` locks the mask and enables complete mouse click-through.
   - `×` deletes that mask without quitting the application.
4. Right-clicking an editable mask opens the same menu as `⋯`.
5. Once locked, the mask intentionally has no clickable controls. Open the system tray menu and choose **Show and Edit** for one mask or **Unlock All**.
6. Closing every mask leaves the application running in the tray. Choose **New Mask** to create another or **Quit** to exit.

## Development

Requirements: Node.js 22, Rust 1.97.1, and the platform dependencies required by Tauri 2.

```bash
npm ci
npm run check
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
npm run tauri build
```

GitHub Actions performs frontend checks, Rust formatting/lint/tests, and package builds for Windows, macOS Intel, macOS Apple Silicon, and Linux. Branch builds never publish a release; publishing requires a `v*` tag.

## Documentation

- [Software Requirements Specification](./docs/SRS.md) — v0.3 implementation baseline for v0.1.5.
- [Historical Implementation Plan](./docs/implement_plan.md) — archived description of the original four-sprint implementation.

## Current Roadmap

- [x] Cross-platform package automation.
- [x] Stable multi-mask state and persistence.
- [x] Native move and eight-direction resize interactions.
- [x] Transactional click-through and tray recovery.
- [x] Monitor/DPI-aware restoration and off-screen recovery.
- [ ] Platform code signing and macOS notarization.
- [ ] Optional profiles and automatic updates.

## License

[MIT License](./LICENSE)
