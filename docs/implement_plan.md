# Useless Subtitle — Historical Implementation Plan

> **Status:** Archived
>
> **Original target:** Tauri v2 (Rust) + Vue 3 (TypeScript)
>
> **Historical requirements baseline:** SRS v0.3 draft
>
> **Notice:** This document records the original implementation approach. It is retained for project history only and must not be used as the current delivery plan or source of truth. The maintained SRS documents define current product behavior.

The original work was divided into four incremental sprints. Each sprint was intended to produce a runnable build while validating high-risk desktop behaviors early.

---

## Sprint 1: Application Shell, Dragging, and Resizing

**Historical goal:** Build a transparent, borderless, always-on-top mask window that could be moved and resized.

- [x] Initialize a Vue 3 + TypeScript Tauri project.
- [x] Configure the mask window with:
  - `decorations: false`
  - `transparent: true`
  - `alwaysOnTop: true`
  - `skipTaskbar: true` where supported
- [x] Render a full-window translucent mask surface.
- [x] Add a window drag region.
- [ ] Implement correct native eight-direction resize dragging.

The original implementation added eight HTML resize handles, but routed them to window dragging instead of directional resizing. This item remained incomplete and is addressed by the later core-functionality refactor.

---

## Sprint 2: System Tray and Multiple Mask Windows

**Historical goal:** Move global controls into the system tray and allow multiple independent mask windows.

- [x] Create a native tray icon and menu.
- [x] Add menu actions for creating a mask, toggling visibility, unlocking masks, and quitting.
- [x] Generate dynamic mask window labels.
- [x] Add local lock and close controls.
- [x] Allow individual mask windows to close.
- [ ] Guarantee stable mask identifiers across application restarts.

The initial counter-based label strategy restarted at `mask_1` for every process and could collide with restored windows. Stable persistent identifiers were not part of this historical implementation.

---

## Sprint 3: Appearance Controls and Click-Through

**Historical goal:** Support per-mask appearance settings and full mouse click-through while locked.

- [x] Add color selection.
- [x] Add opacity settings from 10% through 100%.
- [x] Add a lock action that enables native cursor-event pass-through.
- [x] Hide local controls while locked.
- [x] Add a tray action that disables click-through for all masks.
- [ ] Provide a transactional lock transition with failure recovery.
- [ ] Provide a native or independent settings surface that works for very small masks.

The custom context menu was rendered inside each mask webview and could be clipped by small window dimensions. A locked mask intentionally had no clickable in-window unlock control because full click-through prevents the window from receiving pointer events.

---

## Sprint 4: Persistence and Safe Startup

**Historical goal:** Restore multiple mask windows without trapping the user in an inaccessible click-through state.

- [x] Store mask geometry and appearance in a local JSON-backed store.
- [x] Restore saved windows on startup.
- [x] Start restored masks unlocked.
- [x] Remove a closed mask from persisted state.
- [ ] Serialize multi-window state mutations through a single writer.
- [ ] Validate restored geometry against connected displays.
- [ ] Convert correctly between logical and physical pixels.
- [ ] Recover from invalid or corrupted configuration data.
- [ ] Flush pending state before application exit.

The historical frontend used read-modify-write operations against a shared mask array. Concurrent window updates could overwrite each other, and physical geometry was later interpreted as logical geometry during restoration.

---

## Historical Release Checklist

- [x] Produce Windows installer and portable artifacts.
- [x] Produce macOS Intel and Apple Silicon DMG and portable artifacts.
- [x] Produce Linux AppImage, DEB, and RPM artifacts.
- [x] Build all supported targets in GitHub Actions.
- [ ] Complete interactive behavior tests on physical Windows and macOS systems.
- [ ] Add macOS signing and notarization.
- [ ] Add Windows code signing.

---

## Superseding Work

The `refactor/core-functionality` effort supersedes this historical plan. Its objectives are:

1. Define an explicit lifecycle, visibility, and interaction-mode state model.
2. Make Rust the single owner of mask state and persistence.
3. Implement reliable dragging, eight-direction resizing, locking, unlocking, and deletion.
4. Add stable identifiers and race-free multi-mask persistence.
5. Restore geometry safely across monitors and DPI scale factors.
6. Align the English and Chinese SRS documents and expose only implemented behavior in the README files.
7. Add automated and full-platform release validation before publishing v0.1.5.
