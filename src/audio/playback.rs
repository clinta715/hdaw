use crate::audio::buffer::AudioBuffer;
use crate::audio::effects::Effect;
use crate::audio::effects::factory;
use crate::audio::loader;
use crate::audio::record::RecordState;
use crate::project::clip::{PitchShiftParams, TimeStretchParams};
use crate::project::track::{EffectInstance, Track};
use crate::project::bus::Bus;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PlaybackClip {
    pub buffer_key: String,
    pub start: u64,
    pub offset: u64,
    pub length: u64,
    pub fade_in: u64,
    pub fade_out: u64,
    pub gain: f32,
    pub track_id: Uuid,
    pub time_stretch: Option<TimeStretchParams>,
    pub pitch_shift: Option<PitchShiftParams>,
}

#[derive(Debug, Clone)]
pub struct PlaybackSend {
    pub target_id: Uuid,
    pub level: f32,
    pub is_active: bool,
    pub pre_fader: bool,
}

#[derive(Debug, Clone)]
pub struct PlaybackTrack {
    pub id: Uuid,
    pub volume: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub armed: bool,
    pub input_monitoring: bool,
    pub output_id: Option<Uuid>,
    pub sends: Vec<PlaybackSend>,
}

#[derive(Debug, Clone)]
pub struct PlaybackBus {
    pub id: Uuid,
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub output_id: Option<Uuid>,
    pub sends: Vec<PlaybackSend>,
}

pub struct PlaybackState {
    pub playing: bool,
    pub position: u64,
    pub project_sample_rate: u32,
    pub clips: Vec<PlaybackClip>,
    pub tracks: Vec<PlaybackTrack>,
    pub buffers: HashMap<String, AudioBuffer>,
    pub buses: Vec<PlaybackBus>,
    pub loop_enabled: bool,
    pub loop_start: u64,
    pub loop_end: u64,
    pub recording: RecordState,
    scratch_master: Vec<f32>,
    scratch_tracks: HashMap<Uuid, Vec<f32>>,
    scratch_buses: HashMap<Uuid, Vec<f32>>,
    track_effects: HashMap<Uuid, Vec<Box<dyn Effect>>>,
    bus_effects: HashMap<Uuid, Vec<Box<dyn Effect>>>,
}

impl PlaybackState {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            playing: false,
            position: 0,
            project_sample_rate: sample_rate,
            clips: Vec::new(),
            tracks: Vec::new(),
            buffers: HashMap::new(),
            buses: Vec::new(),
            loop_enabled: false,
            loop_start: 0,
            loop_end: 0,
            recording: RecordState::new(),
            scratch_master: Vec::new(),
            scratch_tracks: HashMap::new(),
            scratch_buses: HashMap::new(),
            track_effects: HashMap::new(),
            bus_effects: HashMap::new(),
        }
    }

    fn ensure_scratch(&mut self, size: usize) {
        if self.scratch_master.len() < size {
            self.scratch_master.resize(size, 0.0);
        }
        let track_ids: Vec<Uuid> = self.tracks.iter().map(|t| t.id).collect();
        for id in track_ids {
            let entry = self.scratch_tracks.entry(id).or_insert_with(Vec::new);
            if entry.len() < size {
                entry.resize(size, 0.0);
            }
        }
        let bus_ids: Vec<Uuid> = self.buses.iter().map(|b| b.id).collect();
        for id in bus_ids {
            let entry = self.scratch_buses.entry(id).or_insert_with(Vec::new);
            if entry.len() < size {
                entry.resize(size, 0.0);
            }
        }
    }
}

#[derive(Clone)]
pub struct PlaybackManager {
    state: Arc<Mutex<PlaybackState>>,
    input_rx: Option<crossbeam::channel::Receiver<Vec<f32>>>,
}

impl PlaybackManager {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            state: Arc::new(Mutex::new(PlaybackState::new(sample_rate))),
            input_rx: None,
        }
    }

    pub fn new_with_recording(sample_rate: u32) -> (Self, crossbeam::channel::Sender<Vec<f32>>) {
        let (tx, rx) = crossbeam::channel::bounded(32);
        (
            Self {
                state: Arc::new(Mutex::new(PlaybackState::new(sample_rate))),
                input_rx: Some(rx),
            },
            tx,
        )
    }

    pub fn has_recording(&self) -> bool {
        self.input_rx.is_some()
    }

    pub fn start_recording(&self, armed_ids: &HashSet<Uuid>, position: u64, sample_rate: u32) {
        if let Ok(mut s) = self.state.lock() {
            s.recording.start(armed_ids, position, sample_rate);
        }
    }

    pub fn stop_recording(&self) -> Option<(HashMap<Uuid, Vec<f32>>, u64)> {
        if let Ok(mut s) = self.state.lock() {
            if s.recording.is_recording() {
                Some(s.recording.stop())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn is_recording(&self) -> bool {
        self.state.lock().map(|s| s.recording.is_recording()).unwrap_or(false)
    }

    pub fn set_armed(&self, track_id: Uuid, armed: bool) {
        if let Ok(mut s) = self.state.lock() {
            if let Some(track) = s.tracks.iter_mut().find(|t| t.id == track_id) {
                track.armed = armed;
            }
            if armed {
                s.recording.armed_track_ids.insert(track_id);
            } else {
                s.recording.armed_track_ids.remove(&track_id);
            }
        }
    }

    pub fn set_input_monitoring(&self, track_id: Uuid, enabled: bool) {
        if let Ok(mut s) = self.state.lock() {
            if let Some(track) = s.tracks.iter_mut().find(|t| t.id == track_id) {
                track.input_monitoring = enabled;
            }
        }
    }

    pub fn state(&self) -> &Arc<Mutex<PlaybackState>> {
        &self.state
    }

    pub fn set_playing(&self, playing: bool) {
        if let Ok(mut s) = self.state.lock() {
            s.playing = playing;
        }
    }

    pub fn is_playing(&self) -> bool {
        self.state.lock().map(|s| s.playing).unwrap_or(false)
    }

    pub fn set_position(&self, position: u64) {
        if let Ok(mut s) = self.state.lock() {
            s.position = position;
        }
    }

    pub fn get_position(&self) -> u64 {
        self.state.lock().map(|s| s.position).unwrap_or(0)
    }

    pub fn set_loop(&self, enabled: bool, start: u64, end: u64) {
        if let Ok(mut s) = self.state.lock() {
            s.loop_enabled = enabled;
            s.loop_start = start;
            s.loop_end = end;
        }
    }

    pub fn set_loop_enabled(&self, enabled: bool) {
        if let Ok(mut s) = self.state.lock() {
            s.loop_enabled = enabled;
        }
    }

    pub fn is_loop_enabled(&self) -> bool {
        self.state.lock().map(|s| s.loop_enabled).unwrap_or(false)
    }

    pub fn update_track_params(&self, track_id: Uuid, volume: f32, pan: f32, mute: bool, solo: bool) {
        if let Ok(mut s) = self.state.lock() {
            if let Some(track) = s.tracks.iter_mut().find(|t| t.id == track_id) {
                track.volume = if mute { 0.0 } else { volume };
                track.pan = pan;
                track.mute = mute;
                track.solo = solo;
            }
        }
    }

    pub fn update_bus_params(&self, bus_id: Uuid, volume: f32, pan: f32, mute: bool, solo: bool) {
        if let Ok(mut s) = self.state.lock() {
            for bus in s.buses.iter_mut() {
                if bus.id == bus_id {
                    bus.volume = volume;
                    bus.pan = pan;
                    bus.mute = mute;
                    bus.solo = solo;
                }
            }
        }
    }

    pub fn update_track_effect_param(&self, track_id: Uuid, effect_index: usize, param_name: &str, value: f32) {
        if let Ok(mut s) = self.state.lock() {
            if let Some(chain) = s.track_effects.get_mut(&track_id) {
                if let Some(effect) = chain.get_mut(effect_index) {
                    effect.set_parameter(param_name, value);
                }
            }
        }
    }

    pub fn update_bus_effect_param(&self, bus_id: Uuid, effect_index: usize, param_name: &str, value: f32) {
        if let Ok(mut s) = self.state.lock() {
            if let Some(chain) = s.bus_effects.get_mut(&bus_id) {
                if let Some(effect) = chain.get_mut(effect_index) {
                    effect.set_parameter(param_name, value);
                }
            }
        }
    }

    pub fn set_track_effect_bypass(&self, track_id: Uuid, effect_index: usize, bypass: bool) {
        if let Ok(mut s) = self.state.lock() {
            if let Some(chain) = s.track_effects.get_mut(&track_id) {
                if let Some(effect) = chain.get_mut(effect_index) {
                    effect.set_bypassed(bypass);
                }
            }
        }
    }

    pub fn set_bus_effect_bypass(&self, bus_id: Uuid, effect_index: usize, bypass: bool) {
        if let Ok(mut s) = self.state.lock() {
            if let Some(chain) = s.bus_effects.get_mut(&bus_id) {
                if let Some(effect) = chain.get_mut(effect_index) {
                    effect.set_bypassed(bypass);
                }
            }
        }
    }

    pub fn reload_track_effects(&self, track_id: Uuid, instances: &[EffectInstance], sample_rate: u32) {
        if let Ok(mut s) = self.state.lock() {
            s.track_effects.insert(track_id, factory::create_effect_chain(instances, sample_rate));
        }
    }

    pub fn reload_bus_effects(&self, bus_id: Uuid, instances: &[EffectInstance], sample_rate: u32) {
        if let Ok(mut s) = self.state.lock() {
            s.bus_effects.insert(bus_id, factory::create_effect_chain(instances, sample_rate));
        }
    }

    pub fn load_project_clips(&self, tracks: &[Track], buses: &[Bus], project_sample_rate: u32) {
        let mut clips = Vec::new();
        let mut pb_tracks = Vec::new();
        let mut pb_buses = Vec::new();
        let mut buffers: HashMap<String, AudioBuffer> = HashMap::new();
        let mut track_fx: HashMap<Uuid, Vec<Box<dyn Effect>>> = HashMap::new();
        let mut bus_fx: HashMap<Uuid, Vec<Box<dyn Effect>>> = HashMap::new();

        for bus in buses {
            let pb_sends: Vec<PlaybackSend> = bus.sends.iter().map(|s| PlaybackSend {
                target_id: s.target_id,
                level: s.level,
                is_active: s.is_active,
                pre_fader: s.pre_fader,
            }).collect();

            bus_fx.insert(bus.id, factory::create_effect_chain(&bus.effects_chain, project_sample_rate));

            pb_buses.push(PlaybackBus {
                id: bus.id,
                name: bus.name.clone(),
                volume: bus.volume,
                pan: bus.pan,
                mute: bus.mute,
                solo: bus.solo,
                output_id: bus.output_id,
                sends: pb_sends,
            });
        }

        let sorted_buses = Self::sort_buses_topologically(&pb_buses);

        for track in tracks {
            let pb_sends: Vec<PlaybackSend> = track.sends.iter().map(|s| PlaybackSend {
                target_id: s.target_id,
                level: s.level,
                is_active: s.is_active,
                pre_fader: s.pre_fader,
            }).collect();

            track_fx.insert(track.id, factory::create_effect_chain(&track.effects_chain, project_sample_rate));

            pb_tracks.push(PlaybackTrack {
                id: track.id,
                volume: track.volume,
                pan: track.pan,
                mute: track.mute,
                solo: track.solo,
                armed: track.armed,
                input_monitoring: track.input_monitoring,
                output_id: track.output_id,
                sends: pb_sends,
            });

            for clip in &track.clips {
                let path_str = clip.source_path.clone();
                let buffer_key = if path_str.is_empty() {
                    "_test_tone_".to_string()
                } else {
                    path_str.clone()
                };

                if !buffers.contains_key(&buffer_key) {
                    let buffer = if path_str.is_empty() {
                        Some(loader::generate_test_tone(project_sample_rate, 2.0))
                    } else if Path::new(&path_str).exists() {
                        loader::load_wav(Path::new(&path_str)).ok()
                    } else {
                        None
                    };
                    if let Some(buf) = buffer {
                        buffers.insert(buffer_key.clone(), buf);
                    }
                }

                if buffers.contains_key(&buffer_key) {
                    clips.push(PlaybackClip {
                        buffer_key,
                        start: clip.position,
                        offset: clip.offset,
                        length: clip.length,
                        fade_in: clip.fade_in,
                        fade_out: clip.fade_out,
                        gain: clip.gain,
                        track_id: track.id,
                        time_stretch: clip.time_stretch.clone(),
                        pitch_shift: clip.pitch_shift.clone(),
                    });
                }
            }
        }

        clips.sort_by_key(|c| c.start);

        if let Ok(mut s) = self.state.lock() {
            s.clips = clips;
            s.tracks = pb_tracks;
            s.buffers = buffers;
            s.buses = sorted_buses;
            s.project_sample_rate = project_sample_rate;
            s.track_effects = track_fx;
            s.bus_effects = bus_fx;
        }
    }

    fn sort_buses_topologically(buses: &[PlaybackBus]) -> Vec<PlaybackBus> {
        let mut sorted = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();

        fn visit(
            bus_id: Uuid,
            buses: &[PlaybackBus],
            sorted: &mut Vec<PlaybackBus>,
            visited: &mut std::collections::HashSet<Uuid>,
            visiting: &mut std::collections::HashSet<Uuid>,
        ) {
            if visited.contains(&bus_id) { return; }
            if visiting.contains(&bus_id) {
                tracing::warn!("Feedback loop detected in bus routing for bus {}", bus_id);
                return;
            }

            visiting.insert(bus_id);

            if let Some(bus) = buses.iter().find(|b| b.id == bus_id) {
                if let Some(output_id) = bus.output_id {
                    visit(output_id, buses, sorted, visited, visiting);
                }
                for send in &bus.sends {
                    if send.is_active {
                        visit(send.target_id, buses, sorted, visited, visiting);
                    }
                }
                sorted.push(bus.clone());
            }

            visiting.remove(&bus_id);
            visited.insert(bus_id);
        }

        for bus in buses {
            visit(bus.id, buses, &mut sorted, &mut visited, &mut visiting);
        }

        sorted.reverse();
        sorted
    }

    fn read_clip_sample(
        buf: &AudioBuffer,
        src_pos: f64,
        channel: usize,
        project_sr: u32,
    ) -> f32 {
        let src_sr = buf.sample_rate.max(1);
        let ratio = project_sr as f64 / src_sr as f64;
        let src_pos_adjusted = src_pos * ratio;
        let src_frame = src_pos_adjusted as usize;
        let frac = (src_pos_adjusted - src_frame as f64) as f32;
        let src_channels = buf.channels as usize;
        let read_ch = channel.min(src_channels.saturating_sub(1));

        let sample_0 = if src_channels == 1 {
            buf.get_sample(src_frame)
        } else {
            buf.get_sample(src_frame * src_channels + read_ch)
        }.unwrap_or(0.0);
        let sample_1 = if src_channels == 1 {
            buf.get_sample(src_frame + 1)
        } else {
            buf.get_sample((src_frame + 1) * src_channels + read_ch)
        }.unwrap_or(0.0);

        sample_0 + frac * (sample_1 - sample_0)
    }

    fn constant_power_pan(pan: f32, ch: usize) -> f32 {
        if ch == 0 {
            (1.0 - pan).max(0.0).sqrt()
        } else {
            (1.0 + pan).max(0.0).sqrt()
        }
    }

    pub fn fill_buffer(&self, output: &mut [f32], output_sample_rate: u32) {
        let mut state = match self.state.lock() {
            Ok(s) => s,
            Err(_) => {
                for s in output.iter_mut() { *s = 0.0; }
                return;
            }
        };

        let any_monitoring = state.tracks.iter().any(|t| t.input_monitoring);
        let is_recording = state.recording.is_recording();
        let has_work = state.playing || any_monitoring || is_recording;

        if !has_work {
            for s in output.iter_mut() { *s = 0.0; }
            return;
        }

        let project_sr = state.project_sample_rate.max(1);
        let output_sr = output_sample_rate.max(1);
        let num_frames_out = output.len() / 2;
        let num_channels_out: usize = 2;
        let current_pos = state.position;
        let frames_project = (num_frames_out as u64 * project_sr as u64) / output_sr as u64;

        let scratch_size = output.len();
        state.ensure_scratch(scratch_size);

        let mut scratch_master = std::mem::take(&mut state.scratch_master);
        let mut scratch_tracks = std::mem::take(&mut state.scratch_tracks);
        let mut scratch_buses = std::mem::take(&mut state.scratch_buses);
        let mut track_effects = std::mem::take(&mut state.track_effects);
        let mut bus_effects = std::mem::take(&mut state.bus_effects);

        if scratch_master.len() < scratch_size {
            scratch_master.resize(scratch_size, 0.0);
        }
        for i in 0..scratch_size {
            scratch_master[i] = 0.0f32;
        }
        for buf in scratch_tracks.values_mut() {
            if buf.len() < scratch_size { buf.resize(scratch_size, 0.0); }
            for i in 0..scratch_size { buf[i] = 0.0; }
        }
        for buf in scratch_buses.values_mut() {
            if buf.len() < scratch_size { buf.resize(scratch_size, 0.0); }
            for i in 0..scratch_size { buf[i] = 0.0; }
        }

        // Drain input samples from recording channel
        if let Some(ref rx) = self.input_rx {
            while let Ok(input_chunk) = rx.try_recv() {
                if state.recording.is_recording() {
                    state.recording.append_samples(&input_chunk);
                }
                // Software monitoring: mix into armed+monitoring track scratch buffers
                for track in &state.tracks {
                    if track.input_monitoring {
                        if let Some(track_buf) = scratch_tracks.get_mut(&track.id) {
                            let monitor_gain = 0.7f32;
                            for (i, &s) in input_chunk.iter().enumerate() {
                                if i < track_buf.len() {
                                    track_buf[i] += s * monitor_gain;
                                }
                            }
                        }
                    }
                }
            }
        }

        let clips = state.clips.clone();
        let buffers = state.buffers.clone();
        let tracks = state.tracks.clone();
        let buses = state.buses.clone();

        let track_map: HashMap<Uuid, &PlaybackTrack> = tracks.iter().map(|t| (t.id, t)).collect();
        let any_track_soloed = tracks.iter().any(|t| t.solo);
        let any_bus_soloed = buses.iter().any(|b| b.solo);

        if state.playing {
            for clip in &clips {
                let clip_end = clip.start + clip.length;
                if clip_end <= current_pos || clip.start >= current_pos + frames_project {
                    continue;
                }

                let track = match track_map.get(&clip.track_id) {
                    Some(t) => *t,
                    None => continue,
                };

                if track.mute { continue; }
                if any_track_soloed && !track.solo { continue; }

                let src_buf = match buffers.get(&clip.buffer_key) {
                    Some(b) => b,
                    None => continue,
                };

                let local_start = current_pos.saturating_sub(clip.start);
                let local_end = (current_pos + frames_project - clip.start).min(clip.length);

                for local_frame in local_start..local_end {
                    let project_frame = clip.start + local_frame - current_pos;
                    if project_frame >= frames_project { break; }
                    let out_idx = project_frame as usize;
                    let src_project_pos = clip.offset as f64
                        + local_frame as f64 / clip.time_stretch.as_ref().map(|ts| ts.ratio as f64).unwrap_or(1.0);

                    let fade_gain = if local_frame < clip.fade_in {
                        local_frame as f32 / clip.fade_in.max(1) as f32
                    } else if clip.length - local_frame <= clip.fade_out {
                        (clip.length - local_frame) as f32 / clip.fade_out.max(1) as f32
                    } else {
                        1.0
                    };

                    let clip_gain = fade_gain * clip.gain;

                    for ch in 0..num_channels_out {
                        let sample = Self::read_clip_sample(src_buf, src_project_pos, ch, project_sr);
                        let idx = out_idx * num_channels_out + ch;

                        if let Some(track_buf) = scratch_tracks.get_mut(&clip.track_id) {
                            if idx < track_buf.len() {
                                track_buf[idx] += sample * clip_gain;
                            }
                        }

                        for send in &track.sends {
                            if !send.is_active { continue; }
                            if send.pre_fader {
                                if let Some(send_buf) = scratch_buses.get_mut(&send.target_id) {
                                    if idx < send_buf.len() {
                                        send_buf[idx] += sample * clip_gain * send.level;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        for track in &tracks {
            if track.mute { continue; }
            if any_track_soloed && !track.solo { continue; }

            if let Some(track_buf) = scratch_tracks.get_mut(&track.id) {
                if let Some(fx_chain) = track_effects.get_mut(&track.id) {
                    let samples = std::mem::take(track_buf);
                    let mut audio_buf = AudioBuffer::from_samples(samples, 2, project_sr);
                    for effect in fx_chain.iter_mut() {
                        effect.process(&mut audio_buf);
                    }
                    *track_buf = audio_buf.samples;
                }
            }
        }

        for track in &tracks {
            if track.mute { continue; }
            if any_track_soloed && !track.solo { continue; }

            let track_buf = match scratch_tracks.get(&track.id) {
                Some(b) => b,
                None => continue,
            };
            let track_data = track_buf.clone();

            for i in 0..num_frames_out {
                for ch in 0..num_channels_out {
                    let idx = i * num_channels_out + ch;
                    if idx >= track_data.len() { continue; }
                    let sample = track_data[idx];
                    let pan_gain = Self::constant_power_pan(track.pan, ch);
                    let final_sample = sample * track.volume * pan_gain;

                    if let Some(bus_id) = track.output_id {
                        if let Some(target_buf) = scratch_buses.get_mut(&bus_id) {
                            if idx < target_buf.len() {
                                target_buf[idx] += final_sample;
                            }
                        } else {
                            if idx < scratch_master.len() { scratch_master[idx] += final_sample; }
                        }
                    } else {
                        if idx < scratch_master.len() { scratch_master[idx] += final_sample; }
                    }

                    for send in &track.sends {
                        if !send.is_active || send.pre_fader { continue; }
                        if let Some(send_buf) = scratch_buses.get_mut(&send.target_id) {
                            if idx < send_buf.len() {
                                send_buf[idx] += sample * track.volume * send.level * pan_gain;
                            }
                        }
                    }
                }
            }
        }

        for bus in &buses {
            if bus.mute { continue; }
            if any_bus_soloed && !bus.solo { continue; }

            if let Some(bus_buf) = scratch_buses.get_mut(&bus.id) {
                if let Some(fx_chain) = bus_effects.get_mut(&bus.id) {
                    let samples = std::mem::take(bus_buf);
                    let mut audio_buf = AudioBuffer::from_samples(samples, 2, project_sr);
                    for effect in fx_chain.iter_mut() {
                        effect.process(&mut audio_buf);
                    }
                    *bus_buf = audio_buf.samples;
                }
            }
        }

        for bus in &buses {
            if bus.mute { continue; }
            if any_bus_soloed && !bus.solo { continue; }

            let bus_buf = match scratch_buses.get(&bus.id) {
                Some(b) => b,
                None => continue,
            };
            let bus_data = bus_buf.clone();

            for i in 0..num_frames_out {
                for ch in 0..num_channels_out {
                    let idx = i * num_channels_out + ch;
                    if idx >= bus_data.len() { continue; }
                    let sample = bus_data[idx];
                    let pan_gain = Self::constant_power_pan(bus.pan, ch);
                    let final_sample = sample * bus.volume * pan_gain;

                    if let Some(target_id) = bus.output_id {
                        if let Some(target_buf) = scratch_buses.get_mut(&target_id) {
                            if idx < target_buf.len() {
                                target_buf[idx] += final_sample;
                            }
                        } else {
                            if idx < scratch_master.len() { scratch_master[idx] += final_sample; }
                        }
                    } else {
                        if idx < scratch_master.len() { scratch_master[idx] += final_sample; }
                    }

                    for send in &bus.sends {
                        if !send.is_active { continue; }
                        if let Some(send_buf) = scratch_buses.get_mut(&send.target_id) {
                            if idx < send_buf.len() {
                                let send_gain = if send.pre_fader { send.level } else { bus.volume * send.level };
                                send_buf[idx] += sample * send_gain * pan_gain;
                            }
                        }
                    }
                }
            }
        }

        for i in 0..output.len() {
            output[i] = scratch_master.get(i).copied().unwrap_or(0.0).clamp(-1.0, 1.0);
        }

        state.scratch_master = scratch_master;
        state.scratch_tracks = scratch_tracks;
        state.scratch_buses = scratch_buses;
        state.track_effects = track_effects;
        state.bus_effects = bus_effects;

        if state.playing {
            let next_pos = current_pos + frames_project;
            if state.loop_enabled && state.loop_end > state.loop_start {
                if next_pos >= state.loop_end {
                    let loop_len = state.loop_end - state.loop_start;
                    state.position = state.loop_start + (next_pos - state.loop_end) % loop_len;
                } else {
                    state.position = next_pos;
                }
            } else {
                state.position = next_pos;
            }
        }
    }
}
