use crate::audio::buffer::AudioBuffer;
use timestretch::StretchParams;

pub struct TimeStretchProcessor {
    sample_rate: u32,
}

impl TimeStretchProcessor {
    pub fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    pub fn stretch(&self, buffer: &AudioBuffer, ratio: f32) -> Result<AudioBuffer, String> {
        let params = StretchParams::new(ratio as f64)
            .with_sample_rate(self.sample_rate);

        let input: Vec<f32> = buffer.samples.clone();

        let stretched = timestretch::stretch(&input, &params)
            .map_err(|e| format!("Time stretch error: {:?}", e))?;

        Ok(AudioBuffer::from_samples(
            stretched,
            buffer.channels,
            buffer.sample_rate,
        ))
    }

    pub fn stretch_to_duration(&self, buffer: &AudioBuffer, target_samples: usize) -> Result<AudioBuffer, String> {
        let current_samples = buffer.samples.len();
        if current_samples == 0 {
            return Ok(buffer.clone());
        }

        let ratio = target_samples as f32 / current_samples as f32;
        self.stretch(buffer, ratio)
    }
}

pub fn pitch_shift(_semitones: f32) -> f32 {
    1.0
}