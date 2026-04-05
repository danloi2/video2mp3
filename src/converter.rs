use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use serde_json::Value;

// ─── Pista de audio ───────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct PistaAudio {
    pub indice_stream: u64,
    pub codec: String,
    pub idioma: String,
}

/// Devuelve el índice (en el Vec) de la primera pista en español, o 0.
pub fn elegir_pista_defecto(pistas: &[PistaAudio]) -> usize {
    pistas
        .iter()
        .position(|p| {
            let lang = p.idioma.to_lowercase();
            lang == "spa" || lang == "es"
        })
        .unwrap_or(0)
}

// ─── ffmpeg / ffprobe helpers ────────────────────────────────────────────────

pub fn verificar_ffmpeg() -> bool {
    Command::new("ffmpeg").arg("-version").output().is_ok()
        && Command::new("ffprobe").arg("-version").output().is_ok()
}

fn obtener_audios_json(archivo: &str) -> Vec<Value> {
    let Ok(output) = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_entries", "stream=index,codec_name:stream_tags=language",
            "-select_streams", "a",
            "-of", "json",
            archivo,
        ])
        .output()
    else {
        return vec![];
    };

    let Ok(json): Result<Value, _> = serde_json::from_slice(&output.stdout) else {
        return vec![];
    };

    match json["streams"].as_array() {
        Some(arr) => arr
            .iter()
            .map(|s| {
                let mut s = s.clone();
                if s.get("tags").is_none() {
                    s["tags"] = Value::Object(serde_json::Map::new());
                }
                if s["tags"].get("language").is_none() {
                    s["tags"]["language"] = Value::String("desconocido".to_string());
                }
                s
            })
            .collect(),
        None => vec![],
    }
}

/// Retorna las pistas de audio disponibles en el archivo.
pub fn obtener_pistas(archivo: &str) -> Vec<PistaAudio> {
    obtener_audios_json(archivo)
        .into_iter()
        .map(|s| PistaAudio {
            indice_stream: s["index"].as_u64().unwrap_or(0),
            codec: s["codec_name"].as_str().unwrap_or("?").to_string(),
            idioma: s["tags"]["language"]
                .as_str()
                .unwrap_or("desconocido")
                .to_string(),
        })
        .collect()
}

fn obtener_duracion_s(archivo: &str) -> Option<f64> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "json",
            archivo,
        ])
        .output()
        .ok()?;

    let json: Value = serde_json::from_slice(&output.stdout).ok()?;
    json["format"]["duration"]
        .as_str()?
        .trim()
        .parse::<f64>()
        .ok()
}

/// Convierte un archivo de vídeo a MP3 usando el stream `audio_stream`.
/// Si `destino` es None, el MP3 se guarda junto al origen.
/// Llama `on_progress(ratio)` con ratio ∈ [0.0, 1.0] durante la conversión.
pub fn convertir_archivo<F>(
    origen: &Path,
    destino: Option<&Path>,
    audio_stream: u64,
    sobreescribir: bool,
    cancelar: Arc<AtomicBool>,
    on_progress: F,
) -> Result<String, String>
where
    F: Fn(f32),
{
    let stem = origen
        .file_stem()
        .ok_or_else(|| "Nombre de archivo inválido".to_string())?
        .to_string_lossy();

    let destino_path: PathBuf = match destino {
        Some(d) => d.to_path_buf(),
        None => origen
            .parent()
            .unwrap_or(Path::new("."))
            .join(format!("{}.mp3", stem)),
    };

    if destino_path.exists() && !sobreescribir {
        return Err(format!(
            "⚠ Ya existe: {}",
            destino_path.file_name().unwrap_or_default().to_string_lossy()
        ));
    }

    let duracion_s = obtener_duracion_s(&origen.to_string_lossy()).unwrap_or(0.0);

    let mut child = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &origen.to_string_lossy(),
            "-map",
            &format!("0:{}", audio_stream),
            "-c:a",
            "libmp3lame",
            "-q:a",
            "2",
            "-progress",
            "pipe:1",
            "-nostats",
            &destino_path.to_string_lossy(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("No se pudo lanzar ffmpeg: {}", e))?;

    if let Some(stdout) = child.stdout.take() {
        let mut cancelado = false;
        for line in std::io::BufReader::new(stdout).lines().map_while(Result::ok) {
            // Comprobar señal de cancelación
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
                        on_progress(ratio);
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
