use eframe::egui::{self, Color32, Margin, RichText};
use crate::gui::helpers::cr;

/// Helper function to render a set of hardware acceleration capability tags.
/// 
/// It scans the provided `HWCapabilities` struct and displays a stylized tag
/// for each supported chipset (NVENC, QSV, AMF, VAAPI, VideoToolbox).
pub fn render_hw_tags(ui: &mut egui::Ui, c: &crate::core::HWCapabilities) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        
        // Base CPU tag (always shown as fallback)
        tag(ui, "CPU", Color32::from_rgb(140, 145, 155));
        
        // Conditional chipset tags based on hardware detection
        if c.nvenc { tag(ui, "NVENC", Color32::from_rgb(118, 185, 0)); }
        if c.qsv   { tag(ui, "QSV",   Color32::from_rgb(0, 104, 181)); }
        if c.amf   { tag(ui, "AMF",   Color32::from_rgb(237, 28, 36)); }
        if c.vaapi { tag(ui, "VAAPI", Color32::from_rgb(255, 140, 0)); }
        if c.vtb   { tag(ui, "APPLE", Color32::from_rgb(160, 160, 160)); }
    });
}

/// A generic stylized tag with a subtle background and colored text.
/// 
/// Used mainly for hardware capabilities where a low-key aesthetic is preferred.
pub fn tag(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.15)) // Subtle transparent fill
        .stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.5))) // Colored outline
        .corner_radius(cr(4))
        .inner_margin(Margin::symmetric(6_i8, 2_i8))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(12.0).color(color).strong());
        });
}

/// A high-contrast version badge for important metadata.
/// 
/// Features a solid primary color background and white text to ensure maximum 
/// legibility and a premium "Shields.io" look.
pub fn render_version_badge(ui: &mut egui::Ui, text: &str) {
    egui::Frame::new()
        .fill(Color32::from_rgb(105, 75, 215)) // Deep purple theme color
        .corner_radius(cr(6))
        .inner_margin(Margin::symmetric(8_i8, 2_i8))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(13.0).color(Color32::WHITE).strong());
        });
}
