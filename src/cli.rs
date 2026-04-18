use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::{Arc, atomic::AtomicBool};

use crate::core::{verificar_ffmpeg, convertir_archivo, elegir_pista_defecto, obtener_pistas, TipoConversion, OpcionesVideo, AceleracionHW};

pub fn run_cli_mode(args: Vec<String>) {
    if !verificar_ffmpeg() {
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

        println!("📂 Se encontraron {} archivos de vídeo. Iniciando conversión...\n", archivos.len());

        let mut exitosos = 0;
        let cancelar     = Arc::new(AtomicBool::new(false));

        for archivo in archivos {
            let pistas = obtener_pistas(&archivo.to_string_lossy());
            let stream = pistas
                .get(elegir_pista_defecto(&pistas))
                .map(|p| p.indice_stream)
                .unwrap_or(0);

            print!("⏳ Convirtiendo '{}'... ", archivo.file_name().unwrap_or_default().to_string_lossy());
            use std::io::Write;
            let _ = std::io::stdout().flush();

            let cancel_clone = cancelar.clone();
            match convertir_archivo(
                &archivo,
                None,
                stream,
                false, // CLI por defecto no sobreescribe
                TipoConversion::AudioMP3,
                OpcionesVideo { preservar_grano: false, optimizar_color: false, aceleracion: AceleracionHW::Ninguna },
                cancel_clone,
                |_ratio| {}, 
            ) {
                Ok(_) => {
                    println!("[ OK ]");
                    exitosos += 1;
                }
                Err(e) => {
                    println!("\n  └─ Error: {}", e);
                }
            }
        }

        println!("\n🎉 Conversión múltiple completada: {} con éxito.", exitosos);
        return;
    }

    let mut origen_path = PathBuf::from(argumento);

    if !origen_path.exists() {
        if !origen_path.extension().is_some() {
            let posibles_origenes = [
                origen_path.with_extension("mkv"),
                origen_path.with_extension("mp4"),
                origen_path.with_extension("avi"),
            ];

            let or = posibles_origenes.iter().find(|p| p.exists());
            if let Some(p) = or {
                origen_path = p.clone();
            } else {
                eprintln!("❌ El archivo de origen '{}' no existe y no se encontró una variante .mkv, .mp4 o .avi.", origen_path.display());
                exit(1);
            }
        } else {
            eprintln!("❌ El archivo de origen '{}' no existe.", origen_path.display());
            exit(1);
        }
    }

    let destino_path = if args.len() > 2 {
        let arg_dest = Path::new(&args[2]);
        if arg_dest.extension().is_none() {
            arg_dest.with_extension("mp3")
        } else {
            arg_dest.to_path_buf()
        }
    } else {
        let stem = origen_path.file_stem().unwrap_or_default();
        origen_path.with_file_name(format!("{}.mp3", stem.to_string_lossy()))
    };

    println!("Iniciando conversión...");

    let pistas = obtener_pistas(&origen_path.to_string_lossy());
    let stream = pistas
        .get(elegir_pista_defecto(&pistas))
        .map(|p| p.indice_stream)
        .unwrap_or(0);

    let cancelar = Arc::new(AtomicBool::new(false));

    match convertir_archivo(
        &origen_path,
        Some(&destino_path),
        stream,
        false, 
        TipoConversion::AudioMP3,
        OpcionesVideo { preservar_grano: false, optimizar_color: false, aceleracion: AceleracionHW::Ninguna },
        cancelar,
        |_ratio| {}, 
    ) {
        Ok(msg) => println!("{}", msg),
        Err(e) => eprintln!("{}", e),
    }
}
