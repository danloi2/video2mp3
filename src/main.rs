mod converter;
mod gui;

use std::path::{Path, PathBuf};
use std::fs;
use std::process::exit;
use std::sync::{Arc, atomic::AtomicBool};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Sin argumentos → interfaz gráfica
    if args.len() < 2 {
        let options = eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_title("convmp3 — Conversor de vídeo a MP3")
                .with_inner_size([860.0, 620.0])
                .with_min_inner_size([640.0, 480.0])
                .with_drag_and_drop(true),
            ..Default::default()
        };

        eframe::run_native(
            "convmp3",
            options,
            Box::new(|cc| Ok(Box::new(gui::ConvApp::new(cc)))),
        )
        .expect("Error iniciando la interfaz gráfica");
        return;
    }

    // Modo línea de comandos
    if !converter::verificar_ffmpeg() {
        eprintln!("❌ FFmpeg o FFprobe no están instalados o no están en el PATH.");
        exit(1);
    }
    println!("✅ FFmpeg y FFprobe están instalados.");

    let argumento = &args[1];

    if argumento.to_lowercase() == "all" {
        let mut archivos: Vec<PathBuf> = fs::read_dir(".")
            .expect("No se puede leer el directorio actual")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                matches!(
                    p.extension()
                        .map(|e| e.to_string_lossy().to_lowercase())
                        .as_deref(),
                    Some("mkv") | Some("mp4") | Some("avi")
                )
            })
            .collect();

        archivos.sort();

        if archivos.is_empty() {
            println!("⚠ No se encontraron archivos MKV, MP4 o AVI en el directorio actual.");
            return;
        }

        println!("🎵 {} archivo(s) encontrado(s). Iniciando conversión...\n", archivos.len());

        for archivo in &archivos {
            let pistas    = converter::obtener_pistas(&archivo.to_string_lossy());
            let pista_idx = converter::elegir_pista_defecto(&pistas);
            if pistas.is_empty() {
                println!("⚠ Sin pistas de audio: {}", archivo.display());
                continue;
            }
            let stream = pistas[pista_idx].indice_stream;
            let dummy_cancel = Arc::new(AtomicBool::new(false));
            match converter::convertir_archivo(archivo, None, stream, false, dummy_cancel, |_| {}) {
                Ok(msg)  => println!("{}", msg),
                Err(err) => println!("❌ {}", err),
            }
        }

        println!("\n🎉 Conversión masiva finalizada.");
    } else {
        let origen = Path::new(argumento);
        if !origen.exists() {
            eprintln!("❌ El archivo '{}' no existe", origen.display());
            exit(1);
        }

        // Destino opcional como segundo argumento (solo en modo CLI)
        let destino = args.get(2).map(|s| Path::new(s.as_str()));

        let pistas    = converter::obtener_pistas(&origen.to_string_lossy());
        let pista_idx = converter::elegir_pista_defecto(&pistas);
        if pistas.is_empty() {
            eprintln!("❌ No se encontró ninguna pista de audio en '{}'", origen.display());
            exit(1);
        }
        let stream = pistas[pista_idx].indice_stream;

        let dummy_cancel = Arc::new(AtomicBool::new(false));
        match converter::convertir_archivo(origen, destino, stream, false, dummy_cancel, |_| {}) {
            Ok(msg)  => println!("{}", msg),
            Err(err) => {
                eprintln!("❌ {}", err);
                exit(1);
            }
        }
    }
}