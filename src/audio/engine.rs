use crate::audio::playback::PlaybackManager;
use crate::audio::transport::Transport;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, OutputCallbackInfo, SampleFormat, Stream, StreamConfig, SupportedBufferSize};
use std::sync::atomic::{AtomicBool, Ordering};

use tracing::{error, info, warn};

pub struct AudioEngine {
    device: Option<Device>,
    output_stream: Option<Stream>,
    input_stream: Option<Stream>,
    stream_config: Option<StreamConfig>,
    sample_format: Option<SampleFormat>,
    transport: Transport,
    playback: Option<PlaybackManager>,
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
                SupportedBufferSize::Range { min, .. } => (*min as u32).max(64),
                SupportedBufferSize::Unknown => 512,
            };
            (cfg.sample_rate().0, buf, Some(cfg.config()), Some(cfg.sample_format()))
        } else {
            (44100, 512, None, None)
        };

        info!("Audio engine initialized: {} Hz, {} buffer size", sample_rate, buffer_size);

        Self {
            device, output_stream: None, input_stream: None,
            stream_config, sample_format,
            transport: Transport::new(),
            playback: None,
            is_running: AtomicBool::new(false), sample_rate, buffer_size,
        }
    }

    pub fn set_playback(&mut self, playback: PlaybackManager) {
        self.playback = Some(playback);
    }

    pub fn playback(&self) -> Option<&PlaybackManager> {
        self.playback.as_ref()
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.is_running.load(Ordering::SeqCst) { return Ok(()); }

        let device = self.device.as_ref().ok_or("No audio device available")?;
        let config = self.stream_config.as_ref().ok_or("No audio config available")?;
        let sample_format = self.sample_format.ok_or("No sample format available")?;
        let playback = self.playback.clone().ok_or("No playback state set")?;
        let dev_sample_rate = config.sample_rate.0;
        info!("Starting audio stream at {} Hz", dev_sample_rate);

        let err_fn = |err| error!("Audio stream error: {}", err);

        let stream = match sample_format {
            SampleFormat::F32 => {
                device.build_output_stream(config, move |data: &mut [f32], _: &OutputCallbackInfo| {
                    playback.fill_buffer(data, dev_sample_rate);
                }, err_fn, None)
            }
            _ => return Err("Unsupported sample format".to_string()),
        }.map_err(|e| format!("Failed to build stream: {}", e))?;

        stream.play().map_err(|e| format!("Failed to play stream: {}", e))?;

        self.output_stream = Some(stream);
        self.is_running.store(true, Ordering::SeqCst);
        info!("Audio stream started successfully");
        Ok(())
    }

    pub fn start_input(&mut self, input_tx: crossbeam::channel::Sender<Vec<f32>>) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or("No input device found")?;
        let config = device.default_input_config()
            .map_err(|e| format!("Failed to get input config: {}", e))?;

        let num_channels = config.channels() as usize;
        info!("Opening input stream: {} Hz, {} channels", config.sample_rate().0, num_channels);

        let err_fn = |err| error!("Input stream error: {}", err);

        let stream = match config.sample_format() {
            SampleFormat::F32 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _| {
                        let stereo: Vec<f32> = if num_channels == 1 {
                            data.iter().flat_map(|s| [*s, *s]).collect()
                        } else if num_channels >= 2 {
                            data.chunks(num_channels)
                                .flat_map(|chunk| [chunk[0], chunk.get(1).copied().unwrap_or(chunk[0])])
                                .collect()
                        } else {
                            data.to_vec()
                        };
                        let _ = input_tx.try_send(stereo);
                    },
                    err_fn,
                    None,
                )
            }
            _ => return Err("Unsupported input sample format".to_string()),
        }.map_err(|e| format!("Failed to build input stream: {}", e))?;

        stream.play().map_err(|e| format!("Failed to start input stream: {}", e))?;
        self.input_stream = Some(stream);
        info!("Input stream started successfully");
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(stream) = self.output_stream.take() {
            drop(stream);
            self.is_running.store(false, Ordering::SeqCst);
            self.transport.stop();
            if let Some(pb) = self.playback.as_ref() {
                pb.set_playing(false);
            }
            info!("Audio stream stopped");
        }
    }

    pub fn stop_input(&mut self) {
        if let Some(stream) = self.input_stream.take() {
            drop(stream);
            info!("Input stream stopped");
        }
    }

    pub fn is_running(&self) -> bool { self.is_running.load(Ordering::SeqCst) }
    pub fn sample_rate(&self) -> u32 { self.sample_rate }
    pub fn buffer_size(&self) -> u32 { self.buffer_size }
    pub fn transport(&self) -> &Transport { &self.transport }
    pub fn transport_mut(&mut self) -> &mut Transport { &mut self.transport }
}
