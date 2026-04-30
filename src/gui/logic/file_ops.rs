use eframe::egui;
use rfd::FileDialog;
use crate::gui::ConvApp;
use crate::gui::state::{FileItem, Status};
use crate::core::{get_audio_tracks, select_default_track, get_media_info};

impl ConvApp {
    /// Opens a folder picker to override the global output directory.
    pub(crate) fn select_output_directory(&mut self) {
        if let Some(path) = FileDialog::new().pick_folder() {
            self.output_directory = Some(path);
        }
    }

    /// Opens a file picker to add one or more video files to the queue.
    pub(crate) fn add_files(&mut self) {
        let Some(paths) = FileDialog::new()
            .add_filter("Supported Videos", &["mkv", "mp4", "avi", "MKV", "MP4", "AVI"])
            .pick_files()
        else {
            return;
        };

        let mut new_count = 0usize;
        for path in paths {
            if self.register_file_if_valid(path) {
                new_count += 1;
            }
        }
        if new_count > 0 {
            self.log.push((true, format!("📂 {} file(s) added", new_count)));
        }
    }

    /// Opens a folder picker and adds all supported video files found within it.
    pub(crate) fn add_folder(&mut self) {
        let Some(base_path) = FileDialog::new().pick_folder() else {
            return;
        };

        let mut new_count = 0usize;
        if let Ok(entries) = std::fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext = ext.to_string_lossy().to_lowercase();
                        if matches!(ext.as_str(), "mkv" | "mp4" | "avi") {
                            if self.register_file_if_valid(path) {
                                new_count += 1;
                            }
                        }
                    }
                }
            }
        }

        if new_count > 0 {
            self.log.push((true, format!("📂 Folder processed: {} new file(s)", new_count)));
        } else {
            self.log.push((false, "⚠ No compatible video files found in the selected folder.".into()));
        }
    }

    /// Internal helper to validate a file path, probe its metadata, and add it to the state.
    /// 
    /// Prevents duplicate entries and automatically selects the default audio track.
    fn register_file_if_valid(&mut self, path: std::path::PathBuf) -> bool {
        if !self.files.iter().any(|a| a.path == path) {
            let tracks         = get_audio_tracks(&path.to_string_lossy());
            let selected_track = select_default_track(&tracks);
            let info           = get_media_info(&path.to_string_lossy());
            self.files.push(FileItem {
                path,
                status: Status::Pending,
                selected: true,
                tracks,
                selected_track,
                info,
                youtube_url: None,
            });
            return true;
        }
        false
    }

    /// Detects and processes files dragged and dropped into the application window.
    pub(crate) fn handle_drop(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            for f in &i.raw.dropped_files {
                if let Some(path) = &f.path {
                    if let Some(ext) = path.extension() {
                        let ext = ext.to_string_lossy().to_lowercase();
                        // Filter for supported extensions
                        if (ext == "mkv" || ext == "mp4" || ext == "avi")
                            && !self.files.iter().any(|a| &a.path == path)
                        {
                            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            let tracks         = get_audio_tracks(&path.to_string_lossy());
                            let selected_track = select_default_track(&tracks);
                            self.files.push(FileItem {
                                path: path.clone(),
                                status: Status::Pending,
                                selected: true,
                                tracks,
                                selected_track,
                                info: get_media_info(&path.to_string_lossy()),
                                youtube_url: None,
                            });
                            self.log.push((true, format!("↓ Added via drop: {}", name)));
                        }
                    }
                }
            }
        });
    }
}
