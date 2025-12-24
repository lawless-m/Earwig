use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Path to the dedicated mouse device (e.g., /dev/input/by-id/...)
    pub mouse_device: PathBuf,

    /// ALSA audio input device name (e.g., "default" or "hw:1,0")
    pub audio_device: String,

    /// Directory to save WAV files
    pub output_dir: PathBuf,

    /// Whisper server endpoint URL
    pub whisper_url: String,

    /// ntfy.sh topic URL
    pub ntfy_topic: String,
}

impl Config {
    /// Load configuration from file or environment variables
    /// Precedence: CLI args > env vars > config file > defaults
    pub fn load() -> Result<Self> {
        // First try to load from config file
        let config_path = Self::get_config_path();

        let mut config: Config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {:?}", config_path))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {:?}", config_path))?
        } else {
            // Return error if config file doesn't exist - we need configuration
            anyhow::bail!(
                "Config file not found at {:?}. Please create it using config.example.toml as a template.",
                config_path
            );
        };

        // Override with environment variables if present
        if let Ok(device) = std::env::var("MEMO_MOUSE_DEVICE") {
            config.mouse_device = PathBuf::from(device);
        }
        if let Ok(device) = std::env::var("MEMO_AUDIO_DEVICE") {
            config.audio_device = device;
        }
        if let Ok(dir) = std::env::var("MEMO_OUTPUT_DIR") {
            config.output_dir = PathBuf::from(dir);
        }
        if let Ok(url) = std::env::var("MEMO_WHISPER_URL") {
            config.whisper_url = url;
        }
        if let Ok(topic) = std::env::var("MEMO_NTFY_TOPIC") {
            config.ntfy_topic = topic;
        }

        // Ensure output directory exists
        if !config.output_dir.exists() {
            std::fs::create_dir_all(&config.output_dir)
                .with_context(|| format!("Failed to create output directory: {:?}", config.output_dir))?;
        }

        Ok(config)
    }

    /// Get the default config file path: ~/.config/voice-memo/config.toml
    fn get_config_path() -> PathBuf {
        // Check for --config CLI argument first
        let args: Vec<String> = std::env::args().collect();
        if let Some(pos) = args.iter().position(|arg| arg == "--config") {
            if let Some(path) = args.get(pos + 1) {
                return PathBuf::from(path);
            }
        }

        // Default to ~/.config/voice-memo/config.toml
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("voice-memo");
        path.push("config.toml");
        path
    }
}
