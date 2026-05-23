use crate::audio::buffer::AudioBuffer;

#[derive(Debug, Clone)]
pub struct WaveformPeaks {
    pub min: Vec<f32>,
    pub max: Vec<f32>,
    pub samples_per_peak: usize,
}

impl WaveformPeaks {
    pub fn generate(buffer: &AudioBuffer, num_peaks: usize) -> Self {
        let total_samples = buffer.samples.len();
        if total_samples == 0 || num_peaks == 0 {
            return Self {
                min: Vec::new(),
                max: Vec::new(),
                samples_per_peak: 0,
            };
        }

        let samples_per_peak = (total_samples / num_peaks).max(1);
        let mut min = Vec::with_capacity(num_peaks);
        let mut max = Vec::with_capacity(num_peaks);

        for chunk in buffer.samples.chunks(samples_per_peak) {
            let mut chunk_min = f32::MAX;
            let mut chunk_max = f32::MIN;

            for &sample in chunk {
                chunk_min = chunk_min.min(sample);
                chunk_max = chunk_max.max(sample);
            }

            min.push(chunk_min);
            max.push(chunk_max);
        }

        Self {
            min,
            max,
            samples_per_peak,
        }
    }

    pub fn get_peak(&self, index: usize) -> (f32, f32) {
        if index < self.min.len() {
            (self.min[index], self.max[index])
        } else {
            (0.0, 0.0)
        }
    }

    pub fn num_peaks(&self) -> usize {
        self.min.len()
    }
}

pub fn generate_waveform_cache(buffer: &AudioBuffer, width: usize) -> WaveformPeaks {
    WaveformPeaks::generate(buffer, width)
}