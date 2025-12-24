use anyhow::{Context, Result};
use chrono::Local;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, StreamConfig};
use hound::{WavSpec, WavWriter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::input::RecordingCommand;

pub struct AudioRecorder {
    output_dir: PathBuf,
    device_name: String,
}

impl AudioRecorder {
    pub fn new(output_dir: PathBuf, device_name: String) -> Self {
        Self {
            output_dir,
            device_name,
        }
    }

    /// Recording task that receives commands and manages audio capture
    pub async fn recording_task(
        &self,
        mut rx: mpsc::Receiver<RecordingCommand>,
        file_tx: mpsc::Sender<PathBuf>,
    ) -> Result<()> {
        info!("Starting recording task");

        let mut is_recording = false;
        let mut current_recorder: Option<ActiveRecorder> = None;

        while let Some(cmd) = rx.recv().await {
            match cmd {
                RecordingCommand::Start => {
                    if is_recording {
                        warn!("Already recording, ignoring start command");
                        continue;
                    }

                    info!("Starting new recording");
                    match self.start_recording().await {
                        Ok(recorder) => {
                            is_recording = true;
                            current_recorder = Some(recorder);
                        }
                        Err(e) => {
                            error!("Failed to start recording: {:#}", e);
                        }
                    }
                }
                RecordingCommand::Stop => {
                    if !is_recording {
                        warn!("Not recording, ignoring stop command");
                        continue;
                    }

                    info!("Stopping recording");
                    if let Some(recorder) = current_recorder.take() {
                        match recorder.stop_and_save().await {
                            Ok(path) => {
                                info!("Recording saved: {:?}", path);
                                if let Err(e) = file_tx.send(path).await {
                                    error!("Failed to send file path for transcription: {:#}", e);
                                }
                            }
                            Err(e) => {
                                error!("Failed to save recording: {:#}", e);
                            }
                        }
                    }
                    is_recording = false;
                }
            }
        }

        info!("Recording task ended");
        Ok(())
    }

    async fn start_recording(&self) -> Result<ActiveRecorder> {
        let host = cpal::default_host();

        // Try to find the device by name, or use default
        let device = if self.device_name == "default" {
            host.default_input_device()
                .context("No default input device available")?
        } else {
            // Try to find device by name
            host.input_devices()
                .context("Failed to enumerate input devices")?
                .find(|d| d.name().map(|n| n == self.device_name).unwrap_or(false))
                .with_context(|| format!("Could not find audio device: {}", self.device_name))?
        };

        info!("Using audio device: {}", device.name().unwrap_or("Unknown".to_string()));

        // Configure for 16kHz mono
        let config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(16000),
            buffer_size: cpal::BufferSize::Default,
        };

        let samples: Arc<Mutex<Vec<i16>>> = Arc::new(Mutex::new(Vec::new()));
        let samples_clone = samples.clone();

        // Build the input stream
        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Convert f32 samples to i16
                    let mut samples = samples_clone.lock().unwrap();
                    for &sample in data {
                        let sample_i16 = (sample * i16::MAX as f32) as i16;
                        samples.push(sample_i16);
                    }
                },
                |err| {
                    error!("Stream error: {}", err);
                },
                None,
            )
            .context("Failed to build input stream")?;

        stream.play().context("Failed to start stream")?;

        Ok(ActiveRecorder {
            stream,
            samples,
            output_dir: self.output_dir.clone(),
        })
    }
}

struct ActiveRecorder {
    stream: cpal::Stream,
    samples: Arc<Mutex<Vec<i16>>>,
    output_dir: PathBuf,
}

impl ActiveRecorder {
    async fn stop_and_save(self) -> Result<PathBuf> {
        // Stop the stream
        drop(self.stream);

        // Generate filename with timestamp
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("memo_{}.wav", timestamp);
        let path = self.output_dir.join(&filename);

        // Get the samples
        let samples = self.samples.lock().unwrap();

        if samples.is_empty() {
            warn!("No audio data recorded");
        }

        // Write WAV file
        let spec = WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::create(&path, spec)
            .with_context(|| format!("Failed to create WAV file: {:?}", path))?;

        for &sample in samples.iter() {
            writer
                .write_sample(sample)
                .context("Failed to write sample")?;
        }

        writer.finalize().context("Failed to finalize WAV file")?;

        info!("Wrote {} samples to {:?}", samples.len(), path);

        Ok(path)
    }
}
