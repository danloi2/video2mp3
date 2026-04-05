use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use super::probe::obtener_duracion_s;

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
