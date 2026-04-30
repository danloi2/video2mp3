pub mod state;
pub mod theme;
pub mod helpers;
pub mod logic;
pub mod layout;

use std::sync::{Arc, atomic::AtomicBool};
use std::sync::mpsc::Receiver;
use eframe::egui;

use crate::core::{verify_ffmpeg, ConversionType, VideoOptions, get_ffmpeg_version, get_ytdlp_version, HWCapabilities, detect_hw_capabilities, HWAcceleration};
use state::{FileItem, Msg};

/// The main application state for the video2mp3 GUI.
pub struct ConvApp {
    pub(crate) ffmpeg_ok:       bool,
    pub(crate) ytdlp_ok:        bool,
    pub(crate) ffmpeg_version:  String,
    pub(crate) ytdlp_version:   String,
    pub(crate) capabilities:    HWCapabilities,
    pub(crate) files:           Vec<FileItem>,
    pub(crate) log:             Vec<(bool, String)>,
    pub(crate) is_converting:   bool,
    pub(crate) progress:        (usize, usize),
    pub(crate) current_progress: f32,
    pub(crate) cancel:          Arc<AtomicBool>,
    pub(crate) rx:              Option<Receiver<Msg>>,
    pub(crate) logo_texture:    Option<egui::TextureHandle>,
    
    // --- User Configuration ---
    pub(crate) conversion_type: ConversionType,
    pub(crate) video_options:   VideoOptions,
    pub(crate) first_time:      bool,
    pub(crate) youtube_url:     String,
    pub(crate) output_directory: Option<std::path::PathBuf>,
}

impl ConvApp {
    /// Initializes a new application state and sets up the theme and assets.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Apply the custom visual theme to the egui context
        theme::apply_theme(&cc.egui_ctx);
        
        // Load the logo icon as a texture
        let logo_texture = match image::load_from_memory(include_bytes!("../../resources/icon.png")) {
            Ok(img) => {
                let img = img.to_rgba8();
                let size = [img.width() as usize, img.height() as usize];
                let pixels = img.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                Some(cc.egui_ctx.load_texture("logo", color_image, egui::TextureOptions::LINEAR))
            }
            Err(_) => None,
        };

        use crate::core::verify_ytdlp;

        Self {
            ffmpeg_ok:       verify_ffmpeg(),
            ytdlp_ok:        verify_ytdlp(),
            ffmpeg_version:  get_ffmpeg_version(),
            ytdlp_version:   get_ytdlp_version(),
            capabilities:    detect_hw_capabilities(),
            files:           vec![],
            log:             vec![],
            is_converting:   false,
            progress:        (0, 0),
            current_progress: 0.0,
            cancel:          Arc::new(AtomicBool::new(false)),
            rx:              None,
            logo_texture,
            conversion_type: ConversionType::AudioMP3,
            video_options:   VideoOptions { preserve_grain: false, optimize_color: true, acceleration: HWAcceleration::None },
            first_time:      true,
            youtube_url:     "".to_string(),
            output_directory: None,
        }
    }
}

impl eframe::App for ConvApp {
    /// Main UI entry point for the eframe application.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // One-time window configuration on startup
        if self.first_time {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            self.first_time = false;
        }

        // Divide the UI into panels
        let ctx = ui.ctx().clone();
        egui::CentralPanel::default().show_inside(ui, |ui| {
            layout::render_ui(self, ui, &ctx);
        });
    }
}
