# Undo/Redo & Editing Tools — Design Spec

## Overview

Design for the undo/redo system and core editing tools for HDAW, built on a command pattern with an enum-based approach.

## 1. Undo/Redo Core

### Command Enum

All editable operations are represented as variants of a single `EditCommand` enum:

```rust
enum EditCommand {
    MoveClip { track_id: Uuid, clip_id: Uuid, old_pos: TimePos, new_pos: TimePos },
    ResizeClip { track_id: Uuid, clip_id: Uuid, old_start: TimePos, old_end: TimePos, new_start: TimePos, new_end: TimePos },
    SplitClip { track_id: Uuid, original_clip: ClipSnapshot, new_clips: (ClipSnapshot, ClipSnapshot) },
    DeleteClips { track_id: Uuid, clips: Vec<ClipSnapshot> },
    ChangeVolume { track_id: Uuid, old_val: f32, new_val: f32 },
    ChangePan { track_id: Uuid, old_val: f32, new_val: f32 },
    SetFade { track_id: Uuid, clip_id: Uuid, edge: FadeEdge, old_dur: TimeDuration, new_dur: TimeDuration },
    TimeStretch { track_id: Uuid, clip_id: Uuid, old_length: TimeDuration, old_stretch: StretchParams, new_length: TimeDuration, new_stretch: StretchParams },
    AddTrack { snapshot: TrackSnapshot, index: usize },
    RemoveTrack { snapshot: TrackSnapshot, index: usize },
}
```

- Commands hold both old and new state so `execute` and `undo` are symmetric operations applied to a `&mut Project`.
- `ClipSnapshot` / `TrackSnapshot` are serializable value types that capture the full state of a clip or track at a point in time.

### Undo Stack

- **Scope:** Per-project. Cleared on new/open project.
- **Depth:** Max 100 entries. When full, the oldest entry is dropped.
- **Implementation:** `UndoStack { commands: Vec<EditCommand>, cursor: usize }`
  - `cursor` points to the next command to undo (starts at `commands.len()`).
  - Undo: decrements cursor, calls `command.undo(&mut project)`.
  - Redo: calls `command.execute(&mut project)` at cursor, increments cursor.
  - New action: truncate `commands[cursor..]`, push new command, reset cursor.
- **Empty undo:** cursor == 0, nothing to undo.
- **Empty redo:** cursor == commands.len(), nothing to redo.

### Coalescing

- **CoalesceKey:** an enum identifying continuous-parameter gestures:
  ```rust
  enum CoalesceKey { Volume(Uuid), Pan(Uuid), Fade(Uuid, FadeEdge), TimeStretch(Uuid) }
  ```
- When a new command with a matching `CoalesceKey` is pushed while `cursor == commands.len()`, it replaces the top entry rather than appending.
- Coalescing ends when the user releases the control (mouse up / touch end). The command is finalized and subsequent changes push new entries.
- Commands that are NOT coalesced (splits, deletes, add/remove) push unconditionally.

### ProjectState Command Application

```rust
trait Command {
    fn execute(&mut self, project: &mut Project);
    fn undo(&mut self, project: &mut Project);
    fn coalesce_key(&self) -> Option<CoaleskeKey> { None }
}
```

- `EditCommand` implements this trait.
- `execute` applies `new_*` values; `undo` restores `old_*` values.
- Simple parameter changes (volume, pan) swap old ↔ new by destructuring the enum variant.

---

## 2. Fade Handles & Crossfades

### Fade Handles

- Each `AudioClip` already has `fade_in: TimeDuration` and `fade_out: TimeDuration`.
- In the timeline UI, dragable triangular handles appear at the top-left (fade in) and top-right (fade out) of each clip.
- Dragging changes the duration. Command: `EditCommand::SetFade`.
- Fade curve: equal-power (cosine/sqrt) by default.

### Auto Crossfade

- When a clip is dragged to overlap another clip on the same track, an automatic crossfade is created.
- The crossfade = `fade_out` of the earlier clip + `fade_in` of the later clip = overlap duration.
- If clips are separated (drag apart), fade handles reset to zero.
- No new data structure required — a crossfade is simply the intersection of two clips' fades.
- Fade curves modulate waveform amplitude in the timeline rendering.

---

## 3. Razor Tool (Split Clip)

- Activated from toolbar or keyboard shortcut (`S`).
- Click on a clip at a cursor position → splits into two clips at that boundary.
- Both clips reference the same `AudioSource`, with adjusted `offset` and `length`:
  - Left clip: `offset` unchanged, `length` = split_time - clip_start
  - Right clip: `offset` = split_time - clip_start, `length` = original_length - new_left_length
- Split at an exact clip boundary → no-op.
- Grid snap applies when enabled.
- Command: `EditCommand::SplitClip` stores the original clip snapshot plus both resulting clip snapshots for symmetric undo/redo.
- Split at playhead position (when not in razor mode) via menu or keyboard shortcut.

---

## 4. Grid Snap

### Snap Modes (single-select)

| Mode | Sub-options |
|------|-------------|
| Off | Free placement |
| Beats | 1/1, 1/2, 1/4, 1/8, 1/16, 1/32 |
| Time | Seconds, milliseconds |
| Frames | 24, 25, 30, 30-drop, 60 fps |

### Adaptive Snap

- When `Beats` mode is active, an additional **adaptive** toggle sets snap resolution dynamically based on zoom level.
- Wider zoom → coarser snap (e.g., 1/1 at low zoom, 1/16 at high zoom).
- Adaptive is on by default; user can override to a fixed division.

### Behavior

- Clip drag start/end, split point, and playhead seek all snap to nearest grid line.
- **Absolute snap** by default: clip start snaps to grid.
- **Relative snap modifier** (Alt held): preserves the clip's offset from the nearest grid line.
- Grid lines rendered at snap interval on the timeline. Current snap division shown in transport bar.

---

## 5. Time-Stretch Drag

- **Gesture:** Alt-drag the left or right edge of a clip.
- The clip's `length` changes; `source` and `offset` remain the same.
- `StretchParams.ratio = new_length / original_length`.
- Waveform renders at the stretched width; time mapping is applied during playback.
- **Non-destructive:** original audio file unchanged. Stretch computed offline via worker thread and cached.
- **Bounds:** 0.25× to 4× stretch ratio.
- Command: `EditCommand::TimeStretch`.

---

## Undo/Redo — Self-Review Checklist

- [x] No "TBD", "TODO", or incomplete sections.
- [x] Internally consistent: command enum variants match the editing features defined in sections 2-5.
- [x] Scope is focused on undo/redo + editing tools only (not buses, MIDI, recording, etc.).
- [x] Requirements are unambiguous — each command has explicit old/new state.
