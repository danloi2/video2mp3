//! Business logic layer for the video2mp3 GUI.
//! 
//! This module partitions the application's interactive behavior into specialized domains:
//! - File operations (Adding/Picking files).
//! - YouTube tasks (URL scanning).
//! - Conversion engine (Thread management).
//! - Message handling (UI state synchronization).

pub mod file_ops;
pub mod youtube_ops;
pub mod conversion_ops;
pub mod message_handler;
