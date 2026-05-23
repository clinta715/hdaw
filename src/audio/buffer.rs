#[derive(Clone, Debug)]
pub struct AudioBuffer {
    pub samples: Vec<f32>,
    pub channels: u16,
    pub sample_rate: u32,
}

impl AudioBuffer {
    pub fn new(channels: u16, sample_rate: u32) -> Self {
        Self { samples: Vec::new(), channels, sample_rate }
    }

    pub fn with_capacity(channels: u16, sample_rate: u32, capacity: usize) -> Self {
        Self { samples: Vec::with_capacity(capacity), channels, sample_rate }
    }

    pub fn from_samples(samples: Vec<f32>, channels: u16, sample_rate: u32) -> Self {
        Self { samples, channels, sample_rate }
    }

    pub fn len(&self) -> usize { self.samples.len() }
    pub fn is_empty(&self) -> bool { self.samples.is_empty() }
    pub fn frames(&self) -> usize { if self.channels > 0 { self.samples.len() / self.channels as usize } else { 0 } }
    pub fn duration_secs(&self) -> f64 { self.frames() as f64 / self.sample_rate as f64 }
    pub fn clear(&mut self) { self.samples.clear(); }
    pub fn resize(&mut self, new_len: usize, value: f32) { self.samples.resize(new_len, value); }
    pub fn push_sample(&mut self, sample: f32) { self.samples.push(sample); }
    pub fn get_sample(&self, index: usize) -> Option<f32> { self.samples.get(index).copied() }

    pub fn get_frame(&self, frame: usize) -> Option<Vec<f32>> {
        if self.channels == 0 { return None; }
        let start = frame * self.channels as usize;
        let end = start + self.channels as usize;
        if end <= self.samples.len() { Some(self.samples[start..end].to_vec()) } else { None }
    }

    pub fn fill_silence(&mut self) {
        for sample in &mut self.samples { *sample = 0.0; }
    }

    pub fn mix(&mut self, other: &AudioBuffer, volume: f32) {
        assert_eq!(self.channels, other.channels);
        assert_eq!(self.sample_rate, other.sample_rate);
        let min_len = self.samples.len().min(other.samples.len());
        for i in 0..min_len { self.samples[i] += other.samples[i] * volume; }
    }

    pub fn apply_gain(&mut self, gain: f32) {
        for sample in &mut self.samples { *sample *= gain; }
    }

    pub fn apply_pan(&mut self, pan: f32) {
        if self.channels != 2 { return; }
        let left_gain = if pan < 0.0 { 1.0 } else { 1.0 - pan };
        let right_gain = if pan > 0.0 { 1.0 } else { 1.0 + pan };
        for frame in 0..self.frames() {
            if let Some(mut frame_data) = self.get_frame(frame) {
                if frame_data.len() >= 2 {
                    frame_data[0] *= left_gain;
                    frame_data[1] *= right_gain;
                    let idx = frame * 2;
                    self.samples[idx] = frame_data[0];
                    self.samples[idx + 1] = frame_data[1];
                }
            }
        }
    }
}

impl Default for AudioBuffer {
    fn default() -> Self { Self::new(2, 44100) }
}