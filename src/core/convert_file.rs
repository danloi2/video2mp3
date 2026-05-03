//! Local file transcoding and remuxing.
//!
//! This module implements the conversion logic for files already present on the
//! local filesystem, utilizing FFmpeg profiles defined in the configuration.

use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use super::probe::get_duration_seconds;
use super::types::{VideoOptions, ConversionType, HWAcceleration};
use super::config::load_ffmpeg_config;

/// Core function to perform file-to-file conversion using FFmpeg.
pub fn convert_file<F>(
    source: &Path,
    destination: Option<&Path>,
    audio_stream: u64,
    overwrite: bool,
    conv_type: ConversionType,
    options: VideoOptions,
    cancel: Arc<AtomicBool>,
    on_progress: F,
) -> Result<String, String>
where
    F: Fn(crate::core::ProgressUpdate),
{
    // Load config
    let config = load_ffmpeg_config()?;

    // Extract file base name for destination formatting
    let stem = source
        .file_stem()
        .ok_or_else(|| "Invalid file name".to_string())?
        .to_string_lossy();

    // Determine target extension based on conversion type
    let ext = match conv_type {
        ConversionType::AudioMP3 => "mp3",
        _ => "mkv",
    };

    // Resolve final destination path
    let destination_path: PathBuf = match destination {
        Some(d) => d.to_path_buf(),
        None => source
            .parent()
            .unwrap_or(Path::new("."))
            .join(format!("{}.{}", stem, ext)),
    };

    // Prevent accidental data loss
    if destination_path.exists() && !overwrite {
        return Err(format!(
            "⚠ Already exists: {}",
            destination_path.file_name().unwrap_or_default().to_string_lossy()
        ));
    }

    // Get source duration for progress percentage calculation
    let duration_s = get_duration_seconds(&source.to_string_lossy()).unwrap_or(0.0);

    // --- Profile Selection ---
    let profile_name = match conv_type {
        ConversionType::AudioMP3 => "extract_audio_mp3",
        ConversionType::VideoMKV => "remux_mkv",
        ConversionType::VideoH264 => match options.acceleration {
            HWAcceleration::None => "encode_h264_software",
            HWAcceleration::NVENC => "encode_h264_nvenc",
            _ => "encode_h264_hw",
        },
        ConversionType::VideoH265 => match options.acceleration {
            HWAcceleration::None => "encode_h265_software",
            HWAcceleration::NVENC => "encode_h265_nvenc",
            _ => "encode_h265_hw",
        },
    };

    let profile = config.profiles.get(profile_name)
        .ok_or_else(|| format!("Profile '{}' not found in config", profile_name))?;
    
    let mut args: Vec<String> = profile.args.as_ref()
        .ok_or_else(|| format!("Profile '{}' has no arguments", profile_name))?
        .clone();

    // --- Placeholder Replacement ---
    let hw_codec = match (conv_type, options.acceleration) {
        (ConversionType::VideoH264, HWAcceleration::QSV)   => "h264_qsv",
        (ConversionType::VideoH264, HWAcceleration::AMF)   => "h264_amf",
        (ConversionType::VideoH264, HWAcceleration::VAAPI) => "h264_vaapi",
        (ConversionType::VideoH264, HWAcceleration::VideoToolbox) => "h264_videotoolbox",
        (ConversionType::VideoH265, HWAcceleration::QSV)   => "hevc_qsv",
        (ConversionType::VideoH265, HWAcceleration::AMF)   => "hevc_amf",
        (ConversionType::VideoH265, HWAcceleration::VAAPI) => "hevc_vaapi",
        (ConversionType::VideoH265, HWAcceleration::VideoToolbox) => "hevc_videotoolbox",
        _ => "",
    };

    let tune = if options.preserve_grain { "grain" } else { "film" };

    for arg in args.iter_mut() {
        *arg = arg.replace("{input}", &source.to_string_lossy())
                  .replace("{output}", &destination_path.to_string_lossy())
                  .replace("{audio_stream}", &audio_stream.to_string())
                  .replace("{hw_codec}", hw_codec)
                  .replace("{tune}", tune);
    }

    // --- Optional Color Correction ---
    if options.optimize_color {
        if let Some(color_profile) = config.profiles.get("color_correction") {
            if let Some(extra) = &color_profile.extra_args {
                // Insert before the last argument (output path)
                let pos = args.len().saturating_sub(1);
                for (i, extra_arg) in extra.iter().enumerate() {
                    args.insert(pos + i, extra_arg.clone());
                }
            }
        }
    }

    let mut child = Command::new(&config.program)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Could not launch FFmpeg: {}", e))?;

    if let Some(stdout) = child.stdout.take() {
        let mut cancelled = false;
        for line in std::io::BufReader::new(stdout).lines().map_while(Result::ok) {
            if cancel.load(Ordering::Relaxed) {
                child.kill().ok();
                cancelled = true;
                break;
            }
            if let Some(val) = line.strip_prefix("out_time_us=") {
                if let Ok(us) = val.trim().parse::<i64>() {
                    if us > 0 && duration_s > 0.0 {
                        let ratio =
                            ((us as f64 / 1_000_000.0) / duration_s).clamp(0.0, 1.0) as f32;
                        on_progress(crate::core::ProgressUpdate::Ratio(ratio));
                    }
                }
            }
        }
        if cancelled {
            let _ = child.wait();
            let _ = std::fs::remove_file(&destination_path);
            return Err("⏹ Conversion cancelled".to_string());
        }
    }

    let status = child.wait().map_err(|e| e.to_string())?;

    if status.success() {
        Ok(format!(
            "✅ {} → {}",
            source.file_name().unwrap_or_default().to_string_lossy(),
            destination_path.file_name().unwrap_or_default().to_string_lossy()
        ))
    } else {
        let _ = std::fs::remove_file(&destination_path);
        Err(format!(
            "❌ FFmpeg failed to convert '{}'",
            source.file_name().unwrap_or_default().to_string_lossy()
        ))
    }
}
