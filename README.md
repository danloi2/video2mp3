# video2mp3 🎵

<p align="center">
  <img src="resources/icon.png" width="180" alt="video2mp3 Logo">
</p>

> **A blazingly fast, professional media suite for high-performance transcoding and YouTube downloading.**

**video2mp3** is an industry-grade media processing suite designed for effortless downloading and high-speed re-encoding. By bridging the raw efficiency of **FFmpeg** and **yt-dlp** with a sleek, modern **Svelte** and **Tauri 2.0** interface, it provides a powerful yet intuitive workspace for both single-file tasks and massive batch conversions—all boosted by full hardware acceleration.

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri_2.0-FFC131?style=for-the-badge&logo=tauri&logoColor=white)
![Svelte](https://img.shields.io/badge/Svelte-FF3E00?style=for-the-badge&logo=svelte&logoColor=white)
![FFmpeg](https://img.shields.io/badge/FFmpeg-007808?style=for-the-badge&logo=ffmpeg&logoColor=white)
![Hardware Accel](https://img.shields.io/badge/HW--Acceleration-Active-blue?style=for-the-badge)

---

## 🏛️ Project Philosophy

The project is built on the pillars of **native performance**, **professional architecture**, and **beautiful UI**. Recently migrated to a **Tauri 2.0 + Svelte** stack, **video2mp3** serves as a template for how to build robust, thread-safe, and visually stunning cross-platform media desktop applications.

---

## ✨ Key Features

### 🌍 YouTube & Playlists
- **Smart Staged Workflow**: Analyze YouTube URLs in the background while managing your queue.
- **Full Playlist Support**: Automatically detect and expand entire playlists for batch processing.
- **Progress Tracking**: Real-time feedback for both download and post-processing phases via IPC Events.

### 🚀 Hardware Acceleration (Pro Grade)
- **Real-time Probing**: Dynamically detects available GPU encoders (NVENC, QSV, AMF, VAAPI, VideoToolbox).
- **Dynamic Optimization**: Automatically configures encoder flags for the best balance between speed and quality.
- **Visual Status**: Integrated UI tags show exactly which hardware features are currently usable on your system.

### 🎥 Professional Media Workspace
- **Deep Media Probing**: Detailed inspection of containers and codecs (MKV, MP4, etc.) using `ffprobe`.
- **Intelligent Track Selection**: Scans all audio streams; automatically pre-selects primary language tracks.
- **Custom Design System**: A premium, high-contrast dark theme powered by modern CSS tokens and Vite.

---

## 🏗️ Architectural Overview (v2.0.0+)

The project has been refactored into a modern, decoupled web/native structure:

```mermaid
graph TD
    A[Svelte UI - src/] -->|Tauri IPC Commands| B[Tauri Backend - src-tauri/]
    A -->|State Management| C[stores.js]
    B -->|Async Events| A
    B --> D[Core Engine]
    D --> E[FFmpeg/yt-dlp Wrappers]
    D --> F[Hardware Probing]
    D --> G[YAML Configs]
```

- **`src/`**: The entire frontend built with Svelte, Vanilla CSS, and Vite. Handles UI rendering and reactive state.
- **`src-tauri/src/commands/`**: Rust endpoints exposed to the frontend via the Tauri IPC bridge.
- **`src-tauri/src/core/`**: Pure business logic, hardware detection, and media wrappers.
- **`src-tauri/src/config/`**: External YAML profiles defining FFmpeg, yt-dlp, and FFprobe command architectures.

---

## ⚙️ Advanced Configuration (YAML)

**video2mp3** externalizes its conversion and download logic into YAML configuration files. This allows advanced modifications to FFmpeg parameters without touching the Rust core.

### 📄 Configuration Files (`src-tauri/src/config/`)
- **`ffmpeg.yaml`**: Defines profiles for audio extraction, remuxing, and hardware-accelerated transcoding.
- **`ytdlp.yaml`**: Manages yt-dlp arguments for metadata extraction and various download modes.
- **`ffprobe.yaml`**: Configuration for media inspection, duration probing, and stream analysis.

---

## 🛠️ Tech Stack

| Domain                | Technology                                                                                 |
| :-------------------- | :----------------------------------------------------------------------------------------- |
| **Backend**           | **Rust** + **Tauri 2.0** (System access, multi-threading, IPC)                             |
| **Frontend**          | **Svelte 5** + **Vite** + Vanilla CSS                                                      |
| **Engines**           | [FFmpeg](https://ffmpeg.org/) + [yt-dlp](https://github.com/yt-dlp/yt-dlp)                 |
| **Package Manager**   | **pnpm**                                                                                   |

---

## 🚀 Getting Started

### Prerequisites

- **FFmpeg (v5.0+)** available in your system's `$PATH`.
- **yt-dlp** for YouTube integration features.
- **Node.js (v20+)** & **pnpm**.
- **Rust Toolchain**.

### Build from source

1. **Clone the repository**:
   ```bash
   git clone https://github.com/danloi2/video2mp3.git
   cd video2mp3
   ```

2. **Install frontend dependencies**:
   ```bash
   pnpm install
   ```

3. **Run in Development Mode (Hot-Reload)**:
   ```bash
   pnpm tauri dev
   ```

4. **Build Production Installers**:
   ```bash
   pnpm tauri build
   ```

---

## 🤝 Contributing

This project is maintained as a high-quality open-source media suite. Contributions regarding new hardware acceleration profiles or UI refinements are welcome.

**Author**: Daniel Losada - [![GitHub](https://img.shields.io/badge/danloi2-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/danloi2)

---

## ⚖️ License

Licensed under the **MIT License**. See [LICENSE](LICENSE) for details.
