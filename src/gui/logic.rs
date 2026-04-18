use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;
use eframe::egui;
use rfd::FileDialog;

use super::ConvApp;
use super::state::{Archivo, Estado, Msg};
use crate::core::{obtener_pistas, elegir_pista_defecto, convertir_archivo, descargar_youtube, TipoConversion};
use std::path::PathBuf;

impl ConvApp {
    pub(crate) fn seleccionar_directorio_salida(&mut self) {
        if let Some(ruta) = FileDialog::new().pick_folder() {
            self.directorio_salida = Some(ruta);
        }
    }

    pub(crate) fn anadir_desde_youtube(&mut self, ctx: &egui::Context) {
        let url = self.youtube_url.trim().to_string();
        if url.is_empty() { return; }

        let tipo = self.tipo_conversion;
        let opciones = self.opciones_video;
        let solo_audio = tipo == TipoConversion::AudioMP3;

        self.archivos.push(Archivo {
            ruta: PathBuf::from(if solo_audio { "YouTube (Audio...)" } else { "YouTube (Vídeo...)" }),
            estado: Estado::Convirtiendo,
            seleccionado: false,
            pistas: vec![],
            pista_sel: 0,
        });
        
        let idx = self.archivos.len() - 1;
        self.convirtiendo    = true;
        self.progreso        = (0, 1);
        self.progreso_actual = 0.0;
        self.cancelar.store(false, Ordering::Relaxed);
        self.log.push((true, format!("🌍 Iniciando YouTube: {}", url)));

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        let ctx2 = ctx.clone();
        let cancel_flag = self.cancelar.clone();
        
        let destino = self.directorio_salida.clone().unwrap_or_else(|| {
            dirs::download_dir().unwrap_or_else(|| {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            })
        });

        thread::spawn(move || {
            let tx2 = tx.clone();
            let ctx3 = ctx2.clone();
            
            // 1. Descarga
            let res_descarga = descargar_youtube(&url, &destino, solo_audio, cancel_flag.clone(), |update| {
                match update {
                    crate::core::ProgressUpdate::Ratio(ratio) => {
                        let _ = tx2.send(Msg::Progreso(idx, ratio * 0.5)); // 50% para descarga
                    }
                    crate::core::ProgressUpdate::Playlist(cur, tot) => {
                        let _ = tx2.send(Msg::PlaylistProgress(idx, cur, tot));
                    }
                }
                ctx3.request_repaint();
            });

            match res_descarga {
                Ok(ruta_descargada) => {
                    if solo_audio {
                        let msg = format!("✅ Audio descargado: {}", ruta_descargada.file_name().unwrap_or_default().to_string_lossy());
                        let _ = tx.send(Msg::Resultado(idx, true, msg));
                    } else {
                        // 2. Conversión
                        let _ = tx.send(Msg::Progreso(idx, 0.5));
                        let _ = tx.send(Msg::Iniciando(idx));
                        
                        let ext = if tipo == TipoConversion::VideoH265 { "mkv" } else { "mkv" };
                        let stem = ruta_descargada.file_stem().unwrap_or_default().to_string_lossy();
                        let final_destino = destino.join(format!("{}.{}", stem, ext));

                        let res_conv = convertir_archivo(
                            &ruta_descargada,
                            Some(&final_destino),
                            0,    // Pista defecto
                            true, // Sobreescribir si existe
                            tipo,
                            opciones,
                            cancel_flag,
                            |update| {
                                if let crate::core::ProgressUpdate::Ratio(ratio) = update {
                                    let _ = tx2.send(Msg::Progreso(idx, 0.5 + (ratio * 0.5))); // Otros 50%
                                    ctx3.request_repaint();
                                }
                            }
                        );

                        match res_conv {
                            Ok(m) => {
                                let _ = tx.send(Msg::Resultado(idx, true, format!("✅ YouTube → {}", m)));
                            }
                            Err(e) => {
                                let _ = tx.send(Msg::Resultado(idx, false, format!("❌ Error conversión: {}", e)));
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Msg::Resultado(idx, false, e));
                }
            }
            let _ = tx.send(Msg::Finalizado);
            ctx2.request_repaint();
        });
        
        self.youtube_url.clear();
    }
    pub(crate) fn anadir_archivos(&mut self) {
        let Some(rutas) = FileDialog::new()
            .add_filter("Vídeos soportados", &["mkv", "mp4", "avi", "MKV", "MP4", "AVI"])
            .pick_files()
        else {
            return;
        };

        let mut nuevos = 0usize;
        for ruta in rutas {
            if self.registrar_archivo_si_valido(ruta) {
                nuevos += 1;
            }
        }
        if nuevos > 0 {
            self.log.push((true, format!("📂 {} archivo(s) añadido(s)", nuevos)));
        }
    }

    pub(crate) fn anadir_carpeta(&mut self) {
        let Some(ruta_base) = FileDialog::new().pick_folder() else {
            return;
        };

        let mut nuevos = 0usize;
        if let Ok(entradas) = std::fs::read_dir(ruta_base) {
            for entrada in entradas.flatten() {
                let ruta = entrada.path();
                if ruta.is_file() {
                    if let Some(ext) = ruta.extension() {
                        let ext = ext.to_string_lossy().to_lowercase();
                        if matches!(ext.as_str(), "mkv" | "mp4" | "avi") {
                            if self.registrar_archivo_si_valido(ruta) {
                                nuevos += 1;
                            }
                        }
                    }
                }
            }
        }

        if nuevos > 0 {
            self.log.push((true, format!("📂 Carpeta procesada: {} archivo(s) nuevo(s)", nuevos)));
        } else {
            self.log.push((false, "⚠ No se encontraron archivos de vídeo compatibles en la carpeta.".into()));
        }
    }

    fn registrar_archivo_si_valido(&mut self, ruta: std::path::PathBuf) -> bool {
        if !self.archivos.iter().any(|a| a.ruta == ruta) {
            let pistas    = obtener_pistas(&ruta.to_string_lossy());
            let pista_sel = elegir_pista_defecto(&pistas);
            self.archivos.push(Archivo {
                ruta,
                estado: Estado::Pendiente,
                seleccionado: true,
                pistas,
                pista_sel,
            });
            return true;
        }
        false
    }


    pub(crate) fn iniciar_conversion(&mut self, ctx: &egui::Context) {
        let mut pendientes = Vec::new();

        for (i, a) in self.archivos.iter().enumerate() {
            if a.seleccionado && a.estado == Estado::Pendiente {
                let stream = a.pistas.get(a.pista_sel)
                    .map(|p| p.indice_stream)
                    .unwrap_or(0);
                
                let ext = match self.tipo_conversion {
                    TipoConversion::AudioMP3 => "mp3",
                    _ => "mkv",
                };

                let stem = a.ruta.file_stem().unwrap_or_default().to_string_lossy();
                
                let destino_dir = self.directorio_salida.clone().unwrap_or_else(|| {
                    a.ruta.parent().unwrap_or(Path::new(".")).to_path_buf()
                });
                let destino_path = destino_dir.join(format!("{}.{}", stem, ext));
                
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
                        continue;
                    }
                }
                pendientes.push((i, a.ruta.clone(), destino_path, stream, sobreescribir));
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
        let tipo = self.tipo_conversion;
        let opciones = self.opciones_video;

        thread::spawn(move || {
            for (idx, ruta, destino_path, stream, sobreescribir) in pendientes {
                let _ = tx.send(Msg::Iniciando(idx));
                ctx2.request_repaint();

                let tx2  = tx.clone();
                let ctx3 = ctx2.clone();
                let cancel_clone = cancel_flag.clone();
                let (ok, msg) = match convertir_archivo(
                    &ruta,
                    Some(&destino_path),
                    stream,
                    sobreescribir,
                    tipo,
                    opciones,
                    cancel_clone,
                    move |update| {
                        if let crate::core::ProgressUpdate::Ratio(ratio) = update {
                            let _ = tx2.send(Msg::Progreso(idx, ratio));
                            ctx3.request_repaint();
                        }
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

    pub(crate) fn procesar_mensajes(&mut self) {
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
                Msg::PlaylistProgress(_idx, actual, total) => {
                    // Si detectamos una lista, ajustamos el total global
                    // para que refleje los items de la lista.
                    self.progreso.1 = total;
                    self.progreso.0 = actual - 1;
                    self.progreso_actual = 0.0;
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

    pub(crate) fn manejar_drop(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            for f in &i.raw.dropped_files {
                if let Some(ruta) = &f.path {
                    if let Some(ext) = ruta.extension() {
                        let ext = ext.to_string_lossy().to_lowercase();
                        if (ext == "mkv" || ext == "mp4" || ext == "avi")
                            && !self.archivos.iter().any(|a| &a.ruta == ruta)
                        {
                            let nombre = ruta.file_name().unwrap_or_default().to_string_lossy().to_string();
                            let pistas    = obtener_pistas(&ruta.to_string_lossy());
                            let pista_sel = elegir_pista_defecto(&pistas);
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
