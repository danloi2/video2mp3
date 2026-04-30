use std::path::PathBuf;
use crate::gui::ConvApp;
use crate::gui::state::{FileItem, Status, Msg};

impl ConvApp {
    /// Consumes all pending messages from the worker thread channel and updates the app state.
    /// 
    /// This function acts as the state synchronization bridge, transforming async events 
    /// into UI updates (progress bars, status icons, logs).
    pub(crate) fn process_messages(&mut self) {
        let messages: Vec<Msg> = {
            let Some(rx) = &self.rx else { return };
            let mut buf = vec![];
            // Non-blocking drain of the channel
            while let Ok(m) = rx.try_recv() { buf.push(m); }
            buf
        };

        for msg in messages {
            match msg {
                Msg::Starting(idx) => {
                    if let Some(a) = self.files.get_mut(idx) {
                        let name = a.path.file_name().unwrap_or_default().to_string_lossy().to_string();
                        // Determine starting state based on source type
                        if a.youtube_url.is_some() {
                            a.status = Status::Downloading;
                            self.log.push((true, format!("⬇  Downloading: {}", name)));
                        } else {
                            a.status = Status::Converting;
                            self.log.push((true, format!("⚙  Converting: {}", name)));
                        }
                    }
                    self.current_progress = 0.0;
                }
                
                Msg::Progress(_idx, ratio) => {
                    self.current_progress = ratio;
                }
                
                Msg::PlaylistProgress(_idx, actual, total) => {
                    // Update global progress counters for multi-item YouTube playlists
                    self.progress.1 = total;
                    self.progress.0 = actual - 1;
                    self.current_progress = 0.0;
                }
                
                Msg::Result(idx, ok, text) => {
                    if let Some(a) = self.files.get_mut(idx) {
                        a.status = if ok { Status::Ready } else { Status::Error(text.clone()) };
                    }
                    self.log.push((ok, text));
                    self.progress.0  += 1;
                    self.current_progress = 0.0;
                }
                
                Msg::AddYouTubeItems(videos) => {
                    // Batch addition of metadata-discovered YouTube videos
                    for (url, title) in videos {
                        self.files.push(FileItem {
                            path: PathBuf::from(title),
                            status: Status::Pending,
                            selected: true,
                            tracks: vec![],
                            selected_track: 0,
                            info: None,
                            youtube_url: Some(url),
                        });
                    }
                }
                
                Msg::Finished => {
                    // Cleanup after entire batch is done
                    self.is_converting    = false;
                    self.current_progress = 0.0;
                    self.rx              = None;
                    self.log.push((true, "🎉 Batch conversion completed!".into()));
                }
                
                Msg::PlaylistLoaded => {
                    // End of metadata scanning phase
                    self.is_converting = false;
                    self.rx           = None;
                }
                
                Msg::LogLine(ok, text) => {
                    self.log.push((ok, text));
                }
                
                Msg::YouTubePhase(idx, phase) => {
                    // Handle sub-phase transitions reported by yt-dlp/FFmpeg
                    match phase {
                        "downloading" => {
                            if let Some(a) = self.files.get_mut(idx) {
                                a.status = Status::Downloading;
                            }
                            self.current_progress = 0.0;
                        }
                        "converting" => {
                            if let Some(a) = self.files.get_mut(idx) {
                                if a.status != Status::Converting {
                                    a.status = Status::Converting;
                                    self.log.push((true, "⚙  Processing audio...".into()));
                                }
                            }
                            // Post-processing is usually fast but indeterminate, set to 100% 
                            // to trigger the pulsing bar animation.
                            self.current_progress = 1.0;
                        }
                        _ => {}
                    }
                }
                
                Msg::ChangeStatus(idx, new_status) => {
                    if let Some(a) = self.files.get_mut(idx) {
                        a.status = new_status;
                    }
                }
            }
        }
    }
}
