//! video2mp3 - A high-performance, professional-grade media converter.
//!
//! This application provides both a graphical user interface (GUI) and a 
//! command-line interface (CLI) for batch processing video files into 
//! high-quality MP3 or transcoded video formats (H.264/H.265).

mod core;
mod gui;
mod cli;

use std::env;
use eframe::egui;

fn main() -> eframe::Result {
    let args: Vec<String> = env::args().collect();

    // --- Execution Mode Dispatch ---
    // If command-line arguments are provided, launch in headless CLI mode.
    // Otherwise, initialize the eframe GUI environment.
    if args.len() > 1 {
        cli::run_cli(&args);
        Ok(())
    } else {
        // GUI Window Configuration
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1100.0, 800.0])
                .with_min_inner_size([800.0, 600.0])
                .with_icon(
                    eframe::icon_data::from_png_bytes(include_bytes!("../resources/icon.png"))
                        .expect("Failed to load application icon"),
                ),
            ..Default::default()
        };

        // Launch the eframe application
        eframe::run_native(
            "video2mp3 - Pro Media Converter",
            options,
            Box::new(|cc| Ok(Box::new(gui::ConvApp::new(cc)))),
        )
    }
}