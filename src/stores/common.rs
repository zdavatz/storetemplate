use std::collections::HashMap;
use std::sync::mpsc;

use eframe::egui;

use crate::icon_gen;
use crate::state::CommonState;
use crate::stl_render::StlMesh;
use crate::translate;
use crate::widgets;

pub const PRICING_CHOICES: &[&str] = &["free", "paid", "freemium", "subscription"];
pub const AGE_RATING_CHOICES: &[&str] = &["4+", "9+", "12+", "17+"];

#[allow(clippy::too_many_arguments)]
pub fn ui_section(
    ui: &mut egui::Ui,
    state: &mut CommonState,
    languages: &[String],
    icon_gen_receiver: &mut Option<icon_gen::IconReceiver>,
    icon_gen_status: &mut Option<String>,
    icon_texture: Option<&egui::TextureHandle>,
    translate_receiver: &mut Option<translate::TranslateReceiver>,
    translate_status: &mut Option<String>,
    stl_mesh: &mut Option<StlMesh>,
    stl_mesh_source: &mut String,
    stl_mesh_loading: &mut Option<mpsc::Receiver<Result<StlMesh, String>>>,
    stl_preview_texture: Option<&egui::TextureHandle>,
) {
    ui.heading("Common Information");
    ui.add_space(8.0);

    widgets::text_field(ui, "App name", &mut state.app_name, None, true);
    widgets::text_field(ui, "Display name", &mut state.display_name, None, true);

    // Auto-suggest bundle ID from app name
    if state.bundle_id.is_empty() && !state.app_name.is_empty() {
        let sanitized: String = state.app_name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '.' })
            .collect::<String>()
            .replace("..", ".");
        let sanitized = sanitized.trim_matches('.').to_string();
        state.bundle_id = format!("com.example.{}", sanitized);
    }
    widgets::text_field(ui, "Bundle/Package ID", &mut state.bundle_id, None, true);
    widgets::text_field(ui, "Version", &mut state.version, None, true);

    ui.add_space(12.0);
    ui.separator();
    ui.heading("Descriptions & Keywords");
    ui.add_space(4.0);

    widgets::per_language_text(ui, "short_desc", "Short description", &mut state.short_description, languages, Some(80), true);
    ui.add_space(4.0);
    widgets::per_language_multiline(ui, "full_desc", "Full description", &mut state.full_description, languages, Some(4000), true);
    ui.add_space(4.0);
    widgets::per_language_list(ui, "keywords", "Keywords (comma-separated)", &mut state.keywords, languages, None);

    ui.add_space(8.0);
    translate_buttons(ui, state, translate_receiver, translate_status);

    ui.add_space(12.0);
    ui.separator();
    ui.heading("URLs & Contact");
    ui.add_space(4.0);

    widgets::url_field(ui, "Privacy policy URL", &mut state.privacy_policy_url, false);
    widgets::url_field(ui, "Support URL", &mut state.support_url, false);
    widgets::url_field(ui, "Website URL", &mut state.website_url, false);
    widgets::email_field(ui, "Contact email", &mut state.contact_email);

    ui.add_space(12.0);
    ui.separator();
    ui.heading("Metadata");
    ui.add_space(4.0);

    widgets::text_field(ui, "Copyright", &mut state.copyright, None, true);
    widgets::choice_field(ui, "Pricing", &mut state.pricing, PRICING_CHOICES);
    widgets::choice_field(ui, "Age rating", &mut state.age_rating, AGE_RATING_CHOICES);
    widgets::path_field(ui, "App icon (512x512 PNG)", &mut state.app_icon_path);

    ui.add_space(4.0);
    widgets::multiline_field(ui, "Icon description (for AI generation)", &mut state.icon_description, None, false);

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("STL model (path or URL, optional):");
        ui.text_edit_singleline(&mut state.icon_stl_path);
        if ui.button("Browse…").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("STL model", &["stl"])
                .pick_file()
            {
                state.icon_stl_path = path.display().to_string();
            }
        }
    });

    ui.horizontal(|ui| {
        ui.label("STL view angle:");
        ui.label("azimuth°");
        ui.add(
            egui::DragValue::new(&mut state.icon_stl_azimuth)
                .speed(1.0)
                .range(-360.0..=360.0),
        );
        ui.label("elevation°");
        ui.add(
            egui::DragValue::new(&mut state.icon_stl_elevation)
                .speed(1.0)
                .range(-90.0..=90.0),
        );
        ui.checkbox(&mut state.icon_stl_z_up, "Z is up");

        let has_stl = !state.icon_stl_path.trim().is_empty();
        let is_loading = stl_mesh_loading.is_some();
        let is_loaded = stl_mesh.is_some()
            && stl_mesh_source.as_str() == state.icon_stl_path.trim();
        ui.add_enabled_ui(has_stl && !is_loading, |ui| {
            let label = if is_loaded { "Reload STL preview" } else { "Load STL preview" };
            if ui.button(label).clicked() {
                *stl_mesh_loading = Some(crate::stl_render::load_stl_async(
                    state.icon_stl_path.trim(),
                ));
                *icon_gen_status = Some("Loading STL...".to_string());
            }
        });
        if is_loading {
            ui.spinner();
        }
    });

    if stl_mesh.is_some() {
        ui.horizontal(|ui| {
            ui.label("View presets:");
            if ui.small_button("Iso").clicked() {
                state.icon_stl_azimuth = 30.0;
                state.icon_stl_elevation = 25.0;
            }
            if ui.small_button("Top").clicked() {
                state.icon_stl_azimuth = 0.0;
                state.icon_stl_elevation = 90.0;
            }
            if ui.small_button("Bottom").clicked() {
                state.icon_stl_azimuth = 0.0;
                state.icon_stl_elevation = -90.0;
            }
            if ui.small_button("Front").clicked() {
                state.icon_stl_azimuth = 0.0;
                state.icon_stl_elevation = 0.0;
            }
            if ui.small_button("Back").clicked() {
                state.icon_stl_azimuth = 180.0;
                state.icon_stl_elevation = 0.0;
            }
            if ui.small_button("Left").clicked() {
                state.icon_stl_azimuth = -90.0;
                state.icon_stl_elevation = 0.0;
            }
            if ui.small_button("Right").clicked() {
                state.icon_stl_azimuth = 90.0;
                state.icon_stl_elevation = 0.0;
            }
        });
    }

    if let Some(tex) = stl_preview_texture {
        ui.add_space(4.0);
        ui.label("Drag to rotate (horizontal = azimuth, vertical = elevation):");
        let size = egui::vec2(256.0, 256.0);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::drag());
        ui.painter().rect_filled(rect, egui::CornerRadius::same(4), egui::Color32::from_gray(220));
        ui.painter().image(
            tex.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        if response.dragged() {
            let delta = response.drag_delta();
            state.icon_stl_azimuth += delta.x * 0.6;
            state.icon_stl_elevation =
                (state.icon_stl_elevation - delta.y * 0.6).clamp(-90.0, 90.0);
            // Wrap azimuth to [-360, 360] so DragValue ranges stay valid
            if state.icon_stl_azimuth > 360.0 { state.icon_stl_azimuth -= 360.0; }
            if state.icon_stl_azimuth < -360.0 { state.icon_stl_azimuth += 360.0; }
        }
    }

    // Generate icon buttons
    ui.horizontal(|ui| {
        let is_generating = icon_gen_receiver.is_some();
        let has_description = !state.icon_description.trim().is_empty();
        let has_icon = !state.app_icon_path.is_empty()
            && std::path::Path::new(&state.app_icon_path).exists();
        let has_stl = !state.icon_stl_path.trim().is_empty();

        let name = if state.app_name.is_empty() {
            "App".to_string()
        } else {
            state.app_name.clone()
        };

        ui.add_enabled_ui(!is_generating && has_description, |ui| {
            if ui.button("Generate New Icon").clicked() {
                *icon_gen_receiver = Some(icon_gen::generate_icon(
                    &state.icon_description,
                    &name,
                    None,
                ));
                *icon_gen_status = Some("Generating icon...".to_string());
            }
        });

        ui.add_enabled_ui(!is_generating && has_description && has_icon, |ui| {
            if ui.button("Iterate on Icon").clicked() {
                *icon_gen_receiver = Some(icon_gen::generate_icon(
                    &state.icon_description,
                    &name,
                    Some(&state.app_icon_path),
                ));
                *icon_gen_status = Some("Iterating on icon...".to_string());
            }
        });

        ui.add_enabled_ui(!is_generating && has_description && has_stl, |ui| {
            if ui.button("Generate from STL").clicked() {
                *icon_gen_receiver = Some(icon_gen::generate_icon_from_stl(
                    &state.icon_description,
                    &name,
                    state.icon_stl_path.trim(),
                    state.icon_stl_azimuth,
                    state.icon_stl_elevation,
                    state.icon_stl_z_up,
                ));
                *icon_gen_status = Some("Rendering STL & generating icon...".to_string());
            }
        });

        ui.add_enabled_ui(!is_generating && has_icon, |ui| {
            if ui.button("Generate 4K Version").clicked() {
                *icon_gen_receiver = Some(icon_gen::upscale_to_4k(&state.app_icon_path, &name));
                *icon_gen_status = Some("Upscaling current icon to 4K...".to_string());
            }
        });

        ui.add_enabled_ui(!is_generating, |ui| {
            if ui.button("Upscale other PNG…").clicked() {
                let png_dir = std::env::current_dir().unwrap_or_default().join("png");
                let mut dialog = rfd::FileDialog::new().add_filter("PNG image", &["png"]);
                if png_dir.exists() {
                    dialog = dialog.set_directory(&png_dir);
                }
                if let Some(path) = dialog.pick_file() {
                    let path_str = path.display().to_string();
                    *icon_gen_receiver = Some(icon_gen::upscale_to_4k(&path_str, &name));
                    *icon_gen_status = Some("Upscaling to 4K...".to_string());
                }
            }
        });

        if is_generating {
            ui.spinner();
            ui.label("Generating...");
        }
    });

    if let Some(ref status) = icon_gen_status {
        if status.starts_with("Error:") {
            ui.colored_label(egui::Color32::RED, status);
        } else if status.starts_with("Icon saved:") {
            ui.colored_label(egui::Color32::DARK_GREEN, status);
        } else {
            ui.label(status);
        }
    }

    // Show icon preview
    if let Some(texture) = icon_texture {
        ui.add_space(8.0);
        ui.label("Icon preview:");
        let size = egui::vec2(128.0, 128.0);
        // Light gray background so transparency is visible
        egui::Frame::new()
            .fill(egui::Color32::from_gray(220))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(4))
            .show(ui, |ui| {
                ui.image(egui::load::SizedTexture::new(texture.id(), size));
            });
    }

    // Extra space so the button isn't hidden behind the footer panel
    ui.add_space(60.0);
}

/// Pack a single field's source-language value into the batch translation payload
/// using a plain field-name key (no language suffix, since the request is one-directional).
fn pack(field: &str, map: &HashMap<String, String>, source_lang: &str, out: &mut HashMap<String, String>) {
    if let Some(val) = map.get(source_lang) {
        if !val.trim().is_empty() {
            out.insert(field.to_string(), val.clone());
        }
    }
}

fn unpack(translated: &HashMap<String, String>, field: &str, target_lang: &str, target: &mut HashMap<String, String>) {
    if let Some(v) = translated.get(field) {
        target.insert(target_lang.to_string(), v.clone());
    }
}

fn translate_buttons(
    ui: &mut egui::Ui,
    state: &mut CommonState,
    translate_receiver: &mut Option<translate::TranslateReceiver>,
    translate_status: &mut Option<String>,
) {
    let is_translating = translate_receiver.is_some();

    let kick_off = |from: &str, to: &str, state: &CommonState| {
        let mut payload: HashMap<String, String> = HashMap::new();
        pack("short_description", &state.short_description, from, &mut payload);
        pack("full_description", &state.full_description, from, &mut payload);
        pack("keywords", &state.keywords, from, &mut payload);
        translate::translate_fields(payload, from, to)
    };

    ui.horizontal(|ui| {
        ui.label("Translate descriptions & keywords:");

        ui.add_enabled_ui(!is_translating, |ui| {
            if ui.button("DE -> EN").clicked() {
                *translate_receiver = Some(kick_off("de", "en", state));
                *translate_status = Some("Translating DE -> EN...".to_string());
            }
            if ui.button("EN -> DE").clicked() {
                *translate_receiver = Some(kick_off("en", "de", state));
                *translate_status = Some("Translating EN -> DE...".to_string());
            }
        });

        if is_translating {
            ui.spinner();
        }
    });

    if let Some(ref status) = translate_status {
        if status.starts_with("Error:") {
            ui.colored_label(egui::Color32::RED, status);
        } else if status.starts_with("Translated") {
            ui.colored_label(egui::Color32::DARK_GREEN, status);
        } else {
            ui.label(status);
        }
    }
}

/// Apply translated fields produced by `translate::translate_fields` back into the common state.
/// Public so main.rs can call it after polling the receiver.
pub fn apply_translation(state: &mut CommonState, to_lang: &str, translated: &HashMap<String, String>) {
    unpack(translated, "short_description", to_lang, &mut state.short_description);
    unpack(translated, "full_description", to_lang, &mut state.full_description);
    unpack(translated, "keywords", to_lang, &mut state.keywords);
}
