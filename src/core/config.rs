use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandProfile {
    pub program: Option<String>,
    pub args: Option<Vec<String>>,
    pub extra_args: Option<Vec<String>>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FFmpegConfig {
    pub program: String,
    #[serde(flatten)]
    pub profiles: HashMap<String, CommandProfile>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YtDlpConfig {
    pub program: String,
    #[serde(flatten)]
    pub profiles: HashMap<String, CommandProfile>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub ffmpeg: FFmpegConfig,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YtDlpWrapper {
    pub ytdlp: YtDlpConfig,
}

pub fn load_ffmpeg_config() -> Result<FFmpegConfig, String> {
    let path = Path::new("src/config/ffmpeg.yaml");
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Could not read ffmpeg.yaml: {}", e))?;
    let wrapper: AppConfig = serde_yaml::from_str(&content)
        .map_err(|e| format!("Could not parse ffmpeg.yaml: {}", e))?;
    Ok(wrapper.ffmpeg)
}

pub fn load_ytdlp_config() -> Result<YtDlpConfig, String> {
    let path = Path::new("src/config/ytdlp.yaml");
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Could not read ytdlp.yaml: {}", e))?;
    let wrapper: YtDlpWrapper = serde_yaml::from_str(&content)
        .map_err(|e| format!("Could not parse ytdlp.yaml: {}", e))?;
    Ok(wrapper.ytdlp)
}
