//! System probing commands: tool detection, version checks, hardware capabilities.

use serde::{Deserialize, Serialize};
use crate::core::probe;

/// System status payload returned to the frontend on startup.
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub ffmpeg_ok:      bool,
    pub ffmpeg_version: String,
    pub ytdlp_ok:       bool,
    pub ytdlp_version:  String,
    pub hw_nvenc:       bool,
    pub hw_qsv:         bool,
    pub hw_amf:         bool,
    pub hw_vaapi:       bool,
    pub hw_vtb:         bool,
}

/// Scanned media file metadata payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub path:      String,
    pub container: String,
    pub v_codec:   Option<String>,
    pub a_codec:   Option<String>,
    pub tracks:    Vec<AudioTrackInfo>,
}

/// Audio track metadata payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct AudioTrackInfo {
    pub stream_index:  u64,
    pub codec:         String,
    pub language:      String,
}

/// Probes the system for all required tools and hardware acceleration capabilities.
///
/// Called once on application startup to populate the status bar and settings panel.
#[tauri::command]
pub async fn probe_system() -> Result<SystemStatus, String> {
    let ffmpeg_ok      = probe::verify_ffmpeg();
    let ytdlp_ok       = probe::verify_ytdlp();
    let ffmpeg_version = probe::get_ffmpeg_version();
    let ytdlp_version  = probe::get_ytdlp_version();
    let hw             = probe::detect_hw_capabilities();

    Ok(SystemStatus {
        ffmpeg_ok,
        ffmpeg_version,
        ytdlp_ok,
        ytdlp_version,
        hw_nvenc: hw.nvenc,
        hw_qsv:   hw.qsv,
        hw_amf:   hw.amf,
        hw_vaapi: hw.vaapi,
        hw_vtb:   hw.vtb,
    })
}

/// Scans a local media file and returns its codec metadata and audio tracks.
#[tauri::command]
pub async fn scan_file(path: String) -> Result<FileInfo, String> {
    let tracks = probe::get_audio_tracks(&path)
        .into_iter()
        .map(|t| AudioTrackInfo {
            stream_index: t.stream_index,
            codec:        t.codec,
            language:     t.language,
        })
        .collect();

    let info = probe::get_media_info(&path);

    Ok(FileInfo {
        path,
        container: info.as_ref().map(|i| i.container.clone()).unwrap_or_default(),
        v_codec:   info.as_ref().and_then(|i| i.v_codec.clone()),
        a_codec:   info.as_ref().and_then(|i| i.a_codec.clone()),
        tracks,
    })
}
