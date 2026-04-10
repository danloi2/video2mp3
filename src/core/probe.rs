use super::types::PistaAudio;
use serde_json::Value;
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct CapacidadesHardware {
    pub nvenc: bool,
    pub qsv:   bool,
    pub amf:   bool,
    pub vaapi: bool,
    pub vtb:   bool,
}

pub fn verificar_ffmpeg() -> bool {
    Command::new("ffmpeg").arg("-version").output().is_ok()
        && Command::new("ffprobe").arg("-version").output().is_ok()
}

pub fn obtener_version_ffmpeg() -> String {
    if let Ok(output) = Command::new("ffmpeg").arg("-version").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().next() {
            // "ffmpeg version 7.1-static ..." -> "7.1-static"
            return line.replace("ffmpeg version ", "")
                       .split_whitespace()
                       .next()
                       .unwrap_or("?")
                       .to_string();
        }
    }
    "desconocido".to_string()
}

pub fn detectar_capacidades_hardware() -> CapacidadesHardware {
    let mut caps = CapacidadesHardware::default();
    
    // 1. Detección por software (ver lo que FFmpeg sabe hacer)
    let b_vaapi = probar_encoder("vaapi");
    let b_nvenc = probar_encoder("nvenc");
    let b_qsv   = probar_encoder("_qsv");
    let b_amf   = probar_encoder("_amf");
    let b_vtb   = probar_encoder("videotoolbox");

    // 2. Detección por hardware real (ver lo que el PC tiene)
    // El comando -init_hw_device intenta inicializar el driver.
    if b_nvenc && probar_dispositivo("cuda")  { caps.nvenc = true; }
    if b_qsv   && probar_dispositivo("qsv")   { caps.qsv   = true; }
    if b_vaapi && probar_dispositivo("vaapi") { caps.vaapi = true; }
    
    // VideoToolbox en macOS no suele necesitar init_hw_device pero lo comprobamos por hwaccels
    if b_vtb && probar_hwaccel("videotoolbox") { caps.vtb = true; }

    // AMF en Linux es poco común, usamos una comprobación simple de encoder
    if b_amf && (caps.vaapi || caps.nvenc) { caps.amf = true; }

    caps
}

fn probar_hwaccel(tipo: &str) -> bool {
    if let Ok(output) = Command::new("ffmpeg").arg("-hwaccels").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.contains(tipo);
    }
    false
}

fn probar_encoder(paci: &str) -> bool {
    if let Ok(output) = Command::new("ffmpeg").arg("-encoders").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.to_lowercase().contains(paci);
    }
    false
}

fn probar_dispositivo(tipo: &str) -> bool {
    // Intentamos inicializar el dispositivo de harware. 
    // Si falla, es que el driver o la placa no están presentes.
    if let Ok(output) = Command::new("ffmpeg")
        .args(["-hide_banner", "-init_hw_device", tipo, "-f", "null", "-"])
        .output() {
        return output.status.success();
    }
    false
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
