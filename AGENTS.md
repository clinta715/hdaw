# AGENTS.md — HDAW Development Guide

## Build & Run

```powershell
# Build (takes ~15s incremental, ~90s from clean)
cargo build

# Run
cargo run

# Check lint/type errors only
cargo build 2>&1 | Select-String "^error"
```

No test suite yet. Visual verification is manual — run the app and interact.

### Qt Feature Build (optional)

```powershell
# Prerequisite: Install Qt 6.7+ SDK (tested with 6.11.1)
# Select MSVC 2022 64-bit, Qt Quick Controls + Layouts
$env:QMAKE = "C:\Qt\6.11.1\msvc2022_64\bin\qmake.exe"

# Build with hybrid Qt+Slint transport bar
cargo build --features qt

# Run
cargo run --features qt
```

The `qt` feature is gated — default `cargo build` is Slint-only. With `--features qt`, a floating Qt transport toolbar spawns alongside the Slint main window.

---

## Architecture Notes

### Audio Thread Safety (critical)
The cpal audio callback runs on a high-priority real-time thread. Rules:
- **No heap allocation** in `fill_buffer()`
- **No mutex lock** in the audio callback — `PlaybackState` is locked once at the top of `fill_buffer` and released at the bottom
- Params are communicated via `Arc<Mutex<PlaybackState>>` — UI thread writes params between callbacks

### Slint Macro Quirks
- All UI is in the `slint::slint!{}` macro in `src/ui/main_window.rs` — no separate `.slint` files
- **No `ScrollView`** element in Slint 1.16 — use `Rectangle { clip: true; }` instead
- **No `format()`** function in Slint 1.16 for float formatting — use simple text displays
- **`horizontal-stretch`** takes a unitless int, NOT a length (use `1` not `1px`). Use `horizontal-stretch: 0;` with a fixed `width` to prevent widget from stretching in a HorizontalLayout.
- **`min-width`** alone does NOT prevent stretching in HorizontalLayout — use `width: Npx; horizontal-stretch: 0;`
- **`border-radius`** only takes a single value in Slint 1.16 (e.g., `border-radius: 3px;`). Four-value form (`0px 0px 3px 3px`) is NOT supported. Individual corner overrides (`border-top-left-radius`) exist but may cause layout issues.
- **TouchArea `pointer-event`** fires on down, up, move, and cancel — check `ta-xxx.pressed` for drag
- **`Math.max`/`Math.min`/`Math.sqrt`** are available but `Math.log` may not be
- **Property types**: `in-out property <float>`, `<int>`, `<length>`, `<bool>`, `<string>`, `<brush>`, `<image>`
- **Struct exports**: `export struct Foo { field: type, }` — must be before the component
- **Model arrays**: Set from Rust with `slint::ModelRc::new(slint::VecModel::from(vec))`

### Callback Pattern (in app.rs)
Every UI callback follows the same pattern:
```rust
let p_foo = project.clone();
let pb_foo = playback.clone();
let us_foo = undo_stack.clone();
let wp_foo = waveform_peaks.clone();
let ww_foo = window_weak.clone();
self.window.on_xxx(move |params| {
    // 1. Parse string IDs to UUIDs
    // 2. Lock project, mutate state
    // 3. Push undo command
    // 4. Update PlaybackManager
    // 5. Drop mutex
    // 6. Re-sync UI via sync_project_to_timeline_with_waveforms
});
```

### Keyboard Shortcuts
Windows-only via `GetAsyncKeyState` polling in the 60fps timer. Key state tracked per-key with edge detection (`was_pressed` only fires on rising edge). Non-Windows platforms get a no-op stub.

---

## Data Flow

```
User Input (Slint TouchArea)
    → callback → app.rs (lock project, push undo, update playback)
        → PlaybackManager::update_*()
            → PlaybackState (Arc<Mutex>)
                → cpal audio callback → fill_buffer() → output

Qt QML Input
    → QObject qinvokable → state.rs callbacks or bridge methods
        → project.lock() + undo + PlaybackManager
        → timeline_dirty flag → picked up by Slint 60fps timer

UI Sync: app.rs re-reads project + calls sync_project_to_timeline_with_waveforms()
    → updates all Slint models (clips, tracks, buses, ruler ticks, etc.)

Timer (60fps Slint): reads position, peaks, GR from PlaybackManager
    → updates playhead-x, peak bars, compressor-gr

Timer (60fps Qt): reads compressor GR via EffectEditor::sync_gr()
    → updates compressor_gr qproperty → GR meter bar width
```

---

## Common Pitfalls

1. **Borrow conflicts in callbacks**: Clone before mutation then drop. Never hold `project.lock()` while calling `sync_project_to_timeline_with_waveforms()` which also locks project.
2. **Missing `pub mod` declarations**: Every new file in `src/` needs a corresponding `pub mod` in the parent `mod.rs`.
3. **TrackInfo/BusInfo/ClipInfo field changes**: Must update BOTH the struct definition in `main_window.rs` AND the construction in `timeline.rs` sync function AND any hardcoded constructors (test data in `mod.rs`).
4. **Undo command additions** require changes in 3 places: enum variant, `execute_command` match, `undo_command` match, and optionally `coalesce_key`.
5. **Main window filename**: The `slint!` macro embedded an image path `@image-url("")` — this MUST match what Slint generates from the filename. If the file is renamed, update the component name `MainWindow` accordingly.
6. **`fill_buffer()` brace structure**: There are TWO separate `for track in &tracks` loops (effects processing then mixing) and TWO `for bus in &buses` loops. When editing either block, use full-context oldString to avoid matching the wrong one and creating orphaned braces.
7. **Automation lane parameter names**: Lane's `parameter_id` is a globally unique string (e.g., `"volume"`, `"eq_low_gain"`). Effect param automation is matched to effects at playback time via `effect.get_parameter(name)` — no effect index needed.
8. **Qt build missing `.file()`**: CXX-Qt 0.8 requires explicit source file paths in `build.rs` via `.file("src/ui_qt/mod.rs")`. Without this, no C++ code is generated for bridge modules, causing linker errors (`unresolved external symbol ...`).
9. **CXX-Qt bridge syntax (0.8)**: Use `extern "RustQt"` block for QObject + qinvokable declarations. Method implementations go OUTSIDE the bridge module: `impl transport_object::TransportBar { ... }`. `use` items go outside the bridge; `unsafe extern "C++"` goes inside for type declarations.
10. **`QString` qproperty issues**: Declaring `type QString = cxx_qt_lib::QString;` inside a bridge causes CXX ExternType::Id mismatch. Avoid QString in qproperties for now — use primitive types (`bool`, `i32`, `f64`).
11. **Qt app lifecycle**: `QGuiApplication::new()` takes no args (auto-collects). Event loop runs via `app.exec()` not `cxx_qt::exec()`. QML is loaded via `engine.load(&QUrl::from("qrc:/qt/qml/com/hdaw/src/ui_qt/main.qml"))` — uses Qt resource system (qrc), NOT `load_data()` or `load_url()` which don't set up module resolution. See pitfall #15 for the full module setup.
12. **QML slider + ListModel**: When using `Repeater { model: ListModel }` for parameter sliders, `onMoved` fires during user drag while `onValueChanged` fires for any value change (including programmatic). Use `onMoved` for drag-to-update. Do NOT call `refresh()` after `set_param` — that would clear+repopulate the model, recreating all delegates and resetting slider positions.
13. **Multiple ApplicationWindows in one QML file**: `QQmlApplicationEngine` loads a single root QML file. Additional windows MUST be `ApplicationWindow` instances declared inside that file (not separate files). QML properties from one window can reference objects in other windows by ID.
14. **JSON to QML pattern**: Dynamic models (pool entries, effect params) are serialized to JSON strings in Rust and `JSON.parse`d in QML to populate `ListModel` entries. Each bridge has an `on<Property>Changed` signal + `updateXxx()` JS function.
15. **QML module discovery (critical)**: CXX-Qt 0.8's `#[qml_element]` types are NOT automatically discoverable by the QML engine. Three things are required:
    - **`build.rs` MUST have `.qml_module()`** — Without it, `QML_NAMED_ELEMENT` compiles into C++ but no `.qmldir`/plugin is generated, and QML sees "module is not installed" errors.
    - **`main.rs` MUST use `engine.load(&QUrl::from("qrc:/qt/qml/..."))`** — `load_data()`/`load_url()` don't set up Qt's QML module resolution. The qrc URL pattern follows the QML file path: `qrc:/qt/qml/<uri_with_slashes>/<file_path>` (e.g., `qrc:/qt/qml/com/hdaw/src/ui_qt/main.qml`).
    - **`main.qml` imports MUST match the module URI** — `import com.hdaw 1.0` (not `import ui_qt.*`). All `#[qml_element]` types become available under this single import.
    - **`cxx-qt-build` MUST have `link_qt_object_files` feature** — Required for pure Cargo builds (no CMake) to statically link Qt object files.
    - Example `build.rs`:
      ```rust
      CxxQtBuilder::new_qml_module(
          QmlModule::new("com.hdaw").qml_file("src/ui_qt/main.qml"),
      )
      .qt_module("Quick")
      .files(["src/ui_qt/mod.rs", ...])
      .build();
      ```

---

## Automation Architecture

```
Data model:
  Track.automation: HashMap<String, AutomationLane>   (param_name → lane with points)
  Track.selected_automation_param: Option<String>     (which lane is visible/editable on timeline)
  AutomationLane: points: Vec<AutomationPoint>, enabled, read_enabled, write_enabled, color

Playback flow:
  load_project_clips() → copies track.automation → PlaybackState.track_automation
  fill_buffer() → each block:
    1. Process effects with automation:
       for each effect in track chain:
         for each automation lane: if effect.get_parameter(name).is_some():
           effect.set_parameter(name, lane.get_value_at(current_time))
         effect.process(buffer)
    2. Read volume/pan automation → override track.volume / track.pan
    3. Mix with automated volume/pan values

UI flow:
  sync_project_to_timeline_with_waveforms() → renders automation image for selected param only
  Click in lane area → add/move AutomationPoint for the selected param
  "+A" button in track strip → cycles: volume → pan → effect_params → (none) → volume...
  Point editing: click-to-add, drag-to-move, undo/redo supported
```

---

## Shared AppState Architecture (Qt Migration)

### State Overview (`src/ui_qt/state.rs`)

```rust
pub struct AppState {
    pub project: Arc<Mutex<Project>>,
    pub playback: PlaybackManager,
    pub undo_stack: Arc<Mutex<UndoStack>>,
    pub waveform_peaks: Arc<Mutex<HashMap<String, WaveformPeaks>>>,
    pub pool_visible: Arc<AtomicBool>,
    pub timeline_dirty: Arc<AtomicBool>,
    pub selected_effect_target: Arc<Mutex<Option<String>>>,   // UUID of track/bus
    pub selected_effect_index: Arc<Mutex<Option<i32>>>,       // index in effects_chain
    pub selected_effect_is_track: Arc<AtomicBool>,            // true=track, false=bus
}
```

Stored in a `OnceLock<AppState>` initialized by `state::init(state)` in `main.rs`. Access via `state::get()` which returns `Option<&'static AppState>`. All fields are thread-safe (`Arc<Mutex<>>` or `Arc<AtomicBool>`).

### Sync From Slint to Qt
Slint callbacks in `callbacks.rs` conditionally write to AppState with `#[cfg(feature = "qt")]`:
```rust
#[cfg(feature = "qt")]
if let Some(state) = crate::ui_qt::state::get() {
    if let Ok(mut target) = state.selected_effect_target.lock() { *target = Some(...); }
}
```

### Sync From Qt to Slint
Qt bridges set `timeline_dirty = true` or `pool_visible = true` atomics. The Slint 60fps timer in `app/mod.rs` reads these and syncs.

---

## Qt Migration (Phase 5 — Stage 1 Complete, QML Loading Fixed)

The Slint main window is being incrementally replaced with Qt QML. All panels (Transport, Pool, Effect Editor, Mixer, Timeline, State Bridge) now use a single integrated Qt window.

### Current State
- **Single unified Qt window**: All panels integrated in `main.qml` under a single `ApplicationWindow`
- **Feature-gated**: `cargo build` = Slint-only; `cargo build --features qt` = Qt
- **CXX-Qt 0.8.1** bridges defined in `src/ui_qt/mod.rs`, `src/ui_qt/pool.rs`, `src/ui_qt/effects.rs`, `src/ui_qt/mixer.rs`, `src/ui_qt/timeline.rs`, `src/ui_qt/state_bridge.rs`, `src/ui_qt/shortcut_handler.rs`
- **Shared state**: `OnceLock<AppState>` in `src/ui_qt/state.rs`
- **QML**: `src/ui_qt/main.qml` — single `ApplicationWindow` with integrated panels
- **Main entry**: `main.rs` spawns Qt event loop on a separate thread
- **QML module loading**: See pitfall #14 below for critical build.rs/main.rs/QML import pattern

### Key Files
- `src/ui_qt/mod.rs` — TransportBar QObject (play/stop/record/toggle-pool), module declarations
- `src/ui_qt/state.rs` — `AppState` struct, `init()`/`get()`, `on_play()`/`on_stop()`/`on_toggle_record()`/`on_import_file()`
- `src/ui_qt/state_bridge.rs` — StateBridge QObject (time_display, bpm_display, time_sig_display, undo/redo, loop toggle, sync_state)
- `src/ui_qt/pool.rs` — PoolModel QObject (pool_json qproperty, refresh, insert_pool_audio)
- `src/ui_qt/effects.rs` — EffectEditor QObject (effect_json + compressor_gr qproperties, refresh, set_param, toggle_bypass, sync_gr, auto-refresh on selection change)
- `src/ui_qt/mixer.rs` — MixerModel QObject (mixer_json + peaks_json qproperties, refresh, sync_peaks, set_volume, set_pan, toggle_mute, toggle_solo, toggle_arm, select_effect)
- `src/ui_qt/timeline.rs` — TimelineModel QObject (timeline_json + playhead_x + pixels_per_second qproperties, refresh, sync_playhead, zoom_in, zoom_out)
- `src/ui_qt/main.qml` — Single root ApplicationWindow with all panels integrated (replaces transport.qml)
- `src/app/callbacks.rs` — Slint-to-Qt effect selection sync (cfg-gated)
- `build.rs` — `.file(...)` entries for all 6 bridge modules

### Effect Editor Bridge Details

**QProperties:**
- `effect_json: QString` — JSON string with structure:
  ```json
  {
    "title": "Compressor",
    "bypassed": false,
    "effect_type": "Compressor",
    "idx": 1,
    "params": [{"name":"comp_threshold","label":"Threshold","value":-20.0,"min":-60.0,"max":0.0,"display":"-20.0dB"}],
    "chain": [{"name":"Equalizer","idx":0,"selected":false}]
  }
  ```
- `compressor_gr: f64` — raw gain reduction value, polled at 60fps

**QInvokables:**
- `refresh()` — reads AppState selection, builds JSON, sets `effect_json`
- `set_param(param_name, value)` — reads selection, locks project, updates EffectInstance + undo + PlaybackManager DSP
- `toggle_bypass()` — reads selection, toggles bypass in project + undo + PlaybackManager, refreshes JSON
- `sync_gr()` — reads selection, calls `PlaybackManager::get_compressor_gr()`, sets `compressor_gr`

**QML pattern for param sliders:**
```qml
Repeater {
    model: paramModel  // ListModel populated from JSON
    Slider {
        from: model.min; to: model.max; value: model.value
        onMoved: fx.setParam(model.name, value)  // NOT fx.refresh()
    }
}
```
`set_param` does NOT update the JSON — the slider position is the truth during drag. JSON only updates on `refresh()` (new selection) or `toggle_bypass()`.

### Mixer Panel Bridge Details

**QProperties:**
- `mixer_json: QString` — JSON array of strip objects (id, name, type, vol, pan, mut, sol, arm, out, fx[])
- `peaks_json: QString` — JSON object mapping strip IDs to `{"l":0.5,"r":0.6}` peak pairs

**QInvokables:**
- `refresh()` — reads AppState project, builds full `mixer_json` (tracks + buses + master)
- `sync_peaks()` — reads `PlaybackManager::get_track_peaks()`, `get_bus_peaks()`, `get_master_peak()`, sets `peaks_json`
- `set_volume(strip_id, value)` — locks project, updates track/bus/master volume + undo + PlaybackManager
- `set_pan(strip_id, value)` — same for pan
- `toggle_mute(strip_id)` — toggles mute in project + PlaybackManager
- `toggle_solo(strip_id)` — toggles solo in project + PlaybackManager
- `toggle_arm(strip_id)` — toggles record arm (tracks only)
- `select_effect(strip_id, effect_idx)` — writes to AppState selection atomics (opens effect editor)

**Split data strategy** (avoids 60fps delegate recreation):
1. `mixer_json` is updated by `refresh()` on a 300ms timer (picks up Slint-side changes) and on `Component.onCompleted`.
2. `peaks_json` is updated by `sync_peaks()` on a 16ms timer (60fps peak meters).
3. QML `updatePeaks()` updates peaks in-place via `item.peakL = value` — no delegate recreation.
4. QML `buildStrips()` compares strip IDs before clearing/repopulating the model. If same IDs exist, skips rebuild entirely.

**Auto-refresh on selection change (in effects.rs):**
The effect editor's `sync_gr()` method (called 60fps) detects selection changes by comparing current AppState selection against a `static LAST_SELECTION: Mutex<Option<(String, i32, bool)>>`. When the selection differs, it auto-refreshes the effect JSON. This ensures clicking an effect slot in the mixer immediately updates the effect editor panel.

### Next Steps (Phase 4+)
1. **Phase 4**: Qt Timeline + Track Headers — waveform clips, ruler, automation lanes
2. **Phase 5**: Unified Main Window — replace Slint `run()` entirely

---

## Effects Architecture Reference

### Effect Trait
```rust
pub trait Effect: Send + Sync {
    fn process(&mut self, buffer: &mut AudioBuffer);
    fn get_parameter(&self, name: &str) -> Option<f32>;
    fn set_parameter(&mut self, name: &str, value: f32);
    fn get_name(&self) -> &str;
    fn is_bypassed(&self) -> bool;
    fn set_bypassed(&mut self, bypassed: bool);
    fn get_gain_reduction(&self) -> f32 { 0.0 }  // override by Compressor
}
```

### EffectInstance (project data model)
```rust
pub struct EffectInstance {
    pub id: Uuid,
    pub effect_type: String,  // "Equalizer" | "Compressor" | "Reverb" | "Delay"
    pub bypass: bool,
    pub parameters: HashMap<String, f32>,
}
```

### Parameters by Effect Type
| Effect | Parameter | Min | Max | Display |
|--------|-----------|-----|-----|---------|
| Equalizer | eq_low_freq | 20 | 500 Hz | {:.0}Hz |
| | eq_low_gain | -12 | +12 dB | {:.1}dB |
| | eq_mid_freq | 200 | 5000 Hz | {:.0}Hz |
| | eq_mid_gain | -12 | +12 dB | {:.1}dB |
| | eq_mid_q | 0.1 | 10 | {:.2} |
| | eq_high_freq | 2000 | 20000 Hz | {:.0}Hz |
| | eq_high_gain | -12 | +12 dB | {:.1}dB |
| Compressor | comp_threshold | -60 | 0 dB | {:.1}dB |
| | comp_ratio | 1 | 20 :1 | {:.1}:1 |
| | comp_attack | 0.001 | 1.0 s | {:.1}ms |
| | comp_release | 0.01 | 2.0 s | {:.0}ms |
| | comp_makeup | -20 | +20 dB | {:.1}dB |
| Reverb | reverb_room_size | 0 | 1 | {:.0}% |
| | reverb_damping | 0 | 1 | {:.0}% |
| | reverb_wet_dry | 0 | 1 | {:.0}% |
| Delay | delay_time | 0.001 | 5.0 s | {:.0}ms |
| | delay_feedback | 0 | 0.95 | {:.0}% |
| | delay_mix | 0 | 1 | {:.0}% |

---

## Future Development Ideas

### Near-term (polish current features)
- **Effect presets**: Save/load named `HashMap<String, f32>` per effect type to RON files in `presets/` directory. Add dropdown to effect editor.
- **Middle-mouse timeline panning**: Add `pointer-event` handler on timeline TouchArea for middle button.
- **Send management UI**: Add send knobs and pre/post toggle to mixer panel. Sends CRUD already wired — just need the visual widgets.
- **Waveform rendering in clips**: `render_waveform_image()` exists in `src/utils/waveform.rs`. Clips already reference waveform images. Need to ensure peak cache is populated for imported WAVs.
- **dB-scale volume faders**: Current volume is linear 0-1. Map to dB scale (-inf to +6dB) for professional feel.
- **Pan law selection**: Current is constant-power (-3dB center). Add options for -4.5dB, -6dB, linear.

### Medium-term (new major features)
- **Automation write/capture**: Record real-time fader/knob movements as automation points during playback. `write_enabled` field exists but unused.
- **Bus automation UI editing**: Bus automation is stored and read during playback but has no timeline UI for adding/editing points.
- **MIDI sequencing**: New spec needed. Would add `MidiClip` type, piano roll UI, MIDI file I/O via `midly` crate, virtual instrument hosting.
- **Plugin hosting**: CLAP format is Rust-friendly. `clap-sys` crate provides raw bindings. VST3 via `vst3-sys`. Would need a plugin scanner, parameter bridging, and UI hosting.

### Long-term (architectural improvements)
- **Separate Slint files**: Move UI out of the `slint!` macro into actual `.slint` files for better editor support and readability.
- **Lock-free audio params**: Replace `Arc<Mutex<PlaybackState>>` with atomic values + ring buffers for real-time-safe parameter updates.
- **Proper DSP library**: Replace hand-written biquad/compressor/reverb with a proper DSP crate like `funtydsp` or `nih-plug`.
- **Test suite**: Add audio unit tests (sine tone → process → verify) and UI integration tests.
- **CI/CD**: GitHub Actions workflow for Windows build + test.

---

## git Notes

The git history has a checkpoint at `f5ec49c` ("checkpoint: effects, recording, cross-track drag, editing features - partial"). This checkpoint is **broken** (missing deps, incomplete structs, non-compiling app.rs). After restoring from this checkpoint, use this AGENTS.md as the source of truth rather than git history.

To commit current state:
```powershell
git add -A
git commit -m "Qt Phase 3: Mixer panel bridge + QML; auto-refresh effect editor on selection change; hybrid Slint/Qt with 4 floating windows"
```
