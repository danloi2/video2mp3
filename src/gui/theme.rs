use eframe::egui::{self, Color32, CornerRadius, Stroke};

pub fn cr(v: u8) -> CornerRadius {
    CornerRadius { nw: v, ne: v, sw: v, se: v }
}

pub fn aplicar_tema(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    let bg_base   = Color32::from_rgb(248, 249, 252);
    let bg_panel  = Color32::from_rgb(238, 240, 245);
    let bg_widget = Color32::from_rgb(228, 232, 240);
    
    let accent_primary = Color32::from_rgb(105, 75, 215);

    visuals.window_fill = bg_base;
    visuals.panel_fill = bg_panel;
    visuals.faint_bg_color = bg_panel;
    
    visuals.widgets.noninteractive.bg_fill = bg_widget;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(215, 220, 225));
    visuals.widgets.noninteractive.corner_radius = cr(6);
    
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(235, 238, 245);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(205, 210, 220));
    visuals.widgets.inactive.corner_radius = cr(6);

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(225, 228, 235);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, accent_primary);
    visuals.widgets.hovered.corner_radius = cr(6);

    visuals.widgets.active.bg_fill = Color32::from_rgb(215, 218, 225);
    visuals.widgets.active.bg_stroke = Stroke::new(1.5, accent_primary);
    visuals.widgets.active.corner_radius = cr(6);

    visuals.selection.bg_fill = accent_primary;
    visuals.selection.stroke = Stroke::new(1.0, accent_primary);

    ctx.set_visuals(visuals);
}
