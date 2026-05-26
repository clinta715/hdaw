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

pub fn render_waveform_image(peaks: &WaveformPeaks, width: u32, height: u32) -> slint::Image {
    let w = width as usize;
    let h = height as usize;
    let mut pixels = vec![0u8; w * h * 4];
    let num_peaks = peaks.min.len();
    if num_peaks == 0 {
        let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(width, height);
        buf.make_mut_bytes().fill(0);
        return slint::Image::from_rgba8_premultiplied(buf);
    }

    let mid = h as f32 * 0.5;
    for x in 0..w {
        let idx = (x as f32 / (w - 1).max(1) as f32) * (num_peaks - 1) as f32;
        let idx0 = idx.floor() as usize;
        let idx1 = (idx0 + 1).min(num_peaks - 1);
        let frac = idx - idx0 as f32;
        let (min0, max0) = peaks.get_peak(idx0);
        let (min1, max1) = peaks.get_peak(idx1);
        let min = min0 * (1.0 - frac) + min1 * frac;
        let max = max0 * (1.0 - frac) + max1 * frac;

        let y_min = ((mid - max * mid) as i32).clamp(0, h as i32 - 1);
        let y_max = ((mid - min * mid) as i32).clamp(0, h as i32 - 1);

        for y in 0..h {
            let pi = (y * w + x) * 4;
            if (y as i32) >= y_min && (y as i32) <= y_max {
                pixels[pi] = 80;
                pixels[pi + 1] = 160;
                pixels[pi + 2] = 255;
                pixels[pi + 3] = 220;
            } else {
                pixels[pi + 3] = 0;
            }
        }
    }

    let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(width, height);
    buf.make_mut_bytes().copy_from_slice(&pixels);
    slint::Image::from_rgba8_premultiplied(buf)
}