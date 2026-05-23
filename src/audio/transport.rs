#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportState { Stopped, Playing, Paused, Recording }

#[derive(Clone, Debug)]
pub struct Transport {
    state: TransportState,
    position_samples: u64,
    loop_enabled: bool,
    loop_start: u64,
    loop_end: u64,
    bpm: f64,
    time_signature_numerator: u8,
    time_signature_denominator: u8,
}

impl Transport {
    pub fn new() -> Self {
        Self {
            state: TransportState::Stopped, position_samples: 0,
            loop_enabled: false, loop_start: 0, loop_end: 0,
            bpm: 120.0, time_signature_numerator: 4, time_signature_denominator: 4,
        }
    }

    pub fn play(&mut self) { self.state = TransportState::Playing; }
    pub fn pause(&mut self) { self.state = TransportState::Paused; }
    pub fn stop(&mut self) { self.state = TransportState::Stopped; self.position_samples = 0; }
    pub fn set_position_samples(&mut self, position: u64) { self.position_samples = position; }
    pub fn get_position_samples(&self) -> u64 { self.position_samples }
    pub fn get_state(&self) -> TransportState { self.state }
    pub fn is_playing(&self) -> bool { self.state == TransportState::Playing }
    pub fn is_recording(&self) -> bool { self.state == TransportState::Recording }
    pub fn is_paused(&self) -> bool { self.state == TransportState::Paused }
    pub fn is_stopped(&self) -> bool { self.state == TransportState::Stopped }

    pub fn advance_position(&mut self, frames: u64) {
        if self.loop_enabled {
            let loop_length = self.loop_end.saturating_sub(self.loop_start);
            if loop_length > 0 {
                let new_pos = self.position_samples + frames;
                if new_pos >= self.loop_end {
                    self.position_samples = (new_pos - self.loop_start) % loop_length + self.loop_start;
                    return;
                }
            }
        }
        self.position_samples += frames;
    }

    pub fn set_loop(&mut self, enabled: bool, start: u64, end: u64) {
        self.loop_enabled = enabled; self.loop_start = start; self.loop_end = end;
    }
    pub fn is_loop_enabled(&self) -> bool { self.loop_enabled }

    pub fn set_bpm(&mut self, bpm: f64) { self.bpm = bpm.clamp(20.0, 999.0); }
    pub fn get_bpm(&self) -> f64 { self.bpm }

    pub fn set_time_signature(&mut self, numerator: u8, denominator: u8) {
        self.time_signature_numerator = numerator; self.time_signature_denominator = denominator;
    }
    pub fn get_time_signature(&self) -> (u8, u8) { (self.time_signature_numerator, self.time_signature_denominator) }

    pub fn samples_to_beats(&self, samples: u64, sample_rate: u32) -> f64 {
        let beats_per_second = self.bpm / 60.0;
        let seconds = samples as f64 / sample_rate as f64;
        seconds * beats_per_second
    }

    pub fn beats_to_samples(&self, beats: f64, sample_rate: u32) -> u64 {
        let beats_per_second = self.bpm / 60.0;
        let seconds = beats / beats_per_second;
        (seconds * sample_rate as f64) as u64
    }

    pub fn samples_to_time_string(&self, samples: u64, sample_rate: u32) -> String {
        let total_seconds = samples as f64 / sample_rate as f64;
        let minutes = (total_seconds / 60.0).floor() as u32;
        let seconds = (total_seconds % 60.0).floor() as u32;
        let millis = ((total_seconds % 1.0) * 1000.0).round() as u32;
        format!("{:02}:{:02}.{:03}", minutes, seconds, millis)
    }

    pub fn samples_to_bars_beats_ticks(&self, samples: u64, sample_rate: u32) -> String {
        let total_beats = self.samples_to_beats(samples, sample_rate);
        let beats_per_bar = self.time_signature_numerator as f64;
        let bars = (total_beats / beats_per_bar).floor() as u32 + 1;
        let beats = (total_beats % beats_per_bar).floor() as u32 + 1;
        let ticks = ((total_beats % 1.0) * 480.0).round() as u32;
        format!("{:03}:{:02}:{:03}", bars, beats, ticks)
    }
}

impl Default for Transport { fn default() -> Self { Self::new() } }