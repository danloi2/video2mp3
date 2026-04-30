use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::{Arc, atomic::AtomicBool};

use crate::core::{verify_ffmpeg, convert_file, select_default_track, get_audio_tracks, ConversionType, VideoOptions, HWAcceleration};

/// Entry point for the Command Line Interface (CLI) mode.
/// 
/// Handles both single file conversion and batch processing of all supported 
/// video files in the current directory.
pub fn run_cli(args: &[String]) {
    // Check if the system has the required FFmpeg/FFprobe binaries
    if !verify_ffmpeg() {
        eprintln!("❌ FFmpeg or FFprobe are not installed or not in the system PATH.");
        exit(1);
    }
    println!("✅ FFmpeg and FFprobe are ready.");

    let argument = &args[1];

    // --- Case: Batch mode "all" ---
    if argument.to_lowercase() == "all" {
        // Collect all compatible video files in the current directory
        let mut files: Vec<PathBuf> = fs::read_dir(".")
            .expect("Cannot read the current directory")
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

        files.sort();

        if files.is_empty() {
            println!("⚠ No MKV, MP4, or AVI files found in the current directory.");
            return;
        }

        println!("📂 Found {} video files. Starting batch conversion...\n", files.len());

        let mut successful_count = 0;
        let cancel               = Arc::new(AtomicBool::new(false));

        for file_path in files {
            // Find the best audio track for this file
            let tracks = get_audio_tracks(&file_path.to_string_lossy());
            let stream = tracks
                .get(select_default_track(&tracks))
                .map(|t| t.stream_index)
                .unwrap_or(0);

            print!("⏳ Converting '{}'... ", file_path.file_name().unwrap_or_default().to_string_lossy());
            use std::io::Write;
            let _ = std::io::stdout().flush();

            let cancel_clone = cancel.clone();
            // Perform conversion using default audio settings and no HW acceleration for CLI
            match convert_file(
                &file_path,
                None,
                stream,
                false, // CLI defaults to non-overwrite to prevent data loss
                ConversionType::AudioMP3,
                VideoOptions { preserve_grain: false, optimize_color: false, acceleration: HWAcceleration::None },
                cancel_clone,
                |_ratio| {}, 
            ) {
                Ok(_) => {
                    println!("[ OK ]");
                    successful_count += 1;
                }
                Err(e) => {
                    println!("\n  └─ Error: {}", e);
                }
            }
        }

        println!("\n🎉 Batch conversion completed: {} successful conversions.", successful_count);
        return;
    }

    // --- Case: Single file conversion ---
    let mut source_path = PathBuf::from(argument);

    // If the exact file doesn't exist, try searching for it with different extensions
    if !source_path.exists() {
        if source_path.extension().is_none() {
            let potential_sources = [
                source_path.with_extension("mkv"),
                source_path.with_extension("mp4"),
                source_path.with_extension("avi"),
            ];

            let found = potential_sources.iter().find(|p| p.exists());
            if let Some(p) = found {
                source_path = p.clone();
            } else {
                eprintln!("❌ Source file '{}' not found (tried .mkv, .mp4, and .avi variants).", source_path.display());
                exit(1);
            }
        } else {
            eprintln!("❌ Source file '{}' does not exist.", source_path.display());
            exit(1);
        }
    }

    // Determine the destination path (defaults to same name with .mp3 extension)
    let dest_path = if args.len() > 2 {
        let arg_dest = Path::new(&args[2]);
        if arg_dest.extension().is_none() {
            arg_dest.with_extension("mp3")
        } else {
            arg_dest.to_path_buf()
        }
    } else {
        let stem = source_path.file_stem().unwrap_or_default();
        source_path.with_file_name(format!("{}.mp3", stem.to_string_lossy()))
    };

    println!("Starting conversion...");

    let tracks = get_audio_tracks(&source_path.to_string_lossy());
    let stream = tracks
        .get(select_default_track(&tracks))
        .map(|t| t.stream_index)
        .unwrap_or(0);

    let cancel = Arc::new(AtomicBool::new(false));

    match convert_file(
        &source_path,
        Some(&dest_path),
        stream,
        false, 
        ConversionType::AudioMP3,
        VideoOptions { preserve_grain: false, optimize_color: false, acceleration: HWAcceleration::None },
        cancel,
        |_ratio| {}, 
    ) {
        Ok(msg) => println!("{}", msg),
        Err(e) => eprintln!("{}", e),
    }
}
