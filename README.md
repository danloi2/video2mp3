# video2mp3 🎵

<p align="center">
  <img src="assets/icon.png" width="180" alt="video2mp3 Logo">
</p>

> **A blazingly fast, native video & audio re-encoder with hardware acceleration.**

**video2mp3** is a modern, high-performance desktop application designed to effortlessly extract audio or re-encode video files. It combines the raw processing power of FFmpeg with a premium graphical interface, providing all the essential tools for both single-file and batch conversions.

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![FFmpeg](https://img.shields.io/badge/FFmpeg-007808?style=for-the-badge&logo=ffmpeg&logoColor=white)
![yt-dlp](https://img.shields.io/badge/yt--dlp-FF0000?style=for-the-badge&logo=youtube&logoColor=white)
![egui](https://img.shields.io/badge/egui-FF5722?style=for-the-badge&logo=rust&logoColor=white)
![Windows](https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)
![macOS](https://img.shields.io/badge/macOS-000000?style=for-the-badge&logo=apple&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)

---

## 🏛️ Project Philosophy

The project is built on the pillars of **native performance**, **user-centric design**, and **simplicity**. Inspired by the efficiency of systems programming and clear UI principles, **video2mp3** offers a seamless bridge for users to process media without fighting complex command-line interfaces, while retaining full execution speed.

---

## ✨ Key Features

### 🌍 YouTube & Playlists (NEW!)
- **Direct Download**: Paste any YouTube link to extract audio (MP3) or download video in the best quality available.
- **Full Playlist Support**: Detects and processes entire playlists automatically, with item-by-item progress tracking.
- **Smart Conversion**: Downloaded videos can be automatically converted to H.264/H.265 MKV in a single step using your preferred hardware acceleration.

### 🚀 Hardware Acceleration
- **Automatic Detection**: Probes your PC to find NVIDIA (NVENC), Intel (QSV), AMD (AMF/VAAPI), or Apple (VideoToolbox) chips.
- **Smart UI Tags**: Visual badges show you exactly what hardware is active and usable in your system.
- **Turbo Encoding**: Up to 10x faster processing in supported GPUs compared to CPU-only encoding.

### 🎥 High-Performance Processing
- **Batch Conversion**: Drag and drop multiple MKV, MP4, or AVI files and process them sequentially in the background.
- **Custom Output Directory**: Choose exactly where your files go. If not set, it defaults to the source folder (for files) or your Downloads folder (for YouTube).
- **Smart Resource Management**: Real-time progress tracking through stdout pipelines without blocking the GUI.

### 🎛️ Smart Audio Management
- **Automatic Track Detection**: Uses `ffprobe` to scan all available audio streams inside media containers.
- **Language Priority**: Automatically detects and pre-selects the Spanish track (SPA/ES) by default.
- **Manual Override**: Intuitive dropdown menus allow users to select any specific audio track.

---

## 🛠️ Tech Stack

| Domain                | Technology                                                                                 |
| :-------------------- | :----------------------------------------------------------------------------------------- |
| **Core Language**     | [Rust](https://www.rust-lang.org/)                                                         |
| **GUI Framework**     | [eframe](https://docs.rs/eframe/latest/eframe/) + [egui](https://github.com/emilk/egui)    |
| **Media Engines**     | [FFmpeg](https://ffmpeg.org/) + [ffprobe](https://ffmpeg.org/ffprobe.html) + [yt-dlp](https://github.com/yt-dlp/yt-dlp) |
| **Hardware APIs**     | NVENC, QSV, VAAPI, AMF, Apple VideoToolbox                                                 |
| **Automation**        | GitHub Actions (Multi-platform Builds)                                                     |

---

## 🚀 Getting Started

### Prerequisites

- **FFmpeg (v4.0+)** available in your system's `$PATH`.
- **yt-dlp** (Optional but recommended) for YouTube download features.
- For Hardware Acceleration: Updated GPU drivers (NVIDIA/Intel/AMD/Apple).

**Quick Download:**
You can find the latest pre-compiled binaries for your system in the [**Releases**](https://github.com/danloi2/convmp3/releases) section. Look for the "Latest Build" tag.

### Build from source

1. **Clone the repository** and navigate to the root directory:
   ```bash
   git clone https://github.com/danloi2/convmp3.git
   cd convmp3
   ```

2. **Build and Run**:
   ```bash
   cargo run --release
   ```

---

## ⚖️ Legal & Copyright Notice

### ⚠️ License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for more information.

- **Source Code**: Open-source, free to use, modify, and distribute.
- **Dependencies**: Relies on system-installed FFmpeg and yt-dlp binaries (not bundled), which may have their own licensing guidelines (e.g., GPL/LGPL).

---

## 🤝 Contributing & Support

This project is developed for the benefit of the community to streamline media conversion tasks. Contributions that align with the project's performance and design goals are welcome.

**Author**: Daniel Losada - [![GitHub](https://img.shields.io/badge/Daniel_Losada-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/danloi2)
