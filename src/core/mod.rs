//! Core module for video2mp3.
//! 
//! This module orchestrates the media processing logic, including file probing,
//! hardware capability detection, and the actual conversion/download processes.
//! It serves as the primary bridge between the UI and external binaries (FFmpeg/yt-dlp).

pub mod types;
pub mod probe;
pub mod convert_file;
pub mod convert_youtube;

// Re-export core types for easier access from other modules
pub use types::{AudioTrack, ConversionType, VideoOptions, HWAcceleration, ProgressUpdate, MediaInfo};

// Re-export probe utilities for environment validation and hardware detection
pub use probe::{
    verify_ffmpeg, 
    verify_ytdlp, 
    select_default_track, 
    get_audio_tracks, 
    get_ffmpeg_version, 
    get_ytdlp_version, 
    HWCapabilities, 
    detect_hw_capabilities, 
    get_media_info
};

// Re-export conversion and download operations
pub use convert_file::convert_file;
pub use convert_youtube::{download_youtube, get_playlist_videos};
