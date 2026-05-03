/// Defines the target format for the conversion process.
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum ConversionType {
    /// Extract and encode audio as high-quality MP3.
    AudioMP3,
    /// Remux or convert video to MKV container.
    VideoMKV,
    /// Transcode video to H.264 (AVC) format.
    VideoH264,
    /// Transcode video to H.265 (HEVC) format.
    VideoH265,
}

/// Supported hardware acceleration backends for video encoding.
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum HWAcceleration {
    /// Software-only encoding (highest quality, slowest speed).
    None,
    /// NVIDIA GPU hardware acceleration.
    NVENC,
    /// Intel QuickSync hardware acceleration.
    QSV,
    /// AMD Advanced Media Framework.
    AMF,
    /// Linux generic Video Acceleration API.
    VAAPI,
    /// Apple macOS VideoToolbox acceleration.
    VideoToolbox,
}

/// Fine-tuning parameters for video transcoding.
#[derive(Clone, Debug, Copy)]
pub struct VideoOptions {
    /// Enable "grain" tuning (specifically for x264/x265 software encoders).
    pub preserve_grain: bool,
    /// Enforce BT.709 color space parameters.
    pub optimize_color: bool,
    /// Selected hardware acceleration backend.
    pub acceleration:   HWAcceleration,
}

/// Metadata representation of an audio track found within a container.
#[derive(Clone, Debug)]
pub struct AudioTrack {
    /// Zero-based index of the stream within the file.
    pub stream_index: u64,
    /// Codec name (e.g., "ac3", "aac", "dts").
    pub codec: String,
    /// Language code if available (e.g., "eng", "spa").
    pub language: String,
}

/// Basic media information extracted during probing.
#[derive(Clone, Debug)]
pub struct MediaInfo {
    /// Container format name (e.g., "matroska", "mov").
    pub container: String,
    /// Video codec of the primary video stream.
    pub v_codec:    Option<String>,
    /// Audio codec of the first compatible audio stream.
    pub a_codec:    Option<String>,
}

/// Messages emitted during long-running tasks to update the UI/CLI.
#[derive(Clone, Debug)]
pub enum ProgressUpdate {
    /// Completion percentage (0.0 to 1.0).
    Ratio(f32),
    /// Current item index and total items in a playlist.
    Playlist(usize, usize), // (current, total)
    /// Phase change signal: "downloading" or "converting".
    Phase(&'static str),
}
