use super::types::PistaAudio;
use serde_json::Value;
use std::process::Command;

pub fn verificar_ffmpeg() -> bool {
    Command::new("ffmpeg").arg("-version").output().is_ok()
        && Command::new("ffprobe").arg("-version").output().is_ok()
}

pub fn elegir_pista_defecto(pistas: &[PistaAudio]) -> usize {
    pistas
        .iter()
        .position(|p| {
            let lang = p.idioma.to_lowercase();
            lang == "spa" || lang == "es"
        })
        .unwrap_or(0)
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

pub(crate) fn obtener_duracion_s(archivo: &str) -> Option<f64> {
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
