# Implementation Notes

## Architecture

Single async binary with three main tasks:

1. **Input task** - Watches evdev device for button events, sends Start/Stop signals
2. **Recording task** - Receives signals, manages audio capture, saves files
3. **Transcription task** - Receives completed file paths, handles HTTP to Whisper and ntfy

Use channels (tokio mpsc) between tasks.

## Device Identification

### Finding the mouse

```bash
# List all input devices with stable names
ls -la /dev/input/by-id/

# Watch events from a specific device
sudo evtest /dev/input/by-id/usb-XXXX-event-mouse

# Or use libinput
libinput debug-events --device /dev/input/by-id/usb-XXXX-event-mouse
```

The user will identify the device path and add it to config.

### Finding the microphone

```bash
# List ALSA capture devices
arecord -l

# List by card/device name
arecord -L

# Test recording
arecord -D default -f S16_LE -r 16000 -c 1 test.wav
```

## evdev Button Detection

```rust
use evdev::{Device, InputEventKind, Key};

let device = Device::open(&config.mouse_device)?;

loop {
    for event in device.fetch_events()? {
        if let InputEventKind::Key(Key::BTN_LEFT) = event.kind() {
            match event.value() {
                1 => { /* Button down - start recording */ }
                0 => { /* Button up - stop recording */ }
                _ => {}
            }
        }
    }
}
```

Note: Need to handle device disconnection gracefully - reopen on error.

## Audio Recording with cpal

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

let host = cpal::default_host();
let device = host.default_input_device()?;  // Or find by name

let config = cpal::StreamConfig {
    channels: 1,
    sample_rate: cpal::SampleRate(16000),
    buffer_size: cpal::BufferSize::Default,
};

// Capture samples to a Vec<i16> or ring buffer
// On stop, write to WAV using hound
```

## WAV Writing with hound

```rust
use hound::{WavSpec, WavWriter};

let spec = WavSpec {
    channels: 1,
    sample_rate: 16000,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
};

let mut writer = WavWriter::create(&path, spec)?;
for sample in samples {
    writer.write_sample(sample)?;
}
writer.finalize()?;
```

## HTTP Requests

### Whisper (example - adjust to actual server API)

```rust
let client = reqwest::Client::new();
let file_bytes = std::fs::read(&wav_path)?;

let response = client
    .post(&config.whisper_url)
    .header("Content-Type", "audio/wav")
    .body(file_bytes)
    .send()
    .await?;

let result: WhisperResponse = response.json().await?;
```

### ntfy.sh

```rust
// Success
client
    .post(&config.ntfy_topic)
    .body(transcript)
    .send()
    .await?;

// Failure
client
    .post(&config.ntfy_topic)
    .header("X-Title", "Transcription Failed")
    .header("X-Priority", "3")
    .body(format!("Recording saved: {}\nError: {}", filename, error))
    .send()
    .await?;
```

## Configuration Loading

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    mouse_device: PathBuf,
    audio_device: String,
    output_dir: PathBuf,
    whisper_url: String,
    ntfy_topic: String,
}

// Load from file or env
```

Precedence: CLI args > env vars > config file > defaults

Config file location: `~/.config/voice-memo/config.toml` or specified via `--config`

## systemd User Service

```ini
# ~/.config/systemd/user/voice-memo.service
[Unit]
Description=Voice Memo Daemon
After=default.target

[Service]
ExecStart=/usr/local/bin/voice-memo
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
```

Enable with:
```bash
systemctl --user enable voice-memo
systemctl --user start voice-memo
```

## Permissions

The user needs access to:
- `/dev/input/event*` - Usually requires `input` group membership
- Audio devices - Usually fine for desktop users

```bash
sudo usermod -a -G input $USER
# Log out and back in
```

## Logging

Use `tracing` with stdout subscriber. journald will capture it.

Levels:
- INFO: Recording started, recording saved, transcription sent
- WARN: Device disconnect, transcription failed (but WAV saved)
- ERROR: Critical failures
- DEBUG: Event details, HTTP responses

## Testing Without Hardware

Mock the evdev device with:
- A test mode that uses stdin (press Enter to toggle)
- Or `uinput` to create a virtual device

Mock Whisper with a simple HTTP server that echoes back.
