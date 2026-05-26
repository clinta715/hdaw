# HDAW — Project Progress & Roadmap

## Legend

- ✅ **Implemented** — code written, compiles, tested
- 🔶 **Partially done** — backend exists, UI gaps remain
- 🔲 **Pending** — not yet designed or implemented

---

## Spec A: Undo/Redo & Editing Tools

### Design
- ✅ Design doc: `docs/superpowers/specs/2026-05-23-undo-editing-tools-design.md`

### Implemented
| Area | Files | Status |
|------|-------|--------|
| EditCommand enum (24 variants) | `src/project/undo.rs` | ✅ |
| UndoStack (cursor, cap 100, coalescing) | `src/project/undo.rs` | ✅ |
| execute/undo commands for all variants | `src/project/undo.rs` | ✅ |
| Split clip, grid snap, auto crossfade | `src/project/editing.rs` | ✅ |
| Clip drag/trim/fade/stretch on timeline | `src/app.rs`, `src/ui/timeline.rs` | ✅ |
| Razor tool, pointer tool | `src/app.rs` | ✅ |
| Snap modes (adaptive/beats/time/frames) | `src/app.rs`, `src/project/editing.rs` | ✅ |

---

## Spec B: Buses & Routing

| Item | Files | Status |
|------|-------|--------|
| Bus model (name, volume, pan, mute, solo, sends, effects, routing) | `src/project/bus.rs` | ✅ |
| Track output routing (`output_id`) | `src/project/track.rs` | ✅ |
| AuxSend model (pre/post fader) | `src/project/track.rs` | ✅ |
| Bus routing UI (click label to cycle) | `src/ui/main_window.rs` | ✅ |
| Send CRUD (add/remove/toggle) | `src/app.rs`, `src/ui/main_window.rs` | ✅ |
| Topological bus processing in playback | `src/audio/playback.rs` | ✅ |
| Undo commands for routing + sends | `src/project/undo.rs` | ✅ |

---

## Spec C: Recording Workflow

| Item | Files | Status |
|------|-------|--------|
| RecordState (armed tracks, buffers) | `src/audio/record.rs` | ✅ |
| cpal input stream integration | `src/audio/engine.rs` | ✅ |
| Software input monitoring | `src/audio/playback.rs` | ✅ |
| Arm button in track headers | `src/ui/main_window.rs` | ✅ |
| Record button UI + R key shortcut | `src/app.rs` | ✅ |
| Write recorded audio to WAV + insert as clip | `src/app.rs` | ✅ |
| Punch in/out | — | 🔲 |
| Take/comp management | — | 🔲 |

---

## Spec G: Audio Playback

| Area | Files | Status |
|------|-------|--------|
| cpal output stream | `src/audio/engine.rs` | ✅ |
| WAV loading via hound | `src/audio/loader.rs` | ✅ |
| WAV metadata for pool | `src/audio/loader.rs` | ✅ |
| Test tone generation | `src/audio/loader.rs` | ✅ |
| PlaybackManager (clips, mixing, effects, buses) | `src/audio/playback.rs` | ✅ |
| Audio pool (file info tracking) | `src/audio/pool.rs` | ✅ |
| Transport (play/stop/loop/seek) | `src/app.rs` | ✅ |
| Playhead cursor (red line, 60fps timer) | `src/app.rs`, `src/ui/main_window.rs` | ✅ |
| Auto-scroll when playhead passes 80% | `src/app.rs` | ✅ |
| Keyboard shortcuts (Space, Ctrl+Z/Y, Home, End, ±, L, R, P, M, S, N, Esc) | `src/app.rs` | ✅ |

---

## Effects + Mixer (May 26, 2026 session)

### Effects DSP
| Effect | File | Status |
|--------|------|--------|
| EQ (3-band: low shelf / peaking / high shelf) | `src/audio/effects/eq.rs` | ✅ |
| Compressor (threshold/ratio/attack/release/makeup) | `src/audio/effects/compressor.rs` | ✅ |
| Reverb (room size/damping/wet-dry) | `src/audio/effects/reverb.rs` | ✅ |
| Delay (time/feedback/mix) | `src/audio/effects/delay.rs` | ✅ |
| Effect factory (instantiate from model) | `src/audio/effects/factory.rs` | ✅ |
| Effect trait with `get_gain_reduction()` | `src/audio/effects/mod.rs` | ✅ |
| Compressor GR tracking per buffer | `src/audio/effects/compressor.rs` | ✅ |
| EQ frequency response computation + image render | `src/audio/effects/eq.rs` | ✅ |

### Mixer Backend
| Feature | File | Status |
|---------|------|--------|
| Per-track peak tracking (L/R, 0.92 falloff) | `src/audio/playback.rs` | ✅ |
| Per-bus peak tracking (L/R) | `src/audio/playback.rs` | ✅ |
| Master peak tracking (L/R) | `src/audio/playback.rs` | ✅ |
| Master volume/pan/mute in output path | `src/audio/playback.rs`, `src/project/mod.rs` | ✅ |
| Getter methods: `get_track_peaks()`, `get_bus_peaks()`, `get_master_peak()`, `get_compressor_gr()` | `src/audio/playback.rs` | ✅ |

### Mixer UI
| Feature | File | Status |
|---------|------|--------|
| Interactive volume faders (drag horizontal) | `src/ui/main_window.rs` | ✅ |
| Interactive pan knobs (drag horizontal) | `src/ui/main_window.rs` | ✅ |
| Peak meters per track (L/R, green→yellow→red) | `src/ui/main_window.rs` | ✅ |
| Peak meters per bus (L/R) | `src/ui/main_window.rs` | ✅ |
| Master channel strip (volume/pan/meters/mute) | `src/ui/main_window.rs` | ✅ |
| Right-side mixer panel (toggleable, View > Show Mixer) | `src/ui/main_window.rs` | ✅ |
| Vertical faders in mixer panel | `src/ui/main_window.rs` | ✅ |
| Stereo meter bars in mixer panel | `src/ui/main_window.rs` | ✅ |
| M/S buttons in mixer panel | `src/ui/main_window.rs` | ✅ |
| Effect slot chips in mixer panel | `src/ui/main_window.rs` | ✅ |
| Peak meter updates in 60fps timer | `src/app.rs` | ✅ |
| Compressor GR updates in 60fps timer | `src/app.rs` | ✅ |

### Effects UI
| Feature | File | Status |
|---------|------|--------|
| Effect CRUD (add/remove/move/bypass) | `src/app.rs` | ✅ |
| Parameter sliders with live DSP update | `src/app.rs` | ✅ |
| Bypass toggle button in editor | `src/ui/main_window.rs` | ✅ |
| EQ frequency response curve visualization | `src/ui/main_window.rs` | ✅ |
| Compressor gain reduction meter bar | `src/ui/main_window.rs` | ✅ |
| Effect preset system | — | 🔲 |

### Menus
| Menu | Items | Status |
|------|-------|--------|
| File | New, Open, Save, Import Audio, Quit | ✅ |
| Edit | Undo, Redo, Copy, Paste, Delete, Select All | ✅ |
| Track | Add Track, Delete Track, Delete Bus | ✅ |
| Transport | Play, Stop, Record, Loop, Go to Start, Go to End | ✅ |
| View | Audio Pool, Snap Enabled, Show Mixer, Snap Config | ✅ |

---

## Project I/O

| Feature | File | Status |
|---------|------|--------|
| RON serialize/deserialize | `src/project/io.rs` | ✅ |
| New/Open/Save from menus | `src/app.rs` | ✅ |
| Auto-save / recovery | — | 🔲 |

---

## Not Started

| Spec | Description |
|------|-------------|
| D: MIDI Sequencing | Piano roll, note editing, virtual instruments |
| E: Plugin Hosting | VST3/CLAP loading, parameter bridging |
| F: Video Timeline | Video import, thumbnail rendering, timecode sync |
| Automation | Lane UI, point editing, playback read/write (model exists) |
| Effect presets | Save/load named parameter sets |
| Middle-mouse panning | Timeline scroll via middle mouse |
| BPM-based ruler | Musical grid lines on ruler |
| Snap indicators | Visual grid lines on timeline |

---

## File Roster

```
hdaw/
├── Cargo.toml
├── SPEC.md
├── docs/
│   ├── PROGRESS.md
│   └── superpowers/specs/
│       ├── 2026-05-23-undo-editing-tools-design.md
│       ├── 2026-05-23-audio-playback-design.md
│       ├── 2026-05-23-editing-ui-interactions-design.md
│       └── 2026-05-23-playhead-cursor-design.md
├── src/
│   ├── main.rs
│   ├── app.rs                          (~920 lines — main orchestrator)
│   ├── audio/
│   │   ├── mod.rs, buffer.rs, engine.rs, loader.rs, mixer.rs
│   │   ├── playback.rs                 (~900 lines — audio graph)
│   │   ├── pool.rs, record.rs, transport.rs
│   │   └── effects/
│   │       ├── mod.rs (Effect trait)
│   │       ├── eq.rs, compressor.rs, reverb.rs, delay.rs
│   │       └── factory.rs
│   ├── project/
│   │   ├── mod.rs (Project model)
│   │   ├── track.rs, clip.rs, bus.rs
│   │   ├── undo.rs                     (~600 lines — command pattern)
│   │   ├── editing.rs, automation.rs, io.rs
│   ├── ui/
│   │   ├── mod.rs, main_window.rs      (~2200 lines — Slint UI)
│   │   └── timeline.rs
│   └── utils/
│       ├── mod.rs, waveform.rs, timestretch.rs
└── ui/
    └── (empty — all UI in slint! macro)
```
