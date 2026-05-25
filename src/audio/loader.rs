use crate::audio::buffer::AudioBuffer;
use std::path::Path;

pub fn load_wav(path: &Path) -> Result<AudioBuffer, String> {
    let reader = hound::WavReader::open(path).map_err(|e| format!("Failed to open WAV: {}", e))?;
    let spec = reader.spec();
    let bits_per_sample = spec.bits_per_sample;
    let channels = spec.channels;
    let sample_rate = spec.sample_rate;

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => {
            reader.into_samples::<f32>()
                .filter_map(|s| s.ok())
                .collect()
        }
        hound::SampleFormat::Int => {
            let max_val = (1u64 << (bits_per_sample - 1)) as f32;
            reader.into_samples::<i32>()
                .filter_map(|s| s.ok())
                .map(|s| (s as f32 / max_val).clamp(-1.0, 1.0))
                .collect()
        }
    };

    Ok(AudioBuffer::from_samples(samples, channels, sample_rate))
}

pub fn generate_test_tone(sample_rate: u32, duration_secs: f64) -> AudioBuffer {
    let num_samples = (sample_rate as f64 * duration_secs) as usize;
    let mut samples = Vec::with_capacity(num_samples * 2);
    let freq = 440.0;
    let amplitude = 0.25;
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let value = (t * freq * 2.0 * std::f64::consts::PI).sin() as f32 * amplitude;
        samples.push(value);
        samples.push(value);
    }
    AudioBuffer::from_samples(samples, 2, sample_rate)
}

pub fn write_wav(path: &Path, samples: &[f32], channels: u16, sample_rate: u32) -> Result<(), String> {
    use hound::{WavWriter, WavSpec, SampleFormat};
    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };
    let mut writer = WavWriter::create(path, spec).map_err(|e| format!("Failed to create WAV: {}", e))?;
    for &sample in samples {
        writer.write_sample(sample).map_err(|e| format!("Failed to write sample: {}", e))?;
    }
    writer.finalize().map_err(|e| format!("Failed to finalize WAV: {}", e))?;
    Ok(())
}
