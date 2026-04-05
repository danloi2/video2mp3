use eframe::egui::{self, Color32, RichText, ScrollArea, Stroke, CornerRadius, Margin};
use std::time::Duration;
use rfd::FileDialog;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use crate::converter::{self, PistaAudio};

// ─── Helpers de display ────────────────────────────────────────────────────────

fn nombre_idioma(codigo: &str) -> String {
    match codigo.to_lowercase().as_str() {
        "spa" | "es" | "esp" => "Español",
        "eng" | "en"         => "Inglés",
        "fre" | "fra" | "fr" => "Francés",
        "ger" | "deu" | "de" => "Alemán",
        "ita" | "it"         => "Italiano",
        "por" | "pt"         => "Portugués",
        "jpn" | "ja"         => "Japonés",
        "chi" | "zho" | "zh" => "Chino",
        "kor" | "ko"         => "Coreano",
        "ara" | "ar"         => "Árabe",
        "rus" | "ru"         => "Ruso",
        "cat" | "ca"         => "Catalán",
        "eus" | "eu"         => "Euskera",
        "glg" | "gl"         => "Gallego",
        "desconocido"        => "Desconocido",
        other                => other,
    }.to_string()
}

fn etiqueta_pista(pista: &PistaAudio, num: usize) -> String {
    format!(
        "Pista {} — {} ({})",
        num + 1,
        nombre_idioma(&pista.idioma),
        pista.codec.to_uppercase()
    )
}

// ─── Estado de cada archivo ───────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
pub enum Estado {
    Pendiente,
    Convirtiendo,
    Listo,
    Error(String),
}

impl Estado {
    fn icono(&self) -> &'static str {
        match self {
            Estado::Pendiente    => "⏳",
            Estado::Convirtiendo => "⚙",
            Estado::Listo        => "✅",
            Estado::Error(_)     => "❌",
        }
    }
    fn color(&self) -> Color32 {
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
}

// ─── Mensajes del hilo trabajador ─────────────────────────────────────────────

enum Msg {
    Iniciando(usize),
    Progreso(usize, f32),
    Resultado(usize, bool, String),
    Finalizado,
}

// ─── App principal ────────────────────────────────────────────────────────────

pub struct ConvApp {
    ffmpeg_ok:       bool,
    archivos:        Vec<Archivo>,
    log:             Vec<(bool, String)>,
    convirtiendo:    bool,
    progreso:        (usize, usize), // (completados, total)
    progreso_actual: f32,
    cancelar:        Arc<AtomicBool>,
    rx:              Option<Receiver<Msg>>,
}

fn cr(v: u8) -> CornerRadius {
    CornerRadius { nw: v, ne: v, sw: v, se: v }
}

impl ConvApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::aplicar_tema(&cc.egui_ctx);
        Self {
            ffmpeg_ok:       converter::verificar_ffmpeg(),
            archivos:        vec![],
            log:             vec![],
            convirtiendo:    false,
            progreso:        (0, 0),
            progreso_actual: 0.0,
            cancelar:        Arc::new(AtomicBool::new(false)),
            rx:              None,
        }
    }

    fn aplicar_tema(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::light();
        let bg_base   = Color32::from_rgb(248, 249, 252);
        let bg_panel  = Color32::from_rgb(238, 240, 245);
        let bg_widget = Color32::from_rgb(228, 232, 240);
        let accent    = Color32::from_rgb(115, 85, 225);

        visuals.panel_fill        = bg_panel;
        visuals.window_fill       = bg_base;
        visuals.extreme_bg_color  = bg_base;
        visuals.selection.bg_fill = accent;

        visuals.widgets.noninteractive.bg_fill   = bg_widget;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(200, 205, 215));
        visuals.widgets.inactive.bg_fill = bg_widget;
        visuals.widgets.hovered.bg_fill  = Color32::from_rgb(215, 220, 230);
        visuals.widgets.active.bg_fill   = accent;

        let mut style = (*ctx.global_style()).clone();
        style.visuals = visuals;
        style.spacing.item_spacing   = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
        ctx.set_global_style(style);
    }

    fn anadir_archivos(&mut self) {
        let Some(rutas) = FileDialog::new()
            .add_filter("Vídeos soportados", &["mkv", "mp4", "avi", "MKV", "MP4", "AVI"])
            .pick_files()
        else {
            return;
        };

        let mut nuevos = 0usize;
        for ruta in rutas {
            if !self.archivos.iter().any(|a| a.ruta == ruta) {
                // Sondear pistas de audio al añadir el archivo
                let pistas    = converter::obtener_pistas(&ruta.to_string_lossy());
                let pista_sel = converter::elegir_pista_defecto(&pistas);
                self.archivos.push(Archivo {
                    ruta,
                    estado: Estado::Pendiente,
                    seleccionado: true,
                    pistas,
                    pista_sel,
                });
                nuevos += 1;
            }
        }
        if nuevos > 0 {
            self.log.push((true, format!("📂 {} archivo(s) añadido(s)", nuevos)));
        }
    }

    fn iniciar_conversion(&mut self, ctx: &egui::Context) {
        let mut pendientes = Vec::new();

        for (i, a) in self.archivos.iter().enumerate() {
            if a.seleccionado && a.estado == Estado::Pendiente {
                let stream = a.pistas.get(a.pista_sel)
                    .map(|p| p.indice_stream)
                    .unwrap_or(0);
                
                let stem = a.ruta.file_stem().unwrap_or_default().to_string_lossy();
                let destino_path = a.ruta.parent().unwrap_or(Path::new(".")).join(format!("{}.mp3", stem));
                
                let mut sobreescribir = false;
                if destino_path.exists() {
                    let nombre = destino_path.file_name().unwrap_or_default().to_string_lossy();
                    let msg = format!("El archivo '{}' ya existe.\n¿Quieres sobreescribirlo?", nombre);
                    let res = rfd::MessageDialog::new()
                        .set_title("Sobreescribir Archivo")
                        .set_description(&msg)
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .show();
                    
                    if res == rfd::MessageDialogResult::Yes {
                        sobreescribir = true;
                    } else {
                        // El usuario ha elegido no sobreescribir, ignoramos este archivo
                        continue;
                    }
                }
                pendientes.push((i, a.ruta.clone(), stream, sobreescribir));
            }
        }

        if pendientes.is_empty() {
            self.log.push((false, "⚠ No hay archivos pendientes seleccionados.".into()));
            return;
        }

        let total = pendientes.len();
        self.convirtiendo    = true;
        self.progreso        = (0, total);
        self.progreso_actual = 0.0;
        self.cancelar.store(false, Ordering::Relaxed);

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        let ctx2 = ctx.clone();
        let cancel_flag = self.cancelar.clone();
        thread::spawn(move || {
            for (idx, ruta, stream, sobreescribir) in pendientes {
                let _ = tx.send(Msg::Iniciando(idx));
                ctx2.request_repaint();

                let tx2  = tx.clone();
                let ctx3 = ctx2.clone();
                let cancel_clone = cancel_flag.clone();
                let (ok, msg) = match converter::convertir_archivo(
                    &ruta,
                    None,
                    stream,
                    sobreescribir,
                    cancel_clone,
                    move |ratio| {
                        let _ = tx2.send(Msg::Progreso(idx, ratio));
                        ctx3.request_repaint();
                    },
                ) {
                    Ok(m)  => (true,  m),
                    Err(e) => (false, e),
                };

                let _ = tx.send(Msg::Resultado(idx, ok, msg));
                ctx2.request_repaint();
            }
            let _ = tx.send(Msg::Finalizado);
            ctx2.request_repaint();
        });
    }

    fn procesar_mensajes(&mut self) {
        if !self.convirtiendo { return; }

        let mensajes: Vec<Msg> = {
            let Some(rx) = &self.rx else { return };
            let mut buf = vec![];
            while let Ok(m) = rx.try_recv() { buf.push(m); }
            buf
        };

        for msg in mensajes {
            match msg {
                Msg::Iniciando(idx) => {
                    if let Some(a) = self.archivos.get_mut(idx) {
                        let nombre = a.ruta.file_name().unwrap_or_default().to_string_lossy().to_string();
                        a.estado = Estado::Convirtiendo;
                        self.log.push((true, format!("⚙  Convirtiendo: {}", nombre)));
                    }
                    self.progreso_actual = 0.0;
                }
                Msg::Progreso(_idx, ratio) => {
                    self.progreso_actual = ratio;
                }
                Msg::Resultado(idx, ok, text) => {
                    if let Some(a) = self.archivos.get_mut(idx) {
                        a.estado = if ok { Estado::Listo } else { Estado::Error(text.clone()) };
                    }
                    self.log.push((ok, text));
                    self.progreso.0  += 1;
                    self.progreso_actual = 0.0;
                }
                Msg::Finalizado => {
                    self.convirtiendo    = false;
                    self.progreso_actual = 0.0;
                    self.rx              = None;
                    self.log.push((true, "🎉 ¡Conversión completada!".into()));
                }
            }
        }
    }

    fn manejar_drop(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            for f in &i.raw.dropped_files {
                if let Some(ruta) = &f.path {
                    if let Some(ext) = ruta.extension() {
                        let ext = ext.to_string_lossy().to_lowercase();
                        if (ext == "mkv" || ext == "mp4" || ext == "avi")
                            && !self.archivos.iter().any(|a| &a.ruta == ruta)
                        {
                            let nombre = ruta.file_name().unwrap_or_default().to_string_lossy().to_string();
                            let pistas    = converter::obtener_pistas(&ruta.to_string_lossy());
                            let pista_sel = converter::elegir_pista_defecto(&pistas);
                            self.archivos.push(Archivo {
                                ruta: ruta.clone(),
                                estado: Estado::Pendiente,
                                seleccionado: true,
                                pistas,
                                pista_sel,
                            });
                            self.log.push((true, format!("↓ Añadido: {}", nombre)));
                        }
                    }
                }
            }
        });
    }
}

impl eframe::App for ConvApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.procesar_mensajes();
        
        // We still need a handle to context for dropping files and repaint
        let ctx = ui.ctx().clone();
        self.manejar_drop(&ctx);

        if self.convirtiendo {
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        // ── Barra superior ────────────────────────────────────────────────────
        egui::Panel::top("cabecera")
            .exact_size(48.0)
            .show_inside(ui, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("🎵  video2mp3")
                            .size(21.0)
                            .color(Color32::from_rgb(105, 75, 215))
                            .strong(),
                    );
                    ui.label(
                        RichText::new("— Conversor de vídeo a MP3")
                            .size(13.0)
                            .color(Color32::from_rgb(100, 105, 120)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(12.0);
                        let (punto, txt, col) = if self.ffmpeg_ok {
                            ("●", " FFmpeg activo", Color32::from_rgb(45, 175, 80))
                        } else {
                            ("●", " FFmpeg no encontrado", Color32::from_rgb(220, 50, 50))
                        };
                        ui.label(RichText::new(format!("{}{}", punto, txt)).color(col).size(12.0));
                    });
                });
            });

        // ── Panel inferior: log + barra de progreso ───────────────────────────
        egui::Panel::bottom("panel_log")
            .min_size(160.0)
            .max_size(280.0)
            .resizable(true)
            .show_inside(ui, |ui| {
                ui.add_space(4.0);

                if self.progreso.1 > 0 {
                    let ratio = (self.progreso.0 as f32 + self.progreso_actual)
                        / self.progreso.1 as f32;
                    let texto = format!(
                        "{} de {} archivo(s)  —  {:.0}%",
                        self.progreso.0, self.progreso.1, ratio * 100.0
                    );

                    // Barra dibujada a mano (evita interferencias del tema egui)
                    let bar_h = 22.0;
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), bar_h),
                        egui::Sense::hover(),
                    );
                    if ui.is_rect_visible(rect) {
                        let paint = ui.painter();
                        let cr5 = CornerRadius { nw: 5, ne: 5, sw: 5, se: 5 };
                        // Fondo de la barra claro
                        paint.rect_filled(rect, cr5, Color32::from_rgb(215, 220, 230));
                        if ratio > 0.0 {
                            let fill_w = (rect.width() * ratio).clamp(0.0, rect.width());
                            let fill_rect = egui::Rect::from_min_size(
                                rect.min,
                                egui::vec2(fill_w, bar_h),
                            );
                            paint.rect_filled(fill_rect, cr5, Color32::from_rgb(115, 85, 225));
                        }
                        
                        let text_color = Color32::from_rgb(255, 255, 255);

                        paint.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            &texto,
                            egui::FontId::proportional(13.0),
                            if ratio > 0.45 { text_color } else { Color32::from_rgb(90, 95, 110) },
                        );
                    }
                    ui.add_space(4.0);
                }

                ui.label(
                    RichText::new("📋  Registro")
                        .size(11.0)
                        .color(Color32::from_rgb(120, 125, 140)),
                );
                ui.separator();

                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for (ok, linea) in &self.log {
                            let col = if *ok {
                                Color32::from_rgb(60, 65, 80)
                            } else {
                                Color32::from_rgb(210, 50, 50)
                            };
                            ui.label(RichText::new(linea).monospace().size(12.0).color(col));
                        }
                    });
            });

        // ── Panel central ─────────────────────────────────────────────────────
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add_space(4.0);

            let convirtiendo = self.convirtiendo;

            // Botones de acción
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        !convirtiendo,
                        egui::Button::new("📂  Añadir archivos")
                            .fill(Color32::from_rgb(225, 230, 240)),
                    )
                    .on_hover_text("Seleccionar archivos MKV, MP4 o AVI")
                    .clicked()
                {
                    self.anadir_archivos();
                }

                if convirtiendo {
                    if ui
                        .add(egui::Button::new(RichText::new("⏹  Detener").color(Color32::from_rgb(255, 255, 255)))
                            .fill(Color32::from_rgb(220, 60, 60)))
                        .on_hover_text("Cancelar la conversión actual")
                        .clicked()
                    {
                        self.cancelar.store(true, Ordering::Relaxed);
                    }
                } else {
                    let hay_pendientes = self
                        .archivos
                        .iter()
                        .any(|a| a.seleccionado && a.estado == Estado::Pendiente);

                    if ui
                        .add_enabled(
                            self.ffmpeg_ok && hay_pendientes,
                            egui::Button::new(RichText::new("▶  Convertir").color(Color32::from_rgb(255, 255, 255)))
                                .fill(Color32::from_rgb(115, 85, 225)),
                        )
                        .on_hover_text("Convertir los archivos seleccionados a MP3")
                        .clicked()
                    {
                        self.iniciar_conversion(&ctx);
                    }
                }

                if ui
                    .add_enabled(
                        !convirtiendo,
                        egui::Button::new("🗑  Limpiar")
                            .fill(Color32::from_rgb(240, 210, 210)),
                    )
                    .on_hover_text("Vaciar lista y registro")
                    .clicked()
                {
                    self.archivos.clear();
                    self.log.clear();
                    self.progreso = (0, 0);
                }

                ui.separator();

                if ui.add_enabled(!convirtiendo, egui::Button::new("✔ Todos")).clicked() {
                    for a in &mut self.archivos { a.seleccionado = true; }
                }
                if ui.add_enabled(!convirtiendo, egui::Button::new("✗ Ninguno")).clicked() {
                    for a in &mut self.archivos { a.seleccionado = false; }
                }

                let total = self.archivos.len();
                let sel   = self.archivos.iter().filter(|a| a.seleccionado).count();
                if total > 0 {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{} / {} seleccionados", sel, total))
                                .size(12.0)
                                .color(Color32::from_rgb(110, 115, 130)),
                        );
                    });
                }
            });

            ui.add_space(6.0);

            // Lista de archivos
            if self.archivos.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("Arrastra archivos aquí\no usa «Añadir archivos»")
                            .size(17.0)
                            .color(Color32::from_rgb(140, 145, 165)),
                    );
                });
            } else {
                ScrollArea::vertical().show(ui, |ui| {
                    let mut eliminar: Option<usize> = None;

                    for (i, archivo) in self.archivos.iter_mut().enumerate() {
                        egui::Frame::new()
                            .fill(Color32::from_rgb(255, 255, 255))
                            .corner_radius(cr(6))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(225, 230, 235)))
                            .inner_margin(Margin::symmetric(10_i8, 7_i8))
                            .show(ui, |ui| {

                                // ── Fila principal ────────────────────────
                                ui.horizontal(|ui| {
                                    ui.add_enabled(
                                        !convirtiendo,
                                        egui::Checkbox::new(&mut archivo.seleccionado, ""),
                                    );

                                    ui.label(
                                        RichText::new(archivo.estado.icono())
                                            .size(16.0)
                                            .color(archivo.estado.color()),
                                    );

                                    let nombre = archivo
                                        .ruta
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy();
                                    ui.label(
                                        RichText::new(nombre.as_ref())
                                            .size(14.0)
                                            .color(Color32::from_rgb(45, 50, 65)),
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if !convirtiendo
                                                && ui
                                                    .small_button(
                                                        RichText::new("✕")
                                                            .color(Color32::from_rgb(200, 80, 80)),
                                                    )
                                                    .on_hover_text("Quitar de la lista")
                                                    .clicked()
                                            {
                                                eliminar = Some(i);
                                            }

                                            let dir = archivo
                                                .ruta
                                                .parent()
                                                .map(|p| p.to_string_lossy().to_string())
                                                .unwrap_or_default();
                                            ui.label(
                                                RichText::new(dir)
                                                    .size(10.0)
                                                    .color(Color32::from_rgb(140, 145, 160)),
                                            );
                                        },
                                    );
                                });

                                // ── Selector de pista de audio ────────────
                                if !archivo.pistas.is_empty() {
                                    ui.horizontal(|ui| {
                                        ui.add_space(30.0); // alinear con el nombre

                                        ui.label(
                                            RichText::new("🎵")
                                                .size(12.0)
                                                .color(Color32::from_rgb(130, 100, 255)),
                                        );

                                        if archivo.pistas.len() == 1 {
                                            // Solo una pista: mostrar info sin combo
                                            let p = &archivo.pistas[0];
                                            ui.label(
                                                RichText::new(etiqueta_pista(p, 0))
                                                    .size(11.0)
                                                    .color(Color32::from_rgb(110, 115, 130)),
                                            );
                                        } else {
                                            // Varias pistas: mostrar ComboBox
                                            ui.label(
                                                RichText::new(format!(
                                                    "{} pistas — audio:",
                                                    archivo.pistas.len()
                                                ))
                                                .size(11.0)
                                                .color(Color32::from_rgb(110, 115, 130)),
                                            );

                                            let sel_label = etiqueta_pista(
                                                &archivo.pistas[archivo.pista_sel],
                                                archivo.pista_sel,
                                            );

                                            egui::ComboBox::from_id_salt(
                                                egui::Id::new(("pista", i)),
                                            )
                                            .selected_text(
                                                RichText::new(sel_label)
                                                    .size(11.0)
                                                    .color(Color32::from_rgb(60, 65, 80)),
                                            )
                                            .width(280.0)
                                            .show_ui(ui, |ui| {
                                                for (j, pista) in
                                                    archivo.pistas.iter().enumerate()
                                                {
                                                    let etiq = etiqueta_pista(pista, j);
                                                    ui.selectable_value(
                                                        &mut archivo.pista_sel,
                                                        j,
                                                        RichText::new(etiq).size(12.0).color(Color32::from_rgb(40, 45, 60)),
                                                    );
                                                }
                                            });
                                        }
                                    });
                                }

                                // ── Mensaje de error inline ───────────────
                                if let Estado::Error(ref msg) = archivo.estado {
                                    ui.label(
                                        RichText::new(msg)
                                            .size(11.0)
                                            .color(Color32::from_rgb(210, 60, 60)),
                                    );
                                }
                            });
                        ui.add_space(3.0);
                    }

                    if let Some(i) = eliminar {
                        self.archivos.remove(i);
                    }
                });
            }

            if !self.ffmpeg_ok {
                ui.separator();
                ui.label(
                    RichText::new("❌ FFmpeg no encontrado. Instálalo con: sudo apt install ffmpeg")
                        .color(Color32::from_rgb(220, 80, 80))
                        .size(13.0),
                );
            }
        });
    }
}
