#![cfg(feature = "qt")]

use std::pin::Pin;

#[cxx_qt::bridge(namespace = "ui_qt::effect_editor")]
pub mod effect_bridge {
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(f64, compressor_gr)]
        #[qproperty(String, effect_json)]
        type EffectEditor = super::EffectEditorRust;

        #[qinvokable]
        fn refresh(self: Pin<&mut EffectEditor>);
        #[qinvokable]
        fn set_param(self: Pin<&mut EffectEditor>, param_name: &str, value: f64);
        #[qinvokable]
        fn toggle_bypass(self: Pin<&mut EffectEditor>);
        #[qinvokable]
        fn sync_gr(self: Pin<&mut EffectEditor>);
    }
}

#[derive(Default)]
pub struct EffectEditorRust {
    compressor_gr: f64,
    effect_json: String,
}

fn param_display(effect_type: &str, name: &str, value: f32) -> (String, f64, f64, String) {
    let (label, min, max, display) = match (effect_type, name) {
        ("Equalizer", "eq_low_freq") => ("Low Freq".to_string(), 20.0, 500.0, format!("{:.0}Hz", value)),
        ("Equalizer", "eq_low_gain") => ("Low Gain".to_string(), -12.0, 12.0, format!("{:.1}dB", value)),
        ("Equalizer", "eq_mid_freq") => ("Mid Freq".to_string(), 200.0, 5000.0, format!("{:.0}Hz", value)),
        ("Equalizer", "eq_mid_gain") => ("Mid Gain".to_string(), -12.0, 12.0, format!("{:.1}dB", value)),
        ("Equalizer", "eq_mid_q") => ("Q".to_string(), 0.1, 10.0, format!("{:.2}", value)),
        ("Equalizer", "eq_high_freq") => ("High Freq".to_string(), 2000.0, 20000.0, format!("{:.0}Hz", value)),
        ("Equalizer", "eq_high_gain") => ("High Gain".to_string(), -12.0, 12.0, format!("{:.1}dB", value)),
        ("Compressor", "comp_threshold") => ("Threshold".to_string(), -60.0, 0.0, format!("{:.1}dB", value)),
        ("Compressor", "comp_ratio") => ("Ratio".to_string(), 1.0, 20.0, format!("{:.1}:1", value)),
        ("Compressor", "comp_attack") => ("Attack".to_string(), 0.001, 1.0, format!("{:.1}ms", value * 1000.0)),
        ("Compressor", "comp_release") => ("Release".to_string(), 0.01, 2.0, format!("{:.0}ms", value * 1000.0)),
        ("Compressor", "comp_makeup") => ("Makeup".to_string(), -20.0, 20.0, format!("{:.1}dB", value)),
        ("Reverb", "reverb_room_size") => ("Room Size".to_string(), 0.0, 1.0, format!("{:.0}%", value * 100.0)),
        ("Reverb", "reverb_damping") => ("Damping".to_string(), 0.0, 1.0, format!("{:.0}%", value * 100.0)),
        ("Reverb", "reverb_wet_dry") => ("Wet/Dry".to_string(), 0.0, 1.0, format!("{:.0}%", value * 100.0)),
        ("Delay", "delay_time") => ("Time".to_string(), 0.001, 5.0, format!("{:.0}ms", value * 1000.0)),
        ("Delay", "delay_feedback") => ("Feedback".to_string(), 0.0, 0.95, format!("{:.0}%", value * 100.0)),
        ("Delay", "delay_mix") => ("Mix".to_string(), 0.0, 1.0, format!("{:.0}%", value * 100.0)),
        _ => (name.to_string(), 0.0, 1.0, format!("{:.2}", value)),
    };
    (label, min, max, display)
}

impl effect_bridge::EffectEditor {
    fn refresh(mut self: Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            let t_id = state.selected_effect_target.lock().ok().and_then(|g| g.clone());
            let e_idx = state.selected_effect_index.lock().ok().and_then(|g| *g);
            let is_track = state.selected_effect_is_track.load(std::sync::atomic::Ordering::Relaxed);

            let p = match state.project.lock() {
                Ok(p) => p,
                Err(_) => {
                    self.as_mut().set_effect_json(String::from("{}"));
                    return;
                }
            };

            if let Some(track_id) = t_id {
                if let Some(idx) = e_idx {
                    let chain_json: Vec<serde_json::Value> = if is_track {
                        if let Some(track) = p.tracks.iter().find(|t| t.id.to_string() == track_id) {
                            track.effects_chain.iter().enumerate().map(|(i, e)| serde_json::json!({
                                "name": e.effect_type,
                                "idx": i,
                                "selected": i as i32 == idx,
                            })).collect()
                        } else {
                            vec![]
                        }
                    } else {
                        if let Some(bus) = p.buses.iter().find(|b| b.id.to_string() == track_id) {
                            bus.effects_chain.iter().enumerate().map(|(i, e)| serde_json::json!({
                                "name": e.effect_type,
                                "idx": i,
                                "selected": i as i32 == idx,
                            })).collect()
                        } else {
                            vec![]
                        }
                    };

                    let effect = if is_track {
                        p.tracks.iter().find(|t| t.id.to_string() == track_id)
                            .and_then(|t| t.effects_chain.get(idx as usize))
                    } else {
                        p.buses.iter().find(|b| b.id.to_string() == track_id)
                            .and_then(|b| b.effects_chain.get(idx as usize))
                    };

                    if let Some(effect) = effect {
                        let params: Vec<serde_json::Value> = effect.parameters.iter().map(|(name, value)| {
                            let (label, min, max, display) = param_display(&effect.effect_type, name, *value);
                            serde_json::json!({
                                "name": name,
                                "label": label,
                                "value": *value,
                                "min": min,
                                "max": max,
                                "display": display,
                            })
                        }).collect();

                        let json = serde_json::json!({
                            "title": effect.effect_type,
                            "bypassed": effect.bypass,
                            "effect_type": effect.effect_type,
                            "idx": idx,
                            "params": params,
                            "chain": chain_json,
                        });
                        self.as_mut().set_effect_json(json.to_string());
                        return;
                    }
                }
            }
            self.as_mut().set_effect_json(String::from("{}"));
        }
    }

    fn set_param(mut self: Pin<&mut Self>, param_name: &str, value: f64) {
        if let Some(state) = crate::ui_qt::state::get() {
            let t_id = state.selected_effect_target.lock().ok().and_then(|g| g.clone());
            let e_idx = state.selected_effect_index.lock().ok().and_then(|g| *g);
            let is_track = state.selected_effect_is_track.load(std::sync::atomic::Ordering::Relaxed);

            if let Some(track_id) = t_id {
                if let Some(idx) = e_idx {
                    if let Ok(mut p) = state.project.lock() {
                        let effect = if is_track {
                            p.tracks.iter_mut()
                                .find(|t| t.id.to_string() == track_id)
                                .and_then(|t| t.effects_chain.get_mut(idx as usize))
                        } else {
                            p.buses.iter_mut()
                                .find(|b| b.id.to_string() == track_id)
                                .and_then(|b| b.effects_chain.get_mut(idx as usize))
                        };
                        if let Some(effect) = effect {
                            effect.parameters.insert(param_name.to_string(), value as f32);
                            let sr = p.sample_rate;
                            state.playback.load_project_clips(&p.tracks, &p.buses, sr);
                        }
                    }
                }
            }
        }
    }

    fn toggle_bypass(mut self: Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            let t_id = state.selected_effect_target.lock().ok().and_then(|g| g.clone());
            let e_idx = state.selected_effect_index.lock().ok().and_then(|g| *g);
            let is_track = state.selected_effect_is_track.load(std::sync::atomic::Ordering::Relaxed);

            if let Some(track_id) = t_id {
                if let Some(idx) = e_idx {
                    if let Ok(mut p) = state.project.lock() {
                        let effect = if is_track {
                            p.tracks.iter_mut()
                                .find(|t| t.id.to_string() == track_id)
                                .and_then(|t| t.effects_chain.get_mut(idx as usize))
                        } else {
                            p.buses.iter_mut()
                                .find(|b| b.id.to_string() == track_id)
                                .and_then(|b| b.effects_chain.get_mut(idx as usize))
                        };
                        if let Some(effect) = effect {
                            effect.bypass = !effect.bypass;
                            let sr = p.sample_rate;
                            state.playback.load_project_clips(&p.tracks, &p.buses, sr);
                        }
                    }
                    self.as_mut().refresh();
                }
            }
        }
    }

    fn sync_gr(mut self: Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            let t_id = state.selected_effect_target.lock().ok().and_then(|g| g.clone());
            let e_idx = state.selected_effect_index.lock().ok().and_then(|g| *g);
            let is_track = state.selected_effect_is_track.load(std::sync::atomic::Ordering::Relaxed);

            if let (Some(track_id_str), Some(idx)) = (t_id, e_idx) {
                if let Ok(track_uuid) = uuid::Uuid::parse_str(&track_id_str) {
                    let gr = state.playback.get_compressor_gr(track_uuid, is_track, idx as usize);
                    self.as_mut().set_compressor_gr(gr as f64);

                    let refresh_key = format!("{}_{}_{}", track_id_str, idx, is_track);
                    use std::sync::Mutex;
                    use std::sync::OnceLock;
                    static LAST_SELECTION: OnceLock<Mutex<String>> = OnceLock::new();
                    let last = LAST_SELECTION.get_or_init(|| Mutex::new(String::new()));
                    if let Ok(mut last) = last.lock() {
                        if *last != refresh_key {
                            *last = refresh_key.clone();
                            self.as_mut().refresh();
                        }
                    }
                }
            }
        }
    }
}
