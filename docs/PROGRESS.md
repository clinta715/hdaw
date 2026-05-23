# HDAW — Project Progress & Roadmap

## Legend

- ✅ **Implemented** — code written, compiles, ready for use
- 🔶 **Spec'd** — design document approved, not yet implemented
- 🔲 **Pending** — not yet designed or implemented

---

## Spec A: Undo/Redo & Editing Tools (Phase 1)

### Design
- ✅ Design doc: `docs/superpowers/specs/2026-05-23-undo-editing-tools-design.md`

### Implemented (all compile)

| Area | Files | Status |
|------|-------|--------|
| EditCommand enum (10 variants) | `src/project/undo.rs` | ✅ |
| UndoStack (cursor, cap 100, per-project) | `src/project/undo.rs` | ✅ |
| Coalescing (volume, pan, fade, stretch) | `src/project/undo.rs` | ✅ |
| execute_command / undo_command | `src/project/undo.rs` | ✅ |
| Split clip logic | `src/project/editing.rs` | ✅ |
| Grid snap (beats/time/frames/adaptive) | `src/project/editing.rs` | ✅ |
| Auto crossfade | `src/project/editing.rs` | ✅ |
| Time-stretch bounds & params | `src/project/editing.rs` | ✅ |
| Project model (tracks, time sig) | `src/project/mod.rs` | ✅ |
| HdawApp (undo stack, callbacks) | `src/app.rs` | ✅ |
| MainWindow Slint layout | `src/ui/main_window.rs` | ✅ |
| Timeline rendering (clips, fades, tracks) | `src/ui/main_window.rs` | ✅ |
| Timeline ↔ project sync | `src/ui/timeline.rs` | ✅ |
| Zoom in/out (scroll event) | `src/ui/timeline.rs` | ✅ |

### Not yet implemented (UI interactions)

| Feature | Blockers | Priority |
|---------|----------|----------|
| 🔲 Clip drag to move | Needs `pointer-event` for drag tracking | High |
| 🔲 Clip edge resize/trim | Needs drag on clip edges | High |
| 🔲 Razor tool (split on click) | Needs tool mode switching + clip hit-test | High |
| 🔲 Fade handle drag | Needs separate TouchArea on fade regions | Medium |
| 🔲 Alt-drag for time-stretch | Needs modifier key detection in Slint | Medium |
| 🔲 Auto crossfade on overlap | Depends on clip drag implementation | Medium |
| 🔲 Grid snap applied to drags | Depends on clip drag implementation | Medium |
| 🔲 Grid line rendering | Visual, depends on snap mode state | Low |
| 🔲 Toolbar tool selection (pointer/razor) | Needs UI for tool buttons | Low |

---

## Spec B: Buses & Routing (not started)

| Item | Status |
|------|--------|
| Design doc | 🔲 |
| Group tracks | 🔲 |
| Aux sends/returns | 🔲 |
| Flexible routing matrix | 🔲 |
| Implementation | 🔲 |

---

## Spec C: Recording Workflow (not started)

| Item | Status |
|------|--------|
| Design doc | 🔲 |
| Armed tracks, input monitoring | 🔲 |
| Punch in/out | 🔲 |
| Take/comp management | 🔲 |

---

## Spec D: MIDI Sequencing (not started)

| Item | Status |
|------|--------|
| Design doc | 🔲 |
| Piano roll, note editing | 🔲 |
| Virtual instrument interface | 🔲 |
| MIDI file I/O | 🔲 |

---

## Spec E: Plugin Hosting (not started)

| Item | Status |
|------|--------|
| Design doc | 🔲 |
| VST3/CLAP loading | 🔲 |
| Parameter bridging | 🔲 |
| UI hosting | 🔲 |

---

## Spec F: Video Timeline (not started)

| Item | Status |
|------|--------|
| Design doc | 🔲 |
| Video import, thumbnail rendering | 🔲 |
| Timecode sync | 🔲 |
| Export | 🔲 |

---

## Working Next Steps

1. **Implement clip drag** in timeline (pointer-event based) — unlocks most editing UX
2. **Wire up EditCommand execution** from UI interactions (move, split, fade, stretch)
3. **Add test clips** to project on startup for visual verification
4. **Build & verify** the full editing loop visually
5. **Move to Spec B** (Buses & Routing) or continue with remaining editing UI
