mod audio;
mod config;
mod input;
mod transcription;

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use audio::AudioRecorder;
use config::Config;
use input::input_task;
use transcription::TranscriptionService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing/logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Earwig Voice Memo Daemon starting...");

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded successfully");
    info!("  Mouse device: {:?}", config.mouse_device);
    info!("  Audio device: {}", config.audio_device);
    info!("  Output directory: {:?}", config.output_dir);
    info!("  Whisper URL: {}", config.whisper_url);
    info!("  ntfy topic: {}", config.ntfy_topic);

    // Create channels
    // Input -> Recording
    let (recording_tx, recording_rx) = mpsc::channel(32);

    // Recording -> Transcription
    let (file_tx, file_rx) = mpsc::channel(32);

    // Create services
    let audio_recorder = AudioRecorder::new(
        config.output_dir.clone(),
        config.audio_device.clone(),
    );

    let transcription_service = TranscriptionService::new(
        config.whisper_url.clone(),
        config.ntfy_topic.clone(),
    );

    // Spawn tasks
    let input_handle = tokio::spawn(input_task(
        config.mouse_device.clone(),
        recording_tx,
    ));

    let recording_handle = tokio::spawn(audio_recorder.recording_task(
        recording_rx,
        file_tx,
    ));

    let transcription_handle = tokio::spawn(transcription_service.transcription_task(
        file_rx,
    ));

    info!("All tasks started, daemon is running");

    // Wait for any task to complete (or fail)
    tokio::select! {
        result = input_handle => {
            match result {
                Ok(Ok(())) => info!("Input task completed"),
                Ok(Err(e)) => error!("Input task failed: {:#}", e),
                Err(e) => error!("Input task panicked: {:#}", e),
            }
        }
        result = recording_handle => {
            match result {
                Ok(Ok(())) => info!("Recording task completed"),
                Ok(Err(e)) => error!("Recording task failed: {:#}", e),
                Err(e) => error!("Recording task panicked: {:#}", e),
            }
        }
        result = transcription_handle => {
            match result {
                Ok(Ok(())) => info!("Transcription task completed"),
                Ok(Err(e)) => error!("Transcription task failed: {:#}", e),
                Err(e) => error!("Transcription task panicked: {:#}", e),
            }
        }
    }

    info!("Daemon shutting down");
    Ok(())
}
