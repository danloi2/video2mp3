use std::path::PathBuf;
use eframe::egui::Color32;
use crate::core::{AudioTrack, MediaInfo};

/// Represents the current lifecycle stage of an item in the queue.
#[derive(Clone, PartialEq, Debug)]
pub enum Status {
    /// Waiting to be processed.
    Pending,
    /// Actively downloading from an external source (yt-dlp phase).
    Downloading,
    /// Actively transcoding (FFmpeg phase).
    Converting,
    /// Successfully processed and ready.
    Ready,
    /// Processing failed; contains the error message.
    Error(String),
}

impl Status {
    /// Returns a symbolic emoji representing the state.
    pub fn icon(&self) -> &'static str {
        match self {
            Status::Pending     => "⏳",
            Status::Downloading => "⬇",
            Status::Converting  => "⚙",
            Status::Ready       => "✅",
            Status::Error(_)    => "❌",
        }
    }
    
    /// Returns the color associated with the state for UI highlighting.
    pub fn color(&self) -> Color32 {
        match self {
            Status::Pending     => Color32::from_rgb(160, 150, 50),
            Status::Downloading => Color32::from_rgb(220, 140, 50),
            Status::Converting  => Color32::from_rgb(60, 100, 220),
            Status::Ready       => Color32::from_rgb(40, 160, 80),
            Status::Error(_)    => Color32::from_rgb(200, 50, 50),
        }
    }
}

/// A structure representing a single video file or YouTube stream in the UI list.
pub struct FileItem {
    /// Source filesystem path or a temporary path for YouTube downloads.
    pub path:           PathBuf,
    /// Current processing status.
    pub status:         Status,
    /// Whether the user has selected this item for the next conversion batch.
    pub selected:       bool,
    /// List of audio tracks detected during probing.
    pub tracks:         Vec<AudioTrack>,
    /// Index of the audio track selected for extraction.
    pub selected_track: usize,
    /// Extended media information (codecs, container).
    pub info:           Option<MediaInfo>,
    /// Original YouTube URL if the source is not a local file.
    pub youtube_url:    Option<String>,
}

/// Inter-thread messages sent from worker threads to the main UI loop.
pub enum Msg {
    /// Start processing item at the given index.
    Starting(usize),
    /// Update progress ratio (0.0 to 1.0) for a specific item.
    Progress(usize, f32),
    /// Update playlist progress: (item_index, current_item, total_items).
    PlaylistProgress(usize, usize, usize),
    /// Final result of an operation: (index, is_success, message).
    Result(usize, bool, String),
    /// Batch add new YouTube items found during playlist scanning.
    AddYouTubeItems(Vec<(String, String)>),
    /// Signal that the entire conversion batch is complete.
    Finished,
    /// Signal that a YouTube playlist scan has finished.
    PlaylistLoaded,
    /// Append a line to the global log without changing item states.
    LogLine(bool, String),
    /// Notify about internal yt-dlp phase transitions.
    YouTubePhase(usize, &'static str),
    /// Forcefully change the status of a specific item.
    ChangeStatus(usize, Status),
}
