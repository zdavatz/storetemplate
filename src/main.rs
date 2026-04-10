mod icon_gen;
mod languages;
mod state;
mod widgets;
mod stores;
mod json_output;
mod workflow;

use eframe::egui;

use state::{AppState, Tab};

struct StoreTemplateApp {
    state: AppState,
    frame_count: u64,
    icon_texture: Option<egui::TextureHandle>,
    icon_texture_path: String,
    icon_needs_reload: bool,
}

impl StoreTemplateApp {
    fn new() -> Self {
        let mut state = AppState::new();

        // Try to restore the most recently saved state
        if let Some(saved) = state::auto_load_latest() {
            state.load_from_saved(saved);
        }
        state.last_saved_name = state.common.app_name.clone();

        Self {
            state,
            frame_count: 0,
            icon_texture: None,
            icon_texture_path: String::new(),
            icon_needs_reload: false,
        }
    }
}

impl eframe::App for StoreTemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;

        // Auto-save every ~120 frames (~2 seconds)
        if self.frame_count % 120 == 0 {
            state::auto_save(&self.state);
            self.state.last_saved_name = self.state.common.app_name.clone();
        }

        // Poll icon generation result
        if let Some(ref rx) = self.state.icon_gen_receiver {
            if let Ok(status) = rx.try_recv() {
                match status {
                    icon_gen::IconGenStatus::Generating => {
                        self.state.icon_gen_status = Some("Generating icon...".to_string());
                    }
                    icon_gen::IconGenStatus::Done(path) => {
                        self.state.common.app_icon_path = path.clone();
                        self.state.icon_gen_status = Some(format!("Icon saved: {}", path));
                        self.state.icon_gen_receiver = None;
                        self.icon_needs_reload = true;
                    }
                    icon_gen::IconGenStatus::Error(e) => {
                        self.state.icon_gen_status = Some(format!("Error: {}", e));
                        self.state.icon_gen_receiver = None;
                    }
                }
                ctx.request_repaint();
            }
        }

        // Load/reload icon texture when the path changes or a new icon was generated
        let icon_path = &self.state.common.app_icon_path;
        let needs_load = (!icon_path.is_empty() && *icon_path != self.icon_texture_path)
            || self.icon_needs_reload;
        if needs_load && !icon_path.is_empty() {
            self.icon_needs_reload = false;
            if let Ok(img_data) = std::fs::read(icon_path) {
                if let Ok(img) = image::load_from_memory(&img_data) {
                    let rgba = img.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    let pixels = rgba.into_raw();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                    self.icon_texture = Some(ctx.load_texture(
                        "app_icon",
                        color_image,
                        egui::TextureOptions::LINEAR,
                    ));
                    self.icon_texture_path = icon_path.clone();
                }
            }
        } else if icon_path.is_empty() {
            self.icon_texture = None;
            self.icon_texture_path.clear();
        }

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.heading("storetemplate — App Store Submission Template Generator");
            ui.add_space(4.0);

            // Store selection
            ui.horizontal(|ui| {
                ui.label("Stores:");
                ui.checkbox(&mut self.state.store_macos, "macOS");
                ui.checkbox(&mut self.state.store_ios, "iOS");
                ui.checkbox(&mut self.state.store_windows, "Windows");
                ui.checkbox(&mut self.state.store_android, "Android");
                ui.checkbox(&mut self.state.store_github, "GitHub");
            });

            // Language selection
            ui.horizontal_wrapped(|ui| {
                ui.label("Languages:");
                let mut changed = false;
                for (i, (code, name)) in languages::LANGUAGES.iter().enumerate() {
                    if i < self.state.lang_selected.len() {
                        if ui.checkbox(&mut self.state.lang_selected[i], format!("{} ({})", code, name)).changed() {
                            changed = true;
                        }
                    }
                }
                if changed {
                    self.state.update_active_languages();
                }
            });

            ui.add_space(4.0);

            // Tab bar with white background
            egui::Frame::new()
                .fill(egui::Color32::WHITE)
                .inner_margin(egui::Margin::symmetric(6, 4))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.state.active_tab, Tab::Common, "Common");
                        if self.state.has_apple() {
                            ui.selectable_value(&mut self.state.active_tab, Tab::Apple, "Apple");
                        }
                        if self.state.store_android {
                            ui.selectable_value(&mut self.state.active_tab, Tab::Android, "Android");
                        }
                        if self.state.store_windows {
                            ui.selectable_value(&mut self.state.active_tab, Tab::Windows, "Windows");
                        }
                        if self.state.store_github {
                            ui.selectable_value(&mut self.state.active_tab, Tab::GitHub, "GitHub");
                        }
                    });
                });
            ui.add_space(2.0);
        });

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.add_space(4.0);

            // Status / errors
            if let Some(ref status) = self.state.save_status {
                ui.colored_label(egui::Color32::GREEN, status);
            }
            if !self.state.validation_errors.is_empty() {
                for err in &self.state.validation_errors {
                    ui.colored_label(egui::Color32::RED, format!("• {}", err));
                }
            }

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    self.state.validation_errors = json_output::validate(&self.state);
                    if self.state.validation_errors.is_empty() {
                        match json_output::save_to_file(&self.state) {
                            Ok(path) => {
                                self.state.save_status = Some(format!("Saved to: {}", path.display()));
                            }
                            Err(e) => {
                                self.state.save_status = Some(format!("Error: {}", e));
                            }
                        }
                    }
                }
                if ui.button("Load").clicked() {
                    if let Some(saved) = state::load_from_file_dialog() {
                        self.state.load_from_saved(saved);
                        self.icon_needs_reload = true;
                        self.state.save_status = Some("Loaded successfully.".to_string());
                    }
                }
                if ui.button("Clear").clicked() {
                    let old_lang = self.state.lang_selected.clone();
                    self.state = AppState::new();
                    self.state.lang_selected = old_lang;
                    self.state.update_active_languages();
                    self.icon_texture = None;
                    self.icon_texture_path.clear();
                }
            });
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if !self.state.any_store_selected() {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        ui.heading("Select at least one store above to begin.");
                    });
                    return;
                }

                let langs = self.state.active_languages.clone();

                match self.state.active_tab {
                    Tab::Common => {
                        stores::common::ui_section(
                            ui,
                            &mut self.state.common,
                            &langs,
                            &mut self.state.icon_gen_receiver,
                            &mut self.state.icon_gen_status,
                            self.icon_texture.as_ref(),
                        );
                    }
                    Tab::Apple => {
                        let has_macos = self.state.store_macos;
                        let has_ios = self.state.store_ios;
                        let app_name = self.state.common.app_name.clone();
                        stores::apple::ui_section(ui, &mut self.state.apple, &langs, has_macos, has_ios, &app_name);
                    }
                    Tab::Android => {
                        stores::google_play::ui_section(ui, &mut self.state.google_play, &langs);
                    }
                    Tab::Windows => {
                        stores::microsoft::ui_section(ui, &mut self.state.microsoft, &langs);
                    }
                    Tab::GitHub => {
                        stores::github::ui_section(ui, &mut self.state.github);
                    }
                }

                ui.add_space(20.0);
            });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        state::auto_save(&self.state);
    }
}

fn load_app_icon() -> Option<egui::IconData> {
    let icon_path = concat!(env!("CARGO_MANIFEST_DIR"), "/png/Storetemplate_icon_1775851683.png");
    let icon_data = std::fs::read(icon_path).ok()?;
    let img = image::load_from_memory(&icon_data).ok()?;
    let rgba = img.to_rgba8();
    Some(egui::IconData {
        width: rgba.width(),
        height: rgba.height(),
        rgba: rgba.into_raw(),
    })
}

fn main() -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1024.0, 768.0])
        .with_min_inner_size([800.0, 600.0]);

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "storetemplate",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::light());

            Ok(Box::new(StoreTemplateApp::new()))
        }),
    )
}
