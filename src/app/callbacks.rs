use super::drag::{DragEdge, DragState, AutomationDragState, commit_drag};
use super::io::rebuild_waveform_cache;
use crate::audio::playback::PlaybackManager;
use crate::project::automation::AutomationPoint;
use crate::project::clip::AudioClip;
use crate::project::editing::{snap_to_grid, split_clip, SnapMode};
use crate::project::track::Track;
use crate::project::undo::{EditCommand, UndoStack, ClipSnapshot};
use crate::project::Project;
use crate::project::io;
use crate::ui::main_window::{MainWindow, ParamInfo, PoolEntry};
use crate::ui::timeline::{
    clip_at_position, is_on_left_edge, TRACK_HEIGHT, CLIP_Y_OFFSET, CLIP_HEIGHT, LANE_HEIGHT,
    is_on_right_edge, pixels_to_samples, samples_to_pixels,
    sync_project_to_timeline_with_waveforms, sync_selection, hit_test_automation_point,
};
use crate::utils::waveform::WaveformPeaks;
use slint::{ComponentHandle, Model};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub(crate) fn setup_all_callbacks(
    window: &MainWindow,
    project: Arc<Mutex<Project>>,
    undo_stack: Arc<Mutex<UndoStack>>,
    playback: PlaybackManager,
    waveform_peaks: Arc<Mutex<HashMap<String, WaveformPeaks>>>,
    selected_clips: Arc<Mutex<HashSet<Uuid>>>,
    clipboard: Arc<Mutex<Vec<AudioClip>>>,
    drag_state: Arc<Mutex<Option<DragState>>>,
    selected_track_id: Arc<Mutex<Option<Uuid>>>,
    selected_bus_id: Arc<Mutex<Option<Uuid>>>,
    automation_drag_state: Arc<Mutex<Option<AutomationDragState>>>,
) {
    let window_weak = window.as_weak();

    // ---- UNDO / REDO ----
    let p_undo = project.clone();
    let us_undo = undo_stack.clone();
    let ww_undo = window_weak.clone();
    let wp_undo = waveform_peaks.clone();
    window.on_undo(move || {
        let mut p = match p_undo.lock() { Ok(p) => p, Err(_) => return };
        let mut stack = match us_undo.lock() { Ok(s) => s, Err(_) => return };
        stack.undo(&mut p);
        drop(stack);
        if let Some(w) = ww_undo.upgrade() {
            if let Ok(cache) = wp_undo.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); }
            w.set_can_undo(us_undo.lock().map(|s| s.can_undo()).unwrap_or(false));
            w.set_can_redo(us_undo.lock().map(|s| s.can_redo()).unwrap_or(false));
        }
    });

    let p_redo = project.clone();
    let us_redo = undo_stack.clone();
    let ww_redo = window_weak.clone();
    let wp_redo = waveform_peaks.clone();
    window.on_redo(move || {
        let mut p = match p_redo.lock() { Ok(p) => p, Err(_) => return };
        let mut stack = match us_redo.lock() { Ok(s) => s, Err(_) => return };
        stack.redo(&mut p);
        drop(stack);
        if let Some(w) = ww_redo.upgrade() {
            if let Ok(cache) = wp_redo.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); }
            w.set_can_undo(us_redo.lock().map(|s| s.can_undo()).unwrap_or(false));
            w.set_can_redo(us_redo.lock().map(|s| s.can_redo()).unwrap_or(false));
        }
    });

    // ---- MENU ----
    let p_add = project.clone();
    let ww_add = window_weak.clone();
    let wp_add = waveform_peaks.clone();
    let us_add = undo_stack.clone();
    window.on_add_track(move || {
        let mut p = match p_add.lock() { Ok(p) => p, Err(_) => return };
        let track = Track::new(format!("Track {}", p.tracks.len() + 1));
        let snapshot = track.clone();
        let index = p.tracks.len();
        p.tracks.push(track);
        drop(p);
        if let Ok(mut stack) = us_add.lock() { stack.push(EditCommand::AddTrack { snapshot, index }); }
        if let Some(w) = ww_add.upgrade() {
            if let Ok(p) = p_add.lock() {
                if let Ok(cache) = wp_add.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); }
            }
        }
    });

    let p_new = project.clone();
    let pb_new = playback.clone();
    let us_new = undo_stack.clone();
    let wp_new = waveform_peaks.clone();
    let ww_new = window_weak.clone();
    window.on_new_project(move || {
        if let Ok(mut p) = p_new.lock() { *p = Project::new(); }
        if let Ok(mut cache) = wp_new.lock() { cache.clear(); }
        if let Ok(mut stack) = us_new.lock() { stack.clear(); }
        pb_new.load_project_clips(&[], &[], 44100);
        if let Some(w) = ww_new.upgrade() {
            w.set_window_title("Untitled Project - HDAW".into());
            if let Ok(p) = p_new.lock() {
                if let Ok(cache) = wp_new.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); }
            }
        }
    });

    let p_open = project.clone();
    let pb_open = playback.clone();
    let us_open = undo_stack.clone();
    let wp_open = waveform_peaks.clone();
    let ww_open = window_weak.clone();
    window.on_open_project(move || {
        let path = match rfd::FileDialog::new().add_filter("HDAW Project", &["hdaw", "ron"]).pick_file() { Some(p) => p, None => return };
        let loaded = match io::load_project(&path) { Ok(p) => p, Err(e) => { tracing::error!("Failed to load: {}", e); return; } };
        let name = loaded.name.clone();
        let sr = loaded.sample_rate;
        let tracks = loaded.tracks.clone();
        let buses = loaded.buses.clone();
        if let Ok(mut p) = p_open.lock() { *p = loaded; }
        rebuild_waveform_cache(&p_open.lock().unwrap(), &wp_open);
        if let Ok(mut stack) = us_open.lock() { stack.clear(); }
        pb_open.load_project_clips(&tracks, &buses, sr);
        if let Some(w) = ww_open.upgrade() {
            w.set_window_title(format!("{} - HDAW", name).into());
            if let Ok(p) = p_open.lock() {
                if let Ok(cache) = wp_open.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); }
            }
        }
    });

    let p_save = project.clone();
    let ww_save = window_weak.clone();
    window.on_save_project(move || {
        let path = match rfd::FileDialog::new().add_filter("HDAW Project", &["hdaw"]).set_file_name("project.hdaw").save_file() { Some(p) => p, None => return };
        if let Ok(p) = p_save.lock() {
            let name = p.name.clone();
            if let Err(e) = io::save_project_pretty(&p, &path) { tracing::error!("Failed to save: {}", e); }
            else if let Some(w) = ww_save.upgrade() { w.set_window_title(format!("{} - HDAW", name).into()); }
        }
    });

    window.on_quit(move || { std::process::exit(0); });

    // Import file
    let p_import = project.clone();
    let pb_import = playback.clone();
    let ww_import = window_weak.clone();
    let us_import = undo_stack.clone();
    let wp_import = waveform_peaks.clone();
    window.on_import_file(move || {
        use crate::audio::loader;
        let path = match rfd::FileDialog::new().add_filter("WAV", &["wav"]).pick_file() { Some(p) => p, None => return };
        let path_str = path.to_string_lossy().to_string();
        let buffer = match loader::load_wav(&path) { Ok(b) => b, Err(e) => { tracing::error!("Failed to load {}: {}", path_str, e); return; } };
        if let Ok(mut cache) = wp_import.lock() { cache.insert(path_str.clone(), WaveformPeaks::generate(&buffer, 2000)); }
        let mut p = match p_import.lock() { Ok(p) => p, Err(_) => return };
        let sr = p.sample_rate;
        let track_id = if let Some(first) = p.tracks.first() { first.id } else { let t = Track::new("Audio 1".into()); let id = t.id; p.tracks.push(t); id };
        let clip = AudioClip::new(path_str.clone(), path_str.split('/').last().unwrap_or("Clip").into(), 0, buffer.samples.len() as u64 / 2);
        if let Some(track) = p.tracks.iter_mut().find(|t| t.id == track_id) {
            track.add_clip(clip);
            if let Ok(mut stack) = us_import.lock() {
                if let Some(clip_snapshot) = track.clips.last().cloned() {
                    stack.push(EditCommand::AddClip { track_id, clip: clip_snapshot });
                }
            }
        }
        pb_import.load_project_clips(&p.tracks, &p.buses, sr);
        if let Some(w) = ww_import.upgrade() {
            if let Ok(cache) = wp_import.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); }
        }
    });

    // Delete track
    let sid_del = selected_track_id.clone();
    let p_del_track = project.clone();
    let pb_del_track = playback.clone();
    let us_del_track = undo_stack.clone();
    let wp_del_track = waveform_peaks.clone();
    let ww_del_track = window_weak.clone();
    window.on_delete_track(move || {
        let id = match sid_del.lock().ok().and_then(|s| *s) { Some(id) => id, None => return };
        let mut p = match p_del_track.lock() { Ok(p) => p, Err(_) => return };
        if let Some(idx) = p.tracks.iter().position(|t| t.id == id) {
            let snapshot = p.tracks[idx].clone();
            p.tracks.remove(idx);
            if let Ok(mut stack) = us_del_track.lock() { stack.push(EditCommand::RemoveTrack { snapshot, index: idx }); }
            pb_del_track.load_project_clips(&p.tracks, &p.buses, p.sample_rate);
            drop(p);
            if let Ok(mut s) = sid_del.lock() { *s = None; }
            if let Some(w) = ww_del_track.upgrade() {
                w.set_selected_track_id("".into());
                if let Ok(p) = p_del_track.lock() {
                    if let Ok(cache) = wp_del_track.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); }
                }
            }
        }
    });

    // Delete bus
    let sid_bus_del = selected_bus_id.clone();
    let p_del_bus = project.clone();
    let pb_del_bus = playback.clone();
    let us_del_bus = undo_stack.clone();
    let wp_del_bus = waveform_peaks.clone();
    let ww_del_bus = window_weak.clone();
    window.on_delete_bus(move |id_str| {
        let id = match Uuid::parse_str(id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_del_bus.lock() { Ok(p) => p, Err(_) => return };
        if let Some(idx) = p.buses.iter().position(|b| b.id == id) {
            let snapshot = p.buses[idx].clone();
            p.buses.remove(idx);
            if let Ok(mut stack) = us_del_bus.lock() { stack.push(EditCommand::RemoveBus { snapshot, index: idx }); }
            pb_del_bus.load_project_clips(&p.tracks, &p.buses, p.sample_rate);
            drop(p);
            if let Ok(mut s) = sid_bus_del.lock() { *s = None; }
            if let Some(w) = ww_del_bus.upgrade() {
                w.set_selected_bus_id("".into());
                if let Ok(p) = p_del_bus.lock() {
                    if let Ok(cache) = wp_del_bus.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); }
                }
            }
        }
    });

    // ---- TRANSPORT ----
    let pb_stop = playback.clone();
    let p_stop = project.clone();
    let ww_stop = window_weak.clone();
    let wp_stop = waveform_peaks.clone();
    window.on_stop(move || {
        let record_data = pb_stop.stop_recording();
        pb_stop.set_playing(false);
        pb_stop.set_position(0);
        if let Some(w) = ww_stop.upgrade() { w.set_is_recording(false); }
        if let Some((recorded_buffers, record_start_pos)) = record_data {
            let sr = p_stop.lock().map(|p| p.sample_rate).unwrap_or(44100);
            let project = p_stop.clone();
            let playback = pb_stop.clone();
            let window_weak = ww_stop.clone();
            let waveform_peaks = wp_stop.clone();
            tokio::spawn(async move {
                let mut new_clips: Vec<(Uuid, AudioClip)> = Vec::new();
                let recordings_dir = std::path::PathBuf::from("recordings");
                let _ = std::fs::create_dir_all(&recordings_dir);
                for (track_id, samples) in recorded_buffers {
                    if samples.is_empty() { continue; }
                    let filename = format!("recording_{}.wav", Uuid::new_v4());
                    let path = recordings_dir.join(&filename);
                    match crate::audio::loader::write_wav(&path, &samples, 2, sr) {
                        Ok(()) => {
                            let clip = AudioClip::new(path.to_string_lossy().to_string(), format!("Take {}", Uuid::new_v4().to_string().chars().take(4).collect::<String>()), record_start_pos, samples.len() as u64 / 2);
                            new_clips.push((track_id, clip));
                        }
                        Err(e) => { tracing::error!("Failed to write recording WAV: {}", e); }
                    }
                }
                if !new_clips.is_empty() {
                    let mut p = match project.lock() { Ok(p) => p, Err(_) => return };
                    for (track_id, clip) in &new_clips {
                        if let Some(track) = p.tracks.iter_mut().find(|t| t.id == *track_id) { track.add_clip(clip.clone()); }
                    }
                    playback.load_project_clips(&p.tracks, &p.buses, p.sample_rate);
                    drop(p);
                    if let Some(w) = window_weak.upgrade() {
                        if let Ok(p) = project.lock() {
                            if let Ok(cache) = waveform_peaks.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); }
                        }
                    }
                }
            });
        }
    });

    let pb_play = playback.clone();
    window.on_play(move || { pb_play.set_playing(true); });

    let pb_rec = playback.clone();
    let p_rec = project.clone();
    let ww_rec = window_weak.clone();
    window.on_start_recording(move || {
        let p = match p_rec.lock() { Ok(p) => p, Err(_) => return };
        let armed_ids: HashSet<Uuid> = p.tracks.iter().filter(|t| t.armed).map(|t| t.id).collect();
        let sr = p.sample_rate;
        drop(p);
        if armed_ids.is_empty() { return; }
        let pos = pb_rec.get_position();
        if !pb_rec.is_playing() { pb_rec.set_playing(true); }
        pb_rec.start_recording(&armed_ids, pos, sr);
        if let Some(w) = ww_rec.upgrade() { w.set_is_recording(true); }
    });

    let pb_arm = playback.clone();
    let p_arm = project.clone();
    let ww_arm = window_weak.clone();
    let wp_arm = waveform_peaks.clone();
    window.on_track_arm_toggled(move |id_str| {
        let id = match Uuid::parse_str(id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_arm.lock() { Ok(p) => p, Err(_) => return };
        if let Some(track) = p.tracks.iter_mut().find(|t| t.id == id) { track.armed = !track.armed; pb_arm.set_armed(id, track.armed); }
        drop(p);
        if let Some(w) = ww_arm.upgrade() {
            if let Ok(p) = p_arm.lock() { if let Ok(cache) = wp_arm.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } }
        }
    });

    let pb_loop = playback.clone();
    window.on_toggle_loop(move || { let current = pb_loop.is_loop_enabled(); pb_loop.set_loop_enabled(!current); });
    let pb_start = playback.clone();
    window.on_go_to_start(move || { pb_start.set_position(0); });
    let pb_end = playback.clone();
    let p_end = project.clone();
    window.on_go_to_end(move || {
        let max_pos = p_end.lock().map(|p| p.tracks.iter().flat_map(|t| t.clips.iter()).map(|c| c.position + c.length).max().unwrap_or(0)).unwrap_or(0);
        pb_end.set_position(max_pos);
    });

    // ---- TRACK/BUS SELECTION ----
    let sid_sel = selected_track_id.clone();
    window.on_track_selected(move |id_str| {
        if let Ok(id) = Uuid::parse_str(id_str.as_str()) { if let Ok(mut s) = sid_sel.lock() { *s = Some(id); } }
    });
    let sid_bus = selected_bus_id.clone();
    window.on_bus_selected(move |id_str| {
        if let Ok(id) = Uuid::parse_str(id_str.as_str()) { if let Ok(mut s) = sid_bus.lock() { *s = Some(id); } }
    });

    // ---- TIMELINE INTERACTION ----
    let ds_pressed = drag_state.clone();
    let adp = automation_drag_state.clone();
    let w_pressed = window_weak.clone();
    let p_pressed = project.clone();
    let pb_pressed = playback.clone();
    let sel_pressed = selected_clips.clone();
    let us_pressed = undo_stack.clone();
    let wp_pressed = waveform_peaks.clone();
    window.on_timeline_pressed(move |x, y, _alt| {
        let w = match w_pressed.upgrade() { Some(w) => w, None => return };
        let p = match p_pressed.lock() { Ok(p) => p, Err(_) => return };
        let sr = p.sample_rate;
        let scroll_x = w.get_timeline_scroll_x();
        let abs_x = x + scroll_x;
        let pps = w.get_pixels_per_second();
        let alt = {
            #[cfg(target_os = "windows")]
            { unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(0x12) as u32 & 0x8000 != 0 } }
            #[cfg(not(target_os = "windows"))]
            { false }
        };
        let raw_samples = pixels_to_samples(abs_x.max(0.0), pps, sr);
        let snapped_samples = if w.get_snap_enabled() { snap_to_grid(raw_samples, p.bpm, sr, &get_snap_mode(&w), 0, false) } else { raw_samples };
        if let Some((_, clip_info)) = clip_at_position(&w.get_clips(), abs_x, y) {
            let track_idx = clip_info.track_index as usize;
            if let Some(track) = p.tracks.get(track_idx) {
                if let Some(clip) = track.clips.iter().find(|c| c.id.to_string() == clip_info.id.as_str()) {
                    if w.get_tool_mode() == 1 {
                        if let Some((left, right)) = split_clip(clip, snapped_samples) {
                            let original = clip.clone();
                            let tid = track.id;
                            drop(p);
                            if let Ok(mut stack) = us_pressed.lock() { stack.push(EditCommand::SplitClip { track_id: tid, original_clip: original, new_clips: (left, right) }); }
                            if let Ok(p) = p_pressed.lock() {
                                pb_pressed.load_project_clips(&p.tracks, &p.buses, p.sample_rate);
                                if let Ok(cache) = wp_pressed.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); }
                            }
                        }
                        return;
                    }
                    let local_x = abs_x - clip_info.x;
                    let local_y = y - (track_idx as f32 * TRACK_HEIGHT + CLIP_Y_OFFSET);
                    let lane_top = CLIP_HEIGHT + 4.0;

                    if local_y >= lane_top {
                        // Automation lane area
                        let param_name = match &track.selected_automation_param {
                            Some(p) => p.clone(),
                            None => return, // no lane visible, ignore click
                        };
                        let lane_y = local_y - lane_top;
                        let time_secs = snapped_samples as f64 / sr as f64;
                        let value = ((1.0 - lane_y / LANE_HEIGHT) as f32).clamp(0.0, 1.0);
                        let lane = track.automation.get(&param_name);
                        if let Some(lane) = lane {
                            if let Some(idx) = hit_test_automation_point(lane, time_secs, value, 0.1) {
                                if let Some(point) = lane.points.get(idx) {
                                    let orig = point.clone();
                                    if let Ok(mut ad) = adp.lock() {
                                        *ad = Some(AutomationDragState {
                                            track_id: track.id,
                                            parameter_name: param_name,
                                            point_index: idx,
                                            original_point: orig,
                                            click_offset_time: time_secs - point.time,
                                            click_offset_value: value - point.value,
                                        });
                                    }
                                    return;
                                }
                            }
                        }
                        // Add new point
                        let new_point = AutomationPoint {
                            time: time_secs,
                            value,
                            curve_type: crate::project::automation::CurveType::Linear,
                        };
                        let tid = track.id;
                        let pn = param_name.clone();
                        drop(p);
                        if let Ok(mut stack) = us_pressed.lock() {
                            stack.push(EditCommand::AddAutomationPoint {
                                track_id: tid,
                                parameter_name: pn.clone(),
                                point: new_point,
                            });
                        }
                        if let Ok(mut p) = p_pressed.lock() {
                            if let Some(track) = p.tracks.iter_mut().find(|t| t.id == tid) {
                                let lane = track.automation.entry(pn.clone()).or_insert_with(|| crate::project::automation::AutomationLane::new(pn.clone()));
                                lane.add_point(time_secs, value);
                            }
                            pb_pressed.load_project_clips(&p.tracks, &p.buses, p.sample_rate);
                            if let Ok(cache) = wp_pressed.lock() {
                                sync_project_to_timeline_with_waveforms(&p, &w, &cache);
                            }
                        }
                        return;
                    }

                    let edge = if local_y <= 20.0 {
                        if local_x <= 10.0 { Some(DragEdge::FadeIn) } else if local_x >= clip_info.width - 10.0 { Some(DragEdge::FadeOut) } else { None }
                    } else if is_on_left_edge(local_x) { Some(DragEdge::Left) }
                    else if is_on_right_edge(local_x, clip_info.width) { if alt { Some(DragEdge::Stretch) } else { Some(DragEdge::Right) } }
                    else { None };
                    let click_offset = if edge.is_none() { pixels_to_samples(local_x, pps, sr) } else { 0 };
                    if let Ok(mut ds) = ds_pressed.lock() {
                        *ds = Some(DragState {
                            clip_id: clip.id, track_id: track.id, track_index: clip_info.track_index,
                            original_pos: pixels_to_samples(clip_info.x, pps, sr),
                            original_length: pixels_to_samples(clip_info.width.max(4.0), pps, sr),
                            original_fade_in: clip.fade_in, original_fade_out: clip.fade_out,
                            click_offset, drag_edge: edge, destination_track_id: None,
                        });
                    }
                    if let Ok(mut sel) = sel_pressed.lock() {
                        let ctrl_held = {
                            #[cfg(target_os = "windows")]
                            { unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(0x11) as u32 & 0x8000 != 0 } }
                            #[cfg(not(target_os = "windows"))]
                            { false }
                        };
                        if ctrl_held { if sel.contains(&clip.id) { sel.remove(&clip.id); } else { sel.insert(clip.id); } }
                        else { sel.clear(); sel.insert(clip.id); }
                        drop(sel);
                        sync_selection(&w, &sel_pressed.lock().unwrap());
                    }
                }
            }
        } else {
            pb_pressed.set_position(snapped_samples);
            if let Ok(mut sel) = sel_pressed.lock() { sel.clear(); drop(sel); sync_selection(&w, &sel_pressed.lock().unwrap()); }
        }
    });

    let ds_moved = drag_state.clone();
    let adm = automation_drag_state.clone();
    let w_moved = window_weak.clone();
    let p_moved = project.clone();
    let wp_moved = waveform_peaks.clone();
    window.on_timeline_moved(move |x, y, _alt| {
        let w = match w_moved.upgrade() { Some(w) => w, None => return };

        if let Ok(mut ad_lock) = adm.lock() {
            if let Some(ref ad) = *ad_lock {
                let p = match p_moved.lock() { Ok(p) => p, Err(_) => return };
                let pps = w.get_pixels_per_second();
                let scroll_px = w.get_timeline_scroll_x();
                let abs_x = (x + scroll_px).max(0.0);
                let raw_samples = pixels_to_samples(abs_x, pps, p.sample_rate);
                let new_samples = if w.get_snap_enabled() { snap_to_grid(raw_samples, p.bpm, p.sample_rate, &get_snap_mode(&w), 0, false) } else { raw_samples };
                let new_time = new_samples as f64 / p.sample_rate as f64;
                let clips = w.get_clips();
                let new_value = if let Some((_, ci)) = clip_at_position(&clips, abs_x, y) {
                    let lane_top = CLIP_HEIGHT + 4.0;
                    let local_y = y - (ci.track_index as f32 * TRACK_HEIGHT + CLIP_Y_OFFSET);
                    ((1.0 - (local_y - lane_top) / LANE_HEIGHT) as f32).clamp(0.0, 1.0)
                } else {
                    ad.original_point.value
                };
                let new_point = AutomationPoint {
                    time: new_time,
                    value: new_value,
                    curve_type: crate::project::automation::CurveType::Linear,
                };
                drop(p);
                if let Ok(mut p) = p_moved.lock() {
                    if let Some(track) = p.tracks.iter_mut().find(|t| t.id == ad.track_id) {
                        if let Some(lane) = track.automation.get_mut(&ad.parameter_name) {
                            if ad.point_index < lane.points.len() {
                                lane.points[ad.point_index] = new_point;
                                lane.points.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
                            }
                        }
                    }
                    if let Ok(cache) = wp_moved.lock() {
                        sync_project_to_timeline_with_waveforms(&p, &w, &cache);
                    }
                }
                return;
            }
        }

        let p = match p_moved.lock() { Ok(p) => p, Err(_) => return };
        let mut ds_lock = match ds_moved.lock() { Ok(d) => d, Err(_) => return };
        let Some(ref mut ds) = *ds_lock else { return };
        let sr = p.sample_rate;
        let pps = w.get_pixels_per_second();
        let scroll_px = w.get_timeline_scroll_x();
        let effective_x = (x + scroll_px).max(0.0);
        let raw_samples = pixels_to_samples(effective_x, pps, sr);
        let new_sample_pos = if w.get_snap_enabled() { snap_to_grid(raw_samples, p.bpm, sr, &get_snap_mode(&w), 0, false) } else { raw_samples };
        let dest_idx = ((y / TRACK_HEIGHT).floor() as i32).max(0).min(p.tracks.len().saturating_sub(1) as i32);
        let dest_id = p.tracks.get(dest_idx as usize).map(|t| t.id);
        ds.destination_track_id = if dest_id == Some(ds.track_id) { None } else { dest_id };
        let clips = w.get_clips();
        let mut new_clips = clips.iter().collect::<Vec<_>>();
        if let Some(idx) = new_clips.iter().position(|c| c.id.as_str() == ds.clip_id.to_string().as_str()) {
            let mut updated = new_clips[idx].clone();
            match ds.drag_edge {
                None => { let adj = new_sample_pos.saturating_sub(ds.click_offset); updated.x = samples_to_pixels(adj, pps, sr); }
                Some(DragEdge::Left) => { let right_edge = ds.original_pos + ds.original_length; let new_len = right_edge.saturating_sub(new_sample_pos).max(1); updated.x = samples_to_pixels(new_sample_pos, pps, sr); updated.width = samples_to_pixels(new_len, pps, sr).max(4.0); }
                Some(DragEdge::Right) | Some(DragEdge::Stretch) => { let new_len = new_sample_pos.saturating_sub(ds.original_pos).max(1); updated.width = samples_to_pixels(new_len, pps, sr).max(4.0); }
                Some(DragEdge::FadeIn) => { let fade_samples = new_sample_pos.saturating_sub(ds.original_pos); updated.fade_in_width = samples_to_pixels(fade_samples, pps, sr); }
                Some(DragEdge::FadeOut) => { let remaining = ds.original_pos + ds.original_length; let fade_samples = remaining.saturating_sub(new_sample_pos); updated.fade_out_width = samples_to_pixels(fade_samples, pps, sr); }
            }
            if ds.destination_track_id.is_some() { updated.track_index = dest_idx; }
            new_clips[idx] = updated;
            w.set_clips(slint::ModelRc::new(slint::VecModel::from(new_clips)));
        }
    });

    let ds_released = drag_state.clone();
    let adr = automation_drag_state.clone();
    let p_released = project.clone();
    let w_released = window_weak.clone();
    let us_released = undo_stack.clone();
    let wp_released = waveform_peaks.clone();
    let pb_released = playback.clone();
    window.on_timeline_released(move |_x, _y| {
        let w = match w_released.upgrade() { Some(w) => w, None => return };

        if let Ok(mut ad_lock) = adr.lock() {
            let auto_state = ad_lock.take();
            if let Some(ref ad) = auto_state {
                if let Ok(mut p) = p_released.lock() {
                    if let Some(track) = p.tracks.iter_mut().find(|t| t.id == ad.track_id) {
                        if let Some(lane) = track.automation.get(&ad.parameter_name) {
                            if let Some(point) = lane.points.get(ad.point_index) {
                                let new_point = point.clone();
                                let old_point = ad.original_point.clone();
                                if new_point.time != old_point.time || new_point.value != old_point.value {
                                    if let Ok(mut stack) = us_released.lock() {
                                        stack.push(EditCommand::MoveAutomationPoint {
                                            track_id: ad.track_id,
                                            parameter_name: ad.parameter_name.clone(),
                                            old_point,
                                            new_point,
                                            index: ad.point_index,
                                        });
                                    }
                                }
                            }
                        }
                    }
                    pb_released.load_project_clips(&p.tracks, &p.buses, p.sample_rate);
                    if let Ok(cache) = wp_released.lock() {
                        sync_project_to_timeline_with_waveforms(&p, &w, &cache);
                    }
                }
                return;
            }
        }

        if let Ok(mut ds) = ds_released.lock() {
            if let Some(ref state) = ds.take() {
                commit_drag(state, &p_released, &w, &us_released, &wp_released);
            }
        }
    });

    // ---- MIXER ----
    let p_mix = project.clone();
    let pb_mix = playback.clone();
    let us_mix = undo_stack.clone();
    let wp_mix = waveform_peaks.clone();
    let ww_mix = window_weak.clone();
    window.on_track_volume_changed(move |id_str, vol| {
        let id = match Uuid::parse_str(id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_mix.lock() { Ok(p) => p, Err(_) => return };
        if let Some(track) = p.tracks.iter_mut().find(|t| t.id == id) {
            let old_val = track.volume; track.volume = vol;
            if let Ok(mut stack) = us_mix.lock() { stack.push(EditCommand::ChangeVolume { track_id: id, old_val, new_val: vol }); }
            pb_mix.update_track_params(track.id, track.volume, track.pan, track.mute, track.solo);
        }
        drop(p);
        if let Some(w) = ww_mix.upgrade() { if let Ok(p) = p_mix.lock() { if let Ok(cache) = wp_mix.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    let p_pan = project.clone();
    let pb_pan = playback.clone();
    let us_pan = undo_stack.clone();
    let wp_pan = waveform_peaks.clone();
    let ww_pan = window_weak.clone();
    window.on_track_pan_changed(move |id_str, pan| {
        let id = match Uuid::parse_str(id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_pan.lock() { Ok(p) => p, Err(_) => return };
        if let Some(track) = p.tracks.iter_mut().find(|t| t.id == id) {
            let old_val = track.pan; track.pan = pan;
            if let Ok(mut stack) = us_pan.lock() { stack.push(EditCommand::ChangePan { track_id: id, old_val, new_val: pan }); }
            pb_pan.update_track_params(track.id, track.volume, track.pan, track.mute, track.solo);
        }
        drop(p);
        if let Some(w) = ww_pan.upgrade() { if let Ok(p) = p_pan.lock() { if let Ok(cache) = wp_pan.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    let p_mut = project.clone();
    let pb_mut = playback.clone();
    let ww_mut = window_weak.clone();
    let wp_mut = waveform_peaks.clone();
    window.on_track_mute_toggled(move |id_str| {
        let id = match Uuid::parse_str(id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_mut.lock() { Ok(p) => p, Err(_) => return };
        if let Some(track) = p.tracks.iter_mut().find(|t| t.id == id) { track.mute = !track.mute; pb_mut.update_track_params(track.id, track.volume, track.pan, track.mute, track.solo); }
        drop(p);
        if let Some(w) = ww_mut.upgrade() { if let Ok(p) = p_mut.lock() { if let Ok(cache) = wp_mut.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    let p_solo = project.clone();
    let pb_solo = playback.clone();
    let ww_solo = window_weak.clone();
    let wp_solo = waveform_peaks.clone();
    window.on_track_solo_toggled(move |id_str| {
        let id = match Uuid::parse_str(id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_solo.lock() { Ok(p) => p, Err(_) => return };
        if let Some(track) = p.tracks.iter_mut().find(|t| t.id == id) { track.solo = !track.solo; pb_solo.update_track_params(track.id, track.volume, track.pan, track.mute, track.solo); }
        drop(p);
        if let Some(w) = ww_solo.upgrade() { if let Ok(p) = p_solo.lock() { if let Ok(cache) = wp_solo.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    let p_bv = project.clone();
    let pb_bv = playback.clone();
    let us_bv = undo_stack.clone();
    let wp_bv = waveform_peaks.clone();
    let ww_bv = window_weak.clone();
    window.on_bus_volume_changed(move |id_str, vol| {
        let id = match Uuid::parse_str(id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_bv.lock() { Ok(p) => p, Err(_) => return };
        if let Some(bus) = p.buses.iter_mut().find(|b| b.id == id) { let old_val = bus.volume; bus.volume = vol; if let Ok(mut stack) = us_bv.lock() { stack.push(EditCommand::ChangeBusVolume { bus_id: id, old_val, new_val: vol }); } pb_bv.update_bus_params(bus.id, bus.volume, bus.pan, bus.mute, bus.solo); }
        drop(p);
        if let Some(w) = ww_bv.upgrade() { if let Ok(p) = p_bv.lock() { if let Ok(cache) = wp_bv.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    let p_bp = project.clone();
    let pb_bp = playback.clone();
    let us_bp = undo_stack.clone();
    let wp_bp = waveform_peaks.clone();
    let ww_bp = window_weak.clone();
    window.on_bus_pan_changed(move |id_str, pan| {
        let id = match Uuid::parse_str(id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_bp.lock() { Ok(p) => p, Err(_) => return };
        if let Some(bus) = p.buses.iter_mut().find(|b| b.id == id) { let old_val = bus.pan; bus.pan = pan; if let Ok(mut stack) = us_bp.lock() { stack.push(EditCommand::ChangeBusPan { bus_id: id, old_val, new_val: pan }); } pb_bp.update_bus_params(bus.id, bus.volume, bus.pan, bus.mute, bus.solo); }
        drop(p);
        if let Some(w) = ww_bp.upgrade() { if let Ok(p) = p_bp.lock() { if let Ok(cache) = wp_bp.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    let p_bm = project.clone();
    let pb_bm = playback.clone();
    let ww_bm = window_weak.clone();
    let wp_bm = waveform_peaks.clone();
    window.on_bus_mute_toggled(move |id_str| {
        let id = match Uuid::parse_str(id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_bm.lock() { Ok(p) => p, Err(_) => return };
        if let Some(bus) = p.buses.iter_mut().find(|b| b.id == id) { bus.mute = !bus.mute; pb_bm.update_bus_params(bus.id, bus.volume, bus.pan, bus.mute, bus.solo); }
        drop(p);
        if let Some(w) = ww_bm.upgrade() { if let Ok(p) = p_bm.lock() { if let Ok(cache) = wp_bm.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    let p_bs = project.clone();
    let pb_bs = playback.clone();
    let ww_bs = window_weak.clone();
    let wp_bs = waveform_peaks.clone();
    window.on_bus_solo_toggled(move |id_str| {
        let id = match Uuid::parse_str(id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_bs.lock() { Ok(p) => p, Err(_) => return };
        if let Some(bus) = p.buses.iter_mut().find(|b| b.id == id) { bus.solo = !bus.solo; pb_bs.update_bus_params(bus.id, bus.volume, bus.pan, bus.mute, bus.solo); }
        drop(p);
        if let Some(w) = ww_bs.upgrade() { if let Ok(p) = p_bs.lock() { if let Ok(cache) = wp_bs.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    // Input monitoring
    let p_im = project.clone();
    let pb_im = playback.clone();
    window.on_track_input_mon_toggled(move |id_str| {
        let id = match Uuid::parse_str(id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_im.lock() { Ok(p) => p, Err(_) => return };
        if let Some(track) = p.tracks.iter_mut().find(|t| t.id == id) { track.input_monitoring = !track.input_monitoring; pb_im.set_input_monitoring(id, track.input_monitoring); }
    });

    // Send level
    let p_send_t = project.clone();
    window.on_track_send_level_changed(move |id_str, send_id_str, level| {
        let id = match Uuid::parse_str(id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let send_id = match Uuid::parse_str(send_id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_send_t.lock() { Ok(p) => p, Err(_) => return };
        if let Some(track) = p.tracks.iter_mut().find(|t| t.id == id) {
            if let Some(send) = track.sends.iter_mut().find(|s| s.id == send_id) { send.level = level; }
        }
    });

    let p_send_b = project.clone();
    window.on_bus_send_level_changed(move |id_str, send_id_str, level| {
        let id = match Uuid::parse_str(id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let send_id = match Uuid::parse_str(send_id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_send_b.lock() { Ok(p) => p, Err(_) => return };
        if let Some(bus) = p.buses.iter_mut().find(|b| b.id == id) {
            if let Some(send) = bus.sends.iter_mut().find(|s| s.id == send_id) { send.level = level; }
        }
    });

    // ---- MASTER ----
    let p_mv = project.clone();
    let pb_mv = playback.clone();
    let us_mv = undo_stack.clone();
    let ww_mv = window_weak.clone();
    window.on_master_volume_changed(move |vol| {
        let mut p = match p_mv.lock() { Ok(p) => p, Err(_) => return };
        let old_val = p.master_volume; p.master_volume = vol;
        if let Ok(mut stack) = us_mv.lock() { stack.push(EditCommand::ChangeMasterVolume { old_val, new_val: vol }); }
        pb_mv.update_master_params(p.master_volume, p.master_pan, p.master_mute);
        drop(p);
        if let Some(w) = ww_mv.upgrade() { w.set_master_volume(vol); }
    });

    let p_mp = project.clone();
    let pb_mp = playback.clone();
    let us_mp = undo_stack.clone();
    let ww_mp = window_weak.clone();
    window.on_master_pan_changed(move |pan| {
        let mut p = match p_mp.lock() { Ok(p) => p, Err(_) => return };
        let old_val = p.master_pan; p.master_pan = pan;
        if let Ok(mut stack) = us_mp.lock() { stack.push(EditCommand::ChangeMasterPan { old_val, new_val: pan }); }
        pb_mp.update_master_params(p.master_volume, p.master_pan, p.master_mute);
        drop(p);
        if let Some(w) = ww_mp.upgrade() { w.set_master_pan(pan); }
    });

    let p_mm = project.clone();
    let pb_mm = playback.clone();
    let us_mm = undo_stack.clone();
    let ww_mm = window_weak.clone();
    window.on_master_mute_toggled(move || {
        let mut p = match p_mm.lock() { Ok(p) => p, Err(_) => return };
        p.master_mute = !p.master_mute;
        if let Ok(mut stack) = us_mm.lock() { stack.push(EditCommand::ToggleMasterMute); }
        pb_mm.update_master_params(p.master_volume, p.master_pan, p.master_mute);
        let muted = p.master_mute;
        drop(p);
        if let Some(w) = ww_mm.upgrade() { w.set_master_mute(muted); }
    });

    // ---- SEND CRUD ----
    let p_send_add = project.clone();
    let pb_send_add = playback.clone();
    let us_send_add = undo_stack.clone();
    let wp_send_add = waveform_peaks.clone();
    let ww_send_add = window_weak.clone();
    window.on_add_send(move |source_id_str, _target_str, is_track| {
        let source_id = match Uuid::parse_str(source_id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_send_add.lock() { Ok(p) => p, Err(_) => return };
        let target_bus_id = p.buses.first().map(|b| b.id).unwrap_or_else(Uuid::new_v4);
        let send = crate::project::track::AuxSend::new(target_bus_id);
        let undo_send = send.clone();
        if is_track { if let Some(track) = p.tracks.iter_mut().find(|t| t.id == source_id) { track.sends.push(send); } }
        else { if let Some(bus) = p.buses.iter_mut().find(|b| b.id == source_id) { bus.sends.push(send); } }
        if let Ok(mut stack) = us_send_add.lock() { stack.push(EditCommand::AddSend { source_id, is_track, send: undo_send }); }
        pb_send_add.load_project_clips(&p.tracks, &p.buses, p.sample_rate);
        drop(p);
        if let Some(w) = ww_send_add.upgrade() { if let Ok(p) = p_send_add.lock() { if let Ok(cache) = wp_send_add.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    let p_send_rem = project.clone();
    let pb_send_rem = playback.clone();
    let us_send_rem = undo_stack.clone();
    let wp_send_rem = waveform_peaks.clone();
    let ww_send_rem = window_weak.clone();
    window.on_remove_send(move |source_id_str, send_id_str, is_track| {
        let source_id = match Uuid::parse_str(source_id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let send_id = match Uuid::parse_str(send_id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_send_rem.lock() { Ok(p) => p, Err(_) => return };
        let removed = if is_track { p.tracks.iter_mut().find(|t| t.id == source_id).and_then(|t| { let pos = t.sends.iter().position(|s| s.id == send_id); pos.map(|i| t.sends.remove(i)) }) } else { p.buses.iter_mut().find(|b| b.id == source_id).and_then(|b| { let pos = b.sends.iter().position(|s| s.id == send_id); pos.map(|i| b.sends.remove(i)) }) };
        if let Some(send) = removed { if let Ok(mut stack) = us_send_rem.lock() { stack.push(EditCommand::RemoveSend { source_id, is_track, send }); } }
        pb_send_rem.load_project_clips(&p.tracks, &p.buses, p.sample_rate);
        drop(p);
        if let Some(w) = ww_send_rem.upgrade() { if let Ok(p) = p_send_rem.lock() { if let Ok(cache) = wp_send_rem.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    let p_send_act = project.clone();
    let pb_send_act = playback.clone();
    let wp_send_act = waveform_peaks.clone();
    let ww_send_act = window_weak.clone();
    window.on_toggle_send_active(move |source_id_str, send_id_str, is_track| {
        let source_id = match Uuid::parse_str(source_id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let send_id = match Uuid::parse_str(send_id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_send_act.lock() { Ok(p) => p, Err(_) => return };
        if is_track { if let Some(track) = p.tracks.iter_mut().find(|t| t.id == source_id) { if let Some(send) = track.sends.iter_mut().find(|s| s.id == send_id) { send.is_active = !send.is_active; } } } else { if let Some(bus) = p.buses.iter_mut().find(|b| b.id == source_id) { if let Some(send) = bus.sends.iter_mut().find(|s| s.id == send_id) { send.is_active = !send.is_active; } } }
        pb_send_act.load_project_clips(&p.tracks, &p.buses, p.sample_rate);
        drop(p);
        if let Some(w) = ww_send_act.upgrade() { if let Ok(p) = p_send_act.lock() { if let Ok(cache) = wp_send_act.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    let p_send_pf = project.clone();
    let pb_send_pf = playback.clone();
    let wp_send_pf = waveform_peaks.clone();
    let ww_send_pf = window_weak.clone();
    window.on_toggle_send_pre_fader(move |source_id_str, send_id_str, is_track| {
        let source_id = match Uuid::parse_str(source_id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let send_id = match Uuid::parse_str(send_id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_send_pf.lock() { Ok(p) => p, Err(_) => return };
        if is_track { if let Some(track) = p.tracks.iter_mut().find(|t| t.id == source_id) { if let Some(send) = track.sends.iter_mut().find(|s| s.id == send_id) { send.pre_fader = !send.pre_fader; } } } else { if let Some(bus) = p.buses.iter_mut().find(|b| b.id == source_id) { if let Some(send) = bus.sends.iter_mut().find(|s| s.id == send_id) { send.pre_fader = !send.pre_fader; } } }
        pb_send_pf.load_project_clips(&p.tracks, &p.buses, p.sample_rate);
        drop(p);
        if let Some(w) = ww_send_pf.upgrade() { if let Ok(p) = p_send_pf.lock() { if let Ok(cache) = wp_send_pf.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    // ---- TRACK/BUS OUTPUT ROUTING ----
    let p_to = project.clone();
    let pb_to = playback.clone();
    let us_to = undo_stack.clone();
    let wp_to = waveform_peaks.clone();
    let ww_to = window_weak.clone();
    window.on_track_output_changed(move |id_str, target_str| {
        let id = match Uuid::parse_str(id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let new_output = if target_str.is_empty() || target_str == "Mstr" { None } else { match Uuid::parse_str(target_str.as_str()) { Ok(id) => Some(id), Err(_) => None } };
        let mut p = match p_to.lock() { Ok(p) => p, Err(_) => return };
        if let Some(track) = p.tracks.iter_mut().find(|t| t.id == id) {
            let old_output = track.output_id; track.output_id = new_output;
            if let Ok(mut stack) = us_to.lock() { stack.push(EditCommand::ChangeTrackOutput { track_id: id, old_output, new_output }); }
        }
        pb_to.load_project_clips(&p.tracks, &p.buses, p.sample_rate);
        drop(p);
        if let Some(w) = ww_to.upgrade() { if let Ok(p) = p_to.lock() { if let Ok(cache) = wp_to.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    let p_bo = project.clone();
    let pb_bo = playback.clone();
    let us_bo = undo_stack.clone();
    let wp_bo = waveform_peaks.clone();
    let ww_bo = window_weak.clone();
    window.on_bus_output_changed(move |id_str, target_str| {
        let id = match Uuid::parse_str(id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let new_output = if target_str.is_empty() || target_str == "Mstr" { None } else { match Uuid::parse_str(target_str.as_str()) { Ok(id) => Some(id), Err(_) => None } };
        let mut p = match p_bo.lock() { Ok(p) => p, Err(_) => return };
        if let Some(bus) = p.buses.iter_mut().find(|b| b.id == id) {
            let old_output = bus.output_id; bus.output_id = new_output;
            if let Ok(mut stack) = us_bo.lock() { stack.push(EditCommand::ChangeBusOutput { bus_id: id, old_output, new_output }); }
        }
        pb_bo.load_project_clips(&p.tracks, &p.buses, p.sample_rate);
        drop(p);
        if let Some(w) = ww_bo.upgrade() { if let Ok(p) = p_bo.lock() { if let Ok(cache) = wp_bo.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    // ---- AUTOMATION PARAM CYCLING ----
    let p_auto_cycle = project.clone();
    let pb_auto_cycle = playback.clone();
    let wp_auto_cycle = waveform_peaks.clone();
    let ww_auto_cycle = window_weak.clone();
    window.on_track_auto_param_clicked(move |track_id_str| {
        let tid = match Uuid::parse_str(track_id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let w = match ww_auto_cycle.upgrade() { Some(w) => w, None => return };
        let mut p = match p_auto_cycle.lock() { Ok(p) => p, Err(_) => return };
        if let Some(track) = p.tracks.iter_mut().find(|t| t.id == tid) {
            // Cycle through available params: volume, pan, effect params
            let mut params = vec!["volume".to_string(), "pan".to_string()];
            for fx in &track.effects_chain {
                let names: &[&str] = match fx.effect_type.as_str() {
                    "Equalizer" => &["eq_low_freq", "eq_low_gain", "eq_mid_freq", "eq_mid_gain", "eq_mid_q", "eq_high_freq", "eq_high_gain"],
                    "Compressor" => &["comp_threshold", "comp_ratio", "comp_attack", "comp_release", "comp_makeup"],
                    "Reverb" => &["reverb_room_size", "reverb_damping", "reverb_wet_dry"],
                    "Delay" => &["delay_time", "delay_feedback", "delay_mix"],
                    _ => &[],
                };
                for name in names {
                    if !params.contains(&(*name).to_string()) {
                        params.push((*name).to_string());
                    }
                }
            }
            let current = track.selected_automation_param.clone();
            let pos = current.as_ref().and_then(|c| params.iter().position(|p| p == c));
            let next = match pos {
                Some(i) if i + 1 < params.len() => Some(params[i + 1].clone()),
                Some(_) => None, // wrap to None (hide lane)
                None if !params.is_empty() => Some(params[0].clone()),
                None => None,
            };
            track.selected_automation_param = next.clone();
            if let Some(ref param_name) = next {
                track.automation.entry(param_name.clone()).or_insert_with(|| crate::project::automation::AutomationLane::new(param_name.clone()));
            }
        }
        let sr = p.sample_rate;
        pb_auto_cycle.load_project_clips(&p.tracks, &p.buses, sr);
        drop(p);
        if let Ok(cache) = wp_auto_cycle.lock() { if let Ok(p) = p_auto_cycle.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } }
    });

    // ---- EFFECTS ----
    let p_fx_move = project.clone();
    let pb_fx_move = playback.clone();
    let us_fx_move = undo_stack.clone();
    let wp_fx_move = waveform_peaks.clone();
    let ww_fx_move = window_weak.clone();
    window.on_move_effect_left(move |target_id_str, is_track, from_idx| {
        if from_idx <= 0 { return; }
        let target_id = match Uuid::parse_str(target_id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_fx_move.lock() { Ok(p) => p, Err(_) => return };
        let chain = if is_track { p.tracks.iter_mut().find(|t| t.id == target_id).map(|t| &mut t.effects_chain) } else { p.buses.iter_mut().find(|b| b.id == target_id).map(|b| &mut b.effects_chain) };
        if let Some(chain) = chain { let from = from_idx as usize; if from > 0 && from < chain.len() { chain.swap(from, from - 1); if let Ok(mut stack) = us_fx_move.lock() { stack.push(EditCommand::MoveEffect { target_id, is_track, from_index: from, to_index: from - 1 }); } } }
        let sr = p.sample_rate; drop(p);
        if is_track { if let Ok(p) = p_fx_move.lock() { if let Some(track) = p.tracks.iter().find(|t| t.id == target_id) { pb_fx_move.reload_track_effects(target_id, &track.effects_chain, sr); } } }
        else { if let Ok(p) = p_fx_move.lock() { if let Some(bus) = p.buses.iter().find(|b| b.id == target_id) { pb_fx_move.reload_bus_effects(target_id, &bus.effects_chain, sr); } } }
        if let Some(w) = ww_fx_move.upgrade() { if let Ok(p) = p_fx_move.lock() { if let Ok(cache) = wp_fx_move.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    let p_fx_mvr = project.clone();
    let pb_fx_mvr = playback.clone();
    let us_fx_mvr = undo_stack.clone();
    let wp_fx_mvr = waveform_peaks.clone();
    let ww_fx_mvr = window_weak.clone();
    window.on_move_effect_right(move |target_id_str, is_track, from_idx| {
        let target_id = match Uuid::parse_str(target_id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_fx_mvr.lock() { Ok(p) => p, Err(_) => return };
        let chain_len = if is_track { p.tracks.iter().find(|t| t.id == target_id).map(|t| t.effects_chain.len()).unwrap_or(0) } else { p.buses.iter().find(|b| b.id == target_id).map(|b| b.effects_chain.len()).unwrap_or(0) };
        let from = from_idx as usize; if from + 1 >= chain_len { return; }
        let chain = if is_track { p.tracks.iter_mut().find(|t| t.id == target_id).map(|t| &mut t.effects_chain) } else { p.buses.iter_mut().find(|b| b.id == target_id).map(|b| &mut b.effects_chain) };
        if let Some(chain) = chain { if from + 1 < chain.len() { chain.swap(from, from + 1); if let Ok(mut stack) = us_fx_mvr.lock() { stack.push(EditCommand::MoveEffect { target_id, is_track, from_index: from, to_index: from + 1 }); } } }
        let sr = p.sample_rate; drop(p);
        if is_track { if let Ok(p) = p_fx_mvr.lock() { if let Some(track) = p.tracks.iter().find(|t| t.id == target_id) { pb_fx_mvr.reload_track_effects(target_id, &track.effects_chain, sr); } } }
        else { if let Ok(p) = p_fx_mvr.lock() { if let Some(bus) = p.buses.iter().find(|b| b.id == target_id) { pb_fx_mvr.reload_bus_effects(target_id, &bus.effects_chain, sr); } } }
        if let Some(w) = ww_fx_mvr.upgrade() { if let Ok(p) = p_fx_mvr.lock() { if let Ok(cache) = wp_fx_mvr.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    // Effect selection
    let p_fx_sel = project.clone();
    let ww_fx_sel = window_weak.clone();
    window.on_effect_selected(move |target_id_str, effect_idx, is_track| {
        let target_id = match Uuid::parse_str(target_id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let w = match ww_fx_sel.upgrade() { Some(w) => w, None => return };
        let (effect_clone, sr) = {
            let p = match p_fx_sel.lock() { Ok(p) => p, Err(_) => return };
            let sr = p.sample_rate;
            let chain = if is_track { p.tracks.iter().find(|t| t.id == target_id).map(|t| &t.effects_chain) } else { p.buses.iter().find(|b| b.id == target_id).map(|b| &b.effects_chain) };
            if let Some(chain) = chain {
                if let Some(effect) = chain.get(effect_idx as usize) {
                    (effect.clone(), sr)
                } else { return; }
            } else { return; }
        };
        w.set_selected_effect_target(target_id_str.clone().into());
        w.set_selected_effect_index(effect_idx);
        w.set_selected_effect_is_track(is_track);
        w.set_effect_editor_title(effect_clone.effect_type.as_str().into());
        let params = build_param_info(&effect_clone);
        w.set_effect_params(slint::ModelRc::new(slint::VecModel::from(params)));
        update_eq_curve(&w, &effect_clone, sr);
    });

    // Add effect
    let p_fx_add = project.clone();
    let pb_fx_add = playback.clone();
    let us_fx_add = undo_stack.clone();
    let ww_fx_add = window_weak.clone();
    let wp_fx_add = waveform_peaks.clone();
    window.on_add_effect(move |target_id_str, is_track, effect_type| {
        let target_id = match Uuid::parse_str(target_id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_fx_add.lock() { Ok(p) => p, Err(_) => return };
        let mut params = HashMap::new();
        match effect_type.as_str() {
            "Equalizer" => { params.insert("eq_low_freq".to_string(), 100.0f32); params.insert("eq_low_gain".to_string(), 0.0f32); params.insert("eq_mid_freq".to_string(), 1000.0f32); params.insert("eq_mid_gain".to_string(), 0.0f32); params.insert("eq_mid_q".to_string(), 1.0f32); params.insert("eq_high_freq".to_string(), 8000.0f32); params.insert("eq_high_gain".to_string(), 0.0f32); }
            "Compressor" => { params.insert("comp_threshold".to_string(), -20.0f32); params.insert("comp_ratio".to_string(), 4.0f32); params.insert("comp_attack".to_string(), 0.01f32); params.insert("comp_release".to_string(), 0.1f32); params.insert("comp_makeup".to_string(), 0.0f32); }
            "Reverb" => { params.insert("reverb_room_size".to_string(), 0.5f32); params.insert("reverb_damping".to_string(), 0.5f32); params.insert("reverb_wet_dry".to_string(), 0.3f32); }
            "Delay" => { params.insert("delay_time".to_string(), 0.5f32); params.insert("delay_feedback".to_string(), 0.3f32); params.insert("delay_mix".to_string(), 0.3f32); }
            _ => {}
        }
        let instance = crate::project::track::EffectInstance { id: Uuid::new_v4(), effect_type: effect_type.to_string(), bypass: false, parameters: params };
        let index = if is_track { if let Some(track) = p.tracks.iter_mut().find(|t| t.id == target_id) { let idx = track.effects_chain.len(); track.effects_chain.push(instance.clone()); idx } else { return } } else { if let Some(bus) = p.buses.iter_mut().find(|b| b.id == target_id) { let idx = bus.effects_chain.len(); bus.effects_chain.push(instance.clone()); idx } else { return } };
        if let Ok(mut stack) = us_fx_add.lock() { stack.push(EditCommand::AddEffect { target_id, is_track, effect: instance.clone(), index }); }
        let sr = p.sample_rate; drop(p);
        if is_track { if let Ok(p) = p_fx_add.lock() { if let Some(track) = p.tracks.iter().find(|t| t.id == target_id) { pb_fx_add.reload_track_effects(target_id, &track.effects_chain, sr); } } }
        else { if let Ok(p) = p_fx_add.lock() { if let Some(bus) = p.buses.iter().find(|b| b.id == target_id) { pb_fx_add.reload_bus_effects(target_id, &bus.effects_chain, sr); } } }
        if let Some(w) = ww_fx_add.upgrade() { if let Ok(p) = p_fx_add.lock() { if let Ok(cache) = wp_fx_add.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    // Remove effect
    let p_fx_rem = project.clone();
    let pb_fx_rem = playback.clone();
    let us_fx_rem = undo_stack.clone();
    let ww_fx_rem = window_weak.clone();
    let wp_fx_rem = waveform_peaks.clone();
    window.on_remove_effect(move |target_id_str, is_track, effect_idx| {
        let target_id = match Uuid::parse_str(target_id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_fx_rem.lock() { Ok(p) => p, Err(_) => return };
        let removed = if is_track { p.tracks.iter_mut().find(|t| t.id == target_id).and_then(|t| if effect_idx < t.effects_chain.len() as i32 && effect_idx >= 0 { Some(t.effects_chain.remove(effect_idx as usize)) } else { None }) } else { p.buses.iter_mut().find(|b| b.id == target_id).and_then(|b| if effect_idx < b.effects_chain.len() as i32 && effect_idx >= 0 { Some(b.effects_chain.remove(effect_idx as usize)) } else { None }) };
        if let Some(effect) = removed { if let Ok(mut stack) = us_fx_rem.lock() { stack.push(EditCommand::RemoveEffect { target_id, is_track, effect, index: effect_idx as usize }); } }
        let sr = p.sample_rate; drop(p);
        if is_track { if let Ok(p) = p_fx_rem.lock() { if let Some(track) = p.tracks.iter().find(|t| t.id == target_id) { pb_fx_rem.reload_track_effects(target_id, &track.effects_chain, sr); } } }
        else { if let Ok(p) = p_fx_rem.lock() { if let Some(bus) = p.buses.iter().find(|b| b.id == target_id) { pb_fx_rem.reload_bus_effects(target_id, &bus.effects_chain, sr); } } }
        if let Some(w) = ww_fx_rem.upgrade() { w.set_selected_effect_index(-1); w.set_selected_effect_target("".into()); w.set_effect_params(slint::ModelRc::new(slint::VecModel::from(Vec::<ParamInfo>::new()))); if let Ok(p) = p_fx_rem.lock() { if let Ok(cache) = wp_fx_rem.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    // Toggle bypass
    let p_fx_byp = project.clone();
    let pb_fx_byp = playback.clone();
    let us_fx_byp = undo_stack.clone();
    let ww_fx_byp = window_weak.clone();
    let wp_fx_byp = waveform_peaks.clone();
    window.on_toggle_effect_bypass(move |target_id_str, is_track, effect_idx| {
        let target_id = match Uuid::parse_str(target_id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let mut p = match p_fx_byp.lock() { Ok(p) => p, Err(_) => return };
        if let Ok(mut stack) = us_fx_byp.lock() { stack.push(EditCommand::ToggleEffectBypass { target_id, is_track, effect_index: effect_idx as usize }); }
        if is_track { if let Some(track) = p.tracks.iter_mut().find(|t| t.id == target_id) { if let Some(effect) = track.effects_chain.get_mut(effect_idx as usize) { effect.bypass = !effect.bypass; pb_fx_byp.set_track_effect_bypass(target_id, effect_idx as usize, effect.bypass); } } }
        else { if let Some(bus) = p.buses.iter_mut().find(|b| b.id == target_id) { if let Some(effect) = bus.effects_chain.get_mut(effect_idx as usize) { effect.bypass = !effect.bypass; pb_fx_byp.set_bus_effect_bypass(target_id, effect_idx as usize, effect.bypass); } } }
        drop(p);
        if let Some(w) = ww_fx_byp.upgrade() { if let Ok(p) = p_fx_byp.lock() { if let Ok(cache) = wp_fx_byp.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
    });

    // Effect param changed
    let p_fx_par = project.clone();
    let pb_fx_par = playback.clone();
    let us_fx_par = undo_stack.clone();
    let ww_fx_par = window_weak.clone();
    window.on_effect_param_changed(move |target_id_str, effect_idx, param_name, value| {
        let target_id = match Uuid::parse_str(target_id_str.as_str()) { Ok(id) => id, Err(_) => return };
        let is_track = ww_fx_par.upgrade().map(|w| w.get_selected_effect_is_track()).unwrap_or(true);
        let param_key: String = param_name.to_string();
        let mut p = match p_fx_par.lock() { Ok(p) => p, Err(_) => return };
        let effect_before = if is_track { p.tracks.iter().find(|t| t.id == target_id).and_then(|t| t.effects_chain.get(effect_idx as usize)).cloned() } else { p.buses.iter().find(|b| b.id == target_id).and_then(|b| b.effects_chain.get(effect_idx as usize)).cloned() };
        let sr = p.sample_rate;
        let old_val = if is_track { p.tracks.iter_mut().find(|t| t.id == target_id).and_then(|t| t.effects_chain.get_mut(effect_idx as usize)).map(|e| { let old = e.parameters.get(&param_key).copied().unwrap_or(value); e.parameters.insert(param_key.clone(), value); old }) } else { p.buses.iter_mut().find(|b| b.id == target_id).and_then(|b| b.effects_chain.get_mut(effect_idx as usize)).map(|e| { let old = e.parameters.get(&param_key).copied().unwrap_or(value); e.parameters.insert(param_key.clone(), value); old }) };
        if let Some(old) = old_val { if let Ok(mut stack) = us_fx_par.lock() { stack.push(EditCommand::ChangeEffectParam { target_id, is_track, effect_index: effect_idx as usize, param_name: param_key.clone(), old_val: old, new_val: value }); } if is_track { pb_fx_par.update_track_effect_param(target_id, effect_idx as usize, &param_key, value); } else { pb_fx_par.update_bus_effect_param(target_id, effect_idx as usize, &param_key, value); } }
        drop(p);
        if let (Some(w), Some(effect)) = (ww_fx_par.upgrade(), effect_before) {
            update_eq_curve(&w, &effect, sr);
        }
    });

    // ---- POOL ----
    let p_pool = project.clone();
    let pb_pool = playback.clone();
    let wp_pool = waveform_peaks.clone();
    let sid_pool = selected_track_id.clone();
    let ww_pool = window_weak.clone();
    window.on_insert_pool_audio(move |path_str| {
        let path = std::path::PathBuf::from(path_str.to_string());
        let buffer = match crate::audio::loader::load_wav(&path) { Ok(b) => b, Err(_) => return };
        if let Ok(mut cache) = wp_pool.lock() { cache.insert(path_str.to_string(), WaveformPeaks::generate(&buffer, 2000)); }
        let mut p = match p_pool.lock() { Ok(p) => p, Err(_) => return };
        let sr = p.sample_rate;
        let pos = pb_pool.get_position();
        let tid = sid_pool.lock().ok().and_then(|s| *s).or_else(|| p.tracks.first().map(|t| t.id));
        if let Some(tid) = tid {
            let name = std::path::Path::new(&path_str.to_string()).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "Clip".to_string());
            let clip = AudioClip::new(path_str.to_string(), name, pos, buffer.samples.len() as u64 / buffer.channels as u64);
            if let Some(track) = p.tracks.iter_mut().find(|t| t.id == tid) { track.add_clip(clip); }
        }
        pb_pool.load_project_clips(&p.tracks, &p.buses, sr);
        drop(p);
        if let Some(w) = ww_pool.upgrade() { if let Ok(p) = p_pool.lock() { if let Ok(cache) = wp_pool.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
        if let Ok(p) = p_pool.lock() {
            let entries: Vec<PoolEntry> = crate::audio::pool::sync(&p).into_iter().enumerate().map(|(i, info)| {
                let rate_str = if info.sample_rate % 1000 == 0 { format!("{:.1}kHz", info.sample_rate as f64 / 1000.0) } else { format!("{}Hz", info.sample_rate) };
                let ch_str = if info.channels == 1 { "Mono".to_string() } else if info.channels == 2 { "Stereo".to_string() } else { format!("{}ch", info.channels) };
                let bit_str = if info.bit_depth == 32 { "32-bit".to_string() } else { format!("{}-bit", info.bit_depth) };
                PoolEntry { name: info.name.into(), info: format!("{} · {} · {} · {:.1}s", rate_str, ch_str, bit_str, info.duration_secs).into(), usage: info.usage_count as i32, path: info.path.into(), idx: i as i32 }
            }).collect();
            if let Some(w) = ww_pool.upgrade() { w.set_pool_entries(slint::ModelRc::new(slint::VecModel::from(entries))); }
        }
    });

    // ---- COPY/PASTE/DELETE CLIPS ----
    let cb_copy = clipboard.clone();
    let sel_copy = selected_clips.clone();
    let p_copy = project.clone();
    window.on_copy_clips(move || {
        if let Ok(mut sel) = sel_copy.lock() {
            if let Ok(mut cb) = cb_copy.lock() { cb.clear(); if let Ok(p) = p_copy.lock() { for track in p.tracks.iter() { for clip in track.clips.iter() { if sel.contains(&clip.id) { cb.push(clip.clone()); } } } } }
        }
    });

    let cb_paste = clipboard.clone();
    let p_paste = project.clone();
    let pb_paste = playback.clone();
    let sid_paste2 = selected_track_id.clone();
    let wp_paste = waveform_peaks.clone();
    let ww_paste = window_weak.clone();
    let us_paste = undo_stack.clone();
    window.on_paste_clips(move || {
        if let Ok(cb) = cb_paste.lock() { let clips = cb.clone(); if clips.is_empty() { return; } drop(cb);
        let mut created: Vec<(Uuid, AudioClip)> = Vec::new();
        if let Ok(mut p) = p_paste.lock() { let offset = pb_paste.get_position(); let selected_id = sid_paste2.lock().ok().and_then(|s| *s); let target_track_id = selected_id.or_else(|| p.tracks.first().map(|t| t.id));
        for clip in &clips { let mut new_clip = clip.clone(); new_clip.id = Uuid::new_v4(); new_clip.position = clip.position + offset; if let Some(tid) = target_track_id { if let Some(track) = p.tracks.iter_mut().find(|t| t.id == tid) { created.push((tid, new_clip.clone())); track.add_clip(new_clip); } } }
        if !created.is_empty() { if let Ok(mut stack) = us_paste.lock() { for (tid, clip) in created.iter() { stack.push(EditCommand::AddClip { track_id: *tid, clip: clip.clone() }); } } }
        pb_paste.load_project_clips(&p.tracks, &p.buses, p.sample_rate); drop(p);
        if let Some(w) = ww_paste.upgrade() { if let Ok(p) = p_paste.lock() { if let Ok(cache) = wp_paste.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } } } }
    });

    let sel_del = selected_clips.clone();
    let p_del_clips = project.clone();
    let pb_del_clips = playback.clone();
    let us_del_clips = undo_stack.clone();
    let wp_del_clips = waveform_peaks.clone();
    let ww_del_clips = window_weak.clone();
    window.on_delete_selected_clips(move || {
        if let Ok(mut sel) = sel_del.lock() { let ids: Vec<Uuid> = sel.iter().copied().collect(); if ids.is_empty() { return; } drop(sel);
        if let Ok(mut p) = p_del_clips.lock() { let mut track_groups: HashMap<Uuid, Vec<ClipSnapshot>> = HashMap::new();
        for track in p.tracks.iter() { for clip in track.clips.iter() { if ids.contains(&clip.id) { track_groups.entry(track.id).or_default().push(clip.clone()); } } }
        for (tid, clips) in track_groups { if let Some(track) = p.tracks.iter_mut().find(|t| t.id == tid) { track.clips.retain(|c| !clips.iter().any(|cl| cl.id == c.id)); } if let Ok(mut stack) = us_del_clips.lock() { stack.push(EditCommand::DeleteClips { track_id: tid, clips }); } }
        pb_del_clips.load_project_clips(&p.tracks, &p.buses, p.sample_rate); drop(p);
        if let Some(w) = ww_del_clips.upgrade() { if let Ok(p) = p_del_clips.lock() { if let Ok(cache) = wp_del_clips.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
        if let Ok(mut sel) = sel_del.lock() { sel.clear(); } } }
    });

    let sel_all = selected_clips.clone();
    let p_all = project.clone();
    let ww_all = window_weak.clone();
    window.on_select_all_clips(move || {
        if let Ok(mut sel) = sel_all.lock() { sel.clear(); if let Ok(p) = p_all.lock() { for track in p.tracks.iter() { for clip in track.clips.iter() { sel.insert(clip.id); } } } drop(sel); if let Some(w) = ww_all.upgrade() { if let Ok(sel) = sel_all.lock() { sync_selection(&w, &sel); } } }
    });

    let sid_del_bus = selected_bus_id.clone();
    let p_del_bus_menu = project.clone();
    let pb_del_bus_menu = playback.clone();
    let us_del_bus_menu = undo_stack.clone();
    let wp_del_bus_menu = waveform_peaks.clone();
    let ww_del_bus_menu = window_weak.clone();
    window.on_delete_selected_bus(move || {
        let id = match sid_del_bus.lock().ok().and_then(|s| *s) { Some(id) => id, None => return };
        let mut p = match p_del_bus_menu.lock() { Ok(p) => p, Err(_) => return };
        if let Some(idx) = p.buses.iter().position(|b| b.id == id) {
            let snapshot = p.buses[idx].clone(); p.buses.remove(idx);
            if let Ok(mut stack) = us_del_bus_menu.lock() { stack.push(EditCommand::RemoveBus { snapshot, index: idx }); }
            pb_del_bus_menu.load_project_clips(&p.tracks, &p.buses, p.sample_rate); drop(p);
            if let Ok(mut s) = sid_del_bus.lock() { *s = None; }
            if let Some(w) = ww_del_bus_menu.upgrade() { w.set_selected_bus_id("".into()); if let Ok(p) = p_del_bus_menu.lock() { if let Ok(cache) = wp_del_bus_menu.lock() { sync_project_to_timeline_with_waveforms(&p, &w, &cache); } } }
        }
    });
}

pub(crate) fn build_param_info(effect: &crate::project::track::EffectInstance) -> Vec<ParamInfo> {
    let mut params = Vec::new();
    for (name, value) in &effect.parameters {
        let (min, max, display) = match name.as_str() {
            "eq_low_freq" => (20.0, 500.0, format!("{:.0}Hz", value)),
            "eq_low_gain" => (-12.0, 12.0, format!("{:.1}dB", value)),
            "eq_mid_freq" => (200.0, 5000.0, format!("{:.0}Hz", value)),
            "eq_mid_gain" => (-12.0, 12.0, format!("{:.1}dB", value)),
            "eq_mid_q" => (0.1, 10.0, format!("{:.2}", value)),
            "eq_high_freq" => (2000.0, 20000.0, format!("{:.0}Hz", value)),
            "eq_high_gain" => (-12.0, 12.0, format!("{:.1}dB", value)),
            "comp_threshold" => (-60.0, 0.0, format!("{:.1}dB", value)),
            "comp_ratio" => (1.0, 20.0, format!("{:.1}:1", value)),
            "comp_attack" => (0.001, 1.0, format!("{:.1}ms", value * 1000.0)),
            "comp_release" => (0.01, 2.0, format!("{:.0}ms", value * 1000.0)),
            "comp_makeup" => (-20.0, 20.0, format!("{:.1}dB", value)),
            "reverb_room_size" => (0.0, 1.0, format!("{:.0}%", value * 100.0)),
            "reverb_damping" => (0.0, 1.0, format!("{:.0}%", value * 100.0)),
            "reverb_wet_dry" => (0.0, 1.0, format!("{:.0}%", value * 100.0)),
            "delay_time" => (0.001, 5.0, format!("{:.0}ms", value * 1000.0)),
            "delay_feedback" => (0.0, 0.95, format!("{:.0}%", value * 100.0)),
            "delay_mix" => (0.0, 1.0, format!("{:.0}%", value * 100.0)),
            _ => (-1.0, 1.0, format!("{:.2}", value)),
        };
        params.push(ParamInfo { name: name.as_str().into(), value: *value, min, max, display: display.into() });
    }
    params.sort_by(|a, b| a.name.cmp(&b.name));
    params
}

pub(crate) fn update_eq_curve(window: &MainWindow, effect: &crate::project::track::EffectInstance, sample_rate: u32) {
    if effect.effect_type != "Equalizer" { return; }
    let lf = *effect.parameters.get("eq_low_freq").unwrap_or(&100.0);
    let lg = *effect.parameters.get("eq_low_gain").unwrap_or(&0.0);
    let mf = *effect.parameters.get("eq_mid_freq").unwrap_or(&1000.0);
    let mg = *effect.parameters.get("eq_mid_gain").unwrap_or(&0.0);
    let mq = *effect.parameters.get("eq_mid_q").unwrap_or(&1.0);
    let hf = *effect.parameters.get("eq_high_freq").unwrap_or(&8000.0);
    let hg = *effect.parameters.get("eq_high_gain").unwrap_or(&0.0);
    let resp = crate::audio::effects::eq::compute_eq_response(lf, lg, mf, mg, mq, hf, hg, sample_rate, 128);
    let img = crate::audio::effects::eq::render_eq_curve_image(&resp, 152, 80);
    window.set_eq_curve_image(img);
}

pub(crate) fn get_snap_mode(window: &MainWindow) -> SnapMode {
    use crate::project::editing::{BeatDivision, Fps};
    if !window.get_snap_enabled() { return SnapMode::Off; }
    match window.get_snap_mode() {
        1 => {
            let div = match window.get_snap_param() {
                0 => BeatDivision::Whole, 1 => BeatDivision::Half,
                2 => BeatDivision::Quarter, 3 => BeatDivision::Eighth,
                4 => BeatDivision::Sixteenth, _ => BeatDivision::ThirtySecond,
            };
            SnapMode::Beats { division: div }
        }
        2 => {
            let ms = match window.get_snap_param() { 0 => 100, 1 => 250, 2 => 500, _ => 1000 };
            SnapMode::Time { resolution: (ms * 44100 / 1000) as u64 }
        }
        3 => {
            let fps = match window.get_snap_param() { 0 => Fps::Fps24, 1 => Fps::Fps25, 2 => Fps::Fps30, 3 => Fps::Fps30Drop, _ => Fps::Fps60 };
            SnapMode::Frames { fps }
        }
        _ => SnapMode::Adaptive,
    }
}
