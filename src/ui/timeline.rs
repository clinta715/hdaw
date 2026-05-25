use crate::project::undo::UndoStack;
use crate::project::Project;
use crate::ui::main_window::{ClipInfo, MainWindow, TrackInfo, BusInfo, SendInfo, EffectSlotInfo, RulerTick};
use crate::utils::waveform::{WaveformPeaks, render_waveform_image};
use slint::{Model, ModelRc, VecModel};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub fn clip_at_position(
    clips: &impl Model<Data = ClipInfo>,
    x: f32,
    y: f32,
) -> Option<(usize, ClipInfo)> {
    for i in 0..clips.row_count() {
        let clip = clips.row_data(i)?;
        let clip_x = clip.x;
        let clip_y = clip.track_index as f32 * 60.0 + 4.0;
        let clip_w = clip.width.max(4.0);
        let clip_h = 52.0;
        if x >= clip_x && x <= clip_x + clip_w && y >= clip_y && y <= clip_y + clip_h {
            return Some((i, clip));
        }
    }
    None
}

pub fn is_on_left_edge(local_x: f32) -> bool {
    local_x >= 0.0 && local_x <= 5.0
}

pub fn is_on_right_edge(local_x: f32, clip_w: f32) -> bool {
    local_x >= clip_w - 5.0 && local_x <= clip_w
}

pub fn compute_cursor_type(clips: &impl Model<Data = ClipInfo>, x: f32, y: f32, alt: bool) -> i32 {
    if let Some((_i, clip)) = clip_at_position(clips, x, y) {
        let local_x = x - clip.x;
        let local_y = y - (clip.track_index as f32 * 60.0 + 4.0);

        if local_y <= 20.0 {
            if local_x <= 10.0 || local_x >= clip.width - 10.0 {
                return 5;
            }
        } else {
            return if is_on_left_edge(local_x) {
                1
            } else if is_on_right_edge(local_x, clip.width) {
                if alt { 3 } else { 2 }
            } else {
                4
            };
        }
        4
    } else {
        0
    }
}

pub fn pixels_to_samples(pixels: f32, pixels_per_second: f32, sample_rate: u32) -> u64 {
    if pixels_per_second <= 0.0 {
        return 0;
    }
    ((pixels / pixels_per_second) * sample_rate as f32).max(0.0) as u64
}

pub fn samples_to_pixels(samples: u64, pixels_per_second: f32, sample_rate: u32) -> f32 {
    if sample_rate == 0 {
        return 0.0;
    }
    (samples as f32 / sample_rate as f32) * pixels_per_second
}

pub fn compute_ruler_ticks(
    pixels_per_second: f32,
    scroll_x: f32,
    visible_width: f32,
) -> Vec<RulerTick> {
    let pps = pixels_per_second.max(1.0) as f64;
    let start_px = scroll_x as f64;
    let end_px = ((scroll_x + visible_width).max(start_px as f32)) as f64;

    let target_px = 100.0;
    let raw_interval = target_px / pps;
    let nice = [0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0];
    let mut interval = nice[0];
    for n in &nice {
        if (*n - raw_interval).abs() < (interval - raw_interval).abs() {
            interval = *n;
        }
    }

    let start_secs = (start_px / pps / interval).floor() * interval;
    let end_secs = (end_px / pps / interval).ceil() * interval;

    let mut ticks = Vec::new();
    let mut t = start_secs;
    while t <= end_secs + 0.001 {
        let pos = (t * pps) as f32;
        let idx = ((t / interval).round()) as i64;
        let label = if interval >= 1.0 {
            let total_secs = t as u64;
            format!("{:02}:{:02}", total_secs / 60, total_secs % 60)
        } else {
            format!("{:.1}", t)
        };
        ticks.push(RulerTick {
            position: pos,
            label: label.into(),
            is_major: idx % 2 == 0,
        });
        t += interval;
    }
    ticks
}

pub fn sync_selection(window: &MainWindow, selected_ids: &HashSet<Uuid>) {
    let clips = window.get_clips();
    let new_clips: Vec<ClipInfo> = (0..clips.row_count())
        .map(|i| {
            let mut c = clips.row_data(i).unwrap();
            c.selected = Uuid::parse_str(c.id.as_str())
                .map(|id| selected_ids.contains(&id))
                .unwrap_or(false);
            c
        })
        .collect();
    window.set_clips(ModelRc::new(VecModel::from(new_clips)));
}

pub fn sync_undo_state(window: &MainWindow, undo_stack: &UndoStack) {
    window.set_can_undo(undo_stack.can_undo());
    window.set_can_redo(undo_stack.can_redo());
}

pub fn sync_project_to_timeline(project: &Project, window: &MainWindow) {
    sync_project_to_timeline_with_waveforms(project, window, &HashMap::new());
}

pub fn sync_project_to_timeline_with_waveforms(
    project: &Project,
    window: &MainWindow,
    waveform_cache: &HashMap<String, WaveformPeaks>,
) {
    let sample_rate = project.sample_rate;
    let px_per_sec = window.get_pixels_per_second() as f64;
    let clip_infos: Vec<ClipInfo> = project
        .tracks
        .iter()
        .enumerate()
        .flat_map(|(track_idx, track)| {
            track.clips.iter().map(move |clip| {
                let pos_px = (clip.position as f64 / sample_rate as f64 * px_per_sec) as f32;
                let w_px = ((clip.length as f64 / sample_rate as f64 * px_per_sec) as f32)
                    .max(4.0);

                let waveform_img = if clip.source_path.is_empty() {
                    waveform_cache.get("_test_tone_")
                } else {
                    waveform_cache.get(&clip.source_path)
                }.map(|peaks| {
                    let img_w = (w_px as u32).max(4).min(2000);
                    render_waveform_image(peaks, img_w, 34)
                }).unwrap_or_else(|| {
                    let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(1, 34);
                    buf.make_mut_bytes().fill(0);
                    slint::Image::from_rgba8_premultiplied(buf)
                });

                ClipInfo {
                    id: clip.id.to_string().into(),
                    track_index: track_idx as i32,
                    x: pos_px,
                    width: w_px,
                    name: clip.name.as_str().into(),
                    color: slint::Color::from_rgb_u8(clip.color.0, clip.color.1, clip.color.2)
                        .into(),
                    fade_in_width: (clip.fade_in as f64 / sample_rate as f64 * px_per_sec) as f32,
                    fade_out_width: (clip.fade_out as f64 / sample_rate as f64 * px_per_sec) as f32,
                    selected: false,
                    track_id: track.id.to_string().into(),
                    waveform: waveform_img,
                }
            })
        })
        .collect();

    let selected_id = window.get_selected_track_id();
    let selected_bus = window.get_selected_bus_id();
    let track_infos: Vec<TrackInfo> = project
        .tracks
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let pb_sends: Vec<SendInfo> = track.sends.iter().map(|s| {
                let target_name = project.buses.iter().find(|b| b.id == s.target_id)
                    .map(|b| b.name.as_str()).unwrap_or("Unknown Bus");
                SendInfo {
                    id: s.id.to_string().into(),
                    target_name: target_name.into(),
                    level: s.level,
                    is_active: s.is_active,
                }
            }).collect();

            let is_sel = selected_id == track.id.to_string().as_str();

            let fx_slots: Vec<EffectSlotInfo> = track.effects_chain.iter().enumerate().map(|(idx, e)| {
                let short_name = match e.effect_type.as_str() {
                    "Equalizer" => "EQ",
                    "Compressor" => "Comp",
                    "Reverb" => "Rev",
                    "Delay" => "Dly",
                    other => other,
                };
                EffectSlotInfo {
                    id: e.id.to_string().into(),
                    name: short_name.into(),
                    bypassed: e.bypass,
                    selected: false,
                    idx: idx as i32,
                }
            }).collect();

            TrackInfo {
                id: track.id.to_string().into(),
                index: i as i32,
                label: track.name.as_str().into(),
                volume: track.volume,
                pan: track.pan,
                mute: track.mute,
                solo: track.solo,
                armed: track.armed,
                selected: is_sel,
                sends: ModelRc::new(VecModel::from(pb_sends)),
                effects: ModelRc::new(VecModel::from(fx_slots)),
            }
        })
        .collect();

    let bus_infos: Vec<BusInfo> = project
        .buses
        .iter()
        .enumerate()
        .map(|(i, bus)| {
            let pb_sends: Vec<SendInfo> = bus.sends.iter().map(|s| {
                let target_name = project.buses.iter().find(|b| b.id == s.target_id)
                    .map(|b| b.name.as_str()).unwrap_or("Unknown Bus");
                SendInfo {
                    id: s.id.to_string().into(),
                    target_name: target_name.into(),
                    level: s.level,
                    is_active: s.is_active,
                }
            }).collect();

            let fx_slots: Vec<EffectSlotInfo> = bus.effects_chain.iter().enumerate().map(|(idx, e)| {
                let short_name = match e.effect_type.as_str() {
                    "Equalizer" => "EQ",
                    "Compressor" => "Comp",
                    "Reverb" => "Rev",
                    "Delay" => "Dly",
                    other => other,
                };
                EffectSlotInfo {
                    id: e.id.to_string().into(),
                    name: short_name.into(),
                    bypassed: e.bypass,
                    selected: false,
                    idx: idx as i32,
                }
            }).collect();

            BusInfo {
                id: bus.id.to_string().into(),
                index: i as i32,
                label: bus.name.as_str().into(),
                volume: bus.volume,
                pan: bus.pan,
                mute: bus.mute,
                solo: bus.solo,
                selected: selected_bus == bus.id.to_string().as_str(),
                sends: ModelRc::new(VecModel::from(pb_sends)),
                effects: ModelRc::new(VecModel::from(fx_slots)),
            }
        })
        .collect();

    window.set_clips(ModelRc::new(VecModel::from(clip_infos)));
    window.set_tracks(ModelRc::new(VecModel::from(track_infos)));
    window.set_buses(ModelRc::new(VecModel::from(bus_infos)));
    window.set_track_count(project.tracks.len() as i32);
}

pub fn setup_timeline_callbacks(
    _window: &MainWindow,
    _project: Arc<Mutex<Project>>,
) {
}
