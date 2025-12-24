use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
}

pub struct TranscriptionService {
    whisper_url: String,
    ntfy_topic: String,
    client: Client,
}

impl TranscriptionService {
    pub fn new(whisper_url: String, ntfy_topic: String) -> Self {
        Self {
            whisper_url,
            ntfy_topic,
            client: Client::new(),
        }
    }

    /// Transcription task that processes completed recordings
    pub async fn transcription_task(
        &self,
        mut rx: mpsc::Receiver<PathBuf>,
    ) -> Result<()> {
        info!("Starting transcription task");

        while let Some(wav_path) = rx.recv().await {
            info!("Processing recording: {:?}", wav_path);

            // Attempt transcription
            match self.transcribe(&wav_path).await {
                Ok(transcript) => {
                    info!("Transcription successful: {}", transcript);
                    if let Err(e) = self.send_notification(&transcript, false).await {
                        error!("Failed to send notification: {:#}", e);
                    }
                }
                Err(e) => {
                    error!("Transcription failed: {:#}", e);
                    let filename = wav_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    let error_msg = format!("Recording saved: {}\nError: {}", filename, e);
                    if let Err(e) = self.send_notification(&error_msg, true).await {
                        error!("Failed to send error notification: {:#}", e);
                    }
                }
            }
        }

        info!("Transcription task ended");
        Ok(())
    }

    async fn transcribe(&self, wav_path: &PathBuf) -> Result<String> {
        // Read the WAV file
        let wav_bytes = tokio::fs::read(wav_path)
            .await
            .with_context(|| format!("Failed to read WAV file: {:?}", wav_path))?;

        // Send to Whisper server
        let response = self
            .client
            .post(&self.whisper_url)
            .header("Content-Type", "audio/wav")
            .body(wav_bytes)
            .send()
            .await
            .context("Failed to send request to Whisper server")?;

        // Check if the response is successful
        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read response".to_string());
            anyhow::bail!("Whisper server returned error {}: {}", status, body);
        }

        // Parse the response
        let whisper_response: WhisperResponse = response
            .json()
            .await
            .context("Failed to parse Whisper response")?;

        Ok(whisper_response.text)
    }

    async fn send_notification(&self, message: &str, is_error: bool) -> Result<()> {
        let mut request = self
            .client
            .post(&self.ntfy_topic)
            .header("Content-Type", "text/plain")
            .body(message.to_string());

        // Add error-specific headers
        if is_error {
            request = request
                .header("X-Title", "Transcription Failed")
                .header("X-Priority", "3");
        }

        let response = request
            .send()
            .await
            .context("Failed to send notification to ntfy")?;

        if !response.status().is_success() {
            let status = response.status();
            warn!("ntfy returned status: {}", status);
        } else {
            info!("Notification sent successfully");
        }

        Ok(())
    }
}
