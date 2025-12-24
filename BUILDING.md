# Building Earwig

## System Dependencies

Before building, install the required system libraries:

### Debian/Ubuntu

```bash
sudo apt-get update
sudo apt-get install -y \
    libasound2-dev \
    pkg-config \
    build-essential
```

### Fedora/RHEL

```bash
sudo dnf install -y \
    alsa-lib-devel \
    pkg-config \
    gcc
```

### Arch Linux

```bash
sudo pacman -S alsa-lib pkgconf base-devel
```

## Building

Once system dependencies are installed:

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check
```

## Running

### Development Mode

```bash
# Create a test config
mkdir -p ~/.config/voice-memo
cp config.example.toml ~/.config/voice-memo/config.toml
# Edit config with your settings
nano ~/.config/voice-memo/config.toml

# Run with debug logging
RUST_LOG=earwig=debug cargo run
```

### Production Installation

```bash
# Build release binary
cargo build --release

# Install to system
sudo cp target/release/earwig /usr/local/bin/

# Install systemd service
cp earwig.service ~/.config/systemd/user/
systemctl --user enable earwig
systemctl --user start earwig
```

## Cross-Compilation

To build for a different Linux architecture:

```bash
# Install cross-compilation target
rustup target add x86_64-unknown-linux-musl

# Build static binary (no system dependencies)
cargo build --release --target x86_64-unknown-linux-musl
```

## Development Tips

### Enable Rust Backtrace

```bash
RUST_BACKTRACE=1 cargo run
```

### Watch for Changes

Install `cargo-watch`:

```bash
cargo install cargo-watch
cargo watch -x check -x test
```

### Format Code

```bash
cargo fmt
```

### Run Linter

```bash
cargo clippy
```

## Troubleshooting Build Issues

### Missing ALSA

```
error: failed to run custom build command for `alsa-sys`
```

Solution: Install `libasound2-dev` (Debian/Ubuntu) or `alsa-lib-devel` (Fedora/RHEL)

### Missing pkg-config

```
error: failed to find tool `pkg-config`
```

Solution: Install `pkg-config` package

### Permission Denied on /dev/input

This is a runtime issue, not a build issue. See README.md for details.
