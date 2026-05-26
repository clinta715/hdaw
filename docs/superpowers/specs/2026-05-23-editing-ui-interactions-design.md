# Editing UI Interactions — Design Spec

## Overview

Design for the Slint-to-Rust interaction layer that enables timeline editing: clip drag, resize, split, fade handles, and grid snap. This builds on the undo/redo + editing data model already implemented in `src/project/undo.rs` and `src/project/editing.rs`.

## 1. Architecture & State Model

### DragState

Maintained in `src/app.rs` (or a new `src/ui/interaction.rs`):

```rust
struct DragState {
    active: bool,
    tool: ToolMode,
    drag_clip_id: Option<Uuid>,
    drag_track_id: Option<Uuid>,
    pointer_offset_x: f32,       // cursor X offset within clip in pixels
    original_clip_x: f32,        // clip x at drag start (for undo)
    drag_edge: Option<DragEdge>,
}

enum ToolMode { Pointer, Razor }

enum DragEdge { Left, Right, FadeIn, FadeOut }
```

### Interaction Flow

1. **Press** — Slint `TouchArea` `pointer-event(Pressed)` fires → Rust hit-tests clips → records state
2. **Move** — `pointer-event(Moved)` → computes new position in pixels → converts to samples → applies snap → updates ClipInfo model for realtime visual
3. **Release** — `pointer-event(Released)` → creates EditCommand → pushes to UndoStack → re-syncs model

### Tool Mode

- `MainWindow` gets an `in property <int> tool-mode` (0 = Pointer, 1 = Razor)
- Rust reads this to decide how to handle clicks
- UI buttons toggle the property

## 2. Clip Drag (Move)

Earh clip Rectangle has a TouchArea. Only responds when `tool-mode == Pointer`.

**On Pressed:**
- Read `event.x` and `event.y` from pointer event
- Hit-test: iterate clips in current model, find which one contains the event position (matching track row from `track_index * 60px` and clip bounds)
- Skip if event is on a fade-handle or edge zone (those take priority)
- Record `pointer_offset_x = event_x - clip.x`
- Set `drag_state = DragState { active: true, drag_clip_id, drag_track_id, pointer_offset_x, original_clip_x, drag_edge: None }`

**On Moved (while drag active):**
- Compute new pixel X: `event_x - pointer_offset_x`
- Convert to samples: `samples = (pixel_x / pixels_per_second) * sample_rate`
- Apply `snap_to_grid()` with current snap mode
- Convert snapped samples back to pixels for visual display
- Update only the dragged clip's `x` in the Slint model for smooth preview
- Do NOT touch the project model until release

**On Released (while drag active):**
- Compute final snapped position in samples
- Create `EditCommand::MoveClip { track_id, clip_id, old_pos: original_position, new_pos: snapped_position }`
- `execute_command()` on project
- Push to `UndoStack`
- Run auto-crossfade if clip overlaps another on the same track
- `sync_project_to_timeline()` to refresh all clips
- Set `drag_state.active = false`

## 3. Clip Edge Resize & Trim

Narrow hit zones (5px) on left and right edges of each clip. These are checked before the main clip drag in the hit-test.

**Left edge drag:**
- Changes both `offset` (start position within source file) and `length`
- Clip `position` stays the same; `offset` moves forward/backward, `length` adjusts inversely
- Creates `EditCommand::ResizeClip`

**Right edge drag:**
- Changes `length` only
- Clip `position` and `offset` unchanged
- Creates `EditCommand::ResizeClip`

**Edge pointer zones:**
- Left 5px of clip → `DragEdge::Left`
- Right 5px of clip → `DragEdge::Right`
- During move, check if pointer is within 5px of clip edge before treating as drag

## 4. Fade Handles

Two overlay Rectangles per clip:
- Top-left: 10px × 20px, positioned at clip.x
- Top-right: 10px × 20px, positioned at clip.x + clip.width - 10px

Each has its own TouchArea. On drag:
- Compute fade duration from pointer X relative to clip edge
- Clamp: 0 to clip length
- Update `fade_in_width` / `fade_out_width` in ClipInfo model during drag (visual feedback)
- On release: `EditCommand::SetFade { track_id, clip_id, edge: FadeIn/FadeOut, old_dur, new_dur }`

Fade UI: rendered as a triangle overlay (using a Rectangle with a gradient or angled border) — simplified to a semi-transparent darker region at the clip corners (already implemented as `rgba(0,0,0,0.3)` rectangles in the current UI).

## 5. Time-Stretch Drag

Alt-drag right edge:
- Same as right-edge resize, but holding Alt toggles resize vs stretch
- Detected via modifier key — Slint `PointerEvent` has a `modifiers` field; check for Alt
- Stretch: `length` changes, `offset` stays same, `position` unchanged
- `ratio = new_length / original_length`, clamped to [0.25, 4.0]
- Creates `EditCommand::TimeStretch`

## 6. Razor Tool (Split)

When `tool-mode == Razor`:
- All clip TouchAreas ignored for drag; only respond to clicks
- On `clicked` or `pointer-event(Pressed)`:
  - Find which clip the pointer is on (same hit-test as drag)
  - Convert pixel X to sample position
  - Apply grid snap if enabled
  - Call `split_clip(&clip, split_position)` from `editing.rs`
  - Create `EditCommand::SplitClip { track_id, original_clip, new_clips }`
  - Execute and push to UndoStack
  - Re-sync timeline

If click is on empty track area (no clip), no-op.

## 7. Grid Snap

### Snap Mode State

- Added to MainWindow: `in property <int> snap-mode` (0=Off, 1=Beats, 2=Time, 3=Frames)
- Rust holds a `SnapMode` value that maps to this

### During Drags

- All position computations (move, resize, split) call `snap_to_grid()` from `editing.rs`
- Snap mode read from Rust state
- Adaptive mode selected automatically at certain zoom levels (future)

### Grid Line Rendering

- Additional model `grid-lines: [length]` on MainWindow
- Computed in Rust based on snap mode, zoom, and visible timeline width
- Rendered as thin vertical Rectangles in the timeline:
```slint
for gx in root.grid-lines: Rectangle {
    x: gx;
    width: 1px;
    height: 100%;
    background: #333333;
}
```

## 8. Toolbar

Added to the transport bar area:
```slint
HorizontalLayout {
    spacing: 2px;
    Rectangle {
        background: tool-mode == 0 ? #444444 : #2a2a2a;
        Text { text: "◆"; ... }
        TouchArea { clicked => { root.tool-mode = 0; } }
    }
    Rectangle {
        background: tool-mode == 1 ? #444444 : #2a2a2a;
        Text { text: "✂"; ... }
        TouchArea { clicked => { root.tool-mode = 1; } }
    }
}
```

## 9. Real-time Visual Feedback During Drags

For smooth drag experience, we need to update clip positions without pushing to the undo stack on every pixel. The approach:

1. During drag move, directly modify the `ClipInfo` in the Slint `VecModel` for the dragged clip
2. Use `ModelRc::set_row_data()` to update individual clip positions
3. Only touch the project model + undo stack on release

This avoids full re-syncs during drag while keeping the UI responsive.

## 10. Auto-Crossfade on Overlap

After a clip is moved (on release):
1. Check if the moved clip overlaps any other clip in the same track
2. Find the overlapping neighbor(s)
3. Call `create_auto_crossfade()` from `editing.rs` on the pair
4. Record the fade changes as part of the MoveClip command (or as a separate SetFade that gets coalesced)

## 11. Files Changed

| File | Change |
|------|--------|
| `src/ui/main_window.rs` | Add new Slint properties (tool-mode, snap-mode, grid-lines), add TouchAreas on clips, add edge/fade handle zones, add toolbar buttons |
| `src/ui/timeline.rs` | Add hit-test helper, grid line computation, drag update helpers |
| `src/app.rs` | Add DragState, add interaction callbacks (on_clip_pointer_event, on_tool_change, on_snap_change) |

## 12. Implementation Order

1. Add ToolMode and DragState to app.rs
2. Add new callbacks to MainWindow Slint (clip-pointer-event, tool-mode-changed)
3. Implement basic clip drag (move)
4. Implement grid snap integration
5. Implement razor tool
6. Implement fade handles
7. Implement edge resize
8. Implement time-stretch (Alt-drag)
9. Add grid line rendering
10. Add toolbar UI
11. Add auto-crossfade on overlap
12. Wire up undo/redo button states (can-undo, can-redo)
