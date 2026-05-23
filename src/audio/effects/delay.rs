use crate::audio::buffer::AudioBuffer;
use crate::audio::effects::{params, Effect};
use std::collections::VecDeque;

pub struct Delay {
    time: f32,
    feedback: f32,
    mix: f32,
    bypassed: bool,
    sample_rate: u32,
    buffer: VecDeque<f32>,
}

impl Delay {
    pub fn new(sample_rate: u32) -> Self {
        let max_delay = (sample_rate * 5) as usize;
        Self {
            time: 0.5,
            feedback: 0.3,
            mix: 0.3,
            bypassed: false,
            sample_rate,
            buffer: VecDeque::with_capacity(max_delay),
        }
    }

    fn delay_samples(&self) -> usize {
        (self.time * self.sample_rate as f32) as usize
    }
}

impl Effect for Delay {
    fn process(&mut self, buffer: &mut AudioBuffer) {
        if self.bypassed {
            return;
        }

        let delay_len = self.delay_samples();
        while self.buffer.len() < delay_len {
            self.buffer.push_back(0.0);
        }

        let wet = self.mix;
        let dry = 1.0 - wet;

        for sample in &mut buffer.samples {
            let delayed = if self.buffer.len() > 0 {
                self.buffer.pop_front().unwrap_or(0.0)
            } else {
                0.0
            };

            let feedback_sample = *sample + delayed * self.feedback;

            self.buffer.push_back(feedback_sample);

            *sample = *sample * dry + delayed * wet;
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f32> {
        match name {
            params::DELAY_TIME => Some(self.time),
            params::DELAY_FEEDBACK => Some(self.feedback),
            params::DELAY_MIX => Some(self.mix),
            _ => None,
        }
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        match name {
            params::DELAY_TIME => self.time = value.clamp(0.001, 5.0),
            params::DELAY_FEEDBACK => self.feedback = value.clamp(0.0, 0.95),
            params::DELAY_MIX => self.mix = value.clamp(0.0, 1.0),
            _ => {}
        }
    }

    fn get_name(&self) -> &str {
        "Delay"
    }

    fn is_bypassed(&self) -> bool {
        self.bypassed
    }

    fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }
}