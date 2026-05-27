#![cfg(feature = "qt")]

use std::pin::Pin;
use std::sync::atomic::Ordering;

#[cxx_qt::bridge(namespace = "ui_qt::timeline")]
pub mod timeline_bridge {
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(f64, playhead_x)]
        #[qproperty(f64, pixels_per_second)]
        #[qproperty(f64, timeline_scroll_x)]
        #[qproperty(bool, drag_active)]
        #[qproperty(f64, drag_overlay_x)]
        #[qproperty(f64, drag_overlay_y)]
        #[qproperty(f64, drag_overlay_w)]
        #[qproperty(f64, drag_overlay_h)]
        #[qproperty(i32, cursor_type)]
        #[qproperty(String, timeline_json)]
        type TimelineModel = super::TimelineModelRust;

        #[qinvokable]
        fn refresh(self: Pin<&mut TimelineModel>);
        #[qinvokable]
        fn sync_playhead(self: Pin<&mut TimelineModel>);
        #[qinvokable]
        fn zoom_in(self: Pin<&mut TimelineModel>);
        #[qinvokable]
        fn zoom_out(self: Pin<&mut TimelineModel>);
        #[qinvokable]
        fn on_timeline_pressed(self: Pin<&mut TimelineModel>, x: f64, y: f64);
        #[qinvokable]
        fn on_timeline_moved(self: Pin<&mut TimelineModel>, x: f64, y: f64);
        #[qinvokable]
        fn on_timeline_released(self: Pin<&mut TimelineModel>);
    }
}

pub struct TimelineModelRust {
    playhead_x: f64,
    pixels_per_second: f64,
    timeline_scroll_x: f64,
    drag_active: bool,
    drag_overlay_x: f64,
    drag_overlay_y: f64,
    drag_overlay_w: f64,
    drag_overlay_h: f64,
    cursor_type: i32,
    timeline_json: String,
    pub(crate) drag: std::sync::Arc<std::sync::Mutex<Option<crate::app::drag::DragState>>>,
    pub(crate) automation_drag: std::sync::Arc<std::sync::Mutex<Option<crate::app::drag::AutomationDragState>>>,
    pub(crate) cached_sr: std::sync::Arc<std::sync::Mutex<u32>>,
}

impl Default for TimelineModelRust {
    fn default() -> Self {
        Self {
            playhead_x: 0.0,
            pixels_per_second: 50.0,
            timeline_scroll_x: 0.0,
            drag_active: false,
            drag_overlay_x: 0.0,
            drag_overlay_y: 0.0,
            drag_overlay_w: 0.0,
            drag_overlay_h: 0.0,
            cursor_type: 0,
            timeline_json: String::from("{}"),
            drag: std::sync::Arc::new(std::sync::Mutex::new(None)),
            automation_drag: std::sync::Arc::new(std::sync::Mutex::new(None)),
            cached_sr: std::sync::Arc::new(std::sync::Mutex::new(44100)),
        }
    }
}

const TRACK_HEIGHT: f64 = 90.0;
const CLIP_EDGE_THRESHOLD: f64 = 10.0;
const AUTOMATION_LANE_HEIGHT: f64 = 20.0;

fn samples_to_pixels(samples: u64, pps: f64, sr: u32) -> f64 {
    (samples as f64 / sr as f64) * pps
}

fn pixels_to_samples(x: f64, pps: f64, sr: u32) -> u64 {
    ((x / pps) * sr as f64) as u64
}

impl timeline_bridge::TimelineModel {
    fn refresh(mut self: Pin<&mut Self>) {
        if let Some(state) = crate::ui_qt::state::get() {
            let pps = *self.pixels_per_second();
            let json = build_timeline_json(state, pps);
            self.as_mut().set_timeline_json(json);
            self.as_mut().sync_playhead();
        }
    }

    fn sync_playhead(mut self: Pin<&mut Self>) {
        let state = match crate::ui_qt::state::get() {
            Some(s) => s,
            None => return,
        };
        let pos = state.playback.get_position();
        let pps = *self.pixels_per_second();
        let sr = if let Ok(p) = state.project.lock() { 
            let s = p.sample_rate;
            if let Ok(mut c) = self.cached_sr.lock() { *c = s; }
            s
        } else { 44100 };
        
        let px = samples_to_pixels(pos, pps, sr);
        self.as_mut().set_playhead_x(px);
    }

    fn zoom_in(mut self: Pin<&mut Self>) {
        let pps = *self.pixels_per_second();
        self.as_mut().set_pixels_per_second((pps * 1.25).min(5000.0));
        self.as_mut().refresh();
    }

    fn zoom_out(mut self: Pin<&mut Self>) {
        let pps = *self.pixels_per_second();
        self.as_mut().set_pixels_per_second((pps * 0.8).max(2.0));
        self.as_mut().refresh();
    }

    fn on_timeline_pressed(mut self: Pin<&mut Self>, x: f64, y: f64) {
        let state = match crate::ui_qt::state::get() {
            Some(s) => s,
            None => return,
        };
        let pps = *self.pixels_per_second();
        let abs_x = x;
        let p = match state.project.lock() {
            Ok(p) => p,
            Err(_) => return,
        };
        let sr = p.sample_rate;

        let track_idx = (y / TRACK_HEIGHT) as usize;
        let track = match p.tracks.get(track_idx) {
            Some(t) => t,
            None => {
                drop(p);
                let sample_pos = pixels_to_samples(abs_x, pps, sr);
                state.playback.set_position(sample_pos);
                self.as_mut().set_drag_active(false);
                return;
            }
        };
        let track_y = track_idx as f64 * TRACK_HEIGHT;

        for clip in &track.clips {
            let clip_x = samples_to_pixels(clip.position, pps, sr);
            let clip_w = samples_to_pixels(clip.length, pps, sr).max(4.0);
            let clip_y = track_y;
            let clip_h = 90.0f64;

            if abs_x >= clip_x && abs_x <= clip_x + clip_w && y >= clip_y && y <= clip_y + clip_h {
                let local_x = abs_x - clip_x;
                let local_y = y - clip_y;
                let local_click_samples = pixels_to_samples(local_x, pps, sr);

                let edge = if local_x <= CLIP_EDGE_THRESHOLD {
                    Some(crate::app::drag::DragEdge::Left)
                } else if local_x >= clip_w - CLIP_EDGE_THRESHOLD {
                    Some(crate::app::drag::DragEdge::Right)
                } else {
                    None
                };

                let ds = crate::app::drag::DragState {
                    clip_id: clip.id,
                    track_id: track.id,
                    track_index: track_idx as i32,
                    original_pos: clip.position,
                    original_length: clip.length,
                    original_fade_in: clip.fade_in,
                    original_fade_out: clip.fade_out,
                    click_offset: local_click_samples,
                    drag_edge: edge,
                    destination_track_id: None,
                };
                if let Ok(mut d) = self.drag.lock() { *d = Some(ds); }

                if local_y >= clip_h - AUTOMATION_LANE_HEIGHT {
                    if let Some(ref param_name) = track.selected_automation_param {
                        if let Some(lane) = track.automation.get(param_name) {
                            let clip_start_sec = clip.position as f64 / sr as f64;
                            let click_time = clip_start_sec + local_x / pps;
                            for (pi, pt) in lane.points.iter().enumerate() {
                                if (pt.time - click_time).abs() <= 0.05 {
                                    if let Ok(mut ad) = self.automation_drag.lock() {
                                        *ad = Some(crate::app::drag::AutomationDragState {
                                            track_id: track.id,
                                            parameter_name: param_name.clone(),
                                            point_index: pi,
                                            original_point: pt.clone(),
                                            click_offset_time: click_time - pt.time,
                                            click_offset_value: 0.0,
                                        });
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }

                drop(p);
                self.as_mut().set_drag_active(true);
                self.as_mut().set_drag_overlay_x(clip_x);
                self.as_mut().set_drag_overlay_y(track_y);
                self.as_mut().set_drag_overlay_w(clip_w);
                self.as_mut().set_drag_overlay_h(clip_h);
                self.as_mut().set_cursor_type(5);
                return;
            }
        }

        drop(p);
        let sample_pos = pixels_to_samples(abs_x, pps, sr);
        state.playback.set_position(sample_pos);
        self.as_mut().set_drag_active(false);
    }

    fn on_timeline_moved(mut self: Pin<&mut Self>, x: f64, y: f64) {
        if !*self.drag_active() { return; }
        let state = match crate::ui_qt::state::get() {
            Some(s) => s,
            None => return,
        };
        let pps = *self.pixels_per_second();
        let abs_x = x;
        let sr = if let Ok(c) = self.cached_sr.lock() { *c } else { 44100 };

        if let Ok(mut auto_drag) = self.automation_drag.lock() {
            if let Some(ref ad) = *auto_drag {
                let current_time = abs_x / pps;
                let new_time = (current_time - ad.click_offset_time).max(0.0);
                let relative_y = y % TRACK_HEIGHT;
                let new_val = (1.0 - (relative_y / TRACK_HEIGHT)).clamp(0.0, 1.0) as f32;
                
                if let Ok(mut p) = state.project.lock() {
                    if let Some(track) = p.tracks.iter_mut().find(|t| t.id == ad.track_id) {
                        if let Some(lane) = track.automation.get_mut(&ad.parameter_name) {
                            if let Some(pt) = lane.points.get_mut(ad.point_index) {
                                pt.time = new_time;
                                pt.value = new_val;
                            }
                        }
                    }
                }
                return;
            }
        }

        let mut overlay_updates = None;
        if let Ok(drag_lock) = self.drag.lock() {
            if let Some(ref ds) = *drag_lock {
                if let Some(edge) = &ds.drag_edge {
                    match edge {
                        crate::app::drag::DragEdge::Left => {
                            let new_x = abs_x.max(0.0);
                            let orig_right = samples_to_pixels(ds.original_pos + ds.original_length, pps, sr);
                            let final_x = new_x.min(orig_right - 5.0);
                            overlay_updates = Some((Some(final_x), None, Some(orig_right - final_x)));
                        }
                        crate::app::drag::DragEdge::Right => {
                            let new_right = abs_x;
                            let orig_x = samples_to_pixels(ds.original_pos, pps, sr);
                            let final_right = new_right.max(orig_x + 5.0);
                            overlay_updates = Some((None, None, Some(final_right - orig_x)));
                        }
                        _ => {}
                    }
                } else {
                    let new_x = abs_x - samples_to_pixels(ds.click_offset, pps, sr);
                    let target_track_idx = (y / TRACK_HEIGHT) as i32;
                    overlay_updates = Some((Some(new_x.max(0.0)), Some(target_track_idx as f64 * TRACK_HEIGHT), Some(samples_to_pixels(ds.original_length, pps, sr))));
                }
            }
        }

        if let Some((x, y, w)) = overlay_updates {
            if let Some(val) = x { self.as_mut().set_drag_overlay_x(val); }
            if let Some(val) = y { self.as_mut().set_drag_overlay_y(val); }
            if let Some(val) = w { self.as_mut().set_drag_overlay_w(val); }
        }
    }

    fn on_timeline_released(mut self: Pin<&mut Self>) {
        let state = match crate::ui_qt::state::get() {
            Some(s) => s,
            None => return,
        };
        let pps = *self.pixels_per_second();
        let sr = if let Ok(c) = self.cached_sr.lock() { *c } else { 44100 };

        if let Ok(mut ad) = self.automation_drag.lock() { *ad = None; }

        let drag_info = if let Ok(mut d) = self.drag.lock() { d.take() } else { None };
        if let Some(ds) = drag_info {
            let new_x = *self.drag_overlay_x();
            let new_w = *self.drag_overlay_w();
            let new_track_idx = (*self.drag_overlay_y() / TRACK_HEIGHT) as usize;

            let new_pos = pixels_to_samples(new_x, pps, sr);
            let new_len = pixels_to_samples(new_w, pps, sr).max(1);

            if let Ok(mut p) = state.project.lock() {
                let mut found_clip = None;
                if let Some(old_track) = p.tracks.iter_mut().find(|t| t.id == ds.track_id) {
                    if let Some(clip_idx) = old_track.clips.iter().position(|c| c.id == ds.clip_id) {
                        let mut clip = old_track.clips.remove(clip_idx);
                        clip.position = new_pos;
                        clip.length = new_len;
                        found_clip = Some(clip);
                    }
                }
                
                if let Some(clip) = found_clip {
                    if let Some(new_track) = p.tracks.get_mut(new_track_idx) {
                        new_track.clips.push(clip);
                    } else if let Some(old_track) = p.tracks.iter_mut().find(|t| t.id == ds.track_id) {
                        old_track.clips.push(clip);
                    }
                }
                
                state.playback.load_project_clips(&p.tracks, &p.buses, sr);
            }
        }

        self.as_mut().set_drag_active(false);
        self.as_mut().set_cursor_type(0);
        self.as_mut().refresh();
    }
}

fn build_timeline_json(state: &crate::ui_qt::state::AppState, pps: f64) -> String {
    let p = match state.project.lock() {
        Ok(p) => p,
        Err(_) => return String::from("{}"),
    };
    let sr = p.sample_rate;

    // Calculate total duration
    let mut max_end: u64 = 0;
    for track in &p.tracks {
        for clip in &track.clips {
            let end = clip.position + clip.length;
            if end > max_end { max_end = end; }
        }
    }
    let duration_secs = if max_end > 0 { max_end as f64 / sr as f64 } else { 60.0 };

    // Build ruler ticks
    let mut ruler_ticks: Vec<serde_json::Value> = Vec::new();
    let beat_secs = 60.0 / p.bpm;
    let total_beats = (duration_secs / beat_secs).ceil() as u32 + 1;
    for beat in 0..total_beats {
        let t = beat as f64 * beat_secs;
        let x = t * pps;
        let is_major = beat % 4 == 0;
        let bar = beat / 4 + 1;
        let label = if is_major { format!("{}", bar) } else { String::new() };
        ruler_ticks.push(serde_json::json!({
            "x": x,
            "major": is_major,
            "label": label,
        }));
    }

    // Build tracks
    let track_height = 90.0f64;
    let mut tracks_json: Vec<serde_json::Value> = Vec::new();
    for (ti, track) in p.tracks.iter().enumerate() {
        let color_hex = format!("#{:02x}{:02x}{:02x}", track.color.0, track.color.1, track.color.2);
        let mut clips_json: Vec<serde_json::Value> = Vec::new();

        // Render waveform images for clips that have waveform peaks cached
        let waveform_cache = state.waveform_peaks.lock().ok();

        for clip in &track.clips {
            let start_px = clip.position as f64 / sr as f64 * pps;
            let width_px = (clip.length as f64 / sr as f64 * pps).max(4.0);
            let fade_in_px = clip.fade_in as f64 / sr as f64 * pps;
            let fade_out_px = clip.fade_out as f64 / sr as f64 * pps;

            // Waveform data URL
            let waveform_url = if let Some(ref cache) = waveform_cache {
                if let Some(peaks) = cache.get(&clip.source_path) {
                    crate::utils::waveform::render_waveform_data_url(peaks, width_px as u32, 80)
                } else { String::new() }
            } else { String::new() };

            clips_json.push(serde_json::json!({
                "x": start_px,
                "width": width_px,
                "name": clip.name,
                "waveform_url": waveform_url,
                "automation_url": "",
                "fade_in_px": fade_in_px,
                "fade_out_px": fade_out_px,
            }));
        }

        tracks_json.push(serde_json::json!({
            "id": track.id.to_string(),
            "name": track.name,
            "color": color_hex,
            "y": ti as f64 * track_height,
            "height": track_height,
            "clips": clips_json,
        }));
    }

    let result = serde_json::json!({
        "duration_secs": duration_secs,
        "zoom": pps,
        "ruler_ticks": ruler_ticks,
        "tracks": tracks_json,
    });

    serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
}
