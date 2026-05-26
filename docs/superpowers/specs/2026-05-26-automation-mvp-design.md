# Automation MVP (Phase 0→1→2)

**Date**: 2026-05-26
**Status**: Approved — implementing
**Files touched**: 7 files, ~250 lines net

---

## 1. Problem

The data model for automation exists (`AutomationLane`, `AutomationPoint`, `get_value_at()` interpolation) but is unused. There are conflicting duplicated struct definitions in `track.rs` vs `automation.rs`. No automation data flows into the playback engine, and no visual lanes exist below tracks.

---

## 2. Requirements

### 2.1 Phase 0: Data Model Consolidation
- Remove duplicate `AutomationLane`/`AutomationPoint`/`CurveType` from `track.rs`
- Use the `automation.rs` definitions everywhere (they have `get_value_at()`, `read_enabled`, `write_enabled`)
- Add `color` field for UI rendering

### 2.2 Phase 1: Playback Integration
- Automation values read in `fill_buffer()` modulate volume/pan at block rate (~5ms resolution)
- Track volume/pan only for MVP (effect params deferred)
- Automation data stored in `PlaybackState` alongside track/bus params

### 2.3 Phase 2: Lane UI
- Expand track height from 60px to 90px (30px room for automation lane)
- Render automation curve as an image below each clip
- Show lane region with subtle background

---

## 3. Detailed Design

### 3.1 Phase 0 — `src/project/automation.rs`

Add `color` field to `AutomationLane`:
```rust
pub struct AutomationLane {
    pub parameter_id: String,
    pub points: Vec<AutomationPoint>,
    pub enabled: bool,
    pub read_enabled: bool,
    pub write_enabled: bool,
    pub color: (u8, u8, u8),  // NEW
}
```

Default in `AutomationLane::new()`: `color: (200, 200, 60)`.

### 3.2 Phase 0 — `src/project/track.rs`

Remove lines 54-74 (duplicate struct definitions) and add import:
```rust
use crate::project::automation::AutomationLane;
```

The existing `automation: HashMap<String, AutomationLane>` field on `Track` continues to work — the key maps parameter IDs.

### 3.3 Phase 1 — `src/audio/playback.rs`

**New import**: `use crate::project::automation::AutomationLane;`

**New fields on `PlaybackState`**:
```rust
pub track_automation: HashMap<Uuid, HashMap<String, AutomationLane>>,
pub bus_automation: HashMap<Uuid, HashMap<String, AutomationLane>>,
```

**In `load_project_clips()`**: Clone automation data alongside existing state. Collect into local maps, set at line 459.

**In `fill_buffer()`**: Four insertion points:
1. Before line 718 (track volume/pan): read automation for `"volume"` and `"pan"` at current block position, override local `track` values
2. Before line 776 (bus volume/pan): same pattern for buses
3. Before line 811 (master volume/pan): same for master (future)

Automation read per block (not per sample) — each block is ~256-512 samples (~5-11ms at 44.1kHz). This is fine for volume/pan sweeps.

### 3.4 Phase 2 — `src/ui/timeline.rs`

**Layout constants**:
```rust
pub const TRACK_HEIGHT: f32 = 90.0;
pub const CLIP_HEIGHT: f32 = 52.0;
pub const LANE_HEIGHT: f32 = 26.0;
pub const CLIP_Y_OFFSET: f32 = 4.0;
```

**Hardcoded 60px replacements**:
- Line 18: `60.0` → `TRACK_HEIGHT`
- Line 39: `60.0` → `TRACK_HEIGHT`

**New function** `render_automation_image()`:
- Takes `&[&AutomationLane]`, time range, width, height
- Creates an RGBA pixel buffer
- For each lane, draws line segments between consecutive points using Bresenham-style column iteration
- Returns `slint::Image`

**Updated `sync_project_to_timeline_with_waveforms()`**:
- Collect automation lanes per track into `HashMap<Uuid, Vec<&AutomationLane>>`
- For each clip, render automation image for clip's time range
- Include `auto_image` in `ClipInfo`

### 3.5 Phase 2 — `src/ui/main_window.rs`

**Update `ClipInfo` struct** (Slint): add `auto_image: image` field.

**Update clip rendering**:
- `y: clip.track_index * 90px + 4px;`
- `height: 82px;` (52px clip + 4px gap + 26px lane area)
- Add lane `Rectangle` inside clip, below waveform:
  ```slint
  Rectangle {
      y: 56px; // 52 + 4 gap
      width: 100%;
      height: 26px;
      background: rgba(255, 255, 255, 0.03);
      border-radius: 0px 0px 3px 3px;
      clip: true;
      Image {
          width: 100%;
          height: 100%;
          source: clip.auto_image;
          image-fit: fill;
      }
  }
  ```

**Update track divider line** (line 1533):
```slint
y: (track.index + 1) * 90px;
```

### 3.6 Phase 2 — `src/app/callbacks.rs`

Update hardcoded 60px references:
- Line 369: `60.0` → use imported `timeline::TRACK_HEIGHT`
- Line 419: `60.0` → use `timeline::TRACK_HEIGHT`

---

## 4. Edge Cases

| Scenario | Behavior |
|----------|----------|
| Track with no automation | Lane area renders empty (transparent image) |
| Lane with 0 points | Empty image, subtle lane background visible |
| Clip < 30px wide | Auto image renders at clip width, pixels clamped by `clip: true` |
| Automation extends beyond clip bounds | Image clamped to clip's time range; lane handles edge cases |
| Rapid BPM/tempo change | Not affected — volume/pan automation is time-based, not tempo-based |

---

## 5. Scope Boundaries (Not in This Phase)

- Automation point editing UI (Phase 3)
- Effect parameter automation (Phase 3)
- Write/record automation (Phase 4)
- Automation lane add/remove management
- Track height toggle (fixed 90px for now)

---

## 6. Future Considerations

- Per-sample automation reading for effect parameters (more responsive)
- Curve-type visualization (linear vs bezier vs step rendering)
- Automation lane color per-parameter
- Master channel automation
