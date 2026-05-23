use crate::audio::buffer::AudioBuffer;
use crate::audio::effects::{params, Effect};
use std::collections::VecDeque;

pub struct Reverb {
    room_size: f32,
    damping: f32,
    wet_dry: f32,
    bypassed: bool,
    sample_rate: u32,
    buffer: VecDeque<f32>,
    delay_line: Vec<f32>,
    write_pos: usize,
}

impl Reverb {
    pub fn new(sample_rate: u32) -> Self {
        let max_delay = (sample_rate * 2) as usize;
        Self {
            room_size: 0.5,
            damping: 0.5,
            wet_dry: 0.3,
            bypassed: false,
            sample_rate,
            buffer: VecDeque::with_capacity(max_delay),
            delay_line: vec![0.0; max_delay],
            write_pos: 0,
        }
    }
}

impl Effect for Reverb {
    fn process(&mut self, buffer: &mut AudioBuffer) {
        if self.bypassed {
            return;
        }

        let wet = self.wet_dry;
        let dry = 1.0 - wet;

        for sample in &mut buffer.samples {
            let delayed = self.delay_line[self.write_pos];
            self.delay_line[self.write_pos] = *sample + delayed * self.damping * self.room_size;
            *sample = *sample * dry + delayed * wet;

            self.write_pos = (self.write_pos + 1) % self.delay_line.len();
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f32> {
        match name {
            params::REVERB_ROOM_SIZE => Some(self.room_size),
            params::REVERB_DAMPING => Some(self.damping),
            params::REVERB_WET_DRY => Some(self.wet_dry),
            _ => None,
        }
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        match name {
            params::REVERB_ROOM_SIZE => self.room_size = value.clamp(0.0, 1.0),
            params::REVERB_DAMPING => self.damping = value.clamp(0.0, 1.0),
            params::REVERB_WET_DRY => self.wet_dry = value.clamp(0.0, 1.0),
            _ => {}
        }
    }

    fn get_name(&self) -> &str {
        "Reverb"
    }

    fn is_bypassed(&self) -> bool {
        self.bypassed
    }

    fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }
}