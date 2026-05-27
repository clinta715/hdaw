use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use crate::audio::playback::PlaybackManager;
use crate::project::undo::UndoStack;
use crate::project::clip::AudioClip;
use crate::project::Project;
use crate::utils::waveform::WaveformPeaks;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct AppState {
    pub project: Arc<Mutex<Project>>,
    pub playback: PlaybackManager,
    pub undo_stack: Arc<Mutex<UndoStack>>,
    pub waveform_peaks: Arc<Mutex<HashMap<String, WaveformPeaks>>>,
    pub pool_visible: Arc<AtomicBool>,
    pub timeline_dirty: Arc<AtomicBool>,
    pub selected_effect_target: Arc<Mutex<Option<String>>>,
    pub selected_effect_index: Arc<Mutex<Option<i32>>>,
    pub selected_effect_is_track: Arc<AtomicBool>,
    pub selected_clips: Arc<Mutex<HashSet<Uuid>>>,
    pub clipboard: Arc<Mutex<Vec<AudioClip>>>,
    pub selected_track_id: Arc<Mutex<Option<Uuid>>>,
    pub tool_mode: Arc<Mutex<i32>>,
    pub snap_enabled: Arc<AtomicBool>,
    pub auto_scroll: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(
        project: Arc<Mutex<Project>>,
        playback: PlaybackManager,
        undo_stack: Arc<Mutex<UndoStack>>,
        waveform_peaks: Arc<Mutex<HashMap<String, WaveformPeaks>>>,
        selected_clips: Arc<Mutex<HashSet<Uuid>>>,
        clipboard: Arc<Mutex<Vec<AudioClip>>>,
        selected_track_id: Arc<Mutex<Option<Uuid>>>,
    ) -> Self {
        Self {
            project,
            playback,
            undo_stack,
            waveform_peaks,
            pool_visible: Arc::new(AtomicBool::new(false)),
            timeline_dirty: Arc::new(AtomicBool::new(false)),
            selected_effect_target: Arc::new(Mutex::new(None)),
            selected_effect_index: Arc::new(Mutex::new(None)),
            selected_effect_is_track: Arc::new(AtomicBool::new(true)),
            selected_clips,
            clipboard,
            selected_track_id,
            tool_mode: Arc::new(Mutex::new(0)),
            snap_enabled: Arc::new(AtomicBool::new(true)),
            auto_scroll: Arc::new(AtomicBool::new(true)),
        }
    }
}

static APP_STATE: OnceLock<AppState> = OnceLock::new();

pub fn init(state: AppState) {
    let _ = APP_STATE.set(state);
}

pub fn get() -> Option<&'static AppState> {
    APP_STATE.get()
}

pub fn on_play() {
    if let Some(state) = APP_STATE.get() {
        state.playback.set_playing(true);
    }
}

pub fn on_stop() {
    let state = match APP_STATE.get() { Some(s) => s, None => return };
    let pb = &state.playback;
    let record_data = pb.stop_recording();
    pb.set_playing(false);
    pb.set_position(0);
    if let Some((recorded_buffers, record_start_pos)) = record_data {
        let sr = state.project.lock().map(|p| p.sample_rate).unwrap_or(44100);
        let project = state.project.clone();
        let playback = pb.clone();
        tokio::spawn(async move {
            use crate::project::clip::AudioClip;
            let mut new_clips: Vec<(uuid::Uuid, AudioClip)> = Vec::new();
            let recordings_dir = std::path::PathBuf::from("recordings");
            let _ = std::fs::create_dir_all(&recordings_dir);
            for (track_id, samples) in recorded_buffers {
                if samples.is_empty() { continue; }
                let filename = format!("recording_{}.wav", uuid::Uuid::new_v4());
                let path = recordings_dir.join(&filename);
                match crate::audio::loader::write_wav(&path, &samples, 2, sr) {
                    Ok(()) => {
                        let clip = AudioClip::new(
                            path.to_string_lossy().to_string(),
                            format!("Take {}", uuid::Uuid::new_v4().to_string().chars().take(4).collect::<String>()),
                            record_start_pos,
                            samples.len() as u64 / 2,
                        );
                        new_clips.push((track_id, clip));
                    }
                    Err(e) => { tracing::error!("Failed to write recording WAV: {}", e); }
                }
            }
            if !new_clips.is_empty() {
                let mut p = match project.lock() { Ok(p) => p, Err(_) => return };
                for (track_id, clip) in &new_clips {
                    if let Some(track) = p.tracks.iter_mut().find(|t| t.id == *track_id) {
                        track.add_clip(clip.clone());
                    }
                }
                playback.load_project_clips(&p.tracks, &p.buses, p.sample_rate);
                drop(p);
            }
        });
    }
}

pub fn on_toggle_record(start: bool) {
    let state = match APP_STATE.get() { Some(s) => s, None => return };
    let pb = &state.playback;
    if start {
        let p = match state.project.lock() { Ok(p) => p, Err(_) => return };
        let armed_ids: std::collections::HashSet<uuid::Uuid> = p.tracks.iter().filter(|t| t.armed).map(|t| t.id).collect();
        let sr = p.sample_rate;
        drop(p);
        if armed_ids.is_empty() { return; }
        let pos = pb.get_position();
        if !pb.is_playing() { pb.set_playing(true); }
        pb.start_recording(&armed_ids, pos, sr);
    } else {
        pb.stop_recording();
    }
}

pub fn on_import_file() {
    let state = match APP_STATE.get() { Some(s) => s, None => return };
    use crate::audio::loader;
    use crate::project::track::Track;
    use crate::project::undo::EditCommand;
    use crate::project::clip::AudioClip;
    let path = match rfd::FileDialog::new().add_filter("WAV", &["wav"]).pick_file() { Some(p) => p, None => return };
    let path_str = path.to_string_lossy().to_string();
    let buffer = match loader::load_wav(&path) { Ok(b) => b, Err(e) => { tracing::error!("Failed to load {}: {}", path_str, e); return; } };
    if let Ok(mut cache) = state.waveform_peaks.lock() { cache.insert(path_str.clone(), WaveformPeaks::generate(&buffer, 2000)); }
    let mut p = match state.project.lock() { Ok(p) => p, Err(_) => return };
    let sr = p.sample_rate;
    let track_id = if let Some(first) = p.tracks.first() { first.id } else { let t = Track::new("Audio 1".into()); let id = t.id; p.tracks.push(t); id };
    let clip = AudioClip::new(path_str.clone(), path_str.split('/').last().unwrap_or("Clip").into(), 0, buffer.samples.len() as u64 / 2);
    if let Some(track) = p.tracks.iter_mut().find(|t| t.id == track_id) {
        track.add_clip(clip);
        if let Ok(mut stack) = state.undo_stack.lock() {
            if let Some(clip_snapshot) = track.clips.last().cloned() {
                stack.push(EditCommand::AddClip { track_id, clip: clip_snapshot });
            }
        }
    }
    state.playback.load_project_clips(&p.tracks, &p.buses, sr);
    state.timeline_dirty.store(true, Ordering::Relaxed);
}
