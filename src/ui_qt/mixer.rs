#![cfg(feature = "qt")]

use std::pin::Pin;
use std::sync::atomic::Ordering;

#[cxx_qt::bridge(namespace = "ui_qt::mixer")]
pub mod mixer_bridge {
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(String, mixer_json)]
        #[qproperty(String, peaks_json)]
        type MixerModel = super::MixerModelRust;

        #[qinvokable]
        fn refresh(self: Pin<&mut MixerModel>);
        #[qinvokable]
        fn sync_peaks(self: Pin<&mut MixerModel>);
        #[qinvokable]
        fn set_volume(self: Pin<&mut MixerModel>, strip_id: &str, value: f64);
        #[qinvokable]
        fn set_pan(self: Pin<&mut MixerModel>, strip_id: &str, value: f64);
        #[qinvokable]
        fn toggle_mute(self: Pin<&mut MixerModel>, strip_id: &str);
        #[qinvokable]
        fn toggle_solo(self: Pin<&mut MixerModel>, strip_id: &str);
        #[qinvokable]
        fn toggle_arm(self: Pin<&mut MixerModel>, strip_id: &str);
        #[qinvokable]
        fn select_effect(self: Pin<&mut MixerModel>, strip_id: &str, effect_idx: i32);
    }
}

#[derive(Default)]
pub struct MixerModelRust {
    mixer_json: String,
    peaks_json: String,
}

fn effect_shorthand(effect_type: &str) -> &str {
    match effect_type {
        "Equalizer" => "EQ",
        "Compressor" => "CP",
        "Reverb" => "RV",
        "Delay" => "DL",
        _ => "FX",
    }
}

impl mixer_bridge::MixerModel {
    fn refresh(mut self: Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            let p = match state.project.lock() {
                Ok(p) => p,
                Err(_) => return,
            };
            let mut strips: Vec<serde_json::Value> = Vec::new();

            for track in &p.tracks {
                let fx_abbrevs: Vec<&str> = track.effects_chain.iter().map(|e| effect_shorthand(&e.effect_type)).collect();
                strips.push(serde_json::json!({
                    "id": track.id.to_string(),
                    "name": track.name,
                    "type": "track",
                    "vol": track.volume,
                    "pan": track.pan,
                    "mut": track.mute,
                    "sol": track.solo,
                    "arm": track.armed,
                    "out": track.output_id.map(|id| id.to_string()),
                    "fx": fx_abbrevs,
                }));
            }

            for bus in &p.buses {
                let fx_abbrevs: Vec<&str> = bus.effects_chain.iter().map(|e| effect_shorthand(&e.effect_type)).collect();
                strips.push(serde_json::json!({
                    "id": bus.id.to_string(),
                    "name": bus.name,
                    "type": "bus",
                    "vol": bus.volume,
                    "pan": bus.pan,
                    "mut": bus.mute,
                    "sol": bus.solo,
                    "arm": false,
                    "out": bus.output_id.map(|id| id.to_string()),
                    "fx": fx_abbrevs,
                }));
            }

            let empty_fx: Vec<&str> = vec![];
            strips.push(serde_json::json!({
                "id": "master",
                "name": "Master",
                "type": "master",
                "vol": p.master_volume,
                "pan": 0.0,
                "mut": false,
                "sol": false,
                "arm": false,
                "out": serde_json::Value::Null,
                "fx": empty_fx,
            }));

            let json = serde_json::to_string(&strips).unwrap_or_else(|_| "[]".to_string());
            self.as_mut().set_mixer_json(json);
        }
    }

    fn sync_peaks(mut self: Pin<&mut Self>) {
        let mut peaks = std::collections::HashMap::new();
        if let Some(state) = crate::ui_qt::state::get() {
            let track_peaks = state.playback.get_track_peaks();
            for (id, peak) in &track_peaks {
                peaks.insert(id.to_string(), serde_json::json!({ "l": peak.0, "r": peak.1 }));
            }
            let bus_peaks = state.playback.get_bus_peaks();
            for (id, peak) in &bus_peaks {
                peaks.insert(id.to_string(), serde_json::json!({ "l": peak.0, "r": peak.1 }));
            }
            let master = state.playback.get_master_peak();
            peaks.insert("master".to_string(), serde_json::json!({ "l": master.0, "r": master.1 }));
        }
        let json = serde_json::to_string(&peaks).unwrap_or_else(|_| "{}".to_string());
        self.as_mut().set_peaks_json(json);
    }

    fn set_volume(self: Pin<&mut Self>, strip_id: &str, value: f64) {
        let state = match crate::ui_qt::state::get() { Some(s) => s, None => return };
        let mut p = match state.project.lock() { Ok(p) => p, Err(_) => return };
        let sr = p.sample_rate;
        if strip_id == "master" {
            p.master_volume = value as f32;
        } else if let Some(track) = p.tracks.iter_mut().find(|t| t.id.to_string() == strip_id) {
            track.volume = value as f32;
        } else if let Some(bus) = p.buses.iter_mut().find(|b| b.id.to_string() == strip_id) {
            bus.volume = value as f32;
        }
        state.playback.load_project_clips(&p.tracks, &p.buses, sr);
    }

    fn set_pan(self: Pin<&mut Self>, strip_id: &str, value: f64) {
        let state = match crate::ui_qt::state::get() { Some(s) => s, None => return };
        let mut p = match state.project.lock() { Ok(p) => p, Err(_) => return };
        let sr = p.sample_rate;
        if let Some(track) = p.tracks.iter_mut().find(|t| t.id.to_string() == strip_id) {
            track.pan = value as f32;
        } else if let Some(bus) = p.buses.iter_mut().find(|b| b.id.to_string() == strip_id) {
            bus.pan = value as f32;
        }
        state.playback.load_project_clips(&p.tracks, &p.buses, sr);
    }

    fn toggle_mute(self: Pin<&mut Self>, strip_id: &str) {
        let state = match crate::ui_qt::state::get() { Some(s) => s, None => return };
        let mut p = match state.project.lock() { Ok(p) => p, Err(_) => return };
        let sr = p.sample_rate;
        if let Some(track) = p.tracks.iter_mut().find(|t| t.id.to_string() == strip_id) {
            track.mute = !track.mute;
        } else if let Some(bus) = p.buses.iter_mut().find(|b| b.id.to_string() == strip_id) {
            bus.mute = !bus.mute;
        }
        state.playback.load_project_clips(&p.tracks, &p.buses, sr);
    }

    fn toggle_solo(self: Pin<&mut Self>, strip_id: &str) {
        let state = match crate::ui_qt::state::get() { Some(s) => s, None => return };
        let mut p = match state.project.lock() { Ok(p) => p, Err(_) => return };
        let sr = p.sample_rate;
        if let Some(track) = p.tracks.iter_mut().find(|t| t.id.to_string() == strip_id) {
            track.solo = !track.solo;
        } else if let Some(bus) = p.buses.iter_mut().find(|b| b.id.to_string() == strip_id) {
            bus.solo = !bus.solo;
        }
        state.playback.load_project_clips(&p.tracks, &p.buses, sr);
    }

    fn toggle_arm(self: Pin<&mut Self>, strip_id: &str) {
        let state = match crate::ui_qt::state::get() { Some(s) => s, None => return };
        let mut p = match state.project.lock() { Ok(p) => p, Err(_) => return };
        let sr = p.sample_rate;
        if let Some(track) = p.tracks.iter_mut().find(|t| t.id.to_string() == strip_id) {
            track.armed = !track.armed;
        }
        state.playback.load_project_clips(&p.tracks, &p.buses, sr);
    }

    fn select_effect(self: Pin<&mut Self>, strip_id: &str, effect_idx: i32) {
        let state = match crate::ui_qt::state::get() { Some(s) => s, None => return };
        let p = match state.project.lock() { Ok(p) => p, Err(_) => return };
        let is_track = p.tracks.iter().any(|t| t.id.to_string() == strip_id);
        if let Ok(mut t) = state.selected_effect_target.lock() {
            *t = Some(strip_id.to_string());
        }
        if let Ok(mut e) = state.selected_effect_index.lock() {
            *e = Some(effect_idx);
        }
        state.selected_effect_is_track.store(is_track, Ordering::Relaxed);
    }
}
