use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use super::probe::get_duration_seconds;
use super::types::{VideoOptions, ConversionType};

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

    // --- FFmpeg Command Initialization ---
    let mut args = vec!["-y".to_string()]; 

    use crate::core::types::HWAcceleration;
    // Enable auto-hardware acceleration if requested
    if options.acceleration != HWAcceleration::None {
        args.extend(["-hwaccel".to_string(), "auto".to_string()]);
    }

    args.extend([
        "-i".to_string(),
        source.to_string_lossy().to_string(),
    ]);

    // --- Encoder Configuration ---
    match conv_type {
        ConversionType::AudioMP3 => {
            args.extend([
                "-map".to_string(),
                format!("0:{}", audio_stream),
                "-c:a".to_string(),
                "libmp3lame".to_string(),
                "-q:a".to_string(),
                "2".to_string(),
            ]);
        }
        ConversionType::VideoH264 => {
            let codec = match options.acceleration {
                HWAcceleration::NVENC => "h264_nvenc",
                HWAcceleration::QSV   => "h264_qsv",
                HWAcceleration::AMF   => "h264_amf",
                HWAcceleration::VAAPI => "h264_vaapi",
                HWAcceleration::VideoToolbox => "h264_videotoolbox",
                HWAcceleration::None => "libx264",
            };
            
            args.extend([
                "-map".to_string(), "0:v:0".to_string(),
                "-map".to_string(), format!("0:{}", audio_stream),
                "-c:v".to_string(), codec.to_string(),
            ]);

            match options.acceleration {
                HWAcceleration::None => {
                    args.extend(["-crf".to_string(), "18".to_string()]);
                    if options.preserve_grain {
                        args.extend(["-tune".to_string(), "grain".to_string()]);
                    } else {
                        args.extend(["-tune".to_string(), "film".to_string()]);
                    }
                    args.extend([
                        "-preset".to_string(), "slow".to_string(),
                        "-profile:v".to_string(), "high".to_string(),
                        "-level".to_string(), "4.1".to_string(),
                        "-x264-params".to_string(), "ref=4:bframes=3:aq-mode=2".to_string(),
                    ]);
                }
                HWAcceleration::NVENC => {
                    args.extend([
                        "-preset".to_string(), "slow".to_string(),
                        "-rc".to_string(), "vbr".to_string(),
                        "-cq".to_string(), "19".to_string(),
                        "-profile:v".to_string(), "high".to_string(),
                    ]);
                }
                HWAcceleration::VideoToolbox => {
                    args.extend(["-q:v".to_string(), "60".to_string()]);
                }
                _ => {
                    args.extend(["-q:v".to_string(), "19".to_string()]);
                }
            }

            args.extend([
                "-c:a".to_string(), "ac3".to_string(),
                "-b:a".to_string(), "448k".to_string(),
            ]);
        }
        ConversionType::VideoH265 => {
            let codec = match options.acceleration {
                HWAcceleration::NVENC => "hevc_nvenc",
                HWAcceleration::QSV   => "hevc_qsv",
                HWAcceleration::AMF   => "hevc_amf",
                HWAcceleration::VAAPI => "hevc_vaapi",
                HWAcceleration::VideoToolbox => "hevc_videotoolbox",
                HWAcceleration::None => "libx265",
            };

            args.extend([
                "-map".to_string(), "0:v:0".to_string(),
                "-map".to_string(), format!("0:{}", audio_stream),
                "-c:v".to_string(), codec.to_string(),
            ]);

            match options.acceleration {
                HWAcceleration::None => {
                    args.extend(["-crf".to_string(), "20".to_string()]);
                    if options.preserve_grain {
                        args.extend(["-tune".to_string(), "grain".to_string()]);
                    }
                    args.extend([
                        "-preset".to_string(), "slow".to_string(),
                        "-x265-params".to_string(), "aq-mode=3:aq-strength=1.0:deblock=-1,-1".to_string(),
                    ]);
                }
                HWAcceleration::NVENC => {
                    args.extend([
                        "-preset".to_string(), "slow".to_string(),
                        "-rc".to_string(), "vbr".to_string(),
                        "-cq".to_string(), "21".to_string(),
                    ]);
                }
                HWAcceleration::VideoToolbox => {
                    args.extend(["-q:v".to_string(), "60".to_string()]);
                }
                _ => {
                    args.extend(["-q:v".to_string(), "21".to_string()]);
                }
            }

            args.extend([
                "-c:a".to_string(), "aac".to_string(),
                "-b:a".to_string(), "192k".to_string(),
            ]);
        }
    }

    if options.optimize_color {
        args.extend([
            "-color_primaries".to_string(), "bt709".to_string(),
            "-color_trc".to_string(), "bt709".to_string(),
            "-colorspace".to_string(), "bt709".to_string(),
        ]);
    }

    args.extend([
        "-progress".to_string(),
        "pipe:1".to_string(),
        "-nostats".to_string(),
        destination_path.to_string_lossy().to_string(),
    ]);

    let mut child = Command::new("ffmpeg")
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
