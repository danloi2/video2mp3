# video2mp3 🎵

> **A blazingly fast, native video-to-audio converter.**

**video2mp3** is a modern, high-performance desktop application designed to effortlessly extract MP3 audio from your video files. It combines the raw processing power of FFmpeg with a premium graphical interface, providing all the essential tools for both single-file and batch conversions.

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![FFmpeg](https://img.shields.io/badge/FFmpeg-007808?style=for-the-badge&logo=ffmpeg&logoColor=white)
![egui](https://img.shields.io/badge/egui-FF5722?style=for-the-badge&logo=rust&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)

---

## 🏛️ Project Philosophy

The project is built on the pillars of **native performance**, **user-centric design**, and **simplicity**. Inspired by the efficiency of systems programming and clear UI principles, **video2mp3** offers a seamless bridge for users to extract audio without fighting complex command-line interfaces, while retaining full execution speed.

---

## ✨ Key Features

### 🎥 High-Performance Processing

- **Batch Conversion**: Drag and drop multiple MKV, MP4, or AVI files and process them sequentially in the background.
- **FFmpeg Integration**: Leverages native FFmpeg binaries for hardware-accelerated, high-fidelity MP3 encoding.
- **Smart Resource Management**: Real-time progress tracking through stdout pipelines without blocking the GUI.

### 🎛️ Smart Audio Management

- **Automatic Track Detection**: Uses `ffprobe` to scan all available audio streams inside media containers.
- **Language Priority**: Automatically detects and pre-selects the Spanish track (SPA/ES) by default, saving time.
- **Manual Override**: Intuitive dropdown menus allow users to select any specific audio track before converting.

### 🚀 Desktop Experience

- **Native UI**: Built with `egui` and `eframe` for an instant startup and a lightweight memory footprint.
- **Execution Control**: Cancel active conversions instantly with a dedicated Stop button that safely cleans up partial files.
- **Safe Overwrites**: Interactive prompts warn you before overwriting existing destination files.

---

## 🛠️ Tech Stack

| Domain                | Technology                                                                                 |
| :-------------------- | :----------------------------------------------------------------------------------------- |
| **Core Language**     | [Rust](https://www.rust-lang.org/)                                                         |
| **GUI Framework**     | [eframe](https://docs.rs/eframe/latest/eframe/) + [egui](https://github.com/emilk/egui)    |
| **Media Engine**      | [FFmpeg](https://ffmpeg.org/) + [ffprobe](https://ffmpeg.org/ffprobe.html)                 |
| **File Dialogs**      | [rfd (Rust File Dialogs)](https://github.com/PolyMeilex/rfd)                               |
| **Data Parsing**      | [serde_json](https://github.com/serde-rs/json)                                             |

---

## 🚀 Getting Started

### Prerequisites

- Rust (v1.80+)
- Cargo package manager
- FFmpeg (v4.0+) available in your system's `$PATH`

```bash
# Ubuntu / Debian
sudo apt install ffmpeg
```

### Installation & Build

1. **Clone the repository** (if applicable) and navigate to the root directory:
   ```bash
   cd video2mp3
   ```
2. **Build the application**:
   ```bash
   cargo build --release
   ```
3. **Run the executable**:
   ```bash
   cargo run --release
   ```

---

## ⚖️ Legal & Copyright Notice

### ⚠️ License

This project is intended for daily utility and is provided "as is". Depending on FFmpeg linking, make sure to comply with FFmpeg's distribution guidelines.

- **Source Code**: Open-source and free to adapt.
- **Dependencies**: Relies on system-installed FFmpeg binaries (not bundled).

---

## 🤝 Contributing & Support

This project is developed for the benefit of the community to streamline media conversion tasks. Contributions that align with the project's performance and design goals are welcome.

**Author**: Daniel Losada - [![GitHub](https://img.shields.io/badge/Daniel_Losada-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/danloi2)
