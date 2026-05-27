# Phase 5: Unified Qt Main Window — Design Spec

## Goal

Replace the Slint main window entirely with a single QML `ApplicationWindow` that integrates all panels (timeline, mixer, effect editor, pool, transport) into one traditional DAW layout. Remove the `slint` crate dependency.

---

## Architecture

### Before (hybrid)
```
Main thread:       Slint main window (all panels + 60fps timer)
Separate thread:   Qt event loop (5 floating ApplicationWindows)
                   ↘ AppState (OnceLock) shared via Arc<Mutex>
```

### After (Qt-only)
```
Main thread:       Qt event loop (single root ApplicationWindow)
                   ↘ AppState (OnceLock) — all panels share state
                   ↘ Timer QML objects for 60fps/300ms intervals
```

### Window Layout

```
+------------------------------------------------------------+
| MenuBar (File / Edit / Track / Transport / View)           |
+------------------------------------------------------------+
| Transport toolbar: ▶ ■ ● | ← → | 120BPM | Undo/Redo | ... |
+----------+-----------------------------------+-------------+
| Pool     | Timeline (Flickable)              | FX Editor   |
| (toggle) | ├─ Ruler with ticks + playhead  | (toggle)     |
| 280px    | ├─ Track lanes, clips, waveforms | 280px        |
|          | ├─ Drag overlays (move/resize)   |             |
|          | ├─ Automation lane curves        |             |
|          | └─ Zoom +/-, scrollbars         |             |
+----------+-----------------------------------+-------------+
| Mixer: [Track 1] [Track 2] [Bus 1] [Master] (horizontal)   |
|        L/R peak bars, vol fader, mute/solo/arm, fx slots    |
+------------------------------------------------------------+
```

Layout uses `ColumnLayout` (vertical) with `RowLayout` (horizontal) for the middle band. Pool and FX Editor are visibility-toggled via buttons. Timeline fills remaining space.

---

## Migration Stages

### Stage 1 — Root Window Shell + Existing Panels

**Files to create/modify:**
- **NEW** `src/ui_qt/main.qml` — single root `ApplicationWindow` with layout
- **MODIFY** `src/main.rs` — load `main.qml` instead of `transport.qml`
- **MODIFY** `src/ui_qt/state.rs` — add `selected_track_id`, `selected_bus_id`, `snap_enabled`, `tool_mode` to `AppState`
- **KEEP** existing 5 bridge modules + state code (unmodified)

**What's in main.qml:**
- `ApplicationWindow { id: mainWindow }` — root window, `visible: true`, default size 1280x720
- `MenuBar` with `Menu { title: "File" }` etc. (empty menus for now, just titles)
- Transport toolbar row reusing existing `TransportBar` bridge
- Pool panel (left) reusing existing `PoolModel` bridge
- Timeline (center) reusing existing `TimelineModel` bridge
- Mixer (bottom) reusing existing `MixerModel` bridge
- FX Editor (right) reusing existing `EffectEditor` bridge
- All toggle buttons (Pool/FX/Mix) toggle panel visibility, not window visibility
- 60fps `Timer` for playhead sync; 300ms `Timer` for mixer/timeline/pool refresh
- Remove old `transport.qml` (no longer needed)

**Stationary Slint window:** Slint window still runs in parallel (`app.run()`), with its own timer loop. Both UIs display simultaneously. Qt panels hide their standalone title bars. This lets us verify each stage without breaking existing functionality.

**Ruler ticks:** Compute in `TimelineModel.refresh()` (300ms) and when zoom changes, NOT in the 60fps `syncPlayhead()` — ruler ticks are static between zoom/scroll operations. The 300ms refresh is sufficient.

**Verification:** `cargo build --features qt` compiles, launches single large window with all panels laid out. Toolbar buttons work (play/stop/rec/import). Pool/FX/Mix toggle visibility. Timeline shows tracks and clips. Mixer shows strips.

---

### Stage 2 — Timeline Rendering

**Current state:** Timeline clips are colored rectangles without waveforms, automation curves, fade handles, or edge resize indicators. The timeline bridge (`timeline.rs`) already builds clip JSON with position and size.

**Work:**
1. **Waveform rendering:** `render_waveform_image()` in `src/utils/waveform.rs` currently produces a `slint::Image` from `SharedPixelBuffer`. Port to render a `QImage` (via `image` crate → RGBA bytes → `QImage` constructor). Convert to base64 PNG data URL for QML `Image.source`.
   - Add a helper: `pub fn render_waveform_qimage(peaks: &WaveformPeaks, width: i32, height: i32) -> Vec<u8>` returning PNG bytes.
   - In `timeline.rs` `build_timeline_json()`: for each clip, call `render_waveform_qimage()`, base64-encode, set as `"waveform_url"` in JSON.
   - In QML: `Image { source: model.waveform_url; fillMode: Image.Stretch }` inside clip delegate.
   - **Cache:** Waveform images are regenerated only when clip dimensions change (same as current Slint approach — cached in `wavform_peaks` HashMap).
   - This stage can be deferred to later if it causes performance issues — colored rectangles are functional.

2. **Automation lane rendering:** Port `render_automation_image()` from `timeline.rs` to produce a QImage PNG data URL for the selected automation lane. Overlay on clip as a semi-transparent `Image` element.
   - Add to `build_timeline_json()`: `"automation_url"` and `"automation_visible"` fields per track.

3. **Fade handles:** Add `fade_in` and `fade_out` properties to clip JSON representing fade duration in pixels. Render small triangular overlays on clip left/right edges in QML.
   - QML: `Rectangle` overlaid at clip x (fade_in width) or right edge (fade_out width), with `opacity: 0.5`, `color: "#88ccff"`.

4. **Edge resize indicators:** When hovering near clip edges, show a small grab handle area. This is driven by cursor type polling.

**Impact on existing code:** `build_timeline_json()` grows waveform/automation rendering. QML `Rectangle` delegate for clips gains `Image` children. No changes to bridge API.

---

### Stage 3 — Drag-and-Drop

**The hard part.** Current implementation: `drag.rs` state machine + `callbacks.rs` for press/move/release → live preview via Slint model mutation → commit on release.

**New approach:** overlay-based drag (no model mutation during drag):

1. **Bridge additions** (`TimelineModel` in `timeline.rs`):
   - New qproperties: `drag_active: bool`, `drag_overlay_x: f64`, `drag_overlay_y: f64`, `drag_overlay_w: f64`, `drag_overlay_h: f64`, `drag_overlay_color: QString`
   - New qinvokables: `onTimelinePressed(x: f64, y: f64)`, `onTimelineMoved(x: f64, y: f64)`, `onTimelineReleased()`
   - Internal state: re-use `DragState` and `AutomationDragState` from `drag.rs` (no Slint dependency — they're pure data structs)
   - On `onTimelinePressed()`:
     - Convert pixel coords → sample positions using `pixels_per_second`
     - Hit-test clips (iterate track clips from project)
     - Determine operation: move, resize-left, resize-right, stretch-left, stretch-right, fade-in, fade-out, automation-move
     - Store `DragState` in bridge struct field
     - Set `drag_active = true`
   - On `onTimelineMoved()`:
     - Update `DragState` with new position
     - Compute new overlay position/size → set `drag_overlay_x/y/w/h`
     - For automation: update `drag_point_time` / `drag_point_value` → set overlay (single point highlight)
   - On `onTimelineReleased()`:
     - Lock project, commit mutation (move/resize/stretch/fade/etc.)
     - Push undo command
     - Update PlaybackManager
     - Set `drag_active = false`
     - Call `refresh()` to rebuild timeline JSON

2. **QML additions:**
   - `MouseArea` overlaying the timeline content area
   - Cursor shape binding: `cursorShape: tl.cursorType`
   - Drag overlay `Rectangle`:
     ```qml
     Rectangle {
         visible: tl.dragActive
         x: tl.dragOverlayX - tlFlick.contentX
         y: tl.dragOverlayY
         width: tl.dragOverlayW
         height: tl.dragOverlayH
         color: tl.dragOverlayColor
         opacity: 0.4
         border.color: "#ffffff"
         border.width: 1
     }
     ```
   - Playhead placement on empty-space click without drag

3. **Key design decisions:**
   - **No model rebuild during drag** — overlay Rectangle updates via qproperty bindings (~16ms latency, fine for 60fps)
   - **Snap support:** snaps to grid/bar/beat during drag. Add `snap_enabled` boolean, `snap_mode` string to bridge qproperties
   - **Cross-track drag:** `drag_overlay_y` changes when clip crosses track boundary. On release, move clip to new track
   - **Undo:** single undo command per drag operation (not per frame)
   - **Automation point drag:** separate overlay (small circle) positioned at point time/value

---

### Stage 4 — Keyboard Shortcuts + 60fps Loop

**Current state:** Keyboard shortcuts polled via Win32 `GetAsyncKeyState` in the Slint 60fps timer. QML panels rely on Slint timer for playhead/peaks.

**Work:**

1. **Keyboard shortcuts** → QML `Shortcut` elements:
   ```qml
   Shortcut { sequence: "Space"; onActivated: transport.play() }
   Shortcut { sequence: "Ctrl+Z"; onActivated: state.undo() }
   Shortcut { sequence: "Ctrl+Y"; onActivated: state.redo() }
   Shortcut { sequence: "Delete"; onActivated: state.deleteSelected() }
   Shortcut { sequence: "Ctrl+C"; onActivated: state.copyClips() }
   Shortcut { sequence: "Ctrl+V"; onActivated: state.pasteClips() }
   Shortcut { sequence: "Ctrl+A"; onActivated: state.selectAll() }
   Shortcut { sequence: "Ctrl+T"; onActivated: state.addTrack() }
   Shortcut { sequence: "Home"; onActivated: state.goToStart() }
   Shortcut { sequence: "End"; onActivated: state.goToEnd() }
   Shortcut { sequence: "L"; onActivated: state.toggleLoop() }
   Shortcut { sequence: "M"; onActivated: state.toggleMute() }
   Shortcut { sequence: "S"; onActivated: state.toggleToolMode() }
   Shortcut { sequence: "N"; onActivated: state.toggleSnap() }
   Shortcut { sequence: "+"; onActivated: tl.zoomIn() }
   Shortcut { sequence: "-"; onActivated: tl.zoomOut() }
   Shortcut { sequence: "Left"; onActivated: state.nudgeLeft() }
   Shortcut { sequence: "Right"; onActivated: state.nudgeRight() }
   Shortcut { sequence: "R"; onActivated: transport.toggleRecord() }
   Shortcut { sequence: "P"; onActivated: togglePool() }
   Shortcut { sequence: "Escape"; onActivated: state.setSelectTool() }
   ```
   Add qinvokables for all new actions to a `StateBridge` (new QObject) or distribute across existing bridges.

2. **60fps loop** → single QML `Timer` in root window:
   ```qml
   Timer {
       interval: 16
       running: true
       repeat: true
       onTriggered: {
           transport.syncState()         // is_playing, position
           tl.syncPlayhead()             // playhead_x
           mixer.syncPeaks()             // peak meter values
           fx.syncGr()                   // compressor GR
           state.syncAutoScroll()        // auto-scroll timeline
           state.syncCursorType()        // update cursor based on position
       }
   }
   ```
   - Add `StateBridge` (or use existing `TransportBar`) for cursor type, auto-scroll, selection state
   - Remove `src/app/input.rs` entirely (no more Win32 polling)
   - Auto-scroll logic ported from `app/mod.rs` 60fps timer: if playhead_x > visible_width * 0.8, scroll Flickable to keep playhead at 20% from right

3. **Selection management:**
   - Add `selected_clips: Arc<Mutex<HashSet<Uuid>>>` to `AppState` (move from `HdawApp`)
   - `StateBridge` methods: `selectClip(id)`, `selectAll()`, `clearSelection()`
   - QML clip MouseArea: `onClicked: stateBridge.selectClip(model.id)` + Ctrl for multi-select
   - Selection visually shown by clip border color change (from JSON `selected` field)

---

### Stage 5 — Remove Slint

**Work:**
1. **Remove Slint crate** from `Cargo.toml` `[dependencies]`
2. **Remove Slint-specific modules:**
   - `src/ui/` directory (entirely)
   - `src/app/` becomes pure logic (no Slint window)
3. **Refactor `src/main.rs`:**
   ```rust
   fn main() {
       // Init state, project, playback...
       let state = app::init();
       ui_qt::state::init(state);
       
       // Qt event loop on main thread
       let mut app = QGuiApplication::new();
       let mut engine = QQmlApplicationEngine::new();
       let qml_source = include_str!("ui_qt/main.qml");
       let qba = QByteArray::from(qml_source);
       engine.as_mut().unwrap().load_data(&qba, ...);
       app.as_mut().unwrap().exec();
   }
   ```
4. **Remove `#[cfg(feature = "qt")]`** gating — Qt becomes the only UI path
5. **Make cxx-qt deps non-optional** — move from `[features]` to `[dependencies]`
6. **Rename feature:** `qt` feature no longer needed; default build = Qt

**Files to delete:**
- `src/ui/` (entire directory: `main_window.rs`, `timeline.rs`, `mod.rs`)
- `src/app/callbacks.rs` (logic moved to bridge methods)
- `src/app/drag.rs` (logic moved to timeline bridge)
- `src/app/input.rs` (no-op without Win32)
- `src/ui_qt/transport.qml` (replaced by `main.qml`)

**Files to modify:**
- `src/main.rs` — Qt-only startup
- `src/app/mod.rs` — remove `HdawApp` struct, just init project/playback
- `Cargo.toml` — remove `slint` dep, make cxx-qt non-optional
- `build.rs` — remove `cfg(feature = "qt")` conditions, always build all bridges

**Verification:** `cargo build` succeeds without `--features qt`. `cargo run` launches a single unified window. All functionality works (playback, recording, editing, effects, mixer, pool, timeline drag-and-drop).

---

## Data Flow

```
User input (QML)
  → MouseArea/MouseEvent → qinvokable → Rust bridge
    → lock project, mutate state
    → push undo command
    → update PlaybackManager
    → drop mutex
    → set timeline_dirty flag OR set qproperty directly

60fps Timer (QML)
  → syncState(), syncPlayhead(), syncPeaks(), syncGr()
    → Rust bridge reads PlaybackState
    → updates qproperties (playhead_x, peaks_json, compressor_gr)
    → QML bindings react

300ms Timer (QML)
  → refresh mixer, timeline, pool
    → Rust bridge reads Project, builds JSON
    → QML updateTimeline()/buildStrips()/updatePool()
      → ListModel rebuild (compared IDs → skip if unchanged)

Keyboard shortcut (QML Shortcut)
  → qinvokable → same as user input path

After drag release (Rust bridge)
  → commit mutation to project
  → push undo
  → call refresh() → rebuild timeline JSON
```

---

## Bridge Changes Summary

**Note:** Any new bridge `.rs` file requires `pub mod` in `src/ui_qt/mod.rs` AND a `.file()` entry in `build.rs`.

### New: `StateBridge` (in `src/ui_qt/state_bridge.rs`)

| QProperty | Type | Purpose |
|-----------|------|---------|
| `snap_enabled` | bool | Snap to grid |
| `tool_mode` | i32 | 0=select, 1=cut |
| `cursor_type` | i32 | Qt cursor shape enum |
| `time_display` | QString | "01:23.456" |
| `bpm_display` | QString | "120.00" |
| `time_sig_display` | QString | "4/4" |
| `loop_enabled` | bool | Loop toggle state |
| `undo_available` | bool | Undo button state |
| `redo_available` | bool | Redo button state |

| QInvokable | Purpose |
|------------|---------|
| `undo()` | Undo last command |
| `redo()` | Redo last undone |
| `deleteSelectedClips()` | Delete selected clips |
| `copyClips()` / `pasteClips()` | Clipboard operations |
| `selectAll()` | Select all clips |
| `addTrack()` | Add audio track |
| `addBus()`, `deleteBus()` | Bus management |
| `goToStart()` / `goToEnd()` | Transport position |
| `toggleLoop()` | Toggle loop mode |
| `toggleToolMode()` | Switch select/cut |
| `toggleSnap()` | Toggle snap |
| `nudgeLeft()` / `nudgeRight()` | Nudge playhead |
| `setSelectTool()` | Set to select mode |
| `syncAutoScroll()` | Auto-scroll timeline if playing |
| `syncCursorType()` | Update cursor from mouse position |

### Modified: `TimelineModel` (in `timeline.rs`)

**New qproperties:**
- `drag_active: bool`, `drag_overlay_x/f64`, `drag_overlay_y/f64`, `drag_overlay_w/f64`, `drag_overlay_h/f64`, `drag_overlay_color: QString`

**New qinvokables:**
- `onTimelinePressed(x: f64, y: f64)`
- `onTimelineMoved(x: f64, y: f64)`
- `onTimelineReleased()`
- `selectClip(id: &str)` / `clearSelection()`

### Existing bridges: mostly unchanged
- `TransportBar` — add `syncState()` now returns time_display, bpm_display too
- `MixerModel` — unchanged (already has set_volume/pan/mute/solo/arm/select_effect)
- `PoolModel` — unchanged
- `EffectEditor` — unchanged

---

## Error Handling

- **Bridge methods lock project/playback:** if lock fails (poisoned), log error and no-op. QML state stays stale until next refresh cycle.
- **Drag operations:** if project is locked when `onTimelineReleased()` fires, stash the operation in a pending queue and retry next 60fps tick.
- **File import failures:** `on_import_file()` already returns nothing; errors are logged. QML button click is fire-and-forget.
- **Waveform rendering errors:** if a WAV file can't be loaded or decoded, the clip shows an empty colored rectangle (no crash).

---

## Testing & Verification

1. **Build:** `cargo build` (non-qt) should fail — Qt is now required. `cargo build --features qt` is no longer needed.
2. **Manual checks per stage:**
   - Stage 1: Window opens, all panels visible, transport buttons work, pool shows entries, mixer shows strips, timeline shows clips
   - Stage 2: Waveforms visible in clips, automation curves visible, fade handles render
   - Stage 3: Click + drag clips to move, resize edges, cross-track move, fade in/out via edge drag, automation point drag, undo/redo of all
   - Stage 4: All keyboard shortcuts work, 60fps loop smooth, auto-scroll during playback, cursor changes on hover
   - Stage 5: No Slint in build dependencies, single window, all features work
3. **Edge cases:** Empty project (no tracks, no clips), recording + drag simultaneously, undo during playback, very large projects (100+ tracks)

---

## Open Questions (Deferred)

1. **Waveform image rendering performance:** If `build_timeline_json()` with waveform URLs becomes slow, defer waveform rendering to a background thread or lazy-load per clip.
2. **Mid-mouse panning:** Can be added later as a `MouseArea.onWheel` + middle-button handler.
3. **Rubber-band selection:** Current Slint doesn't have it; can be implemented as a second overlay rectangle during drag with Ctrl held.
4. **Ruler drag to set loop region:** Future enhancement.
