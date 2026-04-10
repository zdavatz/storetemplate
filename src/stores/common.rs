use eframe::egui;

use crate::state::CommonState;
use crate::widgets;

pub const PRICING_CHOICES: &[&str] = &["free", "paid", "freemium", "subscription"];
pub const AGE_RATING_CHOICES: &[&str] = &["4+", "9+", "12+", "17+"];

pub fn ui_section(ui: &mut egui::Ui, state: &mut CommonState, languages: &[String]) {
    ui.heading("Common Information");
    ui.add_space(8.0);

    widgets::text_field(ui, "App name", &mut state.app_name, None, true);
    widgets::text_field(ui, "Display name", &mut state.display_name, None, true);
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
}
