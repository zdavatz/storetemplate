use std::collections::HashMap;

use eframe::egui;

use crate::languages::language_name;

/// Single-line text field with optional max length and required indicator.
pub fn text_field(ui: &mut egui::Ui, label: &str, value: &mut String, max_len: Option<usize>, required: bool) {
    ui.horizontal(|ui| {
        let label_text = if required {
            format!("{}*", label)
        } else {
            label.to_string()
        };
        ui.label(&label_text);
        ui.text_edit_singleline(value);
        if let Some(max) = max_len {
            let len = value.len();
            let color = if len > max {
                egui::Color32::RED
            } else if len > max * 8 / 10 {
                egui::Color32::YELLOW
            } else {
                egui::Color32::GRAY
            };
            ui.colored_label(color, format!("{}/{}", len, max));
        }
    });
}

/// Multiline text field.
pub fn multiline_field(ui: &mut egui::Ui, label: &str, value: &mut String, max_len: Option<usize>, required: bool) {
    let label_text = if required {
        format!("{}*", label)
    } else {
        label.to_string()
    };
    ui.label(&label_text);
    ui.add(egui::TextEdit::multiline(value).desired_rows(4).desired_width(f32::INFINITY));
    if let Some(max) = max_len {
        let len = value.len();
        let color = if len > max {
            egui::Color32::RED
        } else {
            egui::Color32::GRAY
        };
        ui.colored_label(color, format!("{}/{} chars", len, max));
    }
}

/// Combo box choice field.
pub fn choice_field(ui: &mut egui::Ui, label: &str, selected: &mut usize, choices: &[&str]) {
    ui.horizontal(|ui| {
        ui.label(label);
        let current = choices.get(*selected).unwrap_or(&"");
        egui::ComboBox::from_id_salt(label)
            .selected_text(*current)
            .width(300.0)
            .show_ui(ui, |ui| {
                for (i, choice) in choices.iter().enumerate() {
                    ui.selectable_value(selected, i, *choice);
                }
            });
    });
}

/// Boolean checkbox.
pub fn bool_field(ui: &mut egui::Ui, label: &str, value: &mut bool) {
    ui.checkbox(value, label);
}

/// Comma-separated list field.
pub fn list_field(ui: &mut egui::Ui, label: &str, value: &mut String, max_items: Option<usize>) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.text_edit_singleline(value);
        let count = if value.trim().is_empty() {
            0
        } else {
            value.split(',').count()
        };
        if let Some(max) = max_items {
            let color = if count > max {
                egui::Color32::RED
            } else {
                egui::Color32::GRAY
            };
            ui.colored_label(color, format!("{}/{} items", count, max));
        }
    });
}

/// File path field with browse button.
pub fn path_field(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.text_edit_singleline(value);
        if ui.button("Browse…").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                *value = path.display().to_string();
            }
        }
    });
}

/// Directory path field with browse button (opens folder picker).
pub fn dir_field(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.text_edit_singleline(value);
        if ui.button("Browse…").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                *value = path.display().to_string();
            }
        }
    });
}

/// URL field with validation hint.
pub fn url_field(ui: &mut egui::Ui, label: &str, value: &mut String, required: bool) {
    ui.horizontal(|ui| {
        let label_text = if required {
            format!("{}*", label)
        } else {
            label.to_string()
        };
        ui.label(&label_text);
        ui.text_edit_singleline(value);
        if !value.is_empty() && !value.starts_with("http://") && !value.starts_with("https://") {
            ui.colored_label(egui::Color32::RED, "must start with http(s)://");
        }
    });
}

/// Email field with validation hint.
pub fn email_field(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(format!("{}*", label));
        ui.text_edit_singleline(value);
        if !value.is_empty() && !value.contains('@') {
            ui.colored_label(egui::Color32::RED, "invalid email");
        }
    });
}

/// Per-language text field: renders tabs for each active language.
pub fn per_language_text(
    ui: &mut egui::Ui,
    id_salt: &str,
    label: &str,
    values: &mut HashMap<String, String>,
    languages: &[String],
    max_len: Option<usize>,
    required: bool,
) {
    let label_text = if required {
        format!("{}*", label)
    } else {
        label.to_string()
    };
    ui.label(&label_text);
    ui.horizontal(|ui| {
        for lang in languages {
            let display = language_name(lang);
            let val = values.entry(lang.clone()).or_default();
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(format!(" {}", display));
                    ui.text_edit_singleline(val);
                    if let Some(max) = max_len {
                        let len = val.len();
                        let color = if len > max { egui::Color32::RED } else { egui::Color32::GRAY };
                        ui.colored_label(color, format!("{}/{}", len, max));
                    }
                });
            });
        }
    });
    let _ = id_salt; // used for uniqueness disambiguation if needed
}

/// Per-language multiline field.
pub fn per_language_multiline(
    ui: &mut egui::Ui,
    id_salt: &str,
    label: &str,
    values: &mut HashMap<String, String>,
    languages: &[String],
    max_len: Option<usize>,
    required: bool,
) {
    let label_text = if required {
        format!("{}*", label)
    } else {
        label.to_string()
    };
    ui.label(&label_text);
    for lang in languages {
        let display = language_name(lang);
        let val = values.entry(lang.clone()).or_default();
        ui.group(|ui| {
            ui.label(format!(" {}", display));
            ui.add(egui::TextEdit::multiline(val).desired_rows(3).desired_width(f32::INFINITY));
            if let Some(max) = max_len {
                let len = val.len();
                let color = if len > max { egui::Color32::RED } else { egui::Color32::GRAY };
                ui.colored_label(color, format!("{}/{} chars", len, max));
            }
        });
    }
    let _ = id_salt;
}

/// Per-language list (comma-separated) field.
pub fn per_language_list(
    ui: &mut egui::Ui,
    id_salt: &str,
    label: &str,
    values: &mut HashMap<String, String>,
    languages: &[String],
    max_items: Option<usize>,
) {
    ui.label(label);
    ui.horizontal(|ui| {
        for lang in languages {
            let display = language_name(lang);
            let val = values.entry(lang.clone()).or_default();
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(format!(" {}", display));
                    ui.text_edit_singleline(val);
                    if let Some(max) = max_items {
                        let count = if val.trim().is_empty() { 0 } else { val.split(',').count() };
                        let color = if count > max { egui::Color32::RED } else { egui::Color32::GRAY };
                        ui.colored_label(color, format!("{}/{} items", count, max));
                    }
                });
            });
        }
    });
    let _ = id_salt;
}
