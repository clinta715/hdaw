use crate::project::Project;
use crate::utils::waveform::WaveformPeaks;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub(crate) fn rebuild_waveform_cache(
    project: &Project,
    cache: &Arc<Mutex<HashMap<String, WaveformPeaks>>>,
) {
    use crate::audio::loader;
    let mut map = match cache.lock() { Ok(m) => m, Err(_) => return };
    map.clear();
    for track in &project.tracks {
        for clip in &track.clips {
            let key = if clip.source_path.is_empty() {
                "_test_tone_".to_string()
            } else {
                clip.source_path.clone()
            };
            if map.contains_key(&key) { continue; }
            let buffer = if clip.source_path.is_empty() {
                Some(loader::generate_test_tone(project.sample_rate, 2.0))
            } else {
                let path = std::path::Path::new(&clip.source_path);
                if path.exists() {
                    loader::load_wav(path).ok()
                } else {
                    None
                }
            };
            if let Some(buf) = buffer {
                map.insert(key, WaveformPeaks::generate(&buf, 2000));
            }
        }
    }
}
