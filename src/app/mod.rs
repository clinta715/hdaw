pub(crate) mod input;
pub(crate) mod drag;
pub(crate) mod io;
pub(crate) mod callbacks;

use crate::audio::engine::AudioEngine;
use crate::audio::playback::PlaybackManager;
use crate::project::clip::AudioClip;
use crate::project::track::Track;
use crate::project::undo::{EditCommand, UndoStack};
use crate::project::Project;
use crate::ui::main_window::{MainWindow, PoolEntry};
use crate::ui::timeline::{
    compute_cursor_type, compute_ruler_ticks, samples_to_pixels,
    sync_project_to_timeline_with_waveforms, sync_selection,
};
use crate::utils::format_time;
use crate::utils::waveform::WaveformPeaks;
use slint::{ComponentHandle, Model};
use slint::Timer as SlintTimer;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tracing::info;
use uuid::Uuid;

pub struct HdawApp {
    window: MainWindow,
    project: Arc<Mutex<Project>>,
    undo_stack: Arc<Mutex<UndoStack>>,
    audio_engine: Arc<Mutex<AudioEngine>>,
    playback: PlaybackManager,
    playhead_timer: SlintTimer,
    selected_clips: Arc<Mutex<HashSet<Uuid>>>,
    clipboard: Arc<Mutex<Vec<AudioClip>>>,
    waveform_peaks: Arc<Mutex<HashMap<String, WaveformPeaks>>>,
    selected_track_id: Arc<Mutex<Option<Uuid>>>,
    selected_bus_id: Arc<Mutex<Option<Uuid>>>,
    drag_state: Arc<Mutex<Option<drag::DragState>>>,
    automation_drag: Arc<Mutex<Option<drag::AutomationDragState>>>,
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

        let window = MainWindow::new().expect("Failed to create main window");

        let selected_clips = Arc::new(Mutex::new(HashSet::new()));
        let clipboard = Arc::new(Mutex::new(Vec::new()));
        let waveform_peaks = Arc::new(Mutex::new(waveform_peaks));
        let selected_track_id = Arc::new(Mutex::new(None));
        let selected_bus_id = Arc::new(Mutex::new(None));
        let drag_state = Arc::new(Mutex::new(None));
        let automation_drag = Arc::new(Mutex::new(None));

        let app = Self {
            window,
            project: project.clone(),
            undo_stack: undo_stack.clone(),
            audio_engine,
            playback: playback.clone(),
            playhead_timer: SlintTimer::default(),
            selected_clips: selected_clips.clone(),
            clipboard: clipboard.clone(),
            waveform_peaks: waveform_peaks.clone(),
            selected_track_id: selected_track_id.clone(),
            selected_bus_id: selected_bus_id.clone(),
            drag_state: drag_state.clone(),
            automation_drag: automation_drag.clone(),
        };

        {
            let p = app.project.lock().unwrap();
            app.playback.load_project_clips(&p.tracks, &p.buses, p.sample_rate);
        }

        callbacks::setup_all_callbacks(
            &app.window,
            project.clone(),
            undo_stack.clone(),
            playback.clone(),
            waveform_peaks.clone(),
            selected_clips.clone(),
            clipboard.clone(),
            drag_state.clone(),
            selected_track_id.clone(),
            selected_bus_id.clone(),
            automation_drag.clone(),
        );

        if let Ok(mut engine) = app.audio_engine.lock() {
            engine.set_playback(playback.clone());
            if let Err(e) = engine.start() {
                tracing::error!("Failed to start audio engine: {}", e);
            }
            if let Err(e) = engine.start_input(input_tx) {
                tracing::warn!("Failed to start input stream (recording unavailable): {}", e);
            }
        }

        app.sync_timeline();
        app.sync_pool();

        info!("HDAW initialized successfully");
        app
    }

    pub fn run(self) {
        info!("Running HDAW main loop");

        let playback = self.playback.clone();
        let window_weak = self.window.as_weak();
        let undo_stack = self.undo_stack.clone();
        let project = self.project.clone();
        let waveform_peaks = self.waveform_peaks.clone();
        let selected_clips = self.selected_clips.clone();
        let clipboard = self.clipboard.clone();
        let selected_track_id = self.selected_track_id.clone();

        self.playhead_timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(16), move || {
            let w = match window_weak.upgrade() { Some(w) => w, None => return };
            let pos = playback.get_position();
            let sr = 44100u32;
            let pps = w.get_pixels_per_second();
            let x = samples_to_pixels(pos, pps, sr);
            w.set_playhead_x(x);
            w.set_is_playing(playback.is_playing());
            w.set_time_display(format_time(pos as f64 / sr as f64).into());
            let (bpm, ts_num, ts_den) = if let Ok(p) = project.lock() {
                w.set_bpm_display(format!("{:.1} BPM", p.bpm).into());
                w.set_time_sig_display(format!("{}/{}", p.time_signature.0, p.time_signature.1).into());
                (p.bpm, p.time_signature.0, p.time_signature.1)
            } else {
                (120.0, 4u8, 4u8)
            };
            let visible_w = w.get_timeline_visible_width().max(100.0);
            let scroll_x = w.get_timeline_scroll_x();
            let playhead_visual = x - scroll_x;
            if playhead_visual > visible_w * 0.8 && playback.is_playing() {
                w.set_timeline_scroll_x((x - visible_w * 0.2).max(0.0));
            }
            let ticks = compute_ruler_ticks(pps, scroll_x, visible_w, bpm, ts_num, ts_den, w.get_snap_enabled(), w.get_snap_mode(), w.get_snap_param(), sr);
            w.set_ruler_ticks(slint::ModelRc::new(slint::VecModel::from(ticks)));

            // Peak meters
            let track_peaks = playback.get_track_peaks();
            let bus_peaks = playback.get_bus_peaks();
            let (master_l, master_r) = playback.get_master_peak();
            w.set_master_peak_l(master_l);
            w.set_master_peak_r(master_r);
            {
                let tracks_model = w.get_tracks();
                let mut updated: Vec<crate::ui::main_window::TrackInfo> = Vec::new();
                for i in 0..tracks_model.row_count() {
                    if let Some(mut t) = tracks_model.row_data(i) {
                        if let Ok(id) = Uuid::parse_str(t.id.as_str()) {
                            if let Some((pl, pr)) = track_peaks.get(&id) { t.peak_l = *pl; t.peak_r = *pr; }
                        }
                        updated.push(t);
                    }
                }
                w.set_tracks(slint::ModelRc::new(slint::VecModel::from(updated)));
            }
            {
                let buses_model = w.get_buses();
                let mut updated: Vec<crate::ui::main_window::BusInfo> = Vec::new();
                for i in 0..buses_model.row_count() {
                    if let Some(mut b) = buses_model.row_data(i) {
                        if let Ok(id) = Uuid::parse_str(b.id.as_str()) {
                            if let Some((pl, pr)) = bus_peaks.get(&id) { b.peak_l = *pl; b.peak_r = *pr; }
                        }
                        updated.push(b);
                    }
                }
                w.set_buses(slint::ModelRc::new(slint::VecModel::from(updated)));
            }

            // Compressor GR
            let gr_target = w.get_selected_effect_target();
            if !gr_target.is_empty() && w.get_effect_editor_title().as_str() == "Compressor" {
                if let Ok(id) = Uuid::parse_str(gr_target.as_str()) {
                    let is_track = w.get_selected_effect_is_track();
                    let idx = w.get_selected_effect_index();
                    let gr = playback.get_compressor_gr(id, is_track, idx as usize);
                    w.set_compressor_gr(gr);
                }
            }

            // Cursor
            let mx = w.get_pointer_x();
            let my = w.get_pointer_y();
            let alt = { #[cfg(target_os = "windows")] { input::app_has_focus() && unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(0x12) as u32 & 0x8000 != 0 } } #[cfg(not(target_os = "windows"))] { false } };
            let scroll_x_cursor = w.get_timeline_scroll_x();
            let abs_x = mx + scroll_x_cursor;
            let base = compute_cursor_type(&w.get_clips(), abs_x, my, alt);
            let pressed = w.get_pointer_pressed();
            w.set_cursor_type(if base == 4 && pressed { 6 } else { base });

            // Keyboard shortcuts
            #[cfg(target_os = "windows")]
            {
                if input::app_has_focus() {
                use crate::app::input::keyboard::{SPACE, Z, Y, L, T, M, HOME, END, OEM_PLUS, OEM_MINUS, LEFT, RIGHT, CONTROL, DELETE, C, V, A, R, P, S, N, ESCAPE};
                input::keyboard::KEYS.with(|k| {
                    let mut keys = k.borrow_mut();
                    let ctrl = unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(CONTROL as i32) as u32 & 0x8000 != 0 };
                    if keys.was_pressed(SPACE) { if playback.is_playing() { playback.set_playing(false); } else { playback.set_playing(true); } }
                    if ctrl && keys.was_pressed(Z) { if let Ok(mut p) = project.lock() { if let Ok(mut stack) = undo_stack.lock() { stack.undo(&mut p); if let Ok(cache) = waveform_peaks.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } w.set_can_undo(stack.can_undo()); w.set_can_redo(stack.can_redo()); } } }
                    if ctrl && keys.was_pressed(Y) { if let Ok(mut p) = project.lock() { if let Ok(mut stack) = undo_stack.lock() { stack.redo(&mut p); if let Ok(cache) = waveform_peaks.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } w.set_can_undo(stack.can_undo()); w.set_can_redo(stack.can_redo()); } } }
                    if keys.was_pressed(L) { let current = playback.is_loop_enabled(); playback.set_loop_enabled(!current); }
                    if ctrl && keys.was_pressed(T) { if let Ok(mut p) = project.lock() { let track = Track::new(format!("Track {}", p.tracks.len() + 1)); let snapshot = track.clone(); let index = p.tracks.len(); p.tracks.push(track); if let Ok(mut stack) = undo_stack.lock() { stack.push(EditCommand::AddTrack { snapshot, index }); } if let Ok(cache) = waveform_peaks.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
                    if keys.was_pressed(DELETE) { if let Ok(mut sel) = selected_clips.lock() { let ids: Vec<Uuid> = sel.iter().copied().collect(); if !ids.is_empty() { drop(sel); if let Ok(mut p) = project.lock() { let mut track_groups: HashMap<Uuid, Vec<crate::project::undo::ClipSnapshot>> = HashMap::new(); for track in p.tracks.iter() { for clip in track.clips.iter() { if ids.contains(&clip.id) { track_groups.entry(track.id).or_default().push(clip.clone()); } } } for (tid, clips) in track_groups { if let Some(track) = p.tracks.iter_mut().find(|t| t.id == tid) { track.clips.retain(|c| !clips.iter().any(|cl| cl.id == c.id)); } if let Ok(mut stack) = undo_stack.lock() { stack.push(EditCommand::DeleteClips { track_id: tid, clips }); } } playback.load_project_clips(&p.tracks, &p.buses, p.sample_rate); if let Ok(cache) = waveform_peaks.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } if let Ok(mut sel) = selected_clips.lock() { sel.clear(); } } } }
                    if ctrl && keys.was_pressed(C) { if let Ok(mut sel) = selected_clips.lock() { if let Ok(mut cb) = clipboard.lock() { cb.clear(); if let Ok(p) = project.lock() { for track in p.tracks.iter() { for clip in track.clips.iter() { if sel.contains(&clip.id) { cb.push(clip.clone()); } } } } } } }
                    if ctrl && keys.was_pressed(V) { if let Ok(cb) = clipboard.lock() { let clips = cb.clone(); if !clips.is_empty() { drop(cb); let mut created: Vec<(Uuid, AudioClip)> = Vec::new(); if let Ok(mut p) = project.lock() { let offset = pos; let selected_id = selected_track_id.lock().ok().and_then(|s| *s); let target_track_id = selected_id.or_else(|| p.tracks.first().map(|t| t.id)); for clip in &clips { let mut new_clip = clip.clone(); new_clip.id = Uuid::new_v4(); new_clip.position = clip.position + offset; if let Some(tid) = target_track_id { if let Some(track) = p.tracks.iter_mut().find(|t| t.id == tid) { created.push((tid, new_clip.clone())); track.add_clip(new_clip); } } } if !created.is_empty() { if let Ok(mut stack) = undo_stack.lock() { for (tid, clip) in created.iter() { stack.push(EditCommand::AddClip { track_id: *tid, clip: clip.clone() }); } } } playback.load_project_clips(&p.tracks, &p.buses, p.sample_rate); if let Ok(cache) = waveform_peaks.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } } } }
                    if ctrl && keys.was_pressed(A) { if let Ok(mut sel) = selected_clips.lock() { sel.clear(); if let Ok(p) = project.lock() { for track in p.tracks.iter() { for clip in track.clips.iter() { sel.insert(clip.id); } } } drop(sel); if let Ok(sel) = selected_clips.lock() { sync_selection(&w, &sel); } } }
                    if keys.was_pressed(M) { let sid = w.get_selected_track_id(); if !sid.is_empty() { if let Ok(id) = uuid::Uuid::parse_str(&sid) { if let Ok(mut p) = project.lock() { if let Some(track) = p.tracks.iter_mut().find(|t| t.id == id) { track.mute = !track.mute; playback.update_track_params(track.id, track.volume, track.pan, track.mute, track.solo); if let Ok(cache) = waveform_peaks.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } } } } }
                    if keys.was_pressed(HOME) { playback.set_position(0); }
                    if keys.was_pressed(END) { if let Ok(p) = project.lock() { let max_pos = p.tracks.iter().flat_map(|t| t.clips.iter()).map(|c| c.position + c.length).max().unwrap_or(0); playback.set_position(max_pos); } }
                    if keys.was_pressed(S) { let current = w.get_tool_mode(); w.set_tool_mode(if current == 1 { 0 } else { 1 }); }
                    if keys.was_pressed(N) { let current = w.get_snap_enabled(); w.set_snap_enabled(!current); }
                    if keys.was_pressed(ESCAPE) { w.set_tool_mode(0); }
                    if keys.was_pressed(OEM_PLUS) { let old_pps = w.get_pixels_per_second(); if old_pps < 5000.0 { let new_pps = old_pps * 1.25; let mid = scroll_x + visible_w / 2.0; w.set_pixels_per_second(new_pps); w.set_timeline_scroll_x((mid * (new_pps / old_pps) - visible_w / 2.0).max(0.0)); } }
                    if keys.was_pressed(OEM_MINUS) { let old_pps = w.get_pixels_per_second(); if old_pps > 2.0 { let new_pps = old_pps * 0.8; let mid = scroll_x + visible_w / 2.0; w.set_pixels_per_second(new_pps); w.set_timeline_scroll_x((mid * (new_pps / old_pps) - visible_w / 2.0).max(0.0)); } }
                    if keys.was_pressed(LEFT) { let step = (sr as u64 / 10).max(1); playback.set_position(pos.saturating_sub(step)); }
                    if keys.was_pressed(RIGHT) { let step = (sr as u64 / 10).max(1); playback.set_position(pos + step); }
                    if keys.was_pressed(R) { let p = match project.lock() { Ok(p) => p, Err(_) => return }; let armed_ids: HashSet<Uuid> = p.tracks.iter().filter(|t| t.armed).map(|t| t.id).collect(); let sr2 = p.sample_rate; drop(p); if !armed_ids.is_empty() { let rpos = playback.get_position(); if !playback.is_playing() { playback.set_playing(true); } playback.start_recording(&armed_ids, rpos, sr2); w.set_is_recording(true); } }
                    if keys.was_pressed(P) { let visible = w.get_pool_visible(); w.set_pool_visible(!visible); }
                });
                }
            }

            if let Ok(stack) = undo_stack.lock() { w.set_can_undo(stack.can_undo()); w.set_can_redo(stack.can_redo()); }
        });

        self.window.run().unwrap();
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

    fn sync_timeline(&self) {
        if let Ok(project) = self.project.lock() { if let Ok(cache) = self.waveform_peaks.lock() { sync_project_to_timeline_with_waveforms(&project, &self.window, &cache); } }
        self.sync_pool();
        if let Ok(p) = self.project.lock() {
            self.window.set_master_volume(p.master_volume);
            self.window.set_master_pan(p.master_pan);
            self.window.set_master_mute(p.master_mute);
        }
    }

    fn sync_pool(&self) {
        use slint::ModelRc;
        use slint::VecModel;
        if let Ok(project) = self.project.lock() {
            let entries: Vec<PoolEntry> = crate::audio::pool::sync(&project)
                .into_iter()
                .enumerate()
                .map(|(i, info)| {
                    let rate_str = if info.sample_rate % 1000 == 0 { format!("{:.1}kHz", info.sample_rate as f64 / 1000.0) } else { format!("{}Hz", info.sample_rate) };
                    let ch_str = if info.channels == 1 { "Mono".to_string() } else if info.channels == 2 { "Stereo".to_string() } else { format!("{}ch", info.channels) };
                    let bit_str = if info.bit_depth == 32 { "32-bit".to_string() } else { format!("{}-bit", info.bit_depth) };
                    PoolEntry { name: info.name.into(), info: format!("{} · {} · {} · {:.1}s", rate_str, ch_str, bit_str, info.duration_secs).into(), usage: info.usage_count as i32, path: info.path.into(), idx: i as i32 }
                })
                .collect();
            self.window.set_pool_entries(ModelRc::new(VecModel::from(entries)));
        }
    }
}
