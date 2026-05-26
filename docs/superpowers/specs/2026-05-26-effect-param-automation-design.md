# Effect Parameter Automation

**Date**: 2026-05-26
**Status**: Implemented
**Files touched**: 6 files, ~200 lines net

---

## 1. Problem

Automation editing was hardcoded to `"volume"` only. Users couldn't automate effect parameters (EQ gain, compressor threshold, reverb mix, etc.) even though the data model supported it. Effect automation enables dynamic changes like filter sweeps, auto-panning, gated reverb, and tempo-synced delay feedback.

---

## 2. Requirements

1. User can select which parameter to automate per track (volume, pan, or any effect parameter)
2. Available effect parameters come from the track's effects chain at runtime
3. One visible automation lane per track at a time (user cycles through params)
4. Point editing works for any selected parameter
5. Playback reads effect parameter automation and applies via `effect.set_parameter()` per block
6. Track strip height extends to match timeline track height (52px to 90px) with param selector area

---

## 3. Detailed Design

### 3.1 Data Model — `src/project/track.rs`

Added `selected_automation_param: Option<String>` field to `Track` struct. Default: `None`.

When `None`, no automation lane is visible on the timeline for this track. When set to e.g. `"volume"`, `"eq_low_gain"`, etc., that lane is visible/editable.

### 3.2 Slint UI — `src/ui/main_window.rs`

- **TrackInfo struct**: Added `auto_param_name: string` field.
- **Track strip height**: Extended from 52px to 90px to match timeline track height.
- **Param selector area**: Rectangle at y=56px, height=26px with `"+A"` text (or param name). TouchArea cycles through available params.
- **ClipInfo struct**: Added `auto_param_name: string` field.
- **New callback**: `track-auto-param-clicked(string)` — cycles to next available param.

### 3.3 Timeline Sync — `src/ui/timeline.rs`

- `sync_project_to_timeline_with_waveforms()` now passes only the selected lane to `render_automation_image()` (was: all lanes).
- Populates `auto_param_name` on both `TrackInfo` and `ClipInfo` from `track.selected_automation_param`.
- Removed now-unused `track_auto` HashMap.

### 3.4 Callbacks — `src/app/callbacks.rs`

- `on_timeline_pressed`: Replaced hardcoded `"volume"` with `track.selected_automation_param`. If `None`, lane clicks are ignored.
- New `on_track_auto_param_clicked`: Cycles through available params in order:
  ```
  volume -> pan -> eq_low_freq -> eq_low_gain -> ... -> delay_mix -> (none) -> volume
  ```
  Available effect params are computed from the track's `effects_chain` at callback time. Auto-creates `AutomationLane` on first selection.

### 3.5 Playback — `src/audio/playback.rs`

In `fill_buffer()`, before each `effect.process()` call in both track and bus effects loops:

```rust
for effect in fx_chain.iter_mut() {
    if let Some(lanes) = state.track_automation.get(&track.id) {
        let t = current_pos as f64 / project_sr as f64;
        for (param_name, lane) in lanes {
            if lane.enabled && lane.read_enabled && lane.points.len() >= 2 {
                if effect.get_parameter(param_name).is_some() {
                    if let Some(v) = lane.get_value_at(t) {
                        effect.set_parameter(param_name, v);
                    }
                }
            }
        }
    }
    effect.process(&mut audio_buf);
}
```

Automation lanes are matched to effects via `effect.get_parameter(name)` — each effect type recognizes its own parameter names. If two effects share a param name (e.g., two compressors on same track), both respond.

---

## 4. Available Effect Parameters

| Effect Type | Parameters |
|------------|-----------|
| `Equalizer` | `eq_low_freq`, `eq_low_gain`, `eq_mid_freq`, `eq_mid_gain`, `eq_mid_q`, `eq_high_freq`, `eq_high_gain` |
| `Compressor` | `comp_threshold`, `comp_ratio`, `comp_attack`, `comp_release`, `comp_makeup` |
| `Reverb` | `reverb_room_size`, `reverb_damping`, `reverb_wet_dry` |
| `Delay` | `delay_time`, `delay_feedback`, `delay_mix` |

Always available: `volume`, `pan`.

---

## 5. Edge Cases

| Scenario | Behavior |
|----------|----------|
| Track has no effects | Only `volume` and `pan` available |
| Effect removed while lane has points | Lane persists but becomes inert (no effect matches its param) |
| Track strip auto-param label empty | Shows `"+A"` button |
| Two compressors on same track | Both respond to `comp_threshold` automation |
| Param cycled to `None` | Lane area clears, point editing disabled |
| Lane exists but disabled | Points stored but curve not rendered, playback skips it |
