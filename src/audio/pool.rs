use crate::audio::loader;
use crate::project::Project;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum FileOrigin {
    Imported,
    Recorded,
}

#[derive(Debug, Clone)]
pub struct AudioFileInfo {
    pub path: String,
    pub name: String,
    pub file_size: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: u16,
    pub duration_secs: f64,
    pub usage_count: usize,
    pub origin: FileOrigin,
}

pub fn sync(project: &Project) -> Vec<AudioFileInfo> {
    let mut path_counts: HashMap<&str, usize> = HashMap::new();
    for track in &project.tracks {
        for clip in &track.clips {
            if !clip.source_path.is_empty() {
                *path_counts.entry(clip.source_path.as_str()).or_default() += 1;
            }
        }
    }

    let mut entries: Vec<(String, AudioFileInfo)> = Vec::new();
    for (path, count) in path_counts {
        let name = std::path::Path::new(path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let origin = if path.contains("recordings") || path.contains("recording_") {
            FileOrigin::Recorded
        } else {
            FileOrigin::Imported
        };

        let metadata = loader::load_wav_metadata(std::path::Path::new(path)).ok();
        let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        let info = AudioFileInfo {
            path: path.to_string(),
            name,
            file_size,
            sample_rate: metadata.as_ref().map(|m| m.sample_rate).unwrap_or(0),
            channels: metadata.as_ref().map(|m| m.channels).unwrap_or(0),
            bit_depth: metadata.as_ref().map(|m| m.bit_depth).unwrap_or(0),
            duration_secs: metadata.as_ref().map(|m| m.duration_secs).unwrap_or(0.0),
            usage_count: count,
            origin,
        };

        entries.push((info.name.clone(), info));
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries.into_iter().map(|(_, info)| info).collect()
}
