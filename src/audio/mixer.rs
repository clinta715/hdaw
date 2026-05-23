use crate::audio::buffer::AudioBuffer;
use std::vec::Vec;

#[derive(Clone, Debug)]
pub struct ChannelStrip {
    pub volume: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub peak_left: f32,
    pub peak_right: f32,
}

impl ChannelStrip {
    pub fn new() -> Self {
        Self {
            volume: 1.0,
            pan: 0.0,
            mute: false,
            solo: false,
            peak_left: 0.0,
            peak_right: 0.0,
        }
    }

    pub fn process(&mut self, buffer: &mut AudioBuffer) {
        if self.mute {
            buffer.fill_silence();
            return;
        }

        buffer.apply_gain(self.volume);

        if self.channels() == 2 {
            buffer.apply_pan(self.pan);
        }

        self.update_meters(buffer);
    }

    fn channels(&self) -> usize { 2 }

    fn update_meters(&mut self, buffer: &AudioBuffer) {
        let mut peak_l = 0.0f32;
        let mut peak_r = 0.0f32;

        for frame in 0..buffer.frames() {
            if let Some(frame_data) = buffer.get_frame(frame) {
                if frame_data.len() >= 1 { peak_l = peak_l.max(frame_data[0].abs()); }
                if frame_data.len() >= 2 { peak_r = peak_r.max(frame_data[1].abs()); }
            }
        }
        self.peak_left = peak_l;
        self.peak_right = peak_r;
    }
}

impl Default for ChannelStrip {
    fn default() -> Self { Self::new() }
}

#[derive(Clone, Debug)]
pub struct Mixer {
    channels: Vec<ChannelStrip>,
    master: ChannelStrip,
    sample_rate: u32,
}

impl Mixer {
    pub fn new(num_channels: usize, sample_rate: u32) -> Self {
        let channels = (0..num_channels).map(|_| ChannelStrip::new()).collect();
        Self { channels, master: ChannelStrip::new(), sample_rate }
    }

    pub fn set_num_channels(&mut self, num_channels: usize) {
        self.channels.resize_with(num_channels, ChannelStrip::new);
    }

    pub fn num_channels(&self) -> usize { self.channels.len() }

    pub fn get_channel(&self, index: usize) -> Option<&ChannelStrip> {
        self.channels.get(index)
    }

    pub fn get_channel_mut(&mut self, index: usize) -> Option<&mut ChannelStrip> {
        self.channels.get_mut(index)
    }

    pub fn get_master(&self) -> &ChannelStrip { &self.master }
    pub fn get_master_mut(&mut self) -> &mut ChannelStrip { &mut self.master }

    pub fn process(&mut self, buffers: &mut [&mut AudioBuffer]) {
        for (i, buffer) in buffers.iter_mut().enumerate() {
            if let Some(channel) = self.get_channel_mut(i) {
                channel.process(buffer);
            }
        }

        let has_solo = self.channels.iter().any(|c| c.solo);
        for (i, buffer) in buffers.iter_mut().enumerate() {
            if has_solo {
                if let Some(channel) = self.get_channel(i) {
                    if !channel.solo { buffer.fill_silence(); }
                } else { buffer.fill_silence(); }
            }
        }
    }

    pub fn mix_to_stereo(&mut self, buffers: &[AudioBuffer]) -> AudioBuffer {
        let mut output = AudioBuffer::new(2, self.sample_rate);
        let max_frames = buffers.iter().map(|b| b.frames()).max().unwrap_or(0);
        output.samples.resize(max_frames * 2, 0.0);

        for buffer in buffers {
            for frame in 0..buffer.frames() {
                if let Some(frame_data) = buffer.get_frame(frame) {
                    let out_frame = frame * 2;
                    if out_frame + 1 < output.samples.len() {
                        if frame_data.len() >= 1 { output.samples[out_frame] += frame_data[0]; }
                        if frame_data.len() >= 2 { output.samples[out_frame + 1] += frame_data[1]; }
                        else if frame_data.len() >= 1 { output.samples[out_frame + 1] += frame_data[0]; }
                    }
                }
            }
        }

        self.master.process(&mut output);
        for sample in &mut output.samples { *sample = sample.clamp(-1.0, 1.0); }
        output
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) { self.sample_rate = sample_rate; }
    pub fn add_channel(&mut self) { self.channels.push(ChannelStrip::new()); }
    pub fn remove_channel(&mut self, index: usize) { if index < self.channels.len() { self.channels.remove(index); } }
}

impl Default for Mixer {
    fn default() -> Self { Self::new(2, 44100) }
}