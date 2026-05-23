use crate::audio::buffer::AudioBuffer;
use crate::audio::effects::{params, Effect};

pub struct Compressor {
    threshold: f32,
    ratio: f32,
    attack: f32,
    release: f32,
    makeup: f32,
    bypassed: bool,
    envelope: f32,
}

impl Compressor {
    pub fn new() -> Self {
        Self {
            threshold: -20.0,
            ratio: 4.0,
            attack: 0.01,
            release: 0.1,
            makeup: 0.0,
            bypassed: false,
            envelope: 0.0,
        }
    }
}

impl Effect for Compressor {
    fn process(&mut self, buffer: &mut AudioBuffer) {
        if self.bypassed {
            return;
        }

        let attack_coef = (-1.0 / (self.attack * 44100.0)).exp();
        let release_coef = (-1.0 / (self.release * 44100.0)).exp();

        for sample in &mut buffer.samples {
            let input_level = sample.abs();

            if input_level > self.envelope {
                self.envelope += attack_coef * (input_level - self.envelope);
            } else {
                self.envelope += release_coef * (input_level - self.envelope);
            }

            let db_over = (self.envelope - self.threshold).max(0.0);
            let gain_reduction = if db_over > 0.0 {
                -db_over * (1.0 - 1.0 / self.ratio)
            } else {
                0.0
            };

            let linear_gain = 10.0_f32.powf(gain_reduction / 20.0) * 10.0_f32.powf(self.makeup / 20.0);
            *sample *= linear_gain;
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f32> {
        match name {
            params::COMP_THRESHOLD => Some(self.threshold),
            params::COMP_RATIO => Some(self.ratio),
            params::COMP_ATTACK => Some(self.attack),
            params::COMP_RELEASE => Some(self.release),
            params::COMP_MAKEUP => Some(self.makeup),
            _ => None,
        }
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        match name {
            params::COMP_THRESHOLD => self.threshold = value.clamp(-60.0, 0.0),
            params::COMP_RATIO => self.ratio = value.clamp(1.0, 20.0),
            params::COMP_ATTACK => self.attack = value.clamp(0.001, 1.0),
            params::COMP_RELEASE => self.release = value.clamp(0.01, 2.0),
            params::COMP_MAKEUP => self.makeup = value.clamp(-20.0, 20.0),
            _ => {}
        }
    }

    fn get_name(&self) -> &str {
        "Compressor"
    }

    fn is_bypassed(&self) -> bool {
        self.bypassed
    }

    fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }
}

impl Default for Compressor {
    fn default() -> Self {
        Self::new()
    }
}