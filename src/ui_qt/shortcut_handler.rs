#![cfg(feature = "qt")]

use std::pin::Pin;
use std::sync::atomic::Ordering;

#[cxx_qt::bridge(namespace = "ui_qt::shortcut_handler")]
pub mod shortcut_handler_obj {
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(i32, tool_mode)]
        #[qproperty(bool, snap_enabled)]
        #[qproperty(bool, can_undo)]
        #[qproperty(bool, can_redo)]
        type ShortcutHandler = super::ShortcutHandlerRust;

        #[qinvokable]
        fn go_to_start(self: Pin<&mut ShortcutHandler>);
        #[qinvokable]
        fn go_to_end(self: Pin<&mut ShortcutHandler>);
        #[qinvokable]
        fn nudge_left(self: Pin<&mut ShortcutHandler>);
        #[qinvokable]
        fn nudge_right(self: Pin<&mut ShortcutHandler>);
        #[qinvokable]
        fn delete_selected(self: Pin<&mut ShortcutHandler>);
        #[qinvokable]
        fn copy_selected(self: Pin<&mut ShortcutHandler>);
        #[qinvokable]
        fn paste(self: Pin<&mut ShortcutHandler>);
        #[qinvokable]
        fn select_all(self: Pin<&mut ShortcutHandler>);
        #[qinvokable]
        fn add_track(self: Pin<&mut ShortcutHandler>);
        #[qinvokable]
        fn toggle_mute_selected_track(self: Pin<&mut ShortcutHandler>);
        #[qinvokable]
        fn toggle_snap(self: Pin<&mut ShortcutHandler>);
        #[qinvokable]
        fn toggle_tool_mode(self: Pin<&mut ShortcutHandler>);
        #[qinvokable]
        fn reset_tool_mode(self: Pin<&mut ShortcutHandler>);
        #[qinvokable]
        fn sync_state(self: Pin<&mut ShortcutHandler>);
    }
}

pub struct ShortcutHandlerRust {
    tool_mode: i32,
    snap_enabled: bool,
    can_undo: bool,
    can_redo: bool,
}

impl Default for ShortcutHandlerRust {
    fn default() -> Self {
        Self {
            tool_mode: 0,
            snap_enabled: false,
            can_undo: false,
            can_redo: false,
        }
    }
}

impl shortcut_handler_obj::ShortcutHandler {
    fn go_to_start(self: core::pin::Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            state.playback.set_position(0);
            state.timeline_dirty.store(true, Ordering::Relaxed);
        }
    }

    fn go_to_end(self: core::pin::Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            if let Ok(p) = state.project.lock() {
                let max_pos = p.tracks.iter()
                    .flat_map(|t| t.clips.iter())
                    .map(|c| c.position + c.length)
                    .max().unwrap_or(0);
                state.playback.set_position(max_pos);
            }
            state.timeline_dirty.store(true, Ordering::Relaxed);
        }
    }

    fn nudge_left(self: core::pin::Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            let sr = if let Ok(p) = state.project.lock() { p.sample_rate } else { 44100 };
            let step = (sr as u64 / 10).max(1);
            let pos = state.playback.get_position();
            state.playback.set_position(pos.saturating_sub(step));
            state.timeline_dirty.store(true, Ordering::Relaxed);
        }
    }

    fn nudge_right(self: core::pin::Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            let sr = if let Ok(p) = state.project.lock() { p.sample_rate } else { 44100 };
            let step = (sr as u64 / 10).max(1);
            state.playback.set_position(state.playback.get_position() + step);
            state.timeline_dirty.store(true, Ordering::Relaxed);
        }
    }

    fn delete_selected(self: core::pin::Pin<&mut Self>) {
        use uuid::Uuid;
        use std::collections::HashMap;
        if let Some(state) = crate::ui_qt::state::get() {
            let ids: Vec<Uuid> = if let Ok(sel) = state.selected_clips.lock() {
                sel.iter().copied().collect()
            } else { return; };
            if ids.is_empty() { return; }
            if let Ok(mut p) = state.project.lock() {
                let sr = p.sample_rate;
                let mut track_groups: HashMap<Uuid, Vec<crate::project::clip::AudioClip>> = HashMap::new();
                for track in p.tracks.iter() {
                    for clip in track.clips.iter() {
                        if ids.contains(&clip.id) {
                            track_groups.entry(track.id).or_default().push(clip.clone());
                        }
                    }
                }
                for (tid, clips) in &track_groups {
                    if let Some(track) = p.tracks.iter_mut().find(|t| t.id == *tid) {
                        track.clips.retain(|c| !clips.iter().any(|cl| cl.id == c.id));
                    }
                    if let Ok(mut stack) = state.undo_stack.lock() {
                        stack.push(crate::project::undo::EditCommand::DeleteClips {
                            track_id: *tid,
                            clips: clips.clone(),
                        });
                    }
                }
                state.playback.load_project_clips(&p.tracks, &p.buses, sr);
                if let Ok(mut sel) = state.selected_clips.lock() {
                    sel.clear();
                }
            }
            state.timeline_dirty.store(true, Ordering::Relaxed);
        }
    }

    fn copy_selected(self: core::pin::Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            if let Ok(sel) = state.selected_clips.lock() {
                if let Ok(mut cb) = state.clipboard.lock() {
                    cb.clear();
                    if let Ok(p) = state.project.lock() {
                        for track in p.tracks.iter() {
                            for clip in track.clips.iter() {
                                if sel.contains(&clip.id) {
                                    cb.push(clip.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn paste(self: core::pin::Pin<&mut Self>) {
        use uuid::Uuid;
        if let Some(state) = crate::ui_qt::state::get() {
            let clips = if let Ok(cb) = state.clipboard.lock() { cb.clone() } else { return; };
            if clips.is_empty() { return; }
            let pos = state.playback.get_position();
            if let Ok(mut p) = state.project.lock() {
                let sr = p.sample_rate;
                let selected_id = state.selected_track_id.lock().ok().and_then(|s| *s);
                let target_track_id = selected_id.or_else(|| p.tracks.first().map(|t| t.id));
                let mut created: Vec<(Uuid, crate::project::clip::AudioClip)> = Vec::new();
                for clip in &clips {
                    let mut new_clip = clip.clone();
                    new_clip.id = Uuid::new_v4();
                    new_clip.position = clip.position + pos;
                    if let Some(tid) = target_track_id {
                        if let Some(track) = p.tracks.iter_mut().find(|t| t.id == tid) {
                            created.push((tid, new_clip.clone()));
                            track.add_clip(new_clip);
                        }
                    }
                }
                if !created.is_empty() {
                    if let Ok(mut stack) = state.undo_stack.lock() {
                        for (tid, clip) in created.iter() {
                            stack.push(crate::project::undo::EditCommand::AddClip {
                                track_id: *tid,
                                clip: clip.clone(),
                            });
                        }
                    }
                }
                state.playback.load_project_clips(&p.tracks, &p.buses, sr);
            }
            state.timeline_dirty.store(true, Ordering::Relaxed);
        }
    }

    fn select_all(self: core::pin::Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            if let Ok(mut sel) = state.selected_clips.lock() {
                sel.clear();
                if let Ok(p) = state.project.lock() {
                    for track in p.tracks.iter() {
                        for clip in track.clips.iter() {
                            sel.insert(clip.id);
                        }
                    }
                }
            }
        }
    }

    fn add_track(self: core::pin::Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            if let Ok(mut p) = state.project.lock() {
                let sr = p.sample_rate;
                let track = crate::project::track::Track::new(format!("Track {}", p.tracks.len() + 1));
                let snapshot = track.clone();
                let index = p.tracks.len();
                p.tracks.push(track);
                if let Ok(mut stack) = state.undo_stack.lock() {
                    stack.push(crate::project::undo::EditCommand::AddTrack { snapshot, index });
                }
                state.playback.load_project_clips(&p.tracks, &p.buses, sr);
            }
            state.timeline_dirty.store(true, Ordering::Relaxed);
        }
    }

    fn toggle_mute_selected_track(self: core::pin::Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            let tid = if let Ok(s) = state.selected_track_id.lock() { *s } else { return; };
            if let Some(id) = tid {
                if let Ok(mut p) = state.project.lock() {
                    if let Some(track) = p.tracks.iter_mut().find(|t| t.id == id) {
                        track.mute = !track.mute;
                        state.playback.update_track_params(track.id, track.volume, track.pan, track.mute, track.solo);
                    }
                }
                state.timeline_dirty.store(true, Ordering::Relaxed);
            }
        }
    }

    fn toggle_snap(mut self: Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            let new_val = !state.snap_enabled.load(Ordering::Relaxed);
            state.snap_enabled.store(new_val, Ordering::Relaxed);
            self.as_mut().set_snap_enabled(new_val);
        }
    }

    fn toggle_tool_mode(mut self: Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            if let Ok(mut tm) = state.tool_mode.lock() {
                *tm = if *tm == 1 { 0 } else { 1 };
                self.as_mut().set_tool_mode(*tm);
            }
        }
    }

    fn reset_tool_mode(mut self: Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            if let Ok(mut tm) = state.tool_mode.lock() {
                *tm = 0;
                self.as_mut().set_tool_mode(0);
            }
        }
    }

    fn sync_state(mut self: Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            if let Ok(tm) = state.tool_mode.lock() {
                let val = *tm;
                self.as_mut().set_tool_mode(val);
            }
            let snap = state.snap_enabled.load(Ordering::Relaxed);
            self.as_mut().set_snap_enabled(snap);
            if let Ok(stack) = state.undo_stack.lock() {
                let can_undo = stack.can_undo();
                let can_redo = stack.can_redo();
                self.as_mut().set_can_undo(can_undo);
                self.as_mut().set_can_redo(can_redo);
            }
        }
    }
}
