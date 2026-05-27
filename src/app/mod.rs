pub(crate) mod drag;
pub(crate) mod io;

use crate::audio::engine::AudioEngine;
use crate::audio::playback::PlaybackManager;
use crate::project::clip::AudioClip;
use crate::project::track::Track;
use crate::project::undo::UndoStack;
use crate::project::Project;
use crate::utils::waveform::WaveformPeaks;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tracing::info;
use uuid::Uuid;

pub struct HdawApp {
    pub project: Arc<Mutex<Project>>,
    pub undo_stack: Arc<Mutex<UndoStack>>,
    pub audio_engine: Arc<Mutex<AudioEngine>>,
    pub playback: PlaybackManager,
    pub selected_clips: Arc<Mutex<HashSet<Uuid>>>,
    pub clipboard: Arc<Mutex<Vec<AudioClip>>>,
    pub waveform_peaks: Arc<Mutex<HashMap<String, WaveformPeaks>>>,
    pub selected_track_id: Arc<Mutex<Option<Uuid>>>,
    pub selected_bus_id: Arc<Mutex<Option<Uuid>>>,
    pub drag_state: Arc<Mutex<Option<drag::DragState>>>,
    pub automation_drag: Arc<Mutex<Option<drag::AutomationDragState>>>,
}

impl HdawApp {
    pub fn new() -> Self {
        info!("Initializing HDAW application");

        let mut project = Project::new();
        Self::populate_test_project(&mut project);

        let test_tone_buffer = crate::audio::loader::generate_test_tone(project.sample_rate, 2.0);
        let mut waveform_peaks: HashMap<String, WaveformPeaks> = HashMap::new();
        waveform_peaks.insert("_test_tone_".to_string(), WaveformPeaks::generate(&test_tone_buffer, 2000));

        let project = Arc::new(Mutex::new(project));
        let undo_stack = Arc::new(Mutex::new(UndoStack::new()));
        let audio_engine = Arc::new(Mutex::new(AudioEngine::new()));
        let (playback, input_tx) = PlaybackManager::new_with_recording(44100);

        let selected_clips = Arc::new(Mutex::new(HashSet::new()));
        let clipboard = Arc::new(Mutex::new(Vec::new()));
        let waveform_peaks = Arc::new(Mutex::new(waveform_peaks));
        let selected_track_id = Arc::new(Mutex::new(None));
        let selected_bus_id = Arc::new(Mutex::new(None));
        let drag_state = Arc::new(Mutex::new(None));
        let automation_drag = Arc::new(Mutex::new(None));

        {
            let p = project.lock().unwrap();
            playback.load_project_clips(&p.tracks, &p.buses, p.sample_rate);
        }

        if let Ok(mut engine) = audio_engine.lock() {
            engine.set_playback(playback.clone());
            if let Err(e) = engine.start() {
                tracing::error!("Failed to start audio engine: {}", e);
            }
            if let Err(e) = engine.start_input(input_tx) {
                tracing::warn!("Failed to start input stream (recording unavailable): {}", e);
            }
        }

        info!("HDAW initialized successfully");
        Self {
            project,
            undo_stack,
            audio_engine,
            playback,
            selected_clips,
            clipboard,
            waveform_peaks,
            selected_track_id,
            selected_bus_id,
            drag_state,
            automation_drag,
        }
    }

    pub fn playback(&self) -> PlaybackManager {
        self.playback.clone()
    }

    #[cfg(feature = "qt")]
    pub fn app_state(&self) -> crate::ui_qt::state::AppState {
        crate::ui_qt::state::AppState::new(
            self.project.clone(),
            self.playback.clone(),
            self.undo_stack.clone(),
            self.waveform_peaks.clone(),
            self.selected_clips.clone(),
            self.clipboard.clone(),
            self.selected_track_id.clone(),
        )
    }

    pub fn run(self) {
        info!("HDAW running");
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    }

    fn populate_test_project(project: &mut Project) {
        let sr = project.sample_rate as u64;
        let track = Track {
            id: Uuid::new_v4(), name: "Audio 1".into(), color: (80, 160, 255),
            volume: 0.8, pan: 0.0, mute: false, solo: false,
            armed: false, input_monitoring: false, output_id: None, sends: vec![],
            clips: vec![AudioClip {
                id: Uuid::new_v4(), source_path: String::new(), name: "Test Tone".into(),
                position: sr / 4, offset: 0, length: sr, fade_in: 0, fade_out: 0,
                gain: 1.0, time_stretch: None, pitch_shift: None, color: (80, 160, 255),
            }],
            effects_chain: vec![], automation: HashMap::new(), selected_automation_param: None,
        };
        project.tracks.push(track);
    }
}
