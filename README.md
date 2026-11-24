# DLNA SERVER 2.6.0

<p align="center">
  <img src="docs/images/logo.png" alt="DLNA SERVER logo" width="160">
</p>

<p align="center">
  **A lightweight, high-performance DLNA video server for Linux.**
</p>

<p align="center">
  Share your local video library with Smart TVs, mobile devices, and media players over your home network.
</p>

---

## Overview

**DLNA SERVER** is a native desktop application written in Rust for streaming local video libraries to DLNA / UPnP-compatible devices.

It is designed to provide fast library indexing, efficient network streaming, low resource consumption, and a straightforward desktop interface.

The application focuses on local video sharing. Your original video files remain untouched.

![DLNA SERVER main window](docs/images/screen.png)

## Features

- **Automatic device discovery** — zero-configuration SSDP discovery and UPnP ContentDirectory integration make the server visible to compatible devices on the local network.
- **Direct Play** — streams original video files over HTTP with byte-range seeking and no transcoding; direct streaming is designed for 0% additional CPU load.
- **HTTP Range support** — supports byte-range requests for seeking and playback control.
- **Smart transcoding** — optional conversion of unsupported formats (including H.265/HEVC, AV1, and VP9) to H.264/AAC, up to 1080p.
- **Hardware acceleration** — supports NVIDIA NVENC and VAAPI, with safe CPU fallback when available.
- **4K protection** — includes safeguards intended to prevent excessive system load during transcoding.
- **Media library management** — add and remove shared folders, scan directories, and update the active library.
- **Automatic startup** — optional launch at system login, with network-readiness handling.
- **System tray integration** — convenient background operation.
- **Anti-Sleep protection** — uses systemd-inhibit to help prevent system suspension while serving media.
- **12 interface languages** — English, Russian, German, French, Spanish, Italian, Portuguese, Polish, Ukrainian, Chinese, Japanese, and Korean.
- **Built-in help** — an integrated guide for application features.

## Performance

The following results were measured head-to-head on the same Linux desktop hardware, streaming the same 1080p video file. The SMB figure is the measured single-stream CPU usage when the media is accessed over an SMB network share.

| Metric | Previous Python Version | DLNA SERVER 2.6.0 (Rust) | Improvement |
|---|---:|---:|---|
| **CPU Usage (Single Stream)** | 2.1% | **1.2% (0.5% over SMB)** | **~2× lower CPU load** |
| **CPU Usage (2 Simultaneous Clients: TV + Phone)** | — | **0.9% total** | Multi-client streaming under 1% CPU |
| **Memory Footprint (RSS with Native GUI)** | 141.0 MB | **137.8 MB** | Lower RAM even with 12-language Slint UI |
| **Library Scan & Import Time** | ~30 minutes | **~30 seconds (< 0.1s from cache)** | **60× faster indexing** |

These are measurements from the stated test environment, not universal performance guarantees.

## What's New in v2.6.0

- **Instant UI Controls:** Non-blocking server start/stop, background directory scanning, and off-thread native file dialogs for smooth Wayland/X11 responsiveness.
- **Smart Auto-Broadcast on Boot:** Automatically waits for LAN/Wi-Fi readiness on system startup and goes on air when autostart is enabled.
- **TV-Friendly Directory Sorting:** Folders are always listed first alphabetically, followed by video files alphabetically.
- **Live Index Updates:** Removing a media folder immediately updates the active DLNA catalog without restarting the stream.
- **12-Language UI & Built-In Help:** Instant switching between RU, EN, DE, FR, ES, IT, PT, PL, UK, ZH, JA, KO with a localized built-in guide.

## Installation

### Debian / Ubuntu

Download the latest `.deb` package from the [GitHub Releases](../../releases) page and install it:

```bash
sudo apt install ./dlna-video-server_2.6.0_amd64.deb
```

Launch **DLNA SERVER** from your desktop application menu.

## Building from Source

### Requirements

- Linux
- Rust stable toolchain
- Cargo
- FFmpeg
- Development packages for Wayland, X11, Fontconfig, and FreeType
- `pkg-config`

On Ubuntu / Debian:

```bash
sudo apt install build-essential pkg-config \
  libwayland-dev libx11-dev \
  libfontconfig1-dev libfreetype6-dev ffmpeg
```

### Build

Clone the repository:

```bash
git clone https://github.com/arturzailer779-droid/dlna-server-rust.git
cd dlna-server-rust
```

Build the application:

```bash
cargo build --release -p dlna-app
```

The release binary will be located at:

```text
target/release/dlna-app
```

### Run Tests

```bash
cargo test --workspace
```

Run Clippy:

```bash
cargo clippy --workspace --all-targets
```

## Technology

- **Rust** — application and core services
- **Slint** — desktop user interface
- **SSDP / UPnP / DLNA** — device discovery and media sharing
- **HTTP** — video streaming and range requests
- **FFmpeg** — optional video transcoding
- **Tokio** — asynchronous runtime

The project is organized as a modular Rust workspace, separating the application, core logic, media library, networking, DLNA protocol, streaming, compatibility, platform integration, and transcoding components.

## License

DLNA SERVER is free and open-source software licensed under the **GNU General Public License v3.0 or later (GPL-3.0-or-later)**.

See the [LICENSE](LICENSE) file for the complete license text.

---

<p align="center">
  **DLNA SERVER 2.6.0**
  <br>
  Built with Rust.
</p>
