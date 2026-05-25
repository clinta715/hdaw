use crate::audio::buffer::AudioBuffer;
use crate::audio::effects::{params, Effect};
use std::f32::consts::PI;

struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Biquad {
    fn new() -> Self {
        Self { b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0, x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0 }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }

    fn set_low_shelf(&mut self, freq: f32, gain_db: f32, sample_rate: u32) {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq / sample_rate as f32;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let sqrt_a = a.sqrt();
        let alpha = sin_w0 / 2.0 * (2.0_f32).sqrt();

        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * sqrt_a * alpha);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * sqrt_a * alpha);
        let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * sqrt_a * alpha;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * sqrt_a * alpha;

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    fn set_high_shelf(&mut self, freq: f32, gain_db: f32, sample_rate: u32) {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq / sample_rate as f32;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let sqrt_a = a.sqrt();
        let alpha = sin_w0 / 2.0 * (2.0_f32).sqrt();

        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * sqrt_a * alpha);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * sqrt_a * alpha);
        let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * sqrt_a * alpha;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * sqrt_a * alpha;

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    fn set_peaking(&mut self, freq: f32, gain_db: f32, q: f32, sample_rate: u32) {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq / sample_rate as f32;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w0;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha / a;

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    fn set_allpass(&mut self) {
        self.b0 = 1.0;
        self.b1 = 0.0;
        self.b2 = 0.0;
        self.a1 = 0.0;
        self.a2 = 0.0;
    }
}

pub struct Equalizer {
    low_freq: f32,
    low_gain: f32,
    mid_freq: f32,
    mid_gain: f32,
    mid_q: f32,
    high_freq: f32,
    high_gain: f32,
    bypassed: bool,
    sample_rate: u32,
    low_filter: Biquad,
    mid_filter: Biquad,
    high_filter: Biquad,
    needs_recalc: bool,
}

impl Equalizer {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            low_freq: 100.0,
            low_gain: 0.0,
            mid_freq: 1000.0,
            mid_gain: 0.0,
            mid_q: 1.0,
            high_freq: 8000.0,
            high_gain: 0.0,
            bypassed: false,
            sample_rate,
            low_filter: Biquad::new(),
            mid_filter: Biquad::new(),
            high_filter: Biquad::new(),
            needs_recalc: true,
        }
    }

    fn recalc_coefficients(&mut self) {
        if self.low_gain.abs() < 0.01 {
            self.low_filter.set_allpass();
        } else {
            self.low_filter.set_low_shelf(self.low_freq, self.low_gain, self.sample_rate);
        }
        if self.mid_gain.abs() < 0.01 {
            self.mid_filter.set_allpass();
        } else {
            self.mid_filter.set_peaking(self.mid_freq, self.mid_gain, self.mid_q, self.sample_rate);
        }
        if self.high_gain.abs() < 0.01 {
            self.high_filter.set_allpass();
        } else {
            self.high_filter.set_high_shelf(self.high_freq, self.high_gain, self.sample_rate);
        }
        self.needs_recalc = false;
    }
}

impl Effect for Equalizer {
    fn process(&mut self, buffer: &mut AudioBuffer) {
        if self.bypassed {
            return;
        }
        if self.low_gain.abs() < 0.01 && self.mid_gain.abs() < 0.01 && self.high_gain.abs() < 0.01 {
            return;
        }
        if self.needs_recalc {
            self.recalc_coefficients();
        }
        for sample in &mut buffer.samples {
            let s = self.low_filter.process(*sample);
            let s = self.mid_filter.process(s);
            *sample = self.high_filter.process(s);
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f32> {
        match name {
            params::EQ_LOW_FREQ => Some(self.low_freq),
            params::EQ_LOW_GAIN => Some(self.low_gain),
            params::EQ_MID_FREQ => Some(self.mid_freq),
            params::EQ_MID_GAIN => Some(self.mid_gain),
            params::EQ_MID_Q => Some(self.mid_q),
            params::EQ_HIGH_FREQ => Some(self.high_freq),
            params::EQ_HIGH_GAIN => Some(self.high_gain),
            _ => None,
        }
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        let changed = match name {
            params::EQ_LOW_FREQ => { self.low_freq = value.clamp(20.0, 500.0); true }
            params::EQ_LOW_GAIN => { self.low_gain = value.clamp(-12.0, 12.0); true }
            params::EQ_MID_FREQ => { self.mid_freq = value.clamp(200.0, 5000.0); true }
            params::EQ_MID_GAIN => { self.mid_gain = value.clamp(-12.0, 12.0); true }
            params::EQ_MID_Q => { self.mid_q = value.clamp(0.1, 10.0); true }
            params::EQ_HIGH_FREQ => { self.high_freq = value.clamp(2000.0, 20000.0); true }
            params::EQ_HIGH_GAIN => { self.high_gain = value.clamp(-12.0, 12.0); true }
            _ => false,
        };
        if changed {
            self.needs_recalc = true;
        }
    }

    fn get_name(&self) -> &str {
        "Equalizer"
    }

    fn is_bypassed(&self) -> bool {
        self.bypassed
    }

    fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }
}
