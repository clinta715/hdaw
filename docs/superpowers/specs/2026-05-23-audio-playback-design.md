# Audio Playback Engine — Design Spec

## Overview

Wire up the audio engine to play actual audio from WAV files through the mixer. Currently the cpal callback writes silence.

## 1. WAV File Loading

Add `hound = "1.1"` dependency. New module `src/audio/loader.rs`:

```rust
pub fn load_wav(path: &Path) -> Result<AudioBuffer, String>
```

- Reads any standard WAV file (16/24/32-bit, mono/stereo, any sample rate)
- Normalizes integer samples to f32 in range [-1.0, 1.0]
- Returns an `AudioBuffer` with the correct channel count and sample rate
- Resamples to project sample rate if needed (simple linear interpolation for v1)

## 2. PlaybackManager

New module `src/audio/playback.rs`. Contains:

### PlaybackClip (runtime snapshot)

```rust
struct PlaybackClip {
    buffer_key: String,   // maps to loaded AudioBuffer
    start: u64,           // position in project (samples)
    offset: u64,          // offset within source (samples)
    length: u64,          // length to play (samples)
    fade_in: u64,
    fade_out: u64,
    gain: f32,
    track_volume: f32,
    track_pan: f32,
}
```

### PlaybackState (shared between UI and audio threads)

```rust
struct PlaybackState {
    playing: bool,
    position: u64,
    sample_rate: u32,
    clips: Vec<PlaybackClip>,
    buffers: HashMap<String, AudioBuffer>,
}
```

Wrapped in `Arc<Mutex<PlaybackState>>`.

### PlaybackManager

```rust
struct PlaybackManager {
    state: Arc<Mutex<PlaybackState>>,
}
```

**UI-facing methods:**
- `set_playing(bool)` — sets `playing` flag
- `set_position(u64)` — seeks to position
- `load_clips(&[AudioClip], &[Track])` — builds `PlaybackClip` list from project, loads any new WAV files
- `get_position() -> u64` — reads current playhead for UI

**Audio callback method:**
- `fill_buffer(output: &mut [f32], sample_rate: u32)` — called from cpal stream

## 3. Audio Callback Logic

`fill_buffer` implementation:

```
input: output buffer (interleaved stereo f32), sample rate

lock state
if not playing:
  fill output with 0.0
  unlock
  return

let start = state.position
let end = start + (output.len() / 2)  // frames to fill

// Gather clips that overlap [start, end)
for clip in &state.clips:
  if clip.start + clip.length > start && clip.start < end:
    add to active list

// per-clip output buffer (stereo)
let mut mix_buf = vec![0.0f32; output.len()]

for clip in active_clips:
  let clip_start = clip.start
  let clip_end = clip.start + clip.length

  // Read range from source buffer
  let buf = state.buffers[clip.buffer_key]
  let read_start = offset + (start - clip_start)  // adjusted for clip offset
  // Copy samples from buf to per-clip temp buffer
  // Apply fade in/out, clip gain, track volume, track pan
  // Mix into mix_buf

// Apply master volume
// Clamp to [-1.0, 1.0]
copy mix_buf to output

state.position = end
unlock
```

Key detail: the source `AudioBuffer` may have a different sample rate than the project. Simple linear resampling in the read loop.

## 4. Fade & Gain Application

During sample copy from source to output:

- **Fade in**: gain multiplier ramps from 0 to 1 over `fade_in` samples at clip start
- **Fade out**: gain multiplier ramps from 1 to 0 over `fade_out` samples at clip end  
- **Clip gain**: constant multiplier
- **Track volume**: constant multiplier from `ChannelStrip`
- **Pan**: left/right gain per frame

Fade curve: linear for v1 (can upgrade to equal-power later).

## 5. Transport UI Integration

- `Play` button → `transport.play()` + `audio_engine.start()`
- `Stop` button → `transport.stop()` + `audio_engine.stop()`
- Playhead position displayed in transport bar (already rendered as `00.00.00.000`)
- Position updated from `PlaybackManager.get_position()` via a timer or on each frame

For v1, the play/stop buttons in the toolbar call through the app to the audio engine.

## 6. Test Signal

Generate a simple test WAV file at startup (or embed one) to verify the pipeline without requiring external files. A 440Hz sine tone at -6dB for 2 seconds.

## 7. Files Changed/Added

| File | Action |
|------|--------|
| `Cargo.toml` | Add `hound = "1.1"` |
| `src/audio/mod.rs` | Add `loader`, `playback` modules |
| `src/audio/loader.rs` | New — WAV file loading |
| `src/audio/playback.rs` | New — PlaybackManager, PlaybackState |
| `src/audio/engine.rs` | Rewrite — integrate PlaybackManager into cpal callback |
| `src/app.rs` | Wire transport buttons to engine, load clips on startup |
| `src/ui/main_window.rs` | Wire Play/Stop buttons to callbacks |

## 8. Implementation Order

1. Add `hound` to Cargo.toml
2. Implement `src/audio/loader.rs` (load_wav)
3. Implement `src/audio/playback.rs` (PlaybackManager, fill_buffer)
4. Rewrite `src/audio/engine.rs` (integrate playback, cpal callback)
5. Wire transport buttons in `src/app.rs` and `src/ui/main_window.rs`
6. Generate test tone WAV on startup
7. Build and verify with real audio output
