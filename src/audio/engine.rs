use crate::audio::mixer::Mixer;
use crate::audio::transport::Transport;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, OutputCallbackInfo, SampleFormat, Stream, StreamConfig, SupportedBufferSize};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{error, info, warn};

pub struct AudioEngine {
    device: Option<Device>,
    stream: Option<Stream>,
    stream_config: Option<StreamConfig>,
    sample_format: Option<SampleFormat>,
    mixer: Mixer,
    transport: Transport,
    is_running: AtomicBool,
    sample_rate: u32,
    buffer_size: u32,
}

impl AudioEngine {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device();
        if device.is_none() { warn!("No default audio output device found"); }

        let supported = device.as_ref().and_then(|d| d.default_output_config().ok());

        let (sample_rate, buffer_size, stream_config, sample_format) = if let Some(ref cfg) = supported {
            let buf = match cfg.buffer_size() {
                SupportedBufferSize::Range { max, .. } => *max,
                SupportedBufferSize::Unknown => 512,
            };
            (cfg.sample_rate().0, buf, Some(cfg.config()), Some(cfg.sample_format()))
        } else {
            (44100, 512, None, None)
        };

        info!("Audio engine initialized: {} Hz, {} buffer size", sample_rate, buffer_size);

        Self {
            device, stream: None, stream_config, sample_format,
            mixer: Mixer::new(2, 44100), transport: Transport::new(),
            is_running: AtomicBool::new(false), sample_rate, buffer_size,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.is_running.load(Ordering::SeqCst) { return Ok(()); }

        let device = self.device.as_ref().ok_or("No audio device available")?;
        let config = self.stream_config.as_ref().ok_or("No audio config available")?;
        let sample_format = self.sample_format.ok_or("No sample format available")?;
        info!("Starting audio stream");

        let err_fn = |err| error!("Audio stream error: {}", err);

        let stream = match sample_format {
            SampleFormat::F32 => {
                device.build_output_stream(config, |data: &mut [f32], _: &OutputCallbackInfo| {
                    for sample in data.iter_mut() { *sample = 0.0; }
                }, err_fn, None)
            }
            _ => return Err("Unsupported sample format".to_string()),
        }.map_err(|e| format!("Failed to build stream: {}", e))?;

        stream.play().map_err(|e| format!("Failed to play stream: {}", e))?;

        self.stream = Some(stream);
        self.is_running.store(true, Ordering::SeqCst);
        info!("Audio stream started successfully");
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(stream) = self.stream.take() {
            drop(stream);
            self.is_running.store(false, Ordering::SeqCst);
            info!("Audio stream stopped");
        }
    }

    pub fn is_running(&self) -> bool { self.is_running.load(Ordering::SeqCst) }
    pub fn sample_rate(&self) -> u32 { self.sample_rate }
    pub fn buffer_size(&self) -> u32 { self.buffer_size }
    pub fn set_sample_rate(&mut self, rate: u32) { self.sample_rate = rate; self.mixer.set_sample_rate(rate); }
    pub fn set_buffer_size(&mut self, size: u32) { self.buffer_size = size; }
}

impl Clone for AudioEngine {
    fn clone(&self) -> Self {
        Self {
            device: self.device.clone(), stream: None, stream_config: self.stream_config.clone(),
            sample_format: self.sample_format,
            mixer: self.mixer.clone(), transport: self.transport.clone(),
            is_running: AtomicBool::new(self.is_running.load(Ordering::SeqCst)),
            sample_rate: self.sample_rate, buffer_size: self.buffer_size,
        }
    }
}