[中文](./SRS_zh.md) | English

# Software Requirements Specification

> **Product:** Useless Subtitle
>
> **Document version:** v0.3
>
> **Status:** Approved implementation baseline for v0.1.5
>
> **Last updated:** 2026-08-11
>
> **Source of truth:** This document and its Chinese translation define the same requirements. A behavior is release-ready only when both documents and the implementation agree.

## 1. Purpose and Scope

Useless Subtitle is a lightweight desktop utility that places one or more configurable mask windows above video players or browser content. Its primary use is covering unwanted hardcoded subtitles while preserving access to the controls of the application underneath.

The v0.1.5 scope includes mask lifecycle management, appearance controls, native dragging and resizing, safe click-through locking, system-tray recovery, multi-mask persistence, and monitor/DPI-aware restoration.

### 1.1 Supported Platforms

- **Tier 1:** Windows 10/11 x64.
- **Tier 1:** macOS 11 or later, Intel and Apple Silicon.
- **Tier 2 / experimental:** Linux x64. Tray, transparency, click-through, and topmost behavior may vary by desktop environment and display server.

### 1.2 Terms

| Term | Definition |
|---|---|
| Mask | One borderless always-on-top window used to cover part of the screen. |
| Edit mode | The mask receives pointer input and exposes move, resize, settings, lock, and delete controls. |
| Locked mode | The mask ignores pointer input so clicks and scrolling reach the application underneath. |
| Hidden | The mask window is not visible. Visibility is independent from edit/locked mode. |
| System tray | The Windows/Linux notification area or macOS menu-bar status item. |
| Logical pixel | A DPI-independent unit used to preserve visual size across displays. |

## 2. State Model

Each persisted mask has a stable identifier, name, appearance, and geometry. Runtime behavior is represented by three independent dimensions:

- **Lifecycle:** existing or deleting.
- **Visibility:** visible or hidden.
- **Interaction mode:** edit or locked.

Locked and hidden states are runtime safety states and are not restored after application restart. Every restored mask starts visible and in edit mode.

The application may contain zero masks. Closing the last mask does not quit the application; the tray remains available. A default mask is created only when no valid application configuration has ever been initialized.

## 3. Functional Requirements

### 3.1 Mask Creation and Lifecycle

- **FR-MASK-001:** First launch shall create one visible, editable mask centered on the primary display with a default logical size of 600 × 120.
- **FR-MASK-002:** “New Mask” shall create a visible, editable mask with a stable UUID-backed identifier and persist it immediately.
- **FR-MASK-003:** New masks shall use a cascaded position on the active or primary display and shall be clamped to its work area.
- **FR-MASK-004:** A mask may be duplicated, including its appearance and size, but the duplicate shall receive a new identifier and a non-overlapping cascaded position.
- **FR-MASK-005:** Closing a mask shall remove its persisted record and close its window. Persistence failures shall not leave an unresponsive close control; failures shall be reported and retried by the application state service.
- **FR-MASK-006:** Deleting the final mask shall leave a tray-only application. Restarting an initialized empty configuration shall not recreate a mask automatically.
- **FR-MASK-007:** Window creation failures shall not terminate the process. Invalid entries shall be skipped and reported while valid masks continue to load.

### 3.2 Appearance

- **FR-APP-001:** Each mask shall independently store a six-digit `#RRGGBB` color and an integer opacity from 10 through 100.
- **FR-APP-002:** Default appearance shall be `#000000` at 90% opacity.
- **FR-APP-003:** Opacity controls shall use 10% increments. Documentation shall use “opacity” consistently; 100% means fully opaque.
- **FR-APP-004:** Appearance changes shall be previewed immediately and persisted through the centralized state service.
- **FR-APP-005:** The edit-mode mask shall show a subtle border and rounded corners. Locked mode shall hide borders and controls.

### 3.3 Pointer and Click Behavior

- **FR-INPUT-001:** In edit mode, primary-button drag on the mask background shall move the window.
- **FR-INPUT-002:** Eight resize hit zones shall invoke the matching native resize direction: North, South, East, West, NorthEast, NorthWest, SouthEast, and SouthWest.
- **FR-INPUT-003:** Resize and toolbar hit zones shall never initiate window movement.
- **FR-INPUT-004:** A background click without movement shall only activate the mask. Double-click shall not maximize or lock the mask.
- **FR-INPUT-005:** Right-click and the local settings button shall open the same mask menu.
- **FR-INPUT-006:** In edit mode, a compact toolbar shall expose Settings, Lock, and Delete. Controls shall be real accessible buttons and shall remain visible while editing.
- **FR-INPUT-007:** The minimum logical mask size shall be 96 × 32 so all edit controls remain reachable.
- **FR-INPUT-008:** Menus and appearance controls shall not be constrained by the mask webview bounds; native menus or an independent settings window shall be used.

### 3.4 Locking and Click-Through

- **FR-LOCK-001:** Locking shall hide the edit UI and enable full-window pointer-event pass-through, including clicks and scrolling.
- **FR-LOCK-002:** A fully locked mask shall not expose an in-window unlock control because it cannot receive pointer input.
- **FR-LOCK-003:** Lock shall be enabled only after the system tray is available. If the platform cannot provide a recovery entry, lock shall remain disabled.
- **FR-LOCK-004:** A failed native click-through transition shall leave or return the mask to edit mode and display a recoverable error.
- **FR-LOCK-005:** Unlocking shall disable pointer pass-through before showing edit controls.
- **FR-LOCK-006:** The tray shall provide “Unlock All” and a per-mask “Show and Edit” recovery action.
- **FR-LOCK-007:** No global keyboard shortcut is required for v0.1.5.
- **FR-LOCK-008:** Lock state shall not be persisted; every application start shall be safe and editable.

### 3.5 System Tray

- **FR-TRAY-001:** The tray shall remain available while the process runs, including when zero masks exist.
- **FR-TRAY-002:** Both primary and secondary tray activation should expose the menu where the platform permits, avoiding a hidden click-only command.
- **FR-TRAY-003:** The top-level menu shall provide New Mask, Show All or Hide All, Unlock All, per-mask controls, Recover Off-Screen Masks, Preferences when implemented, and Quit.
- **FR-TRAY-004:** Show All and Hide All shall set one explicit state for every mask; they shall not invert each window independently.
- **FR-TRAY-005:** Show All shall preserve lock mode. Unlock All shall preserve visibility.
- **FR-TRAY-006:** Per-mask “Show and Edit” shall show, unlock, and focus the selected mask.
- **FR-TRAY-007:** Tray actions shall operate only on registered mask windows and never on settings or utility windows.
- **FR-TRAY-008:** Quit shall flush pending configuration writes before terminating.

### 3.6 Settings Surface

- **FR-SET-001:** Right-click or Settings shall expose Appearance, Lock, Reset Geometry, Duplicate, and Delete actions.
- **FR-SET-002:** Custom color and opacity editing shall use one reusable non-click-through settings window associated with the selected mask.
- **FR-SET-003:** Closing the settings window shall not close or delete its mask.
- **FR-SET-004:** Escape shall close transient menus or the settings surface where supported.

### 3.7 Persistence and Recovery

- **FR-PERSIST-001:** Rust shall be the single owner of the complete mask collection. Frontend windows shall not perform shared-array read-modify-write operations.
- **FR-PERSIST-002:** Configuration shall contain a schema version and stable mask identifiers.
- **FR-PERSIST-003:** State writes shall be serialized. Geometry events shall be debounced, and application exit shall flush pending changes.
- **FR-PERSIST-004:** Configuration writes shall use an atomic or equivalently crash-safe strategy with a recoverable last-known-good copy.
- **FR-PERSIST-005:** Missing configuration shall produce first-launch defaults. Corrupt configuration shall be preserved for diagnosis and replaced with a safe valid configuration.
- **FR-PERSIST-006:** Color, opacity, dimensions, identifiers, and configuration version shall be validated before use.
- **FR-PERSIST-007:** A failed mask record shall not prevent other valid masks or the tray from starting.

### 3.8 Multi-Monitor and DPI

- **FR-DISPLAY-001:** Saved geometry shall use logical dimensions and monitor-relative logical offsets, together with enough monitor information to choose a restore target.
- **FR-DISPLAY-002:** Physical values returned by native window APIs shall be converted using the relevant monitor scale factor before persistence or restoration.
- **FR-DISPLAY-003:** Restoration shall prefer the original connected monitor and otherwise fall back to the primary monitor.
- **FR-DISPLAY-004:** Restored geometry shall be clamped so at least a 48 × 24 logical-pixel recovery area remains inside a connected display work area.
- **FR-DISPLAY-005:** Show All, application resume, and Recover Off-Screen Masks shall revalidate visible geometry.
- **FR-DISPLAY-006:** Negative monitor coordinates and mixed 100%, 125%, 150%, 200%, and Retina scaling shall be covered by tests.

### 3.9 Topmost and Platform Behavior

- **FR-PLATFORM-001:** Masks shall be borderless, resizable in edit mode, transparent outside their colored surface, and always on top of normal application windows.
- **FR-PLATFORM-002:** Windows builds shall use an appropriate tool-window/taskbar-hidden behavior and per-monitor DPI awareness.
- **FR-PLATFORM-003:** macOS builds shall use an accessory-style application policy and full-screen auxiliary/Space behavior where supported.
- **FR-PLATFORM-004:** The product shall not claim to overlay protected system UI, security prompts, exclusive full-screen applications, or environments that reject topmost hints.
- **FR-PLATFORM-005:** Linux limitations shall be documented rather than represented as Tier 1 guarantees.

## 4. Non-Functional Requirements

- **NFR-001 Reliability:** No user action, malformed mask entry, duplicate identifier, missing display, or persistence error shall panic the application process.
- **NFR-002 Recoverability:** A user shall be able to restore any locked or off-screen mask from the tray in no more than two menu selections.
- **NFR-003 Security:** Frontend capabilities shall be limited to the specific window and command operations required by mask UI. Direct frontend store access shall not be granted after the state refactor.
- **NFR-004 Performance:** Geometry persistence shall not perform a disk write for every native move event. Idle CPU usage shall be measured on release builds and reported rather than described as “near zero.”
- **NFR-005 Resource measurement:** Release validation shall record memory for one, five, and ten masks so requirements reflect the multi-webview architecture.
- **NFR-006 Accessibility:** Icon-only buttons shall expose accessible names and tooltips. Pointer hit targets shall remain usable at the minimum mask size.
- **NFR-007 Localization:** English and Chinese documentation shall remain structurally synchronized. Application UI localization is outside v0.1.5 unless implemented consistently.
- **NFR-008 Release safety:** Branch and pull-request workflows may build artifacts, but only a `v*` tag may publish a GitHub Release.
- **NFR-009 Distribution:** macOS and Windows signing status shall be stated accurately. Unsigned development artifacts shall not be described as warning-free installation packages.

## 5. Acceptance and Release Criteria

v0.1.5 is eligible for release only when:

1. Frontend type checking and production build pass.
2. Rust formatting, linting, unit tests, and locked dependency checks pass.
3. State-model, identifier, persistence, geometry-clamping, and DPI conversion tests pass.
4. Manual interaction checks cover move, all resize directions, lock, click-through, tray unlock, delete, zero-mask behavior, multi-mask restart, and off-screen recovery.
5. GitHub Actions builds Windows x64, macOS Intel, macOS Apple Silicon, and Linux x64 successfully.
6. A pull request is reviewed and merged into `main`.
7. The same `main` commit passes the complete workflow without publishing a release.
8. Tag `v0.1.5` is created from the verified `main` commit.
9. The tag workflow publishes the expected release artifacts and reports success.

## 6. Out of Scope for v0.1.5

- Global keyboard shortcuts.
- Cloud synchronization.
- Automatic subtitle detection.
- Per-video profiles.
- Automatic application updates.
- Guaranteed overlay above protected system interfaces or exclusive full-screen content.
