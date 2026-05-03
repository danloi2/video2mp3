//! YouTube download and playlist scan commands.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tauri::{AppHandle, Emitter, Listener};
use crate::core::{
    convert_youtube::{download_youtube, get_playlist_videos},
    types::ConversionType,
};

/// A single YouTube playlist entry.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistEntry {
    pub url:   String,
    pub title: String,
}

/// Progress event payload for YouTube downloads.
#[derive(Debug, Serialize, Clone)]
pub struct YtProgressPayload {
    pub index:   usize,
    pub ratio:   f32,
    pub phase:   String,
    pub message: Option<String>,
}

/// Scans a YouTube URL and returns all video entries (handles playlists).
#[tauri::command]
pub async fn scan_playlist(url: String) -> Result<Vec<PlaylistEntry>, String> {
    let mut entries = Vec::new();
    get_playlist_videos(&url, |u, t| {
        entries.push(PlaylistEntry { url: u, title: t });
    });
    Ok(entries)
}

/// Downloads one or more YouTube URLs, emitting `yt:progress` events in real-time.
#[tauri::command]
pub async fn download_youtube_cmd(
    app:         AppHandle,
    urls:        Vec<String>,
    destination: String,
    conv_type:   String,
) -> Result<(), String> {
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let cancel_clone = cancel.clone();
        app.listen("yt:cancel", move |_| {
            cancel_clone.store(true, Ordering::Relaxed);
        });
    }

    let ct = match conv_type.as_str() {
        "AudioAAC"  => ConversionType::AudioAAC,
        "VideoMKV"  => ConversionType::VideoMKV,
        "VideoH264" => ConversionType::VideoH264,
        "VideoH265" => ConversionType::VideoH265,
        _           => ConversionType::AudioMP3,
    };

    let dest = if destination.is_empty() {
        dirs::download_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
    } else {
        Path::new(&destination).to_path_buf()
    };

    for (idx, url) in urls.into_iter().enumerate() {
        let app_clone    = app.clone();
        let cancel_clone = cancel.clone();

        let _ = app.emit("yt:progress", YtProgressPayload {
            index: idx, ratio: 0.0, phase: "downloading".into(), message: None,
        });

        let result = download_youtube(&url, &dest, ct, cancel_clone, move |update| {
            use crate::core::types::ProgressUpdate;
            let payload = match update {
                ProgressUpdate::Ratio(r) => YtProgressPayload {
                    index: idx, ratio: r, phase: "downloading".into(), message: None,
                },
                ProgressUpdate::Phase(p) => YtProgressPayload {
                    index: idx, ratio: 0.0, phase: p.to_string(), message: None,
                },
                ProgressUpdate::Playlist(cur, tot) => YtProgressPayload {
                    index: idx,
                    ratio: cur as f32 / tot.max(1) as f32,
                    phase: "downloading".into(),
                    message: Some(format!("{}/{}", cur, tot)),
                },
            };
            let _ = app_clone.emit("yt:progress", payload);
        });

        match result {
            Ok(path) => {
                let _ = app.emit("yt:progress", YtProgressPayload {
                    index:   idx,
                    ratio:   1.0,
                    phase:   "done".into(),
                    message: Some(path.file_name().unwrap_or_default().to_string_lossy().to_string()),
                });
            }
            Err(e) => {
                let _ = app.emit("yt:progress", YtProgressPayload {
                    index:   idx,
                    ratio:   0.0,
                    phase:   "error".into(),
                    message: Some(e),
                });
            }
        }
    }

    let _ = app.emit("yt:finished", ());
    Ok(())
}
