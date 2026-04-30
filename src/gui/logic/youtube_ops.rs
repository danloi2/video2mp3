use std::sync::mpsc;
use std::thread;
use eframe::egui;
use crate::gui::ConvApp;
use crate::gui::state::Msg;
use crate::core::get_playlist_videos;

impl ConvApp {
    /// Dispatches a background thread to scan a YouTube URL for videos.
    /// 
    /// If the URL is a playlist, multiple videos will be added to the queue.
    /// This function switches the app to a "busy" state to prevent concurrent scans.
    pub(crate) fn add_from_youtube(&mut self, ctx: &egui::Context) {
        let url = self.youtube_url.trim().to_string();
        if url.is_empty() { return; }

        // Enter a temporary "loading" state
        self.is_converting = true;
        
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        let ctx_clone = ctx.clone();
        let tx_clone = tx.clone();
        
        // Spawn a background worker to avoid blocking the UI during network I/O
        thread::spawn(move || {
            // Probe YouTube metadata (might find 1 or many videos)
            get_playlist_videos(&url, |u, t| {
                let _ = tx_clone.send(Msg::AddYouTubeItems(vec![(u, t)]));
            });
            
            // Signal that metadata collection has finished
            let _ = tx.send(Msg::PlaylistLoaded); 
            ctx_clone.request_repaint();
        });
        
        // Reset the input field for the next URL
        self.youtube_url.clear();
    }
}
