use eframe::egui::{self, Color32, RichText, ScrollArea};
use crate::gui::ConvApp;
use crate::gui::helpers::cr;

/// Renders the bottom status and log panel.
pub fn render(app: &mut ConvApp, ui: &mut egui::Ui) {
    egui::Panel::bottom("panel_log")
        .min_size(160.0)
        .max_size(280.0)
        .resizable(true)
        .show_inside(ui, |ui| {
            ui.add_space(4.0);

            // --- SECTION: Progress Bar ---
            if app.progress.1 > 0 {
                let ratio = (app.progress.0 as f32 + app.current_progress) / app.progress.1 as f32;

                // Heuristic to detect final post-processing (e.g., yt-dlp merging streams)
                let is_postprocessing = app.is_converting && app.current_progress >= 0.99;

                let progress_text = if is_postprocessing {
                    format!("{} of {} file(s)  —  Processing…", app.progress.0, app.progress.1)
                } else {
                    format!(
                        "{} of {} file(s)  —  {:.0}%",
                        app.progress.0, app.progress.1, ratio * 100.0
                    )
                };

                let bar_h = 22.0;
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), bar_h),
                    egui::Sense::hover(),
                );
                
                if ui.is_rect_visible(rect) {
                    let paint = ui.painter();
                    let cr5 = cr(5);
                    
                    // 1. Background Track
                    paint.rect_filled(rect, cr5, Color32::from_rgb(215, 220, 230));

                    // 2. Foreground Fill / Animation
                    if is_postprocessing {
                        // "Marquee" Pulse Effect for indeterminate state
                        let t = ui.ctx().input(|i| i.time) as f32;
                        let phase = (t * 0.8).fract(); 
                        let pulse_w = rect.width() * 0.35;
                        let pulse_start = rect.min.x + (rect.width() + pulse_w) * phase - pulse_w;
                        let pulse_end = pulse_start + pulse_w;

                        let draw_start = pulse_start.max(rect.min.x);
                        let draw_end = pulse_end.min(rect.max.x);

                        if draw_end > draw_start {
                            let pulse_rect = egui::Rect::from_min_max(
                                egui::pos2(draw_start, rect.min.y),
                                egui::pos2(draw_end, rect.max.y),
                            );
                            paint.rect_filled(pulse_rect, cr5, Color32::from_rgb(115, 85, 225));
                        }
                    } else if ratio > 0.0 {
                        // Standard Fill
                        let fill_w = (rect.width() * ratio).clamp(0.0, rect.width());
                        let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_w, bar_h));
                        paint.rect_filled(fill_rect, cr5, Color32::from_rgb(115, 85, 225));
                    }

                    // 3. Status Text Overlay
                    let text_color = Color32::WHITE;
                    paint.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &progress_text,
                        egui::FontId::proportional(18.0),
                        // Adjust text color for contrast if the bar is mostly empty
                        if is_postprocessing || ratio > 0.45 { text_color } else { Color32::from_rgb(90, 95, 110) },
                    );
                }
                ui.add_space(4.0);
            }

            // --- SECTION: Activity Log ---
            ui.label(
                RichText::new("📋  Log")
                    .size(15.0)
                    .color(Color32::from_rgb(120, 125, 140)),
            );
            ui.separator();

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true) // Auto-scroll as new lines arrive
                .show(ui, |ui| {
                    for (ok, line) in &app.log {
                        let col = if *ok {
                            Color32::from_rgb(60, 65, 80) // Normal text
                        } else {
                            Color32::from_rgb(210, 50, 50) // Error red
                        };
                        ui.label(RichText::new(line).monospace().size(16.0).color(col));
                    }
                });
        });
}
