#![cfg(feature = "qt")]

use std::pin::Pin;

#[cxx_qt::bridge(namespace = "ui_qt::pool")]
pub mod pool_bridge {
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(String, pool_json)]
        type PoolModel = super::PoolModelRust;

        #[qinvokable]
        fn refresh(self: Pin<&mut PoolModel>);
        #[qinvokable]
        fn insert_pool_audio(self: Pin<&mut PoolModel>, path: &str);
    }
}

#[derive(Default)]
pub struct PoolModelRust {
    pool_json: String,
}

impl pool_bridge::PoolModel {
    fn refresh(mut self: Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            let p = match state.project.lock() {
                Ok(p) => p,
                Err(_) => return,
            };
            let sr = p.sample_rate;
            let mut entries: Vec<serde_json::Value> = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for track in &p.tracks {
                for clip in &track.clips {
                    if seen.insert(clip.source_path.clone()) {
                        let duration_secs = clip.length as f64 / sr as f64;
                        entries.push(serde_json::json!({
                            "name": clip.name,
                            "info": format!("{:.1}s  \u{2022}  {}ch", duration_secs, 2),
                            "usage": 1,
                            "path": clip.source_path,
                        }));
                    }
                }
            }
            let json = serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string());
            self.as_mut().set_pool_json(json);
        }
    }

    fn insert_pool_audio(mut self: Pin<&mut Self>, _path: &str) {
        if let Some(state) = crate::ui_qt::state::get() {
            if let Ok(p) = state.project.lock() {
                let sr = p.sample_rate;
                state.playback.load_project_clips(&p.tracks, &p.buses, sr);
            }
        }
        self.as_mut().set_pool_json(String::from("[]"));
    }
}
