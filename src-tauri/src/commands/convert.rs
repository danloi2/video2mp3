//! File conversion command with real-time progress streaming via Tauri events.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tauri::{AppHandle, Emitter, Listener};
use crate::core::{convert_file, types::{ConversionType, HWAcceleration, VideoOptions}};

/// A single file job submitted by the frontend.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConvertJob {
    /// Absolute path to the source file.
    pub source:       String,
    /// Optional absolute path for the output file (if None, placed next to source).
    pub destination:  Option<String>,
    /// Zero-based stream index of the selected audio track.
    pub audio_stream: u64,
    /// Conversion mode: "AudioMP3" | "AudioAAC" | "VideoMKV" | "VideoH264" | "VideoH265"
    pub conv_type:    String,
    /// Hardware acceleration: "None" | "NVENC" | "QSV" | "AMF" | "VAAPI" | "VideoToolbox"
    pub acceleration: String,
    /// Enable grain-tuning for software encoders.
    pub preserve_grain: bool,
    /// Enforce BT.709 color space parameters.
    pub optimize_color: bool,
}

/// Progress event payload emitted for each file during conversion.
#[derive(Debug, Serialize, Clone)]
pub struct ProgressPayload {
    /// Index of the item in the current batch.
    pub index:   usize,
    /// Progress ratio from 0.0 to 1.0.
    pub ratio:   f32,
    /// Current phase label: "converting" | "done" | "error"
    pub phase:   String,
    /// Result message (populated when phase is "done" or "error").
    pub message: Option<String>,
}

fn parse_conv_type(s: &str) -> ConversionType {
    match s {
        "AudioAAC"  => ConversionType::AudioAAC,
        "VideoMKV"  => ConversionType::VideoMKV,
        "VideoH264" => ConversionType::VideoH264,
        "VideoH265" => ConversionType::VideoH265,
        _           => ConversionType::AudioMP3,
    }
}

fn parse_accel(s: &str) -> HWAcceleration {
    match s {
        "NVENC"        => HWAcceleration::NVENC,
        "QSV"          => HWAcceleration::QSV,
        "AMF"          => HWAcceleration::AMF,
        "VAAPI"        => HWAcceleration::VAAPI,
        "VideoToolbox" => HWAcceleration::VideoToolbox,
        _              => HWAcceleration::None,
    }
}

/// Converts a batch of files, emitting `convert:progress` events for each item.
///
/// The command runs in a background thread so the frontend remains responsive.
/// Each progress event carries the item index and a ratio (0.0–1.0).
#[tauri::command]
pub async fn convert_files(app: AppHandle, jobs: Vec<ConvertJob>) -> Result<(), String> {
    let cancel = Arc::new(AtomicBool::new(false));

    // Register a one-shot cancel listener reachable from the frontend
    {
        let cancel_clone = cancel.clone();
        app.listen("convert:cancel", move |_| {
            cancel_clone.store(true, Ordering::Relaxed);
        });
    }

    for (idx, job) in jobs.into_iter().enumerate() {
        let source_path    = Path::new(&job.source).to_path_buf();
        let dest_path      = job.destination.as_ref().map(|d| Path::new(d).to_path_buf());
        let conv_type      = parse_conv_type(&job.conv_type);
        let options        = VideoOptions {
            preserve_grain: job.preserve_grain,
            optimize_color: job.optimize_color,
            acceleration:   parse_accel(&job.acceleration),
        };

        // Notify frontend: starting this item
        let _ = app.emit("convert:progress", ProgressPayload {
            index:   idx,
            ratio:   0.0,
            phase:   "converting".into(),
            message: None,
        });

        let app_clone    = app.clone();
        let cancel_clone = cancel.clone();

        let result = convert_file(
            &source_path,
            dest_path.as_deref(),
            job.audio_stream,
            true,
            conv_type,
            options,
            cancel_clone,
            move |update| {
                if let crate::core::types::ProgressUpdate::Ratio(r) = update {
                    let _ = app_clone.emit("convert:progress", ProgressPayload {
                        index:   idx,
                        ratio:   r,
                        phase:   "converting".into(),
                        message: None,
                    });
                }
            },
        );

        match result {
            Ok(msg) => {
                let _ = app.emit("convert:progress", ProgressPayload {
                    index:   idx,
                    ratio:   1.0,
                    phase:   "done".into(),
                    message: Some(msg),
                });
            }
            Err(e) => {
                let _ = app.emit("convert:progress", ProgressPayload {
                    index:   idx,
                    ratio:   0.0,
                    phase:   "error".into(),
                    message: Some(e),
                });
            }
        }
    }

    let _ = app.emit("convert:finished", ());
    Ok(())
}
