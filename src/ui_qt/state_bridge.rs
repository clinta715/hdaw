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
        #[qproperty(i32, time_secs)]
        #[qproperty(i32, time_ms)]
        #[qproperty(f64, bpm_val)]
        #[qproperty(i32, time_sig_num)]
        #[qproperty(i32, time_sig_denom)]
        type StateBridge = super::StateBridgeRust;

        #[qinvokable]
        fn sync_state(self: Pin<&mut StateBridge>);
        #[qinvokable]
        fn undo(self: Pin<&mut StateBridge>);
        #[qinvokable]
        fn redo(self: Pin<&mut StateBridge>);
        #[qinvokable]
        fn toggle_loop(self: Pin<&mut StateBridge>);
        #[qinvokable]
        fn get_time_display(self: &StateBridge) -> String;
        #[qinvokable]
        fn get_bpm_display(self: &StateBridge) -> String;
        #[qinvokable]
        fn get_time_sig_display(self: &StateBridge) -> String;
    }
}

#[derive(Default)]
pub struct StateBridgeRust {
    undo_available: bool,
    redo_available: bool,
    loop_enabled: bool,
    time_secs: i32,
    time_ms: i32,
    bpm_val: f64,
    time_sig_num: i32,
    time_sig_denom: i32,
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
            let total_secs = (pos as f64 / sr as f64) as i32;
            let ms = ((pos as f64 / sr as f64 - total_secs as f64) * 1000.0) as i32;
            self.as_mut().set_time_secs(total_secs);
            self.as_mut().set_time_ms(ms);
            self.as_mut().set_bpm_val(p.bpm);
            self.as_mut().set_time_sig_num(p.time_signature.0 as i32);
            self.as_mut().set_time_sig_denom(p.time_signature.1 as i32);
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

    fn get_time_display(self: &state_bridge_mod::StateBridge) -> String {
        let mins = self.time_secs / 60;
        let secs = self.time_secs % 60;
        let ms = self.time_ms;
        format!("{:02}:{:02}.{:03}", mins, secs, ms)
    }

    fn get_bpm_display(self: &state_bridge_mod::StateBridge) -> String {
        format!("{:.1} BPM", self.bpm_val)
    }

    fn get_time_sig_display(self: &state_bridge_mod::StateBridge) -> String {
        format!("{}/{}", self.time_sig_num, self.time_sig_denom)
    }
}
