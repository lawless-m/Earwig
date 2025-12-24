# Voice Memo Daemon - Project Specification

A "push-to-talk" voice memo capture daemon for Linux. Hold a button on a dedicated USB mouse, speak, release. Audio is saved locally, transcribed via Whisper, and notification sent via ntfy.sh.

## Contents

| File | Description |
|------|-------------|
| `SPEC.md` | **Start here** - Full specification: behaviour, interfaces, configuration |
| `IMPLEMENTATION.md` | Technical notes: architecture, crate suggestions, code snippets |
| `config.example.toml` | Example configuration file |

## Quick Start for Implementation

1. Read `SPEC.md` for the full requirements
2. Read `IMPLEMENTATION.md` for architecture and code patterns
3. Set up a new Rust project with the suggested crates
4. Implement in order: config loading → evdev input → audio capture → HTTP calls
5. Test with a cheap USB mouse and `arecord` first before integrating

## Hardware Required

- A cheap USB mouse (dedicated to this purpose)
- A USB microphone (or any ALSA-compatible input)
- A Linux box running this daemon

## External Services

- A Whisper HTTP server (user's existing setup, expects 16kHz mono WAV)
- ntfy.sh account/topic (or self-hosted ntfy)

## Key Decisions

- **Hold-to-record**: Button down starts, button up stops
- **Always save**: WAV files kept regardless of transcription success
- **No retry logic**: If Whisper fails, notify and move on (file is safe)
- **No minimum duration**: Even accidental clicks are processed
- **Daemon mode**: Runs as systemd user service, no UI
