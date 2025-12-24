# Earwig - Voice Memo Daemon

A "push-to-talk" voice memo capture daemon for Linux. Hold a button on a dedicated USB mouse, speak your idea, release. The audio is saved locally, transcribed via Whisper, and a notification sent to your phone via ntfy.sh.

![Spellbook](spellbook.png)

## Features

- **Simple Push-to-Talk**: Hold a mouse button to record, release to stop
- **Always Saves**: Audio files are kept regardless of transcription success
- **Auto-Transcription**: Sends recordings to your Whisper server
- **Instant Notifications**: Get transcribed text via ntfy.sh on your phone
- **Reliable**: Automatic device reconnection, runs as systemd service
- **No GUI**: Lightweight daemon, logs to journald

## How It Works

1. **Wait** - Daemon watches for button press on dedicated USB mouse
2. **Record** - Button down starts 16kHz mono WAV recording
3. **Stop** - Button up stops recording, saves with timestamp
4. **Transcribe** - POSTs WAV to Whisper HTTP endpoint
5. **Notify** - Sends transcript (or error) to ntfy.sh
6. **Loop** - Returns to waiting

## Requirements

### Hardware
- A cheap USB mouse (dedicated to this purpose)
- A USB microphone (or any ALSA-compatible input)
- A Linux machine running the daemon

### Software
- Rust toolchain (for building)
- A Whisper HTTP server (expects 16kHz mono WAV)
- ntfy.sh account/topic (or self-hosted ntfy)

### System Access
- User must be in `input` group to access `/dev/input/event*` devices

```bash
sudo usermod -a -G input $USER
# Log out and back in for group to take effect
```

## Installation

### 1. Build the Binary

```bash
cargo build --release
sudo cp target/release/earwig /usr/local/bin/
```

### 2. Configure

Create configuration directory and file:

```bash
mkdir -p ~/.config/voice-memo
cp config.example.toml ~/.config/voice-memo/config.toml
```

Edit `~/.config/voice-memo/config.toml` with your settings:

```toml
mouse_device = "/dev/input/by-id/usb-PixArt_USB_Optical_Mouse-event-mouse"
audio_device = "default"
output_dir = "/home/user/voice-memos"
whisper_url = "http://localhost:9000/transcribe"
ntfy_topic = "https://ntfy.sh/my-secret-voice-memos"
```

#### Finding Your Mouse Device

```bash
# List all input devices
ls -la /dev/input/by-id/

# Test which device is your mouse
sudo evtest /dev/input/by-id/usb-XXXX-event-mouse
```

#### Finding Your Microphone

```bash
# List ALSA capture devices
arecord -l

# Test recording
arecord -D default -f S16_LE -r 16000 -c 1 test.wav
```

### 3. Install as systemd Service

```bash
# Copy service file
cp earwig.service ~/.config/systemd/user/

# Enable and start
systemctl --user enable earwig
systemctl --user start earwig

# Check status
systemctl --user status earwig

# View logs
journalctl --user -u earwig -f
```

## Usage

Once the daemon is running:

1. Grab your dedicated USB mouse
2. Hold down the left button
3. Speak your memo
4. Release the button
5. Wait a moment - your phone will buzz with the transcribed text!

All recordings are saved to your configured output directory with filenames like:
```
memo_20241224_143052.wav
```

## Configuration

Configuration can be provided via:
- Config file: `~/.config/voice-memo/config.toml` (recommended)
- Environment variables: `MEMO_MOUSE_DEVICE`, `MEMO_AUDIO_DEVICE`, etc.
- CLI argument: `--config /path/to/config.toml`

Precedence: CLI args > env vars > config file

## Logging

Set log level via `RUST_LOG` environment variable:

```bash
# In systemd service file
Environment="RUST_LOG=earwig=debug"

# Or when running manually
RUST_LOG=earwig=debug earwig
```

Levels: `trace`, `debug`, `info`, `warn`, `error`

## Troubleshooting

### Device Permission Denied

Make sure you're in the `input` group and have logged out/in:

```bash
groups  # Should show 'input'
sudo usermod -a -G input $USER
```

### No Audio Recorded

Test your microphone with arecord:

```bash
arecord -D default -f S16_LE -r 16000 -c 1 test.wav
# Press Ctrl+C after a few seconds, then play it back:
aplay test.wav
```

### Whisper Connection Failed

Check that your Whisper server is running and accessible:

```bash
curl -X POST -H "Content-Type: audio/wav" --data-binary @test.wav http://localhost:9000/transcribe
```

### ntfy Not Receiving

Test ntfy directly:

```bash
curl -d "Test message" https://ntfy.sh/your-topic
```

## Architecture

The daemon runs three async tasks:

1. **Input Task** - Watches evdev device for button events
2. **Recording Task** - Manages audio capture and WAV saving
3. **Transcription Task** - Handles HTTP to Whisper and ntfy

Tasks communicate via async channels (tokio mpsc).

## Project Structure

```
earwig/
├── src/
│   ├── main.rs           # Main entry point, task orchestration
│   ├── config.rs         # Configuration loading
│   ├── input.rs          # Mouse button monitoring (evdev)
│   ├── audio.rs          # Audio recording (cpal + hound)
│   └── transcription.rs  # Whisper + ntfy HTTP clients
├── Cargo.toml
├── earwig.service        # systemd user service file
├── config.example.toml   # Example configuration
└── voice-memo-spec/      # Full specification documents
```

## Claude Code Skills

This repository includes useful Claude Code skills in `.claude/skills/`:

- **Rust** - Rust development patterns
- **Web Frontend** - React + Tailwind artifacts
- **Databases** - Database access patterns
- **And many more...**

See the skills directory for the full list.

## License

MIT

## Credits

Built with:
- [evdev](https://github.com/cmr/evdev) - Linux input device access
- [cpal](https://github.com/RustAudio/cpal) - Cross-platform audio I/O
- [hound](https://github.com/ruuda/hound) - WAV encoding/decoding
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime
