use std::path::PathBuf;
use eframe::egui::Color32;
use crate::core::{PistaAudio, InfoMedia};

#[derive(Clone, PartialEq, Debug)]
pub enum Estado {
    Pendiente,
    Convirtiendo,
    Listo,
    Error(String),
}

impl Estado {
    pub fn icono(&self) -> &'static str {
        match self {
            Estado::Pendiente    => "⏳",
            Estado::Convirtiendo => "⚙",
            Estado::Listo        => "✅",
            Estado::Error(_)     => "❌",
        }
    }
    
    pub fn color(&self) -> Color32 {
        match self {
            Estado::Pendiente    => Color32::from_rgb(160, 150, 50),
            Estado::Convirtiendo => Color32::from_rgb(60, 100, 220),
            Estado::Listo        => Color32::from_rgb(40, 160, 80),
            Estado::Error(_)     => Color32::from_rgb(200, 50, 50),
        }
    }
}

pub struct Archivo {
    pub ruta:        PathBuf,
    pub estado:      Estado,
    pub seleccionado: bool,
    pub pistas:      Vec<PistaAudio>, // pistas de audio detectadas
    pub pista_sel:   usize,           // índice en `pistas` elegido por el usuario
    pub info:        Option<InfoMedia>,
    pub youtube_url: Option<String>,
}

pub enum Msg {
    Iniciando(usize),
    Progreso(usize, f32),
    PlaylistProgress(usize, usize, usize), // idx, item_actual, item_total
    Resultado(usize, bool, String),
    AnadirArchivosYoutube(Vec<(String, String)>),
    Finalizado,
}
