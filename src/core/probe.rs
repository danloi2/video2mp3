use super::types::AudioTrack;
use serde_json::Value;
use std::process::Command;

/// Represents the hardware acceleration features supported by the host system.
#[derive(Debug, Clone, Default)]
pub struct HWCapabilities {
    pub nvenc: bool,
    pub qsv:   bool,
    pub amf:   bool,
    pub vaapi: bool,
    pub vtb:   bool,
}

/// Validates that both `ffmpeg` and `ffprobe` are installed and available in the system PATH.
pub fn verify_ffmpeg() -> bool {
    let ffmpeg_ok = super::config::load_ffmpeg_config()
        .map(|c| Command::new(c.program).arg("-version").output().is_ok())
        .unwrap_or(false);
    let ffprobe_ok = super::config::load_ffprobe_config()
        .map(|c| Command::new(c.program).arg("-version").output().is_ok())
        .unwrap_or(false);
    ffmpeg_ok && ffprobe_ok
}

/// Validates that `yt-dlp` is installed and available in the system PATH.
pub fn verify_ytdlp() -> bool {
    super::config::load_ytdlp_config()
        .map(|c| Command::new(c.program).arg("--version").output().is_ok())
        .unwrap_or(false)
}

/// Extracts the short version string from the `ffmpeg -version` output.
pub fn get_ffmpeg_version() -> String {
    if let Ok(config) = super::config::load_ffmpeg_config() {
        if let Some(profile) = config.profiles.get("check_version") {
            if let Ok(output) = Command::new(&config.program).args(profile.args.as_ref().unwrap()).output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.lines().next() {
                    return line.replace("ffmpeg version ", "")
                               .split_whitespace()
                               .next()
                               .unwrap_or("?")
                               .to_string();
                }
            }
        }
    }
    "unknown".to_string()
}

/// Retrieves the version string reported by `yt-dlp --version`.
pub fn get_ytdlp_version() -> String {
    if let Ok(config) = super::config::load_ytdlp_config() {
        if let Some(profile) = config.profiles.get("check_version") {
            if let Ok(output) = Command::new(&config.program).args(profile.args.as_ref().unwrap()).output() {
                return String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }
    }
    "unknown".to_string()
}

/// Detects hardware acceleration support by probing both available encoders and hardware devices.
pub fn detect_hw_capabilities() -> HWCapabilities {
    let mut caps = HWCapabilities::default();
    
    // 1. Software Probing: Check which encoders are compiled into the FFmpeg binary.
    let b_vaapi = probe_encoder("vaapi");
    let b_nvenc = probe_encoder("nvenc");
    let b_qsv   = probe_encoder("_qsv");
    let b_amf   = probe_encoder("_amf");
    let b_vtb   = probe_encoder("videotoolbox");

    // 2. Hardware Verification: Attempt to initialize the hardware drivers.
    // If initialization succeeds, the hardware is actually present and usable.
    if b_nvenc && probe_device("cuda")  { caps.nvenc = true; }
    if b_qsv   && probe_device("qsv")   { caps.qsv   = true; }
    if b_vaapi && probe_device("vaapi") { caps.vaapi = true; }
    
    // VideoToolbox on macOS usually doesn't need device init, check via hwaccels list instead.
    if b_vtb && probe_hwaccel("videotoolbox") { caps.vtb = true; }

    // AMF on Linux is rare; if encoder exists and other HW is present, we assume support for now.
    if b_amf && (caps.vaapi || caps.nvenc) { caps.amf = true; }

    caps
}

/// Checks if a specific hardware acceleration type is listed in `ffmpeg -hwaccels`.
fn probe_hwaccel(accel_type: &str) -> bool {
    if let Ok(config) = super::config::load_ffmpeg_config() {
        if let Some(profile) = config.profiles.get("list_hwaccels") {
            if let Ok(output) = Command::new(&config.program).args(profile.args.as_ref().unwrap()).output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return stdout.contains(accel_type);
            }
        }
    }
    false
}

/// Checks if a specific encoder string is present in the `ffmpeg -encoders` list.
fn probe_encoder(encoder_name: &str) -> bool {
    if let Ok(config) = super::config::load_ffmpeg_config() {
        if let Some(profile) = config.profiles.get("list_encoders") {
            if let Ok(output) = Command::new(&config.program).args(profile.args.as_ref().unwrap()).output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return stdout.to_lowercase().contains(encoder_name);
            }
        }
    }
    false
}

/// Attempts to initialize a hardware device via FFmpeg.
/// 
/// This is the most reliable way to check if a specific GPU/driver is actually functional.
fn probe_device(device_type: &str) -> bool {
    if let Ok(config) = super::config::load_ffmpeg_config() {
        if let Some(profile) = config.profiles.get("init_device") {
            let mut args = profile.args.as_ref().unwrap().clone();
            for arg in args.iter_mut() {
                *arg = arg.replace("{device_type}", device_type);
            }
            if let Ok(output) = Command::new(&config.program).args(&args).output() {
                return output.status.success();
            }
        }
    }
    false
}

/// Heuristically selects the best default audio track (prefers Spanish/ES).
pub fn select_default_track(tracks: &[AudioTrack]) -> usize {
    tracks
        .iter()
        .position(|t| {
            let lang = t.language.to_lowercase();
            lang == "spa" || lang == "es"
        })
        .unwrap_or(0)
}

/// Helper function to extract audio stream metadata using ffprobe in JSON format.
fn get_audio_streams_json(file_path: &str) -> Vec<Value> {
    let Ok(config) = super::config::load_ffprobe_config() else {
        return vec![];
    };
    let Some(profile) = config.profiles.get("probe_audio_streams") else {
        return vec![];
    };
    let mut args = match &profile.args {
        Some(a) => a.clone(),
        None => return vec![],
    };

    for arg in args.iter_mut() {
        *arg = arg.replace("{input}", file_path);
    }

    let Ok(output) = Command::new(&config.program)
        .args(&args)
        .output()
    else {
        return vec![];
    };

    let Ok(json): Result<Value, _> = serde_json::from_slice(&output.stdout) else {
        return vec![];
    };

    match json["streams"].as_array() {
        Some(arr) => arr
            .iter()
            .map(|s| {
                let mut s = s.clone();
                // Ensure the tags object exists to avoid runtime errors during access
                if s.get("tags").is_none() {
                    s["tags"] = Value::Object(serde_json::Map::new());
                }
                if s["tags"].get("language").is_none() {
                    s["tags"]["language"] = Value::String("unknown".to_string());
                }
                s
            })
            .collect(),
        None => vec![],
    }
}

/// Returns a list of all audio tracks found within the specified media file.
pub fn get_audio_tracks(file_path: &str) -> Vec<AudioTrack> {
    get_audio_streams_json(file_path)
        .into_iter()
        .map(|s| AudioTrack {
            stream_index: s["index"].as_u64().unwrap_or(0),
            codec: s["codec_name"].as_str().unwrap_or("?").to_string(),
            language: s["tags"]["language"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
        })
        .collect()
}

/// Retrieves the duration of the media file in seconds using ffprobe.
pub(crate) fn get_duration_seconds(file_path: &str) -> Option<f64> {
    let config = super::config::load_ffprobe_config().ok()?;
    let profile = config.profiles.get("probe_duration")?;
    let mut args = profile.args.as_ref()?.clone();

    for arg in args.iter_mut() {
        *arg = arg.replace("{input}", file_path);
    }

    let output = Command::new(&config.program)
        .args(&args)
        .output()
        .ok()?;

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .ok()
}

/// Collects primary media info (container and first video/audio codecs) from a file.
pub fn get_media_info(file_path: &str) -> Option<super::types::MediaInfo> {
    let config = super::config::load_ffprobe_config().ok()?;
    let profile = config.profiles.get("probe_media_info")?;
    let mut args = profile.args.as_ref()?.clone();

    for arg in args.iter_mut() {
        *arg = arg.replace("{input}", file_path);
    }

    let output = Command::new(&config.program)
        .args(&args)
        .output()
        .ok()?;

    let json: Value = serde_json::from_slice(&output.stdout).ok()?;
    
    let container = json["format"]["format_name"]
        .as_str()
        .unwrap_or("?")
        .to_string();

    let mut v_codec = None;
    let mut a_codec = None;

    if let Some(streams) = json["streams"].as_array() {
        for s in streams {
            let codec_type = s["codec_type"].as_str().unwrap_or("");
            let codec_name = s["codec_name"].as_str().map(|c| c.to_string());
            
            if codec_type == "video" && v_codec.is_none() {
                v_codec = codec_name;
            } else if codec_type == "audio" && a_codec.is_none() {
                a_codec = codec_name;
            }
        }
    }

    Some(super::types::MediaInfo {
        container,
        v_codec,
        a_codec,
    })
}
