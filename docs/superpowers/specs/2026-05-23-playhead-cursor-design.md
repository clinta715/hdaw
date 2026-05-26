# Playhead Cursor — Design Spec

## Goal
Add a real-time playhead cursor (vertical line + arrow) to the timeline that shows the current playback position and allows click-to-seek.

## Architecture

### Data Flow
```
Audio callback ──writes──> PlaybackState.position (Arc<Mutex<u64>>)
                                  ↑ reads (60 Hz)
                           slint::Timer (16ms interval)
                                  ↓ samples_to_pixels()
                           MainWindow.playhead-x: length
                                  ↓
                           Slint rendering (vertical line)
```

### MainWindow Property
- Add `in-out property <length> playhead-x: 0px;` to the `MainWindow` component
- The playhead is rendered inside the timeline area (not overlapping the scrollbar/panel on the right)

### Timer & Position Update
- A `slint::imer` (`TimerMode::Repeated`, 16ms interval) polls `PlaybackState.position`
- Converts position (project sample rate samples) to pixels using `samples_to_pixels()` from `timeline.rs`
- Calls `window.set_playhead_x(pixels)`
- The timer is stored in `HdawApp` (e.g., `playhead_timer: slint::Timer`) — must not be dropped while running

### Rendering
Draw the playhead as two stacked elements inside the timeline `Rectangle`:

1. **Vertical line**: `Rectangle { x: root.playhead-x; width: 2px; height: 100%; background: #ff4444; }`
2. **Arrow head**: `Text { x: root.playhead-x - 6px; y: -12px; text: "▼"; color: #ff4444; font-size: 14px; }`

The playhead draws on top of all clips and grid lines.

### Click to Seek
In the `timeline-pressed` callback (before clip interaction handling):
- If `tool-mode != 0` (razor tool active), skip (razor takes priority)
- If a clip is hit at the click position, defer to clip interaction (drag/split/fade)
- Otherwise (empty space clicked):
  - Convert `x` to samples: `pixels_to_samples(x, px_per_sec, sample_rate)`
  - Call `playback.set_position(samples)`
  - The playhead immediately snaps to the new position via the next timer tick

### Play / Stop Behavior
- **Play**: `playback.set_playing(true)` — does NOT reset position. Starts from wherever the playhead currently is.
- **Stop**: `playback.set_playing(false); playback.set_position(0)` — stops audio and returns playhead to start.

## Files Changed

| File | Change |
|------|--------|
| `src/ui/main_window.rs` | Add `playhead-x` property, playhead rendering elements in timeline area |
| `src/app.rs` | Add `slint::Timer` field, timer setup in new method, play/stop behavior update (don't reset on play, reset on stop), click-to-seek in `timeline-pressed` |
| `src/audio/playback.rs` | No changes needed (position already exposed) |
| `docs/PROGRESS.md` | Update status |

## Not in Scope (v1)
- Auto-scroll timeline when playhead moves beyond visible area
- Playhead snapping to grid during seek
- Dragging the playhead
- Loop region markers on the timeline ruler
