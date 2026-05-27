# Qt QML Migration Design

**Date:** 2026-05-26
**Status:** Draft
**Goal:** Replace Slint UI entirely with Qt QML, starting by fixing existing Qt transport bar gaps, then incrementally migrating each view.

---

## Architecture

### Shared State

A single `AppState` struct behind `Arc<Mutex<>>` and a `OnceLock` static, shared between the Slint main thread and the Qt thread:

```rust
pub struct AppState {
    pub project: Arc<Mutex<Project>>,
    pub playback: PlaybackManager,
    pub undo_stack: Arc<Mutex<UndoStack>>,
    pub waveform_peaks: Arc<Mutex<HashMap<String, WaveformPeaks>>>,
    pub pool_visible: Arc<AtomicBool>,
}
```

Initialized in `main.rs` before spawning the Qt thread. The existing `OnceLock<PlaybackManager>` is replaced with `OnceLock<AppState>`.

### Thread Model

- **Main thread:** Slint event loop + 60fps timer (unchanged)
- **Qt thread:** QML engine + event loop (separate thread, spawned after `HdawApp::new()`)
- **Audio thread:** cpal callback (unchanged, high-priority real-time)

Communication: `Arc<Mutex<>>` shared state. Qt reads state via a 60fps QML `Timer` that polls properties from the Rust QObject bridge.

### QML View Strategy

Each major view is a separate QML file with its own CXX-Qt bridge QObject. During migration, new Qt views run as floating `ApplicationWindow` panels alongside the Slint main window. When a view reaches parity, the Slint version is disabled by feature flag.

---

## Phase 0: Fix Existing Qt Transport Bar

### 0a — Shared AppState

Replace `OnceLock<PlaybackManager>` in `state.rs` with `OnceLock<AppState>` containing `project`, `playback`, `undo_stack`, `waveform_peaks`, and `pool_visible`.

**Files:**
- `src/ui_qt/state.rs` — new `AppState` struct, update `init()` and all accessors
- `src/main.rs` — pass shared state to Qt thread

### 0b — Fix `on_stop()`

Match Slint behavior in `callbacks.rs:233`:
1. `stop_recording()` capture recorded buffers
2. `set_playing(false)`
3. `set_position(0)`
4. Spawn tokio task to write WAVs to `recordings/` dir and add clips to project
5. Re-sync timeline

**Files:**
- `src/ui_qt/state.rs` — rewrite `on_stop()`

### 0c — Fix `on_toggle_record()`

Match Slint behavior in `callbacks.rs:283`:
1. Lock project, collect `armed_ids` from armed tracks
2. If none armed, return
3. Get current position, start playing if not already
4. `start_recording(&armed_ids, pos, sr)`

**Files:**
- `src/ui_qt/state.rs` — rewrite `on_toggle_record()`

### 0d — Fix `on_import_file()`

Match Slint behavior in `callbacks.rs:152`:
1. Open file dialog via `rfd::FileDialog`
2. Load WAV via `loader::load_wav()`
3. Generate waveform peaks
4. Add clip to first track
5. Push undo command
6. Reload clips + re-sync timeline

**Files:**
- `src/ui_qt/state.rs` — rewrite `on_import_file()`

### 0e — Bidirectional State Sync

Add a 60fps QML `Timer` in `transport.qml` that polls Rust-side properties:
- `playing` update play button highlight
- `recording` update record button highlight
- Project BPM update BPM label
- `pool_visible` update Pool button state

**Files:**
- `src/ui_qt/transport.qml` — add `Timer { interval: 16; running: true; repeat: true }`
- `src/ui_qt/mod.rs` — add getter methods reading from AppState

### 0f — Pool Button Wiring

Change Pool button to toggle `AppState.pool_visible` (AtomicBool). Slint 60fps timer reads this to show/hide pool panel.

**Files:**
- `src/ui_qt/mod.rs` — `toggle_pool()` invokable
- `src/app/mod.rs` — read `pool_visible` atomic in 60fps timer

### 0g — QML Embedding + Feature Fix

Embed transport.qml at compile time via `include_str!()` to eliminate `current_dir()` dependency. Add `"qt_core"` to cxx-qt-lib features.

**Files:**
- `src/main.rs` — use `set_data()` instead of `load(&url)`
- `Cargo.toml` — add `qt_core` feature

---

## Phase 1: Pool Panel (Qt)

**Files:**
- `src/ui_qt/pool.rs` — CXX-Qt bridge for PoolModel QObject
- `src/ui_qt/pool.qml` — `ApplicationWindow` with `ListView`
- `src/ui_qt/mod.rs` — register PoolModel

**Behavior:**
- Lists imported audio files (name, sample rate, channels, bit depth, duration, usage)
- Data pushed from shared state, refreshed on project changes
- Floating window, toggleable from transport toolbar

---

## Phase 2: Effect Editor (Qt)

**Files:**
- `src/ui_qt/effects.rs` — CXX-Qt bridge for EffectEditor QObject
- `src/ui_qt/effects.qml` — effect editor UI

**Behavior:**
- Shows effect chain for selected track/bus (ID from shared state)
- Effect type name + per-parameter knobs/sliders
- Param changes via `PlaybackManager::update_effect_param()`
- Compressor GR meter from 60fps polling

---

## Phase 3: Mixer (Qt)

**Files:**
- `src/ui_qt/mixer.rs` — CXX-Qt bridge
- `src/ui_qt/mixer.qml` — channel strips panel

**Behavior:**
- One strip per track + per bus + master
- Volume fader, pan knob, mute/solo/arm buttons
- Peak meters at 60fps from `get_track_peaks()`
- Send routing (add/remove, level, pre/post)
- Floating window

---

## Phase 4: Timeline + Track Headers (Qt)

**Files:**
- `src/ui_qt/timeline.rs` — CXX-Qt bridge
- `src/ui_qt/timeline.qml` — timeline canvas
- `src/ui_qt/track_header.qml` — track header component

**Subsystems:**
- Clip rendering with waveform images
- Automation lane curves
- Ruler with time ticks
- Playhead
- Scroll + zoom
- Click-select, drag-move, edge trim, split tool
- Automation point add/drag
- Mute/solo/arm/volume/pan per track
- Scroll sync between headers and timeline
- Shared `selected_clips` set with Slint during transition

---

## Phase 5: Unified Qt Main Window

**Files:**
- `src/ui_qt/main.qml` — top-level window composing all views
- `src/ui_qt/main_window.rs` — bridge for unified window
- `src/ui_qt/menubar.qml` — menu bar

**Behavior:**
- Desktop QML window with menu bar, toolbar, dockable panels
- Timeline + track headers as central widget
- Mixer as right dock
- Pool as left/toggle panel
- Effect editor as bottom panel or popup
- Transport in toolbar area (migrate from floating window)
- Window state persistence

### Cleanup
- Remove `slint` dependency from Cargo.toml
- Remove `src/ui/` directory
- Remove Slint code from `src/app/`
- Make Qt the default build (remove `qt` feature flag)

---

## Non-Goals
- MIDI sequencing / piano roll
- Plugin hosting UI (CLAP/VST)
- Lock-free audio parameters (separate effort)
