pub mod top_panel;
pub mod central_panel;
pub mod bottom_panel;
pub mod widgets;

use eframe::egui;
use crate::gui::ConvApp;

/// Orchestrates the high-level layout of the application.
pub fn render_ui(app: &mut ConvApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    // Background message processing (runs every frame)
    app.process_messages();
    
    // Handle files dropped into the window
    app.handle_drop(ctx);

    top_panel::render(app, ui);
    bottom_panel::render(app, ui);
    central_panel::render(app, ui, ctx);
}
