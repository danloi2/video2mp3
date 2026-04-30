use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;
use eframe::egui;
use crate::gui::ConvApp;
use crate::gui::state::{Status, Msg};
use crate::core::{convert_file, download_youtube, ConversionType};

impl ConvApp {
    /// Orchestrates the start of a conversion batch.
    /// 
    /// This function:
    /// 1. Filters the item list for selected "Pending" items.
    /// 2. Resolves destination paths (and handles overwrite confirmation dialogs).
    /// 3. Spawns a background worker thread to process items sequentially.
    /// 4. Manages the shared `cancel` atomic flag and the message channel.
    pub(crate) fn start_conversion(&mut self, ctx: &egui::Context) {
        let mut pending_items = Vec::new();

        // --- Phase 1: Pre-processing and Path Resolution ---
        for (i, a) in self.files.iter().enumerate() {
            if a.selected && a.status == Status::Pending {
                let stream = a.tracks.get(a.selected_track)
                    .map(|p| p.stream_index)
                    .unwrap_or(0);
                
                let ext = match self.conversion_type {
                    ConversionType::AudioMP3 => "mp3",
                    _ => "mkv",
                };

                let stem = a.path.file_stem().unwrap_or_default().to_string_lossy();

                // Logic: YouTube downloads go to 'Downloads' folder by default.
                // Local conversions stay in the source folder by default.
                let is_youtube = a.youtube_url.is_some();
                let dest_dir = self.output_directory.clone().unwrap_or_else(|| {
                    if is_youtube {
                        dirs::download_dir().unwrap_or_else(|| Path::new(".").to_path_buf())
                    } else {
                        a.path.parent()
                            .filter(|p| !p.as_os_str().is_empty())
                            .unwrap_or(Path::new("."))
                            .to_path_buf()
                    }
                });
                let dest_path = dest_dir.join(format!("{}.{}", stem, ext));
                
                let mut overwrite = false;
                if dest_path.exists() {
                    let name = dest_path.file_name().unwrap_or_default().to_string_lossy();
                    let msg = format!("File '{}' already exists.\nDo you want to overwrite it?", name);
                    let res = rfd::MessageDialog::new()
                        .set_title("Overwrite File")
                        .set_description(&msg)
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .show();
                    
                    if res == rfd::MessageDialogResult::Yes {
                        overwrite = true;
                    } else {
                        continue; // Skip this file if user says No
                    }
                }
                pending_items.push((i, a.path.clone(), dest_path, stream, overwrite, a.youtube_url.clone()));
            }
        }

        if pending_items.is_empty() {
            self.log.push((false, "⚠ No pending selected files to process.".into()));
            return;
        }

        // --- Phase 2: State Initialization ---
        let total = pending_items.len();
        self.is_converting    = true;
        self.progress         = (0, total);
        self.current_progress = 0.0;
        self.cancel.store(false, Ordering::Relaxed);

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        let ctx_clone = ctx.clone();
        let cancel_flag = self.cancel.clone();
        let conv_type = self.conversion_type;
        let options = self.video_options;

        // --- Phase 3: Background Worker Execution ---
        thread::spawn(move || {
            for (idx, source_path, dest_path, stream, overwrite, youtube_url) in pending_items {
                let _ = tx.send(Msg::Starting(idx));
                ctx_clone.request_repaint();

                let tx_loop  = tx.clone();
                let ctx_loop = ctx_clone.clone();
                let cancel_clone = cancel_flag.clone();
                
                let (ok, msg) = if let Some(url) = youtube_url {
                    // --- YouTube Pipeline ---
                    let audio_only = conv_type == ConversionType::AudioMP3;
                    let dest_dir = dest_path.parent().unwrap_or(Path::new("."));

                    let download_res = download_youtube(&url, dest_dir, audio_only, cancel_clone.clone(), |update| {
                        match update {
                            crate::core::ProgressUpdate::Ratio(ratio) => {
                                // If transcoding follows download, each takes 50% of the bar
                                let r = if audio_only { ratio } else { ratio * 0.5 };
                                let _ = tx_loop.send(Msg::Progress(idx, r));
                            }
                            crate::core::ProgressUpdate::Playlist(cur, tot) => {
                                let _ = tx_loop.send(Msg::PlaylistProgress(idx, cur, tot));
                            }
                            crate::core::ProgressUpdate::Phase(phase) => {
                                if phase == "converting" && !audio_only {
                                    // Managed below manually for H264/H265
                                } else {
                                    let _ = tx_loop.send(Msg::YouTubePhase(idx, phase));
                                }
                            }
                        }
                        ctx_loop.request_repaint();
                    });

                    match download_res {
                        Ok(downloaded_path) => {
                            if audio_only {
                                (true, format!("✅ YouTube → {}", downloaded_path.file_name().unwrap_or_default().to_string_lossy()))
                            } else {
                                // For video formats, we follow the download with a custom transcode pass
                                let _ = tx_loop.send(Msg::ChangeStatus(idx, Status::Converting));
                                let _ = tx_loop.send(Msg::LogLine(true, "⚙  Transcoding video...".into()));
                                let _ = tx_loop.send(Msg::Progress(idx, 0.5));
                                ctx_loop.request_repaint();
                                
                                match convert_file(
                                    &downloaded_path,
                                    Some(&dest_path),
                                    0,
                                    true,
                                    conv_type,
                                    options,
                                    cancel_clone,
                                    |update| {
                                        if let crate::core::ProgressUpdate::Ratio(ratio) = update {
                                            let _ = tx_loop.send(Msg::Progress(idx, 0.5 + (ratio * 0.5)));
                                            ctx_loop.request_repaint();
                                        }
                                    }
                                ) {
                                    Ok(m) => (true, format!("✅ YouTube → {}", m)),
                                    Err(e) => (false, format!("❌ Post-conversion error: {}", e)),
                                }
                            }
                        }
                        Err(e) => (false, format!("❌ Download error: {}", e)),
                    }
                } else {
                    // --- Local File Pipeline ---
                    match convert_file(
                        &source_path,
                        Some(&dest_path),
                        stream,
                        overwrite,
                        conv_type,
                        options,
                        cancel_clone,
                        move |update| {
                            if let crate::core::ProgressUpdate::Ratio(ratio) = update {
                                let _ = tx_loop.send(Msg::Progress(idx, ratio));
                                ctx_loop.request_repaint();
                            }
                        },
                    ) {
                        Ok(m)  => (true,  m),
                        Err(e) => (false, e),
                    }
                };

                // Report the final result for this item
                let _ = tx.send(Msg::Result(idx, ok, msg));
                ctx_clone.request_repaint();
            }
            
            // Signal global completion
            let _ = tx.send(Msg::Finished);
            ctx_clone.request_repaint();
        });
    }
}
