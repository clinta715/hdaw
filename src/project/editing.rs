use crate::project::clip::{AudioClip, TimeStretchParams, StretchAlgorithm};
use uuid::Uuid;

pub enum SnapMode {
    Off,
    Beats { division: BeatDivision },
    Time { resolution: u64 },
    Frames { fps: Fps },
    Adaptive,
}

pub enum BeatDivision {
    Whole,      // 1/1
    Half,       // 1/2
    Quarter,    // 1/4
    Eighth,     // 1/8
    Sixteenth,  // 1/16
    ThirtySecond, // 1/32
}

pub enum Fps {
    Fps24,
    Fps25,
    Fps30,
    Fps30Drop,
    Fps60,
}

impl BeatDivision {
    pub fn ticks_per_beat(&self) -> u64 {
        match self {
            BeatDivision::Whole => 4,
            BeatDivision::Half => 2,
            BeatDivision::Quarter => 1,
            BeatDivision::Eighth => 2,
            BeatDivision::Sixteenth => 4,
            BeatDivision::ThirtySecond => 8,
        }
    }
}

pub fn snap_to_grid(position: u64, bpm: f64, sample_rate: u32, mode: &SnapMode, relative_offset: u64, preserve_offset: bool) -> u64 {
    let grid_size = match mode {
        SnapMode::Off => return position,
        SnapMode::Beats { division } => {
            let beats_per_second = bpm / 60.0;
            let samples_per_beat = (sample_rate as f64 / beats_per_second) as u64;
            samples_per_beat / division.ticks_per_beat().max(1)
        }
        SnapMode::Time { resolution } => *resolution,
        SnapMode::Frames { fps } => {
            let fps_val = match fps {
                Fps::Fps24 => 24.0,
                Fps::Fps25 => 25.0,
                Fps::Fps30 | Fps::Fps30Drop => 30.0,
                Fps::Fps60 => 60.0,
            };
            (sample_rate as f64 / fps_val) as u64
        }
        SnapMode::Adaptive => {
            (sample_rate as u64 / 10).max(1)
        }
    };

    if grid_size == 0 {
        return position;
    }

    if preserve_offset && relative_offset > 0 {
        let snapped_base = (position / grid_size) * grid_size;
        snapped_base + relative_offset
    } else {
        let rounded = ((position as f64 / grid_size as f64).round()) as u64;
        (rounded * grid_size).max(0)
    }
}

pub fn split_clip(clip: &AudioClip, split_position: u64) -> Option<(AudioClip, AudioClip)> {
    if split_position <= clip.position || split_position >= clip.end_position() {
        return None;
    }

    let left_length = split_position - clip.position;
    let right_offset = clip.offset + left_length;
    let right_length = clip.length - left_length;

    let left = AudioClip {
        id: Uuid::new_v4(),
        source_path: clip.source_path.clone(),
        name: format!("{} (L)", clip.name),
        position: clip.position,
        offset: clip.offset,
        length: left_length,
        fade_in: clip.fade_in.min(left_length),
        fade_out: clip.fade_out.min(left_length),
        gain: clip.gain,
        time_stretch: clip.time_stretch.clone(),
        pitch_shift: clip.pitch_shift.clone(),
        color: clip.color,
    };

    let right = AudioClip {
        id: Uuid::new_v4(),
        source_path: clip.source_path.clone(),
        name: format!("{} (R)", clip.name),
        position: split_position,
        offset: right_offset,
        length: right_length,
        fade_in: clip.fade_in.min(right_length),
        fade_out: clip.fade_out.min(right_length),
        gain: clip.gain,
        time_stretch: clip.time_stretch.clone(),
        pitch_shift: clip.pitch_shift.clone(),
        color: clip.color,
    };

    Some((left, right))
}

pub const MIN_STRETCH_RATIO: f32 = 0.25;
pub const MAX_STRETCH_RATIO: f32 = 4.0;

pub fn clamp_stretch_ratio(ratio: f32) -> f32 {
    ratio.clamp(MIN_STRETCH_RATIO, MAX_STRETCH_RATIO)
}

pub fn create_time_stretch_params(ratio: f32) -> TimeStretchParams {
    let algorithm = if ratio < 0.5 || ratio > 2.0 {
        StretchAlgorithm::High
    } else {
        StretchAlgorithm::Normal
    };

    TimeStretchParams {
        ratio: clamp_stretch_ratio(ratio),
        algorithm,
    }
}

pub fn create_auto_crossfade(clip_a: &mut AudioClip, clip_b: &mut AudioClip) {
    let overlap_start = clip_a.position.max(clip_b.position);
    let overlap_end = clip_a.end_position().min(clip_b.end_position());

    if overlap_end <= overlap_start {
        return;
    }

    let overlap_dur = overlap_end - overlap_start;
    let half = overlap_dur / 2;

    clip_a.fade_out = half;
    clip_b.fade_in = half;
}

pub fn clear_crossfade(clip_a: &mut AudioClip, clip_b: &mut AudioClip) {
    clip_a.fade_out = 0;
    clip_b.fade_in = 0;
}

pub fn clips_overlap(a: &AudioClip, b: &AudioClip) -> bool {
    a.position < b.end_position() && b.position < a.end_position()
}
