use eframe::egui;

use crate::icon_gen;
use crate::state::CommonState;
use crate::widgets;

pub const PRICING_CHOICES: &[&str] = &["free", "paid", "freemium", "subscription"];
pub const AGE_RATING_CHOICES: &[&str] = &["4+", "9+", "12+", "17+"];

pub fn ui_section(
    ui: &mut egui::Ui,
    state: &mut CommonState,
    languages: &[String],
    icon_gen_receiver: &mut Option<icon_gen::IconReceiver>,
    icon_gen_status: &mut Option<String>,
    icon_texture: Option<&egui::TextureHandle>,
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

    // Generate icon buttons
    ui.horizontal(|ui| {
        let is_generating = icon_gen_receiver.is_some();
        let has_description = !state.icon_description.trim().is_empty();
        let has_icon = !state.app_icon_path.is_empty()
            && std::path::Path::new(&state.app_icon_path).exists();

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

        ui.add_enabled_ui(!is_generating, |ui| {
            if ui.button("Generate 4K Version…").clicked() {
                let mut dialog = rfd::FileDialog::new()
                    .add_filter("PNG image", &["png"]);
                if has_icon {
                    if let Some(parent) = std::path::Path::new(&state.app_icon_path).parent() {
                        dialog = dialog.set_directory(parent);
                    }
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
