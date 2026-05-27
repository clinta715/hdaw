#![cfg(feature = "qt")]

use std::pin::Pin;
use std::sync::atomic::Ordering;

#[cxx_qt::bridge(namespace = "ui_qt::state_bridge")]
pub mod state_bridge_mod {
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, undo_available)]
        #[qproperty(bool, redo_available)]
        #[qproperty(bool, loop_enabled)]
        #[qproperty(String, time_display)]
        #[qproperty(String, bpm_display)]
        #[qproperty(String, time_sig_display)]
        type StateBridge = super::StateBridgeRust;

        #[qinvokable]
        fn sync_state(self: Pin<&mut StateBridge>);
        #[qinvokable]
        fn undo(self: Pin<&mut StateBridge>);
        #[qinvokable]
        fn redo(self: Pin<&mut StateBridge>);
        #[qinvokable]
        fn toggle_loop(self: Pin<&mut StateBridge>);
    }
}

#[derive(Default)]
pub struct StateBridgeRust {
    undo_available: bool,
    redo_available: bool,
    loop_enabled: bool,
    time_display: String,
    bpm_display: String,
    time_sig_display: String,
}

impl state_bridge_mod::StateBridge {
    fn sync_state(mut self: Pin<&mut Self>) {
        let state = match crate::ui_qt::state::get() {
            Some(s) => s,
            None => return,
        };

        let pos = state.playback.get_position();
        if let Ok(p) = state.project.lock() {
            let sr = p.sample_rate;
            let secs = pos as f64 / sr as f64;
            self.as_mut().set_time_display(crate::utils::format_time(secs));
            self.as_mut().set_bpm_display(format!("{:.1} BPM", p.bpm));
            self.as_mut().set_time_sig_display(format!("{}/{}", p.time_signature.0, p.time_signature.1));
        }

        if let Ok(stack) = state.undo_stack.lock() {
            self.as_mut().set_undo_available(stack.can_undo());
            self.as_mut().set_redo_available(stack.can_redo());
        }

        self.as_mut().set_loop_enabled(state.playback.is_loop_enabled());
    }

    fn undo(mut self: Pin<&mut Self>) {
        let state = match crate::ui_qt::state::get() {
            Some(s) => s,
            None => return,
        };
        if let Ok(mut stack) = state.undo_stack.lock() {
            if let Ok(mut p) = state.project.lock() {
                stack.undo(&mut p);
                let sr = p.sample_rate;
                state.playback.load_project_clips(&p.tracks, &p.buses, sr);
                state.timeline_dirty.store(true, Ordering::Relaxed);

                self.as_mut().set_undo_available(stack.can_undo());
                self.as_mut().set_redo_available(stack.can_redo());
            }
        }
    }

    fn redo(mut self: Pin<&mut Self>) {
        let state = match crate::ui_qt::state::get() {
            Some(s) => s,
            None => return,
        };
        if let Ok(mut stack) = state.undo_stack.lock() {
            if let Ok(mut p) = state.project.lock() {
                stack.redo(&mut p);
                let sr = p.sample_rate;
                state.playback.load_project_clips(&p.tracks, &p.buses, sr);
                state.timeline_dirty.store(true, Ordering::Relaxed);

                self.as_mut().set_undo_available(stack.can_undo());
                self.as_mut().set_redo_available(stack.can_redo());
            }
        }
    }

    fn toggle_loop(mut self: Pin<&mut Self>) {
        let state = match crate::ui_qt::state::get() {
            Some(s) => s,
            None => return,
        };
        let enabled = !state.playback.is_loop_enabled();
        state.playback.set_loop_enabled(enabled);
        self.as_mut().set_loop_enabled(enabled);
    }
}
