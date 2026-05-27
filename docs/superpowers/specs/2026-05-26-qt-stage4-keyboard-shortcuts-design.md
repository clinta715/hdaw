# Stage 4: Qt Keyboard Shortcuts + 60fps Timer Port

## Objective

Replace the Win32 `GetAsyncKeyState` polling in the Slint 60fps timer with QML `Shortcut` elements wired to a dedicated Rust bridge, and port auto-scroll logic to the Qt 16ms timer.

## Motivation

- The existing shortcut system is Windows-only (`#[cfg(target_os = "windows")]`) and uses low-level Win32 polling.
- This blocks Stage 5 (full Slint removal) because the 60fps timer also holds all keyboard shortcut logic.
- QML `Shortcut` elements are cross-platform, declarative, and idiomatic.

## Architecture

```
QML Shortcut (key sequence)
    → qinvokable on ShortcutHandler QObject
        → lock AppState fields + project
            → mutate state, push undo, refresh PlaybackManager
        → drop locks
        → call bridge .refresh() (triggers QML model rebuild)
```

### New AppState Fields

Added to `src/ui_qt/state.rs` in the `AppState` struct:

| Field | Type | Default | Purpose |
|-------|------|---------|---------|
| `selected_clips` | `Arc<Mutex<HashSet<Uuid>>>` | empty | Clip selection for delete/copy/paste/select-all |
| `clipboard` | `Arc<Mutex<Vec<AudioClip>>>` | empty | Copy buffer |
| `selected_track_id` | `Arc<Mutex<Option<Uuid>>>` | None | Track header selection / mute target |
| `tool_mode` | `Arc<Mutex<i32>>` | 0 | 0=select, 1=split |
| `snap_enabled` | `Arc<AtomicBool>` | true | Snap-to-grid toggle |
| `auto_scroll` | `Arc<AtomicBool>` | true | Auto-follow playhead during playback (read by QML) |

All fields are initialized in `state::init()`.

### ShortcutHandler Bridge

New file: `src/ui_qt/shortcut_handler.rs`

**CXX-Qt bridge module** `#[cxx_qt::bridge]`:
- Extern "RustQt" block with `ShortcutHandler` QObject
- 13 qinvokables (remaining shortcuts route directly to existing bridges in QML)

**QProperties:**
- `tool_mode: i32` — read by QML for cursor/UI state
- `snap_enabled: bool`
- `can_undo: bool`
- `can_redo: bool`

**QInvokables (only actions that have no existing bridge target):**

| QInvokable | Logic |
|-----------|-------|
| `go_to_start()` | Set playback position to 0, update PlaybackManager |
| `go_to_end()` | Lock project, scan all tracks for max clip end, set position |
| `nudge_left()` | Subtract `sample_rate/10` from position, clamp ≥ 0 |
| `nudge_right()` | Add `sample_rate/10` to position |
| `delete_selected()` | Lock project, collect `selected_clips`, remove from tracks, push `EditCommand::DeleteClips`, reload playback clips |
| `copy_selected()` | Lock project, iterate tracks, clone clips matching `selected_clips` into `clipboard` |
| `paste()` | Lock project, read `clipboard` + playhead position, clone with new UUIDs, add to selected track (or first track), push `EditCommand::AddClip` per clip |
| `select_all()` | Lock project, fill `selected_clips` with every clip ID from every track |
| `add_track()` | Lock project, create `Track::new()`, push `EditCommand::AddTrack`, reload playback |
| `toggle_mute_selected_track()` | Lock project, find track by `selected_track_id`, toggle mute, update PlaybackManager params |
| `toggle_snap()` | Flip `snap_enabled` atomic |
| `toggle_tool_mode()` | Toggle `tool_mode` between 0 and 1 |
| `reset_tool_mode()` | Set `tool_mode` = 0 |
| `sync_state()` | Read AppState → update qproperties (tool_mode, snap, can_undo, can_redo) |

### QML Wiring

In `main.qml`, add inside the root `ApplicationWindow`:

```qml
property var sc: shortcutHandler

// === Route to existing bridge qinvokables ===
Shortcut { sequence: "Space";     onActivated: transport.togglePlayStop() }
Shortcut { sequence: "Ctrl+Z";    onActivated: state.undo() }
Shortcut { sequence: "Ctrl+Y";    onActivated: state.redo() }
Shortcut { sequence: "L";         onActivated: state.toggleLoop() }
Shortcut { sequence: "=";         onActivated: tl.zoomIn() }
Shortcut { sequence: "-";         onActivated: tl.zoomOut() }
Shortcut { sequence: "P";         onActivated: transport.togglePool() }

// === Route to ShortcutHandler for app-level actions ===
Shortcut { sequence: "Ctrl+T";    onActivated: sc.addTrack() }
Shortcut { sequence: "Delete";    onActivated: sc.deleteSelected() }
Shortcut { sequence: "Ctrl+C";    onActivated: sc.copySelected() }
Shortcut { sequence: "Ctrl+V";    onActivated: sc.paste() }
Shortcut { sequence: "Ctrl+A";    onActivated: sc.selectAll() }
Shortcut { sequence: "M";         onActivated: sc.toggleMuteSelectedTrack() }
Shortcut { sequence: "Home";      onActivated: sc.goToStart() }
Shortcut { sequence: "End";       onActivated: sc.goToEnd() }
Shortcut { sequence: "S";         onActivated: sc.toggleToolMode() }
Shortcut { sequence: "N";         onActivated: sc.toggleSnap() }
Shortcut { sequence: "Escape";    onActivated: sc.resetToolMode() }
Shortcut { sequence: "Left";      onActivated: sc.nudgeLeft() }
Shortcut { sequence: "Right";     onActivated: sc.nudgeRight() }
```

TransportBar also gains a `toggle_play_stop()` qinvokable for the Space shortcut.

The 16ms timer gains auto-scroll logic:

```qml
Timer {
    interval: 16
    running: true
    repeat: true
    onTriggered: {
        transport.syncState()
        tl.timelineScrollX = tlFlick.contentX
        tl.syncPlayhead()
        mixer.syncPeaks()
        fx.syncGr()
        state.syncState()
        sc.syncState()

        // auto-scroll during playback
        if (transport.playing) {
            var vw = tlFlick.width
            var sx = tlFlick.contentX
            var px = tl.playheadX
            if (px > sx + vw * 0.8) {
                tlFlick.contentX = px - vw * 0.5
            }
        }
    }
}
```

### Registration

- `mod.rs`: add `pub mod shortcut_handler;`
- `build.rs`: add `.file("src/ui_qt/shortcut_handler.rs")`
- `main.rs`: create `ShortcutHandler`, set as context property

### Migration / Hybrid Mode

The Slint 60fps timer continues to run in hybrid mode. To avoid double-firing keyboard actions:
- The Slint timer's keyboard polling block should be gated with `#[cfg(not(feature = "qt"))]`
- The Slint timer's cursor-type computation (Alt-key-based) should be gated similarly
- Auto-scroll in the Slint timer should be gated with `#[cfg(not(feature = "qt"))]`

This ensures:
- **`cargo build` (no qt)**: Slint timer handles everything as before
- **`cargo build --features qt`**: Qt handles shortcuts + auto-scroll; Slint timer handles only Slint-specific sync

When Stage 5 removes Slint entirely, the Slint timer is deleted.

## File Changes

| File | Change |
|------|--------|
| `src/ui_qt/state.rs` | Add 6 new fields to `AppState`, update `init()` |
| `src/ui_qt/shortcut_handler.rs` | **NEW** — CXX-Qt bridge, ShortcutHandler QObject, 20 qinvokables |
| `src/ui_qt/mod.rs` | Add `pub mod shortcut_handler`; add `toggle_play_stop()` to TransportBar |
| `build.rs` | Add `.file("src/ui_qt/shortcut_handler.rs")` between existing `.file()` entries |
| `src/ui_qt/main.qml` | Add `Shortcut` elements, auto-scroll in 16ms timer, `syncState()` call |
| `src/main.rs` | Create ShortcutHandler, set context property |
| `src/app/mod.rs` | Gate keyboard polling + auto-scroll with `#[cfg(not(feature = "qt"))]` |

## Thread Safety

- All `AppState` fields are `Arc<Mutex<>>` or `Arc<AtomicBool>` — safe for cross-thread access.
- The Slint 60fps timer and Qt event loop run on different threads, but no lock is held across an await point or callback boundary, so there is no deadlock risk.
- `PlaybackManager` uses internal `Arc<Mutex<PlaybackState>>` — already thread-safe.

## Risks

| Risk | Mitigation |
|------|------------|
| Double-firing keyboard actions in hybrid mode | Gate Slint timer shortcuts with `#[cfg(not(feature = "qt"))]` |
| Thread contention on project lock | Project lock held briefly per action; no nested locks across threads |
| QML Shortcut context priority | Use `Shortcut { }` at ApplicationWindow level; context defaults to `Qt.WindowShortcut` |
| Qt 0.8 bridge compat with 20 qinvokables | All follow existing pattern; no exotic types |

## Success Criteria

- All 18 existing keyboard shortcuts work in Qt mode
- Auto-scroll follows playhead during playback
- No regressions in Slint-only mode (`cargo build`)
- No `GetAsyncKeyState` calls in Qt mode
