use std::sync::atomic::Ordering;
use eframe::egui::{self, Color32, Margin, RichText, ScrollArea, Stroke};
use crate::gui::ConvApp;
use crate::gui::state::Status;
use crate::gui::helpers::{cr, get_track_label};
use super::widgets::{tag, render_hw_tags};

/// Renders the central workspace of the application.
pub fn render(app: &mut ConvApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    egui::CentralPanel::default().show_inside(ui, |ui| {
        ui.add_space(4.0);

        let is_converting = app.is_converting;

        // --- SECTION: Action Toolbar ---
        ui.horizontal(|ui| {
            if ui.add_enabled(!is_converting, egui::Button::new("📂  Add files").fill(Color32::from_rgb(225, 230, 240)))
                .on_hover_text("Select MKV, MP4, or AVI files")
                .clicked()
            {
                app.add_files();
            }

            if ui.add_enabled(!is_converting, egui::Button::new("📁  Add folder").fill(Color32::from_rgb(225, 230, 240)))
                .on_hover_text("Add all videos from a directory")
                .clicked()
            {
                app.add_folder();
            }

            // --- Primary Action: Convert / Stop ---
            if is_converting {
                if ui.add(egui::Button::new(RichText::new("⏹  Stop").color(Color32::WHITE)).fill(Color32::from_rgb(220, 60, 60)))
                    .on_hover_text("Cancel current operation")
                    .clicked()
                {
                    app.cancel.store(true, Ordering::Relaxed);
                }
            } else {
                let has_pending = app.files.iter().any(|a| a.selected && a.status == Status::Pending);
                if ui.add_enabled(
                        app.ffmpeg_ok && has_pending,
                        egui::Button::new(RichText::new("▶  Convert").color(Color32::WHITE)).fill(Color32::from_rgb(115, 85, 225)),
                    )
                    .on_hover_text("Process selected files")
                    .clicked()
                {
                    app.start_conversion(ctx);
                }
            }

            if ui.add_enabled(!is_converting, egui::Button::new("🗑  Clear").fill(Color32::from_rgb(240, 210, 210)))
                .on_hover_text("Empty list and log")
                .clicked()
            {
                app.files.clear();
                app.log.clear();
                app.progress = (0, 0);
            }

            ui.separator();

            // --- Selection Helpers ---
            if ui.add_enabled(!is_converting, egui::Button::new("✔ All")).clicked() {
                for a in &mut app.files { a.selected = true; }
            }
            if ui.add_enabled(!is_converting, egui::Button::new("✗ None")).clicked() {
                for a in &mut app.files { a.selected = false; }
            }

            // --- Stats display ---
            let total = app.files.len();
            let selected_count = app.files.iter().filter(|a| a.selected).count();
            if total > 0 {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{} / {} selected", selected_count, total)).size(16.0).color(Color32::from_rgb(110, 115, 130)));
                });
            }
        });

        ui.add_space(8.0);

        // --- SECTION: YouTube Integration ---
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
                    let btn = ui.add_enabled(
                        ytdlp_ok && !app.youtube_url.is_empty(),
                        egui::Button::new(RichText::new("➕ Add to list").color(Color32::WHITE)).fill(Color32::from_rgb(115, 85, 225))
                    );

                    if btn.clicked() {
                        app.add_from_youtube(ctx);
                    }

                    if !ytdlp_ok {
                        ui.label(RichText::new("⚠ yt-dlp not found").color(Color32::RED).size(14.0));
                    }
                });
            });

        ui.add_space(8.0);

        // --- SECTION: Global Conversion Settings ---
        egui::Frame::new()
            .fill(Color32::from_rgb(245, 247, 250))
            .corner_radius(cr(8))
            .inner_margin(Margin::same(10_i8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Conversion Mode:").strong().size(18.0));
                    
                    use crate::core::ConversionType;
                    let type_text = match app.conversion_type {
                        ConversionType::AudioMP3  => "🎵 Audio (MP3)",
                        ConversionType::VideoMKV  => "🎬 Video (MKV - Remux/Copy)",
                        ConversionType::VideoH264 => "🎬 Video (H.264 - Best compatibility)",
                        ConversionType::VideoH265 => "🎬 Video (H.265 - Best compression)",
                    };
 
                     egui::ComboBox::from_id_salt("conv_type")
                         .selected_text(RichText::new(type_text).size(18.0).color(Color32::from_rgb(45, 50, 65)))
                         .width(320.0)
                         .show_ui(ui, |ui| {
                             let mut sel_val = |ui: &mut egui::Ui, val: ConversionType, text: &str| {
                                 let is_selected = app.conversion_type == val;
                                 let col = if is_selected { Color32::WHITE } else { Color32::from_rgb(45, 50, 65) };
                                 ui.selectable_value(&mut app.conversion_type, val, RichText::new(text).color(col));
                             };
 
                             sel_val(ui, ConversionType::AudioMP3, "🎵 Audio (MP3)");
                             sel_val(ui, ConversionType::VideoMKV, "🎬 Video (MKV - Remux/Copy)");
                             sel_val(ui, ConversionType::VideoH264, "🎬 Video (H.264 - Best compatibility)");
                             sel_val(ui, ConversionType::VideoH265, "🎬 Video (H.265 - Best compression)");
                         });

                    // Conditional video encoding flags
                    if app.conversion_type != ConversionType::AudioMP3 && app.conversion_type != ConversionType::VideoMKV {
                        ui.separator();
                        ui.checkbox(&mut app.video_options.preserve_grain, "🌑 Preserve Grain");
                        ui.checkbox(&mut app.video_options.optimize_color, "🎨 Optimize Color (BT.709)");
                    }
                    
                    ui.separator();
                    
                    // Hardware Acceleration Selector
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Acceleration:").size(16.0));
                        render_hw_tags(ui, &app.capabilities);
                    });
                    
                    use crate::core::HWAcceleration;
                    let accel_text = match app.video_options.acceleration {
                        HWAcceleration::None => "❌ CPU Only",
                        HWAcceleration::NVENC   => "🚀 NVIDIA (NVENC)",
                        HWAcceleration::QSV     => "⚡ Intel (QSV)",
                        HWAcceleration::AMF     => "🏎 AMD (AMF)",
                        HWAcceleration::VAAPI   => "🐧 Linux (VAAPI)",
                        HWAcceleration::VideoToolbox => "🍎 Apple (VTB)",
                    };

                    let hw_enabled = app.conversion_type != ConversionType::VideoMKV && app.conversion_type != ConversionType::AudioMP3;
                    ui.add_enabled_ui(hw_enabled, |ui| {
                        egui::ComboBox::from_id_salt("hw_accel")
                            .selected_text(RichText::new(accel_text).size(16.0).color(Color32::from_rgb(60, 65, 80)))
                            .width(160.0)
                            .show_ui(ui, |ui| {
                                let mut hw_button = |ui: &mut egui::Ui, val: HWAcceleration, label: &str, detected: bool| {
                                    let is_selected = app.video_options.acceleration == val;
                                    let col = if is_selected { Color32::WHITE } else { Color32::from_rgb(45, 50, 65) };
                                    
                                    ui.add_enabled_ui(detected, |ui| {
                                        let res = ui.selectable_value(&mut app.video_options.acceleration, val, RichText::new(label).color(col));
                                        if !detected {
                                            res.on_hover_text("This hardware was not detected in your system.");
                                        }
                                    });
                                };

                                hw_button(ui, HWAcceleration::None, "❌ CPU Only", true);
                                hw_button(ui, HWAcceleration::NVENC,   "🚀 NVIDIA (NVENC)", app.capabilities.nvenc);
                                hw_button(ui, HWAcceleration::QSV,     "⚡ Intel (QSV)",     app.capabilities.qsv);
                                hw_button(ui, HWAcceleration::AMF,     "🏎 AMD (AMF)",       app.capabilities.amf);
                                hw_button(ui, HWAcceleration::VAAPI,   "🐧 Linux (VAAPI)",   app.capabilities.vaapi);
                                hw_button(ui, HWAcceleration::VideoToolbox, "🍎 Apple (VTB)", app.capabilities.vtb);
                            });
                    });
                });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // --- Output Directory Configuration ---
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Output Directory:").strong().size(17.0));
                    
                    let dir_text = match &app.output_directory {
                        Some(p) => p.to_string_lossy().to_string(),
                        None => "Default (Source folder / Downloads)".to_string(),
                    };

                    egui::Frame::new()
                        .fill(Color32::from_rgb(230, 235, 245))
                        .corner_radius(cr(4))
                        .inner_margin(Margin::symmetric(8_i8, 4_i8))
                        .show(ui, |ui| {
                            ui.label(RichText::new(dir_text).size(15.0).color(Color32::from_rgb(60, 70, 90)));
                        });

                    if ui.button("📂 Change").clicked() {
                        app.select_output_directory();
                    }

                    if app.output_directory.is_some() {
                        if ui.button("↺ Reset").on_hover_text("Revert to default directory").clicked() {
                            app.output_directory = None;
                        }
                    }
                });
            });

        ui.add_space(8.0);

        // --- SECTION: Queue List ---
        if app.files.is_empty() {
            // Empty state placeholder
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Drag and drop files here\nor use «Add files»").size(22.0).color(Color32::from_rgb(140, 145, 165)));
            });
        } else {
            ScrollArea::vertical().show(ui, |ui| {
                let mut remove_idx: Option<usize> = None;

                for (i, file) in app.files.iter_mut().enumerate() {
                    egui::Frame::new()
                        .fill(Color32::WHITE)
                        .corner_radius(cr(6))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(225, 230, 235)))
                        .inner_margin(Margin::symmetric(10_i8, 7_i8))
                        .show(ui, |ui| {

                            // --- Item Header (Checkbox, Icon, Filename) ---
                            ui.horizontal(|ui| {
                                ui.add_enabled(!is_converting, egui::Checkbox::new(&mut file.selected, ""));
                                ui.label(RichText::new(file.status.icon()).size(21.0).color(file.status.color()));
                                let name = file.path.file_name().unwrap_or_default().to_string_lossy();
                                ui.label(RichText::new(name.as_ref()).size(19.0).color(Color32::from_rgb(45, 50, 65)));

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if !is_converting && ui.small_button(RichText::new("✕").color(Color32::from_rgb(200, 80, 80))).on_hover_text("Remove from list").clicked() {
                                        remove_idx = Some(i);
                                    }
                                    let dir = file.path.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                                    ui.label(RichText::new(dir).size(14.0).color(Color32::from_rgb(140, 145, 160)));
                                });
                            });

                            // --- Media Capability Tags ---
                            if let Some(info) = &file.info {
                                ui.horizontal(|ui| {
                                    ui.add_space(34.0);
                                    ui.spacing_mut().item_spacing.x = 4.0;
                                    tag(ui, &info.container.to_uppercase(), Color32::from_rgb(100, 110, 130));
                                    if let Some(vc) = &info.v_codec {
                                        tag(ui, &vc.to_uppercase(), Color32::from_rgb(60, 120, 200));
                                    }
                                    if let Some(ac) = &info.a_codec {
                                        tag(ui, &ac.to_uppercase(), Color32::from_rgb(180, 100, 40));
                                    }
                                });
                            }

                            // --- Audio Track Selection ---
                            if !file.tracks.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.add_space(30.0);
                                    ui.label(RichText::new("🎵").size(16.0).color(Color32::from_rgb(130, 100, 255)));

                                    if file.tracks.len() == 1 {
                                        let t = &file.tracks[0];
                                        ui.label(RichText::new(get_track_label(t, 0)).size(15.0).color(Color32::from_rgb(110, 115, 130)));
                                    } else {
                                        ui.label(RichText::new(format!("{} tracks — audio:", file.tracks.len())).size(15.0).color(Color32::from_rgb(110, 115, 130)));
                                        let selected_label = get_track_label(&file.tracks[file.selected_track], file.selected_track);
                                        
                                        egui::ComboBox::from_id_salt(egui::Id::new(("track", i)))
                                            .selected_text(RichText::new(selected_label).size(15.0).color(Color32::from_rgb(60, 65, 80)))
                                            .width(280.0)
                                            .show_ui(ui, |ui| {
                                                for (j, track) in file.tracks.iter().enumerate() {
                                                    let label = get_track_label(track, j);
                                                    let is_selected = file.selected_track == j;
                                                    let col = if is_selected { Color32::WHITE } else { Color32::from_rgb(45, 50, 65) };
                                                    ui.selectable_value(&mut file.selected_track, j, RichText::new(label).size(14.0).color(col));
                                                }
                                            });
                                    }
                                });
                            }

                            // --- Error Messaging ---
                            if let Status::Error(ref msg) = file.status {
                                ui.label(RichText::new(msg).size(13.0).color(Color32::from_rgb(210, 60, 60)));
                            }
                        });
                    ui.add_space(3.0);
                }

                if let Some(i) = remove_idx {
                    app.files.remove(i);
                }
            });
        }

        // --- System Warnings ---
        if !app.ffmpeg_ok {
            ui.separator();
            ui.label(RichText::new("❌ FFmpeg not found. Install it via: sudo apt install ffmpeg").color(Color32::from_rgb(220, 80, 80)).size(18.0));
        }
    });
}
