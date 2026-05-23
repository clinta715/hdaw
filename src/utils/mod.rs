pub mod waveform;
pub mod timestretch;

use std::path::Path;

pub fn get_audio_formats() -> Vec<&'static str> {
    vec!["wav", "mp3", "flac", "ogg", "aac", "m4a", "aiff"]
}

pub fn is_audio_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        return get_audio_formats().contains(&ext.as_str());
    }
    false
}

pub fn format_time(seconds: f64) -> String {
    let mins = (seconds / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;
    let ms = ((seconds % 1.0) * 1000.0).round() as u32;
    format!("{:02}:{:02}.{:03}", mins, secs, ms)
}

pub fn db_to_linear(db: f32) -> f32 {
    if db <= -96.0 {
        0.0
    } else {
        10.0_f32.powf(db / 20.0)
    }
}

pub fn linear_to_db(linear: f32) -> f32 {
    if linear <= 0.0 {
        -96.0
    } else {
        20.0 * linear.log10()
    }
}