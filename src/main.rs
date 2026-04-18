mod core;
mod gui;
mod cli;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Sin argumentos → interfaz gráfica
    if args.len() < 2 {
        let mut viewport = eframe::egui::ViewportBuilder::default()
            .with_title("video2mp3 — Conversor de vídeo a MP3")
            .with_inner_size([1100.0, 800.0])
            .with_min_inner_size([1100.0, 800.0])
            .with_drag_and_drop(true);

        if let Ok(img) = image::load_from_memory(include_bytes!("../resources/icon.png")) {
            let img_rgba = img.to_rgba8();
            let (width, height) = img_rgba.dimensions();
            let icon_data = eframe::egui::IconData {
                rgba: img_rgba.into_raw(),
                width,
                height,
            };
            viewport = viewport.with_icon(std::sync::Arc::new(icon_data));
        }

        let options = eframe::NativeOptions {
            viewport,
            persist_window: false,
            ..Default::default()
        };

        eframe::run_native(
            "video2mp3",
            options,
            Box::new(|cc| Ok(Box::new(gui::ConvApp::new(cc)))),
        )
        .expect("Error al iniciar la interfaz gráfica");
        return;
    }

    // Modo línea de comandos
    cli::run_cli_mode(args);
}