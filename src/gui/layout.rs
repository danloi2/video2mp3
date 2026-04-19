use std::sync::atomic::Ordering;
use std::time::Duration;
use eframe::egui::{self, Color32, Margin, RichText, ScrollArea, Stroke};

use super::ConvApp;
use super::state::Estado;
use super::helpers::{cr, etiqueta_pista};

pub(crate) fn render_ui(app: &mut ConvApp, ui: &mut egui::Ui) {
    app.procesar_mensajes();
    
    let ctx = ui.ctx().clone();
    app.manejar_drop(&ctx);

    if app.convirtiendo {
        ctx.request_repaint_after(Duration::from_millis(100));
    }

    render_top_panel(app, ui, &ctx);
    render_bottom_panel(app, ui);
    render_central_panel(app, ui, &ctx);
}

fn render_top_panel(app: &mut ConvApp, ui: &mut egui::Ui, _ctx: &egui::Context) {
    egui::Panel::top("cabecera")
        .exact_size(48.0)
        .show_inside(ui, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(6.0);
                
                if let Some(tex) = &app.logo_texture {
                    ui.add(egui::Image::new(tex).fit_to_exact_size(egui::vec2(32.0, 32.0)));
                    ui.add_space(4.0);
                } else {
                    ui.label(
                        RichText::new("🎵")
                            .size(26.0)
                            .color(Color32::from_rgb(105, 75, 215))
                    );
                }
                
                ui.label(
                    RichText::new("video2mp3")
                        .size(26.0)
                        .color(Color32::from_rgb(105, 75, 215))
                        .strong(),
                );
                ui.label(
                    RichText::new("— Conversor de vídeo a MP3 con ffmpeg y yt-dlp")
                        .size(18.0)
                        .color(Color32::from_rgb(100, 105, 120)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(12.0);
                    let (punto, txt, col) = if app.ffmpeg_ok {
                        ("●", format!(" FFmpeg {}", app.ffmpeg_version), Color32::from_rgb(45, 175, 80))
                    } else {
                        ("●", " Sin ffmpeg".to_string(), Color32::from_rgb(220, 50, 50))
                    };
                    ui.label(RichText::new(format!("{}{}", punto, txt)).color(col).size(16.0));

                    ui.add_space(8.0);

                    let (punto, txt, col) = if app.ytdlp_ok {
                        ("●", " yt-dlp".to_string(), Color32::from_rgb(45, 175, 80))
                    } else {
                        ("●", " Sin yt-dlp".to_string(), Color32::from_rgb(220, 50, 50))
                    };
                    ui.label(RichText::new(format!("{}{}", punto, txt)).color(col).size(16.0));

                    ui.add_space(8.0);
                    render_hw_tags(ui, &app.capacidades);
                });
            });
        });
}

fn render_hw_tags(ui: &mut egui::Ui, c: &crate::core::CapacidadesHardware) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        
        tag(ui, "CPU", Color32::from_rgb(140, 145, 155));
        if c.nvenc { tag(ui, "NVENC", Color32::from_rgb(118, 185, 0)); }
        if c.qsv   { tag(ui, "QSV",   Color32::from_rgb(0, 104, 181)); }
        if c.amf   { tag(ui, "AMF",   Color32::from_rgb(237, 28, 36)); }
        if c.vaapi { tag(ui, "VAAPI", Color32::from_rgb(255, 140, 0)); }
        if c.vtb   { tag(ui, "APPLE", Color32::from_rgb(160, 160, 160)); }
    });
}

fn tag(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.15))
        .stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.5)))
        .corner_radius(cr(4))
        .inner_margin(Margin::symmetric(6_i8, 2_i8))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(12.0).color(color).strong());
        });
}

fn render_bottom_panel(app: &mut ConvApp, ui: &mut egui::Ui) {
    egui::Panel::bottom("panel_log")
        .min_size(160.0)
        .max_size(280.0)
        .resizable(true)
        .show_inside(ui, |ui| {
            ui.add_space(4.0);

            if app.progreso.1 > 0 {
                let ratio = (app.progreso.0 as f32 + app.progreso_actual) / app.progreso.1 as f32;
                let texto = format!(
                    "{} de {} archivo(s)  —  {:.0}%",
                    app.progreso.0, app.progreso.1, ratio * 100.0
                );

                let bar_h = 22.0;
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), bar_h),
                    egui::Sense::hover(),
                );
                if ui.is_rect_visible(rect) {
                    let paint = ui.painter();
                    let cr5 = cr(5);
                    paint.rect_filled(rect, cr5, Color32::from_rgb(215, 220, 230));
                    if ratio > 0.0 {
                        let fill_w = (rect.width() * ratio).clamp(0.0, rect.width());
                        let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_w, bar_h));
                        paint.rect_filled(fill_rect, cr5, Color32::from_rgb(115, 85, 225));
                    }
                    
                    let text_color = Color32::from_rgb(255, 255, 255);
                    paint.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &texto,
                        egui::FontId::proportional(18.0),
                        if ratio > 0.45 { text_color } else { Color32::from_rgb(90, 95, 110) },
                    );
                }
                ui.add_space(4.0);
            }

            ui.label(
                RichText::new("📋  Registro")
                    .size(15.0)
                    .color(Color32::from_rgb(120, 125, 140)),
            );
            ui.separator();

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for (ok, linea) in &app.log {
                        let col = if *ok {
                            Color32::from_rgb(60, 65, 80)
                        } else {
                            Color32::from_rgb(210, 50, 50)
                        };
                        ui.label(RichText::new(linea).monospace().size(16.0).color(col));
                    }
                });
        });
}

fn render_central_panel(app: &mut ConvApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    egui::CentralPanel::default().show_inside(ui, |ui| {
        ui.add_space(4.0);

        let convirtiendo = app.convirtiendo;

        ui.horizontal(|ui| {
            if ui.add_enabled(!convirtiendo, egui::Button::new("📂  Añadir archivos").fill(Color32::from_rgb(225, 230, 240)))
                .on_hover_text("Seleccionar archivos MKV, MP4 o AVI")
                .clicked()
            {
                app.anadir_archivos();
            }

            if ui.add_enabled(!convirtiendo, egui::Button::new("📁  Añadir carpeta").fill(Color32::from_rgb(225, 230, 240)))
                .on_hover_text("Añadir todos los vídeos de una carpeta")
                .clicked()
            {
                app.anadir_carpeta();
            }

            if convirtiendo {
                if ui.add(egui::Button::new(RichText::new("⏹  Detener").color(Color32::from_rgb(255, 255, 255))).fill(Color32::from_rgb(220, 60, 60)))
                    .on_hover_text("Cancelar la conversión actual")
                    .clicked()
                {
                    app.cancelar.store(true, Ordering::Relaxed);
                }
            } else {
                let hay_pendientes = app.archivos.iter().any(|a| a.seleccionado && a.estado == Estado::Pendiente);
                if ui.add_enabled(
                        app.ffmpeg_ok && hay_pendientes,
                        egui::Button::new(RichText::new("▶  Convertir").color(Color32::from_rgb(255, 255, 255))).fill(Color32::from_rgb(115, 85, 225)),
                    )
                    .on_hover_text("Convertir los archivos seleccionados")
                    .clicked()
                {
                    app.iniciar_conversion(ctx);
                }
            }

            if ui.add_enabled(!convirtiendo, egui::Button::new("🗑  Limpiar").fill(Color32::from_rgb(240, 210, 210)))
                .on_hover_text("Vaciar lista y registro")
                .clicked()
            {
                app.archivos.clear();
                app.log.clear();
                app.progreso = (0, 0);
            }

            ui.separator();

            if ui.add_enabled(!convirtiendo, egui::Button::new("✔ Todos")).clicked() {
                for a in &mut app.archivos { a.seleccionado = true; }
            }
            if ui.add_enabled(!convirtiendo, egui::Button::new("✗ Ninguno")).clicked() {
                for a in &mut app.archivos { a.seleccionado = false; }
            }

            let total = app.archivos.len();
            let sel   = app.archivos.iter().filter(|a| a.seleccionado).count();
            if total > 0 {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{} / {} seleccionados", sel, total)).size(16.0).color(Color32::from_rgb(110, 115, 130)));
                });
            }
        });

        ui.add_space(8.0);

        // --- Opción YouTube ---
        egui::Frame::new()
            .fill(Color32::from_rgb(240, 245, 255))
            .corner_radius(cr(8))
            .inner_margin(Margin::same(10_i8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("YouTube:").strong().size(18.0));
                    ui.add(
                        egui::TextEdit::singleline(&mut app.youtube_url)
                            .hint_text("https://www.youtube.com/watch?v=...")
                            .desired_width(400.0)
                    );
                    
                    let ytdlp_ok = app.ytdlp_ok;
                    use crate::core::TipoConversion;
                    let (btn_txt, btn_col) = if app.tipo_conversion == TipoConversion::AudioMP3 {
                        ("📥  Descargar MP3", Color32::from_rgb(230, 33, 23))
                    } else {
                        ("📥  Descargar Vídeo", Color32::from_rgb(30, 150, 230))
                    };

                    let btn = ui.add_enabled(
                        ytdlp_ok && !app.convirtiendo && !app.youtube_url.is_empty(),
                        egui::Button::new(RichText::new(btn_txt).color(Color32::WHITE)).fill(btn_col)
                    );

                    if btn.clicked() {
                        app.anadir_desde_youtube(ctx);
                    }

                    if !ytdlp_ok {
                        ui.label(RichText::new("⚠ yt-dlp no encontrado").color(Color32::RED).size(14.0));
                    }
                });
            });

        ui.add_space(8.0);

        // --- Panel de Opciones de Conversión ---
        egui::Frame::new()
            .fill(Color32::from_rgb(245, 247, 250))
            .corner_radius(cr(8))
            .inner_margin(Margin::same(10_i8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Modo de conversión:").strong().size(18.0));
                    
                    use crate::core::TipoConversion;
                    let tipo_text = match app.tipo_conversion {
                        TipoConversion::AudioMP3  => "🎵 Audio (MP3)",
                        TipoConversion::VideoH264 => "🎬 Vídeo (H.264 - Máxima compatibilidad)",
                        TipoConversion::VideoH265 => "🎬 Vídeo (H.265 - Máxima compresión)",
                    };

                    egui::ComboBox::from_id_salt("tipo_conv")
                        .selected_text(RichText::new(tipo_text).size(18.0).color(Color32::from_rgb(45, 50, 65)))
                        .width(320.0)
                        .show_ui(ui, |ui| {
                            let mut sel_val = |ui: &mut egui::Ui, val: TipoConversion, text: &str| {
                                let is_sel = app.tipo_conversion == val;
                                let col = if is_sel { Color32::WHITE } else { Color32::from_rgb(45, 50, 65) };
                                ui.selectable_value(&mut app.tipo_conversion, val, RichText::new(text).color(col));
                            };

                            sel_val(ui, TipoConversion::AudioMP3, "🎵 Audio (MP3)");
                            sel_val(ui, TipoConversion::VideoH264, "🎬 Vídeo (H.264 - Máxima compatibilidad)");
                            sel_val(ui, TipoConversion::VideoH265, "🎬 Vídeo (H.265 - Máxima compresión)");
                        });

                    if app.tipo_conversion != TipoConversion::AudioMP3 {
                        ui.separator();
                        ui.checkbox(&mut app.opciones_video.preservar_grano, "🌑 Preservar grano");
                        ui.checkbox(&mut app.opciones_video.optimizar_color, "🎨 Optimizar color (BT.709)");
                    }
                    
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Aceleración:").size(16.0));
                        render_hw_tags(ui, &app.capacidades);
                    });
                    
                    use crate::core::AceleracionHW;
                    let ac_text = match app.opciones_video.aceleracion {
                        AceleracionHW::Ninguna => "❌ Solo CPU",
                        AceleracionHW::NVENC   => "🚀 NVIDIA (NVENC)",
                        AceleracionHW::QSV     => "⚡ Intel (QSV)",
                        AceleracionHW::AMF     => "🏎 AMD (AMF)",
                        AceleracionHW::VAAPI   => "🐧 Linux (VAAPI)",
                        AceleracionHW::VideoToolbox => "🍎 Apple (VTB)",
                    };

                    egui::ComboBox::from_id_salt("hw_accel")
                        .selected_text(RichText::new(ac_text).size(16.0).color(Color32::from_rgb(60, 65, 80)))
                        .width(160.0)
                        .show_ui(ui, |ui| {
                            let mut hw_button = |ui: &mut egui::Ui, val: AceleracionHW, label: &str, detectado: bool| {
                                let is_sel = app.opciones_video.aceleracion == val;
                                let col = if is_sel { Color32::WHITE } else { Color32::from_rgb(45, 50, 65) };
                                
                                ui.add_enabled_ui(detectado, |ui| {
                                    let res = ui.selectable_value(&mut app.opciones_video.aceleracion, val, RichText::new(label).color(col));
                                    if !detectado {
                                        res.on_hover_text("Este hardware no fue detectado en el sistema.");
                                    }
                                });
                            };

                            hw_button(ui, AceleracionHW::Ninguna, "❌ Solo CPU", true);
                            hw_button(ui, AceleracionHW::NVENC,   "🚀 NVIDIA (NVENC)", app.capacidades.nvenc);
                            hw_button(ui, AceleracionHW::QSV,     "⚡ Intel (QSV)",     app.capacidades.qsv);
                            hw_button(ui, AceleracionHW::AMF,     "🏎 AMD (AMF)",       app.capacidades.amf);
                            hw_button(ui, AceleracionHW::VAAPI,   "🐧 Linux (VAAPI)",   app.capacidades.vaapi);
                            hw_button(ui, AceleracionHW::VideoToolbox, "🍎 Apple (VTB)", app.capacidades.vtb);
                        });
                });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Directorio de salida:").strong().size(17.0));
                    
                    let txt = match &app.directorio_salida {
                        Some(p) => p.to_string_lossy().to_string(),
                        None => "Predeterminado (mismo que origen / Descargas)".to_string(),
                    };

                    egui::Frame::new()
                        .fill(Color32::from_rgb(230, 235, 245))
                        .corner_radius(cr(4))
                        .inner_margin(Margin::symmetric(8_i8, 4_i8))
                        .show(ui, |ui| {
                            ui.label(RichText::new(txt).size(15.0).color(Color32::from_rgb(60, 70, 90)));
                        });

                    if ui.button("📂 Cambiar").clicked() {
                        app.seleccionar_directorio_salida();
                    }

                    if app.directorio_salida.is_some() {
                        if ui.button("↺ Reset").on_hover_text("Volver al directorio predeterminado").clicked() {
                            app.directorio_salida = None;
                        }
                    }
                });
            });

        ui.add_space(8.0);


        if app.archivos.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Arrastra archivos aquí\no usa «Añadir archivos»").size(22.0).color(Color32::from_rgb(140, 145, 165)));
            });
        } else {
            ScrollArea::vertical().show(ui, |ui| {
                let mut eliminar: Option<usize> = None;

                for (i, archivo) in app.archivos.iter_mut().enumerate() {
                    egui::Frame::new()
                        .fill(Color32::from_rgb(255, 255, 255))
                        .corner_radius(cr(6))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(225, 230, 235)))
                        .inner_margin(Margin::symmetric(10_i8, 7_i8))
                        .show(ui, |ui| {

                            ui.horizontal(|ui| {
                                ui.add_enabled(!convirtiendo, egui::Checkbox::new(&mut archivo.seleccionado, ""));
                                ui.label(RichText::new(archivo.estado.icono()).size(21.0).color(archivo.estado.color()));
                                let nombre = archivo.ruta.file_name().unwrap_or_default().to_string_lossy();
                                ui.label(RichText::new(nombre.as_ref()).size(19.0).color(Color32::from_rgb(45, 50, 65)));

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if !convirtiendo && ui.small_button(RichText::new("✕").color(Color32::from_rgb(200, 80, 80))).on_hover_text("Quitar de la lista").clicked() {
                                        eliminar = Some(i);
                                    }
                                    let dir = archivo.ruta.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                                    ui.label(RichText::new(dir).size(14.0).color(Color32::from_rgb(140, 145, 160)));
                                });
                            });

                            if let Some(info) = &archivo.info {
                                ui.horizontal(|ui| {
                                    ui.add_space(34.0);
                                    ui.spacing_mut().item_spacing.x = 4.0;
                                    tag(ui, &info.contenedor.to_uppercase(), Color32::from_rgb(100, 110, 130));
                                    if let Some(vc) = &info.v_codec {
                                        tag(ui, &vc.to_uppercase(), Color32::from_rgb(60, 120, 200));
                                    }
                                    if let Some(ac) = &info.a_codec {
                                        tag(ui, &ac.to_uppercase(), Color32::from_rgb(180, 100, 40));
                                    }
                                });
                            }

                            if !archivo.pistas.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.add_space(30.0);
                                    ui.label(RichText::new("🎵").size(16.0).color(Color32::from_rgb(130, 100, 255)));

                                    if archivo.pistas.len() == 1 {
                                        let p = &archivo.pistas[0];
                                        ui.label(RichText::new(etiqueta_pista(p, 0)).size(15.0).color(Color32::from_rgb(110, 115, 130)));
                                    } else {
                                        ui.label(RichText::new(format!("{} pistas — audio:", archivo.pistas.len())).size(15.0).color(Color32::from_rgb(110, 115, 130)));
                                        let sel_label = etiqueta_pista(&archivo.pistas[archivo.pista_sel], archivo.pista_sel);
                                        
                                        egui::ComboBox::from_id_salt(egui::Id::new(("pista", i)))
                                            .selected_text(RichText::new(sel_label).size(15.0).color(Color32::from_rgb(60, 65, 80)))
                                            .width(280.0)
                                            .show_ui(ui, |ui| {
                                                for (j, pista) in archivo.pistas.iter().enumerate() {
                                                    let etiq = etiqueta_pista(pista, j);
                                                    let is_sel = archivo.pista_sel == j;
                                                    let col = if is_sel { Color32::WHITE } else { Color32::from_rgb(45, 50, 65) };
                                                    ui.selectable_value(&mut archivo.pista_sel, j, RichText::new(etiq).size(14.0).color(col));
                                                }
                                            });
                                    }
                                });
                            }

                            if let Estado::Error(ref msg) = archivo.estado {
                                ui.label(RichText::new(msg).size(13.0).color(Color32::from_rgb(210, 60, 60)));
                            }
                        });
                    ui.add_space(3.0);
                }

                if let Some(i) = eliminar {
                    app.archivos.remove(i);
                }
            });
        }

        if !app.ffmpeg_ok {
            ui.separator();
            ui.label(RichText::new("❌ FFmpeg no encontrado. Instálalo con: sudo apt install ffmpeg").color(Color32::from_rgb(220, 80, 80)).size(18.0));
        }
    });
}
