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
                    
UI Sync: app.rs re-reads project + calls sync_project_to_timeline_with_waveforms()
    → updates all Slint models (clips, tracks, buses, ruler ticks, etc.)

Timer (60fps): reads position, peaks, GR from PlaybackManager
    → updates playhead-x, peak bars, compressor-gr
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
git commit -m "automation MVP: lane UI, point editing, effect param automation, undo/redo"
```
