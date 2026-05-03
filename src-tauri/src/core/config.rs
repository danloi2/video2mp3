//! Configuration management for external tools.
//!
//! This module handles loading and parsing YAML configuration files that define
//! the command-line arguments and profiles for FFmpeg and yt-dlp.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Defines a specific command execution profile.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandProfile {
    /// The binary to execute (optional, defaults to system default if None).
    pub program: Option<String>,
    /// Base arguments for the command.
    pub args: Option<Vec<String>>,
    /// Additional arguments that can be appended conditionally.
    pub extra_args: Option<Vec<String>>,
}

/// Configuration schema for FFmpeg operations.
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FFmpegConfig {
    /// Default FFmpeg binary name or path.
    pub program: String,
    /// Map of named profiles (e.g., "remux_mkv", "encode_h264").
    #[serde(flatten)]
    pub profiles: HashMap<String, CommandProfile>,
}

/// Configuration schema for yt-dlp operations.
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YtDlpConfig {
    /// Default yt-dlp binary name or path.
    pub program: String,
    /// Map of named profiles (e.g., "download_video", "list_playlist").
    #[serde(flatten)]
    pub profiles: HashMap<String, CommandProfile>,
}

/// Configuration schema for FFprobe operations.
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FFprobeConfig {
    /// Default FFprobe binary name or path.
    pub program: String,
    /// Map of named profiles (e.g., "probe_duration", "probe_media_info").
    #[serde(flatten)]
    pub profiles: HashMap<String, CommandProfile>,
}

/// Internal wrapper for parsing the root of ffmpeg.yaml.
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub ffmpeg: FFmpegConfig,
}

/// Internal wrapper for parsing the root of ytdlp.yaml.
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YtDlpWrapper {
    pub ytdlp: YtDlpConfig,
}

/// Internal wrapper for parsing the root of ffprobe.yaml.
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FFprobeWrapper {
    pub ffprobe: FFprobeConfig,
}

/// Loads the FFmpeg configuration (embedded in the binary).
pub fn load_ffmpeg_config() -> Result<FFmpegConfig, String> {
    let content = include_str!("../config/ffmpeg.yaml");
    let wrapper: AppConfig = yaml_serde::from_str(content)
        .map_err(|e| format!("Could not parse embedded FFmpeg configuration: {}", e))?;
    Ok(wrapper.ffmpeg)
}

/// Loads the yt-dlp configuration (embedded in the binary).
pub fn load_ytdlp_config() -> Result<YtDlpConfig, String> {
    let content = include_str!("../config/ytdlp.yaml");
    let wrapper: YtDlpWrapper = yaml_serde::from_str(content)
        .map_err(|e| format!("Could not parse embedded yt-dlp configuration: {}", e))?;
    Ok(wrapper.ytdlp)
}

/// Loads the FFprobe configuration (embedded in the binary).
pub fn load_ffprobe_config() -> Result<FFprobeConfig, String> {
    let content = include_str!("../config/ffprobe.yaml");
    let wrapper: FFprobeWrapper = yaml_serde::from_str(content)
        .map_err(|e| format!("Could not parse embedded FFprobe configuration: {}", e))?;
    Ok(wrapper.ffprobe)
}
