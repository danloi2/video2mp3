//! Application header and status display.
//!
//! This module renders the top panel of the GUI, which includes the branding,
//! application versioning, and status indicators for external dependencies.

use eframe::egui::{self, Color32, Margin, RichText};
use crate::gui::ConvApp;
use crate::gui::helpers::cr;
use super::widgets::render_version_badge;

/// Renders the top header panel of the application.
///
/// Displays the application logo, name, version, and the status of 
/// essential system tools (FFmpeg and yt-dlp).
pub fn render(app: &mut ConvApp, ui: &mut egui::Ui) {
    egui::Panel::top("panel_top")
        .show_inside(ui, |ui| {
            egui::Frame::new()
                .inner_margin(Margin::same(12_i8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // --- SECTION: Branding ---
                        if let Some(texture) = &app.logo_texture {
                            ui.image((texture.id(), egui::vec2(32.0, 32.0)));
                        }
                        
                        ui.vertical(|ui| {
                            ui.label(RichText::new("video2mp3").strong().size(22.0));
                            ui.label(RichText::new("Professional Media Converter").size(12.0).color(Color32::from_rgb(120, 130, 150)));
                        });

                        ui.add_space(8.0);
                        
                        // --- SECTION: Versioning ---
                        let version = env!("CARGO_PKG_VERSION");
                        render_version_badge(ui, version);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // --- SECTION: System Status ---
                            status_indicator(ui, "FFmpeg", app.ffmpeg_ok, &app.ffmpeg_version);
                            ui.add_space(8.0);
                            status_indicator(ui, "yt-dlp", app.ytdlp_ok, &app.ytdlp_version);
                        });
                    });
                });
        });
}

/// Helper to render a small pill-shaped status indicator for external dependencies.
///
/// Highlights whether a tool is correctly installed and displays its detected version.
fn status_indicator(ui: &mut egui::Ui, name: &str, ok: bool, version: &str) {
    let (bg, fg, text) = if ok {
        (Color32::from_rgb(230, 245, 235), Color32::from_rgb(40, 140, 70), version)
    } else {
        (Color32::from_rgb(255, 235, 235), Color32::from_rgb(200, 60, 60), "Missing")
    };

    egui::Frame::new()
        .fill(bg)
        .corner_radius(cr(12))
        .inner_margin(Margin::symmetric(8_i8, 2_i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(name).strong().size(13.0).color(fg));
                ui.label(RichText::new(text).size(13.0).color(fg));
            });
        });
}
