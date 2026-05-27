#![cfg(feature = "qt")]

pub mod state;
pub mod pool;
pub mod effects;
pub mod mixer;
pub mod timeline;
pub mod state_bridge;
pub mod shortcut_handler;

use std::pin::Pin;

#[cxx_qt::bridge(namespace = "ui_qt::transport_object")]
pub mod transport_object {
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, playing)]
        #[qproperty(bool, recording)]
        #[qproperty(bool, pool_visible)]
        type TransportBar = super::TransportBarRust;

        #[qinvokable]
        fn play(self: Pin<&mut TransportBar>);
        #[qinvokable]
        fn stop(self: Pin<&mut TransportBar>);
        #[qinvokable]
        fn toggle_record(self: Pin<&mut TransportBar>);
        #[qinvokable]
        fn import_file(self: Pin<&mut TransportBar>);
        #[qinvokable]
        fn toggle_pool(self: Pin<&mut TransportBar>);
        #[qinvokable]
        fn sync_state(self: Pin<&mut TransportBar>);
        #[qinvokable]
        fn toggle_play_stop(self: Pin<&mut TransportBar>);
    }
}

pub struct TransportBarRust {
    playing: bool,
    recording: bool,
    pool_visible: bool,
}

impl Default for TransportBarRust {
    fn default() -> Self {
        Self {
            playing: false,
            recording: false,
            pool_visible: false,
        }
    }
}

impl transport_object::TransportBar {
    fn play(self: core::pin::Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            state.playback.set_playing(true);
        }
    }

    fn stop(self: core::pin::Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            state.playback.set_playing(false);
        }
    }

    fn toggle_play_stop(self: core::pin::Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            let next = !state.playback.is_playing();
            state.playback.set_playing(next);
        }
    }

    fn toggle_record(self: core::pin::Pin<&mut Self>) {
        // ...
    }

    fn import_file(self: core::pin::Pin<&mut Self>) {
        crate::ui_qt::state::on_import_file();
    }

    fn toggle_pool(mut self: Pin<&mut Self>) {
        let new_val = !*self.pool_visible();
        self.as_mut().set_pool_visible(new_val);
    }

    fn sync_state(mut self: Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            let is_p = state.playback.is_playing();
            self.as_mut().set_playing(is_p);
        }
    }
}
