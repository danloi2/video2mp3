pub mod state;
pub mod theme;
pub mod helpers;
pub mod logic;
pub mod layout;

use std::sync::{Arc, atomic::AtomicBool};
use std::sync::mpsc::Receiver;
use eframe::egui;

use crate::core::{verificar_ffmpeg, TipoConversion, OpcionesVideo, obtener_version_ffmpeg, CapacidadesHardware, detectar_capacidades_hardware, AceleracionHW};
use state::{Archivo, Msg};

pub struct ConvApp {
    pub(crate) ffmpeg_ok:       bool,
    pub(crate) ffmpeg_version:  String,
    pub(crate) capacidades:     CapacidadesHardware,
    pub(crate) archivos:        Vec<Archivo>,
    pub(crate) log:             Vec<(bool, String)>,
    pub(crate) convirtiendo:    bool,
    pub(crate) progreso:        (usize, usize),
    pub(crate) progreso_actual: f32,
    pub(crate) cancelar:        Arc<AtomicBool>,
    pub(crate) rx:              Option<Receiver<Msg>>,
    pub(crate) logo_texture:    Option<egui::TextureHandle>,
    // Nuevas opciones
    pub(crate) tipo_conversion: TipoConversion,
    pub(crate) opciones_video:  OpcionesVideo,
    pub(crate) primera_vez:     bool,
}

impl ConvApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::aplicar_tema(&cc.egui_ctx);
        
        let logo_texture = match image::load_from_memory(include_bytes!("../../assets/icon.png")) {
            Ok(img) => {
                let img = img.to_rgba8();
                let size = [img.width() as usize, img.height() as usize];
                let pixels = img.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                Some(cc.egui_ctx.load_texture("logo", color_image, egui::TextureOptions::LINEAR))
            }
            Err(_) => None,
        };

        Self {
            ffmpeg_ok:       verificar_ffmpeg(),
            ffmpeg_version:  obtener_version_ffmpeg(),
            capacidades:     detectar_capacidades_hardware(),
            archivos:        vec![],
            log:             vec![],
            convirtiendo:    false,
            progreso:        (0, 0),
            progreso_actual: 0.0,
            cancelar:        Arc::new(AtomicBool::new(false)),
            rx:              None,
            logo_texture,
            tipo_conversion: TipoConversion::AudioMP3,
            opciones_video:  OpcionesVideo { preservar_grano: false, optimizar_color: true, aceleracion: AceleracionHW::Ninguna },
            primera_vez:     true,
        }
    }
}

impl eframe::App for ConvApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.primera_vez {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            self.primera_vez = false;
        }
        layout::render_ui(self, ui);
    }
}
