# AGENTS.md — Agent Guidelines for `video2mp3`

> Guidance and architectural reference for AI coding agents (Antigravity, Claude Code, Cursor, etc.) working on this repository.

---

## 🧭 Repository Overview

`video2mp3` is a cross-platform desktop media conversion suite built with **Tauri 2.0**, **Rust**, and **Svelte 5** (bundled with **Vite**). It wraps **FFmpeg**, **ffprobe**, and **yt-dlp** using decoupled YAML configuration profiles.

- **Primary Goal**: High-performance local media transcoding, audio extraction (MP3/AAC), and YouTube playlist/video downloading with hardware acceleration (NVENC, QSV, AMF, VAAPI, VideoToolbox).
- **License**: MIT

---

## 📁 Project Architecture & Layout

```
.
├── src/                          # Frontend (Svelte 5 + Vanilla CSS)
│   ├── App.svelte                # Root UI shell
│   ├── app.js                    # Frontend orchestrator & Tauri command dispatchers
│   ├── stores.js                 # Svelte reactive stores (queue, settings, progress, logs)
│   ├── styles.css                # Global design system & theme variables
│   └── lib/                      # Modular UI components
│       ├── BottomBar.svelte      # Conversion progress bar & activity log
│       ├── FileQueue.svelte      # File list, track selector, drag-and-drop UI
│       ├── Settings.svelte       # Codec, hardware acceleration & tuning options
│       ├── TopBar.svelte         # System status badges (FFmpeg, yt-dlp, HW accels)
│       └── YouTubeBar.svelte     # URL input, playlist detection, YouTube options
│
├── src-tauri/                    # Tauri Backend (Rust)
│   ├── tauri.conf.json           # Tauri v2 application configuration
│   ├── Cargo.toml                # Rust dependencies & metadata
│   └── src/
│       ├── main.rs               # Desktop executable entrypoint
│       ├── lib.rs                # Tauri builder, plugin registration & command handlers
│       ├── commands/             # Tauri IPC command endpoints
│       │   ├── convert.rs        # Local file batch conversion (`convert_files`)
│       │   ├── probe.rs          # System probing & file metadata scanning (`scan_file`)
│       │   └── youtube.rs        # YouTube queries & downloads (`download_youtube`)
│       ├── core/                 # Core engine & domain logic
│       │   ├── config.rs         # YAML configuration loader (`yaml_serde`)
│       │   ├── convert_file.rs   # Local FFmpeg process execution & progress parser
│       │   ├── convert_youtube.rs# yt-dlp process manager & stream extraction
│       │   ├── probe.rs          # Hardware probing, codec inspection, ffprobe queries
│       │   └── types.rs          # Shared data models & enums
│       └── config/               # Externalized YAML Command Profiles (embedded via include_str!)
│           ├── ffmpeg.yaml       # FFmpeg arguments for all encoding/remuxing profiles
│           ├── ffprobe.yaml      # FFprobe queries for media duration & streams
│           └── ytdlp.yaml        # yt-dlp options for single videos and playlists
│
└── .github/workflows/
    └── build.yml                 # Multi-platform CI/CD build & GitHub Releases
```

---

## 🛠️ Build, Run & Test Commands

### Prerequisites
- **Node.js**: v20+
- **pnpm**: v10+
- **Rust Toolchain**: Stable (2024 edition)
- **External Binaries**: `ffmpeg`, `ffprobe`, `yt-dlp` in `$PATH`

### Frontend Commands
```bash
pnpm install             # Install npm dependencies
pnpm run dev             # Run Vite development server (port 5173)
pnpm run build           # Build production frontend to dist/
pnpm run preview         # Preview built frontend
```

### Tauri / Desktop App Commands
```bash
pnpm tauri dev           # Run desktop app with hot-reload (frontend + backend)
pnpm tauri build         # Build production desktop installers (.deb, .rpm, .msi, .dmg)
```

### Rust Backend Checks
```bash
cargo check --manifest-path src-tauri/Cargo.toml   # Verify Rust compilation
cargo test --manifest-path src-tauri/Cargo.toml    # Run backend tests
```

---

## ⚠️ Critical Conventions & Invariants

### 1. YAML Command Profiles (`src-tauri/src/config/`)
- All CLI arguments for `ffmpeg`, `ffprobe`, and `yt-dlp` are externalized in `src-tauri/src/config/*.yaml` and embedded at compile time using `include_str!`.
- **Audio Extraction Profiles (`extract_audio_mp3`, `extract_audio_aac`)**:
  - Always include `-vn` to suppress video and attached picture/cover art streams.
  - Stream placeholder `{audio_stream}` must resolve to a valid audio stream specifier.
- **Placeholder replacements**:
  - `{input}`: Source file path.
  - `{output}`: Destination file path.
  - `{audio_stream}`: Audio stream specifier (e.g., `"a:0"` or absolute index `"1"`).
  - `{hw_codec}`: Resolved hardware acceleration codec.
  - `{tune}`: Encoder tuning (`film` or `grain`).

### 2. Audio Stream Mapping in Rust (`convert_file.rs`)
- In container files (MP4, MKV, etc.), stream index `0` (`0:0`) is almost always **video**.
- When `audio_stream == 0` (default when no specific track is chosen or tracks list is pending), the backend must resolve `{audio_stream}` to `"a:0"`. This maps `-map 0:a:0` (first audio stream) instead of `-map 0:0` (video).
- Always capture child process `stderr` via `Stdio::piped()` (drained in a background thread) so failures contain actionable error messages instead of being swallowed.

### 3. Versioning & SemVer Sync (`build.yml`)
- Tauri strictly enforces **Semantic Versioning** (`MAJOR.MINOR.PATCH`) for `tauri.conf.json > version`.
- Git release tags might come as `v2.0.8` or `v.2.0.8`. The CI workflow in `.github/workflows/build.yml` extracts the clean SemVer using:
  ```javascript
  const match = raw.match(/(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?)/);
  const tag = match ? match[1] : raw.replace(/^v\.?/, '');
  ```
- Keep version fields synchronized between `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`.

### 4. GitHub Actions CI Runners
- GitHub has retired `macos-14` runner labels.
- Use `macos-latest` (or `macos-15`) for macOS jobs (`aarch64-apple-darwin`).
- Use `ubuntu-22.04` for Linux (`x86_64-unknown-linux-gnu`) and `windows-latest` for Windows (`x86_64-pc-windows-msvc`).

### 5. Frontend & UI Guidelines
- Framework is **Svelte 5**; follow fine-grained reactivity principles.
- Use Tauri's `@tauri-apps/api/core` for `invoke` and `@tauri-apps/api/event` for event listeners (`convert:progress`, `convert:finished`, `yt:progress`, `yt:finished`).
- Clean up all event listeners (`unlisten()`) on teardown or when re-registering.

---

## 🔍 Common Debugging Scenarios

| Issue | Typical Root Cause | Solution |
| :--- | :--- | :--- |
| `tauri.conf.json > version must be a semver string` | Git tag had leading `v.` (e.g., `v.2.0.8`) and was stripped to `.2.0.8`. | Ensure regex in `build.yml` extracts clean `2.0.8`. |
| `[mp3] No audio stream present` | `-map 0:0` mapped video stream instead of audio into MP3. | Ensure `-vn` is present and `audio_stream == 0` resolves to `a:0`. |
| `The configuration 'macos-14...' is not supported` | GitHub Actions deprecated macOS 14 runner image. | Change runner to `macos-latest` in `.github/workflows/build.yml`. |
| `glib-sys` / `webkit2gtk` build error on Linux | Missing GTK3 / WebKit development headers on the host. | Install `libwebkit2gtk-4.1-dev` (Debian/Ubuntu) or `webkit2gtk4.1-devel` (Fedora). |
