//! Core module for video2mp3.
//! 
//! This module orchestrates the media processing logic, including file probing,
//! hardware capability detection, and the actual conversion/download processes.
//! It serves as the primary bridge between the UI and external binaries (FFmpeg/yt-dlp).

pub mod types;
pub mod probe;
pub mod convert_file;
pub mod convert_youtube;
pub mod config;

pub use types::{ConversionType, ProgressUpdate};
pub use convert_file::convert_file;
