//! YouTube downloading and processing.
//!
//! This module manages interaction with `yt-dlp` to fetch metadata, list
//! playlists, and download media directly from YouTube URLs.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use super::config::load_ytdlp_config;

/// Fetches the filename of a YouTube video without downloading it.
pub fn get_youtube_name(url: &str) -> Option<String> {
    let config = load_ytdlp_config().ok()?;
    let profile = config.profiles.get("get_filename")?;
    let mut args = profile.args.as_ref()?.clone();

    for arg in args.iter_mut() {
        *arg = arg.replace("{url}", url);
    }

    let output = Command::new(&config.program)
        .args(&args)
        .output()
        .ok()?;
    
    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() && !name.contains("NA") {
            return Some(name);
        }
    }
    None
}

/// Scans a YouTube URL for multiple videos if it's a playlist.
pub fn get_playlist_videos<F>(url: &str, mut on_video: F)
where
    F: FnMut(String, String),
{
    let config = match load_ytdlp_config() {
        Ok(c) => c,
        Err(_) => return,
    };
    let profile = match config.profiles.get("list_playlist") {
        Some(p) => p,
        None => return,
    };
    let mut args = match &profile.args {
        Some(a) => a.clone(),
        None => return,
    };

    for arg in args.iter_mut() {
        *arg = arg.replace("{url}", url);
    }

    let child = Command::new(&config.program)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok();

    let mut found_count = 0usize;

    if let Some(mut c) = child {
        if let Some(stdout) = c.stdout.take() {
            use std::io::BufRead;
            for line in std::io::BufReader::new(stdout).lines().map_while(Result::ok) {
                let parts: Vec<&str> = line.splitn(2, '\t').collect();
                if parts.len() == 2 {
                    let video_url = parts[0].trim();
                    let title     = parts[1].trim();
                    if !video_url.is_empty()
                        && video_url != "NA"
                        && video_url.starts_with("http")
                        && !title.is_empty()
                        && title != "NA"
                    {
                        on_video(video_url.to_string(), title.to_string());
                        found_count += 1;
                    }
                }
            }
        }
        let _ = c.wait();
    }

    if found_count == 0 {
        if let Some(title) = get_youtube_name(url) {
            on_video(url.to_string(), title);
        }
    }
}

/// Downloads a video from YouTube using `yt-dlp`.
pub fn download_youtube<F>(
    url: &str,
    destination: &Path,
    audio_only: bool,
    cancel: Arc<AtomicBool>,
    on_progress: F,
) -> Result<PathBuf, String>
where
    F: Fn(crate::core::ProgressUpdate),
{
    let config = load_ytdlp_config()?;
    let profile_name = if audio_only { "download_audio_mp3" } else { "download_video" };
    let profile = config.profiles.get(profile_name)
        .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;
    let mut args = profile.args.as_ref()
        .ok_or_else(|| format!("Profile '{}' has no arguments", profile_name))?
        .clone();

    let template = "%(title)s.%(ext)s";
    let full_template = destination.join(template);

    for arg in args.iter_mut() {
        *arg = arg.replace("{url}", url)
                  .replace("{output_template}", &full_template.to_string_lossy());
    }

    let mut child = Command::new(&config.program)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Could not launch yt-dlp: {}", e))?;

    let mut downloaded_paths: Vec<PathBuf> = Vec::new();

    on_progress(crate::core::ProgressUpdate::Phase("downloading"));

    if let Some(stdout) = child.stdout.take() {
        let mut cancelled = false;
        use std::io::BufRead;
        for line in std::io::BufReader::new(stdout).lines().map_while(Result::ok) {
            if cancel.load(Ordering::Relaxed) {
                child.kill().ok();
                cancelled = true;
                break;
            }

            let trimmed = line.trim();
            if !trimmed.starts_with('[') && !trimmed.is_empty() {
                let path = PathBuf::from(trimmed);
                if path.is_absolute() || path.exists() {
                    downloaded_paths.push(path);
                    continue;
                }
            }

            if line.contains("[ExtractAudio]")
                || line.contains("[ffmpeg]")
                || line.contains("[Merger]")
                || line.contains("[VideoConvertor]")
            {
                on_progress(crate::core::ProgressUpdate::Phase("converting"));
                continue;
            }

            if line.contains("Downloading item") && line.contains("of") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let (Some(pos_item), Some(pos_of)) = (parts.iter().position(|&s| s == "item"), parts.iter().position(|&s| s == "of")) {
                    if let (Ok(cur), Ok(tot)) = (parts[pos_item+1].parse::<usize>(), parts[pos_of+1].parse::<usize>()) {
                        on_progress(crate::core::ProgressUpdate::Playlist(cur, tot));
                    }
                }
            }

            if line.contains("[download]") && line.contains('%') {
                if let Some(pos) = line.find('%') {
                    let start = line[..pos].rfind(' ').unwrap_or(0);
                    if let Ok(p) = line[start..pos].trim().parse::<f32>() {
                        on_progress(crate::core::ProgressUpdate::Ratio(p / 100.0));
                    }
                }
            }
        }
        if cancelled {
            let _ = child.wait();
            return Err("⏹ YouTube download cancelled".to_string());
        }
    }

    let mut stderr_content = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        use std::io::Read;
        let _ = stderr.read_to_string(&mut stderr_content);
    }

    let status = child.wait().map_err(|e| e.to_string())?;

    if status.success() {
        if let Some(path) = downloaded_paths.into_iter().last() {
            Ok(path)
        } else {
            Ok(destination.to_path_buf())
        }
    } else {
        let err_msg = if stderr_content.is_empty() {
            "Unknown error in yt-dlp".to_string()
        } else {
            stderr_content.lines().last().unwrap_or("Error in yt-dlp").to_string()
        };
        Err(format!("❌ {}", err_msg))
    }
}
