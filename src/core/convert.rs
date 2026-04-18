use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use super::probe::obtener_duracion_s;
use super::types::{OpcionesVideo, TipoConversion};

pub fn convertir_archivo<F>(
    origen: &Path,
    destino: Option<&Path>,
    audio_stream: u64,
    sobreescribir: bool,
    tipo: TipoConversion,
    opciones: OpcionesVideo,
    cancelar: Arc<AtomicBool>,
    on_progress: F,
) -> Result<String, String>
where
    F: Fn(crate::core::ProgressUpdate),
{
    let stem = origen
        .file_stem()
        .ok_or_else(|| "Nombre de archivo inválido".to_string())?
        .to_string_lossy();

    let ext = match tipo {
        TipoConversion::AudioMP3 => "mp3",
        _ => "mkv",
    };

    let destino_path: PathBuf = match destino {
        Some(d) => d.to_path_buf(),
        None => origen
            .parent()
            .unwrap_or(Path::new("."))
            .join(format!("{}.{}", stem, ext)),
    };

    if destino_path.exists() && !sobreescribir {
        return Err(format!(
            "⚠ Ya existe: {}",
            destino_path.file_name().unwrap_or_default().to_string_lossy()
        ));
    }

    let duracion_s = obtener_duracion_s(&origen.to_string_lossy()).unwrap_or(0.0);

    let mut args = vec!["-y".to_string()];

    use crate::core::types::AceleracionHW;
    if opciones.aceleracion != AceleracionHW::Ninguna {
        args.extend(["-hwaccel".to_string(), "auto".to_string()]);
    }

    args.extend([
        "-i".to_string(),
        origen.to_string_lossy().to_string(),
    ]);

    match tipo {
        TipoConversion::AudioMP3 => {
            args.extend([
                "-map".to_string(),
                format!("0:{}", audio_stream),
                "-c:a".to_string(),
                "libmp3lame".to_string(),
                "-q:a".to_string(),
                "2".to_string(),
            ]);
        }
        TipoConversion::VideoH264 => {
            let codec = match opciones.aceleracion {
                AceleracionHW::NVENC => "h264_nvenc",
                AceleracionHW::QSV   => "h264_qsv",
                AceleracionHW::AMF   => "h264_amf",
                AceleracionHW::VAAPI => "h264_vaapi",
                AceleracionHW::VideoToolbox => "h264_videotoolbox",
                AceleracionHW::Ninguna => "libx264",
            };
            
            args.extend([
                "-map".to_string(), "0:v:0".to_string(),
                "-map".to_string(), format!("0:{}", audio_stream),
                "-c:v".to_string(), codec.to_string(),
            ]);

            match opciones.aceleracion {
                AceleracionHW::Ninguna => {
                    args.extend(["-crf".to_string(), "18".to_string()]);
                    if opciones.preservar_grano {
                        args.extend(["-tune".to_string(), "grain".to_string()]);
                    } else {
                        args.extend(["-tune".to_string(), "film".to_string()]);
                    }
                    args.extend([
                        "-preset".to_string(), "slow".to_string(),
                        "-profile:v".to_string(), "high".to_string(),
                        "-level".to_string(), "4.1".to_string(),
                        "-x264-params".to_string(), "ref=4:bframes=3:aq-mode=2".to_string(),
                    ]);
                }
                AceleracionHW::NVENC => {
                    args.extend([
                        "-preset".to_string(), "slow".to_string(),
                        "-rc".to_string(), "vbr".to_string(),
                        "-cq".to_string(), "19".to_string(),
                        "-profile:v".to_string(), "high".to_string(),
                    ]);
                }
                AceleracionHW::VideoToolbox => {
                    args.extend(["-q:v".to_string(), "60".to_string()]); // Calidad 0-100 en VTB
                }
                _ => {
                    // Genérico para QSV/AMF/VAAPI por ahora
                    args.extend(["-q:v".to_string(), "19".to_string()]);
                }
            }

            args.extend([
                "-c:a".to_string(), "ac3".to_string(),
                "-b:a".to_string(), "448k".to_string(),
            ]);
        }
        TipoConversion::VideoH265 => {
            let codec = match opciones.aceleracion {
                AceleracionHW::NVENC => "hevc_nvenc",
                AceleracionHW::QSV   => "hevc_qsv",
                AceleracionHW::AMF   => "hevc_amf",
                AceleracionHW::VAAPI => "hevc_vaapi",
                AceleracionHW::VideoToolbox => "hevc_videotoolbox",
                AceleracionHW::Ninguna => "libx265",
            };

            args.extend([
                "-map".to_string(), "0:v:0".to_string(),
                "-map".to_string(), format!("0:{}", audio_stream),
                "-c:v".to_string(), codec.to_string(),
            ]);

            match opciones.aceleracion {
                AceleracionHW::Ninguna => {
                    args.extend(["-crf".to_string(), "20".to_string()]);
                    if opciones.preservar_grano {
                        args.extend(["-tune".to_string(), "grain".to_string()]);
                    }
                    args.extend([
                        "-preset".to_string(), "slow".to_string(),
                        "-x265-params".to_string(), "aq-mode=3:aq-strength=1.0:deblock=-1,-1".to_string(),
                    ]);
                }
                AceleracionHW::NVENC => {
                    args.extend([
                        "-preset".to_string(), "slow".to_string(),
                        "-rc".to_string(), "vbr".to_string(),
                        "-cq".to_string(), "21".to_string(),
                    ]);
                }
                AceleracionHW::VideoToolbox => {
                    args.extend(["-q:v".to_string(), "60".to_string()]);
                }
                _ => {
                    args.extend(["-q:v".to_string(), "21".to_string()]);
                }
            }

            args.extend([
                "-c:a".to_string(), "aac".to_string(),
                "-b:a".to_string(), "192k".to_string(),
            ]);
        }
    }

    if opciones.optimizar_color {
        args.extend([
            "-color_primaries".to_string(), "bt709".to_string(),
            "-color_trc".to_string(), "bt709".to_string(),
            "-colorspace".to_string(), "bt709".to_string(),
        ]);
    }

    args.extend([
        "-progress".to_string(),
        "pipe:1".to_string(),
        "-nostats".to_string(),
        destino_path.to_string_lossy().to_string(),
    ]);

    let mut child = Command::new("ffmpeg")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("No se pudo lanzar ffmpeg: {}", e))?;

    if let Some(stdout) = child.stdout.take() {
        let mut cancelado = false;
        for line in std::io::BufReader::new(stdout).lines().map_while(Result::ok) {
            if cancelar.load(Ordering::Relaxed) {
                child.kill().ok();
                cancelado = true;
                break;
            }
            if let Some(val) = line.strip_prefix("out_time_us=") {
                if let Ok(us) = val.trim().parse::<i64>() {
                    if us > 0 && duracion_s > 0.0 {
                        let ratio =
                            ((us as f64 / 1_000_000.0) / duracion_s).clamp(0.0, 1.0) as f32;
                        on_progress(crate::core::ProgressUpdate::Ratio(ratio));
                    }
                }
            }
        }
        if cancelado {
            let _ = child.wait();
            let _ = std::fs::remove_file(&destino_path);
            return Err("⏹ Conversión cancelada".to_string());
        }
    }

    let status = child.wait().map_err(|e| e.to_string())?;

    if status.success() {
        Ok(format!(
            "✅ {} → {}",
            origen.file_name().unwrap_or_default().to_string_lossy(),
            destino_path.file_name().unwrap_or_default().to_string_lossy()
        ))
    } else {
        let _ = std::fs::remove_file(&destino_path);
        Err(format!(
            "❌ ffmpeg falló al convertir '{}'",
            origen.file_name().unwrap_or_default().to_string_lossy()
        ))
    }
}

pub fn obtener_nombre_youtube(url: &str) -> Option<String> {
    let output = Command::new("yt-dlp")
        .args(["--get-filename", "-o", "%(title)s.%(ext)s", url])
        .output()
        .ok()?;
    
    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

pub fn descargar_youtube<F>(
    url: &str,
    destino: &Path,
    solo_audio: bool,
    cancelar: Arc<AtomicBool>,
    on_progress: F,
) -> Result<PathBuf, String>
where
    F: Fn(crate::core::ProgressUpdate),
{
    let mut args = vec!["--newline".to_string()];
    
    if solo_audio {
        args.push("-x".to_string());
        args.push("--audio-format".to_string());
        args.push("mp3".to_string());
    }

    let template = if solo_audio {
        "%(title)s.mp3"
    } else {
        "%(title)s.%(ext)s"
    };

    args.push("-o".to_string());
    args.push(format!("{}/{}", destino.to_string_lossy(), template));
    args.push(url.to_string());

    let mut child = Command::new("yt-dlp")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("No se pudo lanzar yt-dlp: {}", e))?;

    if let Some(stdout) = child.stdout.take() {
        let mut cancelado = false;
        for line in std::io::BufReader::new(stdout).lines().map_while(Result::ok) {
            if cancelar.load(Ordering::Relaxed) {
                child.kill().ok();
                cancelado = true;
                break;
            }
            // Playlist progress: [download] Downloading item 1 of 2
            if line.contains("Downloading item") && line.contains("of") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let (Some(pos_item), Some(pos_of)) = (parts.iter().position(|&s| s == "item"), parts.iter().position(|&s| s == "of")) {
                    if let (Ok(cur), Ok(tot)) = (parts[pos_item+1].parse::<usize>(), parts[pos_of+1].parse::<usize>()) {
                        on_progress(crate::core::ProgressUpdate::Playlist(cur, tot));
                    }
                }
            }

            if line.contains("[download]") && line.contains('%') {
                if let Some(pos) = line.find('%') {
                    let start = line[..pos].rfind(' ').unwrap_or(0);
                    if let Ok(p) = line[start..pos].trim().parse::<f32>() {
                        on_progress(crate::core::ProgressUpdate::Ratio(p / 100.0));
                    }
                }
            }
        }
        if cancelado {
            let _ = child.wait();
            return Err("⏹ Descarga de YouTube cancelada".to_string());
        }
    }

    let status = child.wait().map_err(|e| e.to_string())?;

    if status.success() {
        // Obtenemos el nombre final (un poco redundante pero seguro)
        if let Some(nombre) = obtener_nombre_youtube(url) {
            let mut ruta = destino.to_path_buf().join(nombre);
            if solo_audio {
                ruta.set_extension("mp3");
            }
            Ok(ruta)
        } else {
            Ok(destino.to_path_buf()) // fallback
        }
    } else {
        Err("❌ yt-dlp falló al descargar".to_string())
    }
}
