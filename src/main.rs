mod languages;
mod state;
mod widgets;
mod stores;
mod json_output;

use eframe::egui;

use state::{AppState, Tab};

struct StoreTemplateApp {
    state: AppState,
}

impl StoreTemplateApp {
    fn new() -> Self {
        Self {
            state: AppState::new(),
        }
    }
}

impl eframe::App for StoreTemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

            // Tab bar
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
                if ui.button("Save Template").clicked() {
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
                if ui.button("Clear All").clicked() {
                    let old_lang = self.state.lang_selected.clone();
                    self.state = AppState::new();
                    self.state.lang_selected = old_lang;
                    self.state.update_active_languages();
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
                        stores::common::ui_section(ui, &mut self.state.common, &langs);
                    }
                    Tab::Apple => {
                        let has_macos = self.state.store_macos;
                        let has_ios = self.state.store_ios;
                        stores::apple::ui_section(ui, &mut self.state.apple, &langs, has_macos, has_ios);
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
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "storetemplate",
        options,
        Box::new(|_cc| Ok(Box::new(StoreTemplateApp::new()))),
    )
}
