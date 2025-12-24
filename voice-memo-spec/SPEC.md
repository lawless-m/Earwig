# Voice Memo Daemon Specification

## Overview

A background daemon that captures voice memos via a dedicated USB mouse button, transcribes them via a Whisper server, and sends notifications via ntfy.sh.

## User Story

User is lying in bed, has an idea, grabs dedicated mouse, holds button, speaks idea, releases button. Moments later, phone buzzes with transcribed text. Audio file is kept locally regardless of transcription success.

## Behaviour

1. **Wait** - Daemon idles, watching for button press on specific mouse
2. **Record** - On button down, start recording 16kHz mono WAV from configured mic
3. **Stop** - On button up, stop recording, save WAV with timestamp filename
4. **Transcribe** - POST WAV to Whisper HTTP endpoint
5. **Notify** - POST transcript (or error message) to ntfy.sh topic
6. **Loop** - Return to waiting

## Audio Specification

- Format: WAV
- Sample rate: 16000 Hz
- Channels: 1 (mono)
- Bit depth: 16-bit PCM

## File Storage

- All recordings saved to configurable output directory
- Filename format: `memo_YYYYMMDD_HHMMSS.wav`
- Files are **never** automatically deleted
- On transcription success: keep file
- On transcription failure: keep file

## Configuration

Via config file (TOML or similar) or environment variables:

```
MEMO_MOUSE_DEVICE=/dev/input/by-id/usb-SomeMouse-event-mouse
MEMO_AUDIO_DEVICE=default  # ALSA device name
MEMO_OUTPUT_DIR=/home/user/voice-memos
MEMO_WHISPER_URL=http://localhost:9000/transcribe
MEMO_NTFY_TOPIC=https://ntfy.sh/my-secret-topic
```

## External Interfaces

### Whisper Server

- Method: POST
- Content-Type: audio/wav (or multipart/form-data depending on server)
- Body: Raw WAV file
- Expected response: JSON with transcript text

The actual request/response format should be configurable or at least easily modifiable, as Whisper server implementations vary.

### ntfy.sh

- Method: POST
- URL: `https://ntfy.sh/{topic}` (or self-hosted)
- Body: Plain text transcript
- Headers: Optional priority, title, tags as needed

On success:
```
POST https://ntfy.sh/my-topic
Content-Type: text/plain

[Transcript text here]
```

On failure:
```
POST https://ntfy.sh/my-topic
Content-Type: text/plain
X-Priority: 3
X-Title: Transcription Failed

Recording saved: memo_20241224_030512.wav
Error: Connection refused
```

## Error Handling

| Scenario | Behaviour |
|----------|-----------|
| Mouse disconnected | Log error, keep trying to reconnect |
| Mic unavailable | Log error, notify via ntfy, skip recording |
| Whisper unreachable | Save WAV, notify failure via ntfy |
| Whisper returns error | Save WAV, notify failure via ntfy |
| ntfy unreachable | Log error, continue (WAV is safe) |
| Very short recording (<100ms) | Process anyway, let Whisper handle it |

## Deployment

- Runs as systemd user service
- No GUI, no TUI
- Logs to stdout/journald
- Should start on login and persist

## Suggested Rust Crates

- `evdev` - Reading mouse events from specific device
- `cpal` - Cross-platform audio capture
- `hound` - WAV file writing
- `reqwest` - HTTP client (blocking or async)
- `tokio` - Async runtime
- `serde` / `toml` - Configuration parsing
- `chrono` - Timestamp generation
- `tracing` or `log` - Logging

## Non-Goals

- GUI or TUI
- Audio playback
- Editing/deleting recordings
- Multiple button actions
- Complex audio processing (noise reduction, VAD, etc.)
- Queueing/retry logic for failed transcriptions

## Future Possibilities (Out of Scope)

- Web UI to browse recordings
- Local Whisper fallback
- Keyboard shortcut alternative to mouse
- Audio level indicator (LED on device?)
