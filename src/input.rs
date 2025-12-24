use anyhow::{Context, Result};
use evdev::{Device, InputEventKind, Key};
use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub enum RecordingCommand {
    Start,
    Stop,
}

/// Input task that monitors the mouse device for button events
pub async fn input_task(
    device_path: PathBuf,
    tx: mpsc::Sender<RecordingCommand>,
) -> Result<()> {
    info!("Starting input task, monitoring device: {:?}", device_path);

    loop {
        match open_and_monitor_device(&device_path, &tx).await {
            Ok(_) => {
                info!("Device monitoring ended normally");
                break;
            }
            Err(e) => {
                error!("Device error: {:#}", e);
                warn!("Will retry opening device in 5 seconds...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }

    Ok(())
}

async fn open_and_monitor_device(
    device_path: &PathBuf,
    tx: &mpsc::Sender<RecordingCommand>,
) -> Result<()> {
    let mut device = Device::open(device_path)
        .with_context(|| format!("Failed to open device: {:?}", device_path))?;

    info!("Device opened successfully: {}", device.name().unwrap_or("Unknown"));

    // Spawn a blocking task to read events (evdev is synchronous)
    let (event_tx, mut event_rx) = mpsc::channel(32);

    tokio::task::spawn_blocking(move || {
        loop {
            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        if event_tx.blocking_send(event).is_err() {
                            // Channel closed, exit
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("Error fetching events: {:#}", e);
                    break;
                }
            }
        }
    });

    // Process events in async context
    while let Some(event) = event_rx.recv().await {
        if let InputEventKind::Key(Key::BTN_LEFT) = event.kind() {
            match event.value() {
                1 => {
                    // Button down - start recording
                    info!("Button pressed - starting recording");
                    if let Err(e) = tx.send(RecordingCommand::Start).await {
                        error!("Failed to send Start command: {:#}", e);
                        break;
                    }
                }
                0 => {
                    // Button up - stop recording
                    info!("Button released - stopping recording");
                    if let Err(e) = tx.send(RecordingCommand::Stop).await {
                        error!("Failed to send Stop command: {:#}", e);
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
