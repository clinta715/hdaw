# Stage 5: Remove Slint Dependency Entirely

## Objective

Remove the `slint` crate and all Slint-related code from the project. The Qt bridges in `src/ui_qt/` fully replace all Slint functionality. After this stage, the only build path is `cargo build` (with `qt` as a default feature).

## Motivation

- All UI functionality has been incrementally ported to Qt QML across Phases 1–4 (Transport, Pool, FX Editor, Mixer, Timeline, Keyboard Shortcuts).
- Maintaining both Slint and Qt UI code is wasteful — ~4,500 lines of dead Slint code, duplicated logic, and build complexity.
- Removing Slint eliminates the Windows-only keyboard polling, the 16ms hybrid sync, and the dual-event-loop architecture.

## Changes

### 1. Cargo.toml

- Remove `slint = "1.16"` from `[dependencies]`
- Make `qt` a default feature:
  ```toml
  [features]
  default = ["qt"]
  qt = ["dep:cxx-qt", "dep:cxx-qt-lib", "dep:cxx", "dep:cxx-qt-build"]
  ```

### 2. File Deletions

| File | Lines | Contents |
|------|-------|----------|
| `src/ui/main_window.rs` | 2699 | `slint!` macro — entire Slint UI DSL (MainWindow, 11 structs, 45 properties, 40 callbacks) |
| `src/ui/timeline.rs` | 577 | Slint model sync functions, hit-testing utilities, image rendering |
| `src/ui/mod.rs` | 4 | Module declarations |
| `src/ui/components/mod.rs` | placeholder | Empty |
| `src/ui/mixer.rs` | placeholder | Empty |
| `src/ui/transport.rs` | placeholder | Empty |
| `src/ui/waveform.rs` | placeholder | Empty |
| `src/app/callbacks.rs` | 1238 | 40+ Slint `window.on_xxx()` callback closures |
| `src/app/input.rs` | 60 | Win32 `GetAsyncKeyState` polling + `app_has_focus()` |

### 3. Function Deletions

| Function | File | Reason |
|----------|------|--------|
| `render_waveform_image()` | `src/utils/waveform.rs` | Returns `slint::Image`; only called from `ui/timeline.rs` sync function |
| `render_eq_curve_image()` | `src/audio/effects/eq.rs` | Returns `slint::Image`; only called from `callbacks.rs` |
| `commit_drag()` | `src/app/drag.rs` | Only called from `callbacks.rs`; `DragState`/`AutomationDragState` structs kept |

### 4. Structural Refactors

**`src/app/mod.rs`** — `HdawApp` struct:
- Remove fields: `window: MainWindow`, `playhead_timer: SlintTimer`
- Remove module declarations: `pub(crate) mod callbacks;`, `pub(crate) mod input;`
- Remove imports: `slint::*`, `crate::ui::*`
- Remove `#[cfg(not(feature = "qt"))]` blocks that reference deleted Slint functions
- Simplify `new()`: no Slint window creation, no callback registration, no `sync_timeline()`/`sync_pool()`
- Simplify `run()`:
  - Qt mode: block main thread while Qt event loop runs
  - Headless: no-op

**`src/app/drag.rs`**:
- Remove `use crate::ui::timeline::{pixels_to_samples, sync_project_to_timeline_with_waveforms}`
- Remove entire `commit_drag()` function
- Keep `DragState`, `DragEdge`, `AutomationDragState` struct definitions (used by `src/ui_qt/timeline.rs`)

**`src/main.rs`**:
- Remove `mod ui;`
- `mod ui_qt;` stays gated with `#[cfg(feature = "qt")]`

### 5. No Changes To

- `src/ui_qt/` — all bridge files remain unmodified
- `src/ui_qt/main.qml` — remains the only UI entry point
- Audio engine, project, undo, playback, pool — all pure Rust logic, no Slint dependency

## Architecture After Stage 5

```
main.rs:
  └─ HdawApp::new()
       ├─ Creates Project (with test data)
       ├─ Creates AudioEngine (starts cpal)
       └─ Creates PlaybackManager, UndoStack, etc.

  └─ Qt thread:
       ├─ AppState::init() — shares Arcs with HdawApp
       ├─ QQmlApplicationEngine loads main.qml
       └─ QGuiApplication::exec()

  └─ HdawApp::run()
       └─ std::thread::park() — blocks main thread
```

The 16ms timer in `main.qml` handles all real-time sync (playhead, peaks, GR, state, auto-scroll).
The 300ms timer handles bridge refreshes (pool, timeline, mixer).
QML `Shortcut` elements handle all keyboard shortcuts.

## Risks

| Risk | Mitigation |
|------|------------|
| Missed Slint reference in non-obvious file | Grep for `slint`, `MainWindow`, `ComponentHandle`, `ModelRc` across entire src/ |
| `--no-default-features` fails | Acceptable — headless builds may need minor fixes; `cargo build` (with qt default) is the only supported path |
| Qt SDK not available for testing | Build compiles; runtime test requires Qt SDK |
