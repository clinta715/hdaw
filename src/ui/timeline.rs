use crate::project::undo::UndoStack;
use crate::project::Project;
use crate::project::automation::AutomationLane;
use crate::ui::main_window::{ClipInfo, MainWindow, TrackInfo, BusInfo, SendInfo, EffectSlotInfo, RulerTick};
use crate::utils::waveform::{WaveformPeaks, render_waveform_image};
use slint::{Model, ModelRc, VecModel};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub const TRACK_HEIGHT: f32 = 90.0;
pub const CLIP_HEIGHT: f32 = 52.0;
pub const LANE_HEIGHT: f32 = 26.0;
pub const CLIP_Y_OFFSET: f32 = 4.0;

pub fn clip_at_position(
    clips: &impl Model<Data = ClipInfo>,
    x: f32,
    y: f32,
) -> Option<(usize, ClipInfo)> {
    for i in 0..clips.row_count() {
        let clip = clips.row_data(i)?;
        let clip_x = clip.x;
        let clip_y = clip.track_index as f32 * TRACK_HEIGHT + CLIP_Y_OFFSET;
        let clip_w = clip.width.max(4.0);
        let clip_h = CLIP_HEIGHT;
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
        let local_y = y - (clip.track_index as f32 * TRACK_HEIGHT + CLIP_Y_OFFSET);

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

enum RulerDetail { BarsOnly, Medium, Close }

fn compute_time_ticks(pps: f64, start_px: f64, end_px: f64) -> Vec<RulerTick> {
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

fn compute_bar_beat_ticks(
    pps: f64,
    start_px: f64,
    end_px: f64,
    bpm: f64,
    time_sig_numerator: u8,
    detail: RulerDetail,
) -> Vec<RulerTick> {
    let beats_per_second = bpm / 60.0;
    let beats_per_bar = time_sig_numerator as u64;

    let start_secs = start_px / pps;
    let end_secs = end_px / pps;
    let start_beat = (start_secs * beats_per_second).floor() as u64;
    let end_beat = (end_secs * beats_per_second).ceil() as u64 + 1;

    let mut ticks = Vec::new();
    for beat in start_beat..=end_beat {
        let beat_secs = beat as f64 / beats_per_second;
        let px = beat_secs * pps;
        let is_bar = beat % beats_per_bar == 0;

        match detail {
            RulerDetail::BarsOnly => {
                if !is_bar {
                    continue;
                }
                let bar_num = beat / beats_per_bar + 1;
                ticks.push(RulerTick {
                    position: px as f32,
                    label: format!("{:03}:01", bar_num).into(),
                    is_major: true,
                });
            }
            RulerDetail::Medium => {
                let label = if is_bar {
                    let bar_num = beat / beats_per_bar + 1;
                    format!("{:03}:01", bar_num)
                } else {
                    String::new()
                };
                ticks.push(RulerTick {
                    position: px as f32,
                    label: label.into(),
                    is_major: is_bar,
                });
            }
            RulerDetail::Close => {
                let bar_num = beat / beats_per_bar + 1;
                let beat_num = beat % beats_per_bar + 1;
                ticks.push(RulerTick {
                    position: px as f32,
                    label: format!("{:03}:{:02}", bar_num, beat_num).into(),
                    is_major: is_bar,
                });
            }
        }
    }
    ticks
}

fn compute_frame_ticks(
    pps: f64,
    start_px: f64,
    end_px: f64,
    snap_param: i32,
) -> Vec<RulerTick> {
    let fps: f64 = match snap_param {
        0 => 24.0,
        1 => 25.0,
        2 => 30.0,
        3 => 30.0,
        4 => 60.0,
        _ => 30.0,
    };

    let start_secs = start_px / pps;
    let end_secs = end_px / pps;
    let start_frame = (start_secs * fps).floor() as u64;
    let end_frame = (end_secs * fps).ceil() as u64 + 1;

    let mut ticks = Vec::new();
    for frame in start_frame..=end_frame {
        let frame_secs = frame as f64 / fps;
        let px = frame_secs * pps;
        let is_second = (frame as f64 % fps) < 0.001;
        let label = if is_second {
            let total_secs = frame_secs as u64;
            format!("{:02}:{:02}", total_secs / 60, total_secs % 60)
        } else {
            String::new()
        };
        ticks.push(RulerTick {
            position: px as f32,
            label: label.into(),
            is_major: is_second,
        });
    }
    ticks
}

pub const DISTANT_PPB: f64 = 25.0;
pub const MEDIUM_PPB: f64 = 80.0;

pub fn compute_ruler_ticks(
    pixels_per_second: f32,
    scroll_x: f32,
    visible_width: f32,
    bpm: f64,
    time_sig_numerator: u8,
    _time_sig_denominator: u8,
    snap_enabled: bool,
    snap_mode: i32,
    snap_param: i32,
    _sample_rate: u32,
) -> Vec<RulerTick> {
    let pps = pixels_per_second.max(1.0) as f64;
    let start_px = scroll_x as f64;
    let end_px = ((scroll_x + visible_width).max(start_px as f32)) as f64;
    let pixels_per_beat = pps * 60.0 / bpm;

    if !snap_enabled {
        return compute_time_ticks(pps, start_px, end_px);
    }

    match snap_mode {
        2 /* Time */ => compute_time_ticks(pps, start_px, end_px),
        0 /* Adaptive */ => {
            if pixels_per_beat < DISTANT_PPB {
                compute_time_ticks(pps, start_px, end_px)
            } else if pixels_per_beat < MEDIUM_PPB {
                compute_bar_beat_ticks(pps, start_px, end_px, bpm, time_sig_numerator, RulerDetail::Medium)
            } else {
                compute_bar_beat_ticks(pps, start_px, end_px, bpm, time_sig_numerator, RulerDetail::Close)
            }
        }
        1 /* Beats */ => {
            if pixels_per_beat < DISTANT_PPB {
                compute_bar_beat_ticks(pps, start_px, end_px, bpm, time_sig_numerator, RulerDetail::BarsOnly)
            } else if pixels_per_beat < MEDIUM_PPB {
                compute_bar_beat_ticks(pps, start_px, end_px, bpm, time_sig_numerator, RulerDetail::Medium)
            } else {
                compute_bar_beat_ticks(pps, start_px, end_px, bpm, time_sig_numerator, RulerDetail::Close)
            }
        }
        3 /* Frames */ => {
            if pixels_per_beat < DISTANT_PPB {
                compute_time_ticks(pps, start_px, end_px)
            } else {
                compute_frame_ticks(pps, start_px, end_px, snap_param)
            }
        }
        _ => compute_time_ticks(pps, start_px, end_px),
    }
}

pub fn hit_test_automation_point(
    lane: &AutomationLane,
    time: f64,
    _value: f32,
    tolerance: f64,
) -> Option<usize> {
    if !lane.enabled || lane.points.is_empty() {
        return None;
    }
    for (i, p) in lane.points.iter().enumerate() {
        if (p.time - time).abs() <= tolerance {
            return Some(i);
        }
    }
    None
}

pub fn render_automation_image(
    lanes: &[&AutomationLane],
    start_time: f64,
    end_time: f64,
    width: u32,
    height: u32,
) -> slint::Image {
    let w = width.max(1) as usize;
    let h = height.max(1) as usize;
    let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(w as u32, h as u32);
    let bytes = buf.make_mut_bytes();
    for pixel in bytes.chunks_exact_mut(4) {
        pixel[0] = 0;
        pixel[1] = 0;
        pixel[2] = 0;
        pixel[3] = 8;
    }

    for lane in lanes {
        if !lane.enabled || lane.points.len() < 2 {
            continue;
        }
        let (r, g, b) = lane.color;
        let range = end_time - start_time;
        if range <= 0.0 {
            continue;
        }

        for i in 0..lane.points.len() - 1 {
            let p1 = &lane.points[i];
            let p2 = &lane.points[i + 1];
            if p2.time < start_time || p1.time > end_time {
                continue;
            }
            let t1 = p1.time;
            let t2 = p2.time;
            let v1 = p1.value.clamp(0.0, 1.0);
            let v2 = p2.value.clamp(0.0, 1.0);

            let col_start = ((t1 - start_time) / range * (w as f64)).floor() as isize;
            let col_end = ((t2 - start_time) / range * (w as f64)).ceil() as isize;

            for col in col_start.max(0)..=col_end.min(w as isize - 1) {
                let frac = if t2 == t1 { 0.0 } else {
                    let col_time = start_time + col as f64 * range / w as f64;
                    ((col_time - t1) / (t2 - t1)).clamp(0.0, 1.0)
                };
                let val = v1 + (v2 - v1) * frac as f32;
                let row = ((1.0 - val) * (h - 1) as f32).round() as usize;
                let idx = row * w + col as usize;
                if let Some(pixel) = bytes.get_mut(idx * 4..idx * 4 + 4) {
                    let alpha = pixel[3].saturating_add(192);
                    pixel[0] = r;
                    pixel[1] = g;
                    pixel[2] = b;
                    pixel[3] = alpha;
                }
            }
        }
    }
    slint::Image::from_rgba8_premultiplied(buf)
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

                let auto_image = if let Some(ref param_name) = track.selected_automation_param {
                    if let Some(lane) = track.automation.get(param_name) {
                        let start_sec = clip.position as f64 / sample_rate as f64;
                        let end_sec = (clip.position + clip.length) as f64 / sample_rate as f64;
                        let img_w = (w_px as u32).max(4).min(2000);
                        render_automation_image(&[lane], start_sec, end_sec, img_w, 26)
                    } else {
                        let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(1, 26);
                        buf.make_mut_bytes().fill(0);
                        slint::Image::from_rgba8_premultiplied(buf)
                    }
                } else {
                    let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(1, 26);
                    buf.make_mut_bytes().fill(0);
                    slint::Image::from_rgba8_premultiplied(buf)
                };

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
                    auto_image,
                    auto_param_name: track.selected_automation_param.clone().unwrap_or_default().into(),
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

            let is_sel = selected_id == track.id.to_string().as_str();

            let output_name = match track.output_id {
                Some(oid) => project.buses.iter().find(|b| b.id == oid)
                    .map(|b| b.name.as_str()).unwrap_or("???"),
                None => "Mstr",
            };

            TrackInfo {
                id: track.id.to_string().into(),
                index: i as i32,
                label: track.name.as_str().into(),
                volume: track.volume,
                pan: track.pan,
                mute: track.mute,
                solo: track.solo,
                armed: track.armed,
                input_monitoring: track.input_monitoring,
                selected: is_sel,
                track_color: slint::Color::from_rgb_u8(track.color.0, track.color.1, track.color.2).into(),
                sends: ModelRc::new(VecModel::from(pb_sends)),
                effects: ModelRc::new(VecModel::from(fx_slots)),
                peak_l: 0.0,
                peak_r: 0.0,
                output_name: output_name.into(),
                auto_param_name: track.selected_automation_param.clone().unwrap_or_default().into(),
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

            let bus_output_name = match bus.output_id {
                Some(oid) => project.buses.iter().find(|b| b.id == oid)
                    .map(|b| b.name.as_str()).unwrap_or("???"),
                None => "Mstr",
            };

            BusInfo {
                id: bus.id.to_string().into(),
                index: i as i32,
                label: bus.name.as_str().into(),
                volume: bus.volume,
                pan: bus.pan,
                mute: bus.mute,
                solo: bus.solo,
                selected: selected_bus == bus.id.to_string().as_str(),
                track_color: slint::Color::from_rgb_u8(bus.color.0, bus.color.1, bus.color.2).into(),
                sends: ModelRc::new(VecModel::from(pb_sends)),
                effects: ModelRc::new(VecModel::from(fx_slots)),
                peak_l: 0.0,
                peak_r: 0.0,
                output_name: bus_output_name.into(),
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
