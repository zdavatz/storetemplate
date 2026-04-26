use eframe::egui;

use crate::state::MicrosoftState;
use crate::widgets;

pub const CATEGORIES: &[&str] = &[
    "BooksAndReference", "Business", "DeveloperTools", "Education", "Entertainment",
    "FoodAndDining", "GovernmentAndPolitics", "HealthAndFitness", "KidsAndFamily",
    "Lifestyle", "Medical", "MultimediaDesign", "Music", "NavigationAndMaps",
    "NewsAndWeather", "PersonalFinance", "Personalization", "PhotoAndVideo",
    "Productivity", "Security", "Shopping", "Social", "Sports", "Travel",
    "UtilitiesAndTools",
];

pub const INSTALLER_TYPES: &[&str] = &["exe", "msi", "msix"];
pub const RAM_CHOICES: &[&str] = &["300MB", "750MB", "1GB", "2GB", "4GB", "6GB", "8GB"];

pub fn ui_section(ui: &mut egui::Ui, state: &mut MicrosoftState, languages: &[String]) {
    ui.heading("Microsoft Store (Windows)");
    ui.add_space(8.0);

    ui.label("Name, descriptions and keywords are taken from the Common tab.");
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label("MS Store App ID");
        ui.text_edit_singleline(&mut state.msstore_app_id);
        if ui.link("Open Partner Center").clicked() {
            let _ = open::that("https://partner.microsoft.com/dashboard/apps-and-games/overview");
        }
    });
    widgets::choice_field(ui, "Category", &mut state.category, CATEGORIES);
    widgets::text_field(ui, "Subcategory", &mut state.subcategory, None, false);

    ui.add_space(12.0);
    ui.separator();
    ui.heading("Listing (store-specific)");
    ui.add_space(4.0);

    widgets::per_language_multiline(ui, "ms_whats_new", "What's new", &mut state.whats_new, languages, Some(1500), false);
    ui.weak("Auto-filled from Common.full_description if left empty.");
    ui.add_space(4.0);
    widgets::per_language_list(ui, "ms_features", "Product features (up to 20)", &mut state.product_features, languages, Some(20));
    ui.weak("Auto-filled from Common.short_description if left empty.");
    ui.add_space(4.0);
    ui.weak("Search terms are taken from Common.keywords.");

    ui.add_space(12.0);
    ui.separator();
    ui.heading("Support Info (Properties)");
    ui.add_space(4.0);

    ui.colored_label(
        egui::Color32::from_rgb(160, 80, 0),
        "Note: phone and address are NOT settable via the Microsoft Store API. \
         You must enter these once in Partner Center account settings. \
         Privacy URL / Support contact / Website are set from the Common tab via the v2 API.",
    );
    ui.add_space(4.0);
    widgets::text_field(ui, "Phone number", &mut state.contact_phone, None, false);
    widgets::text_field(ui, "Address line 1", &mut state.support_address1, None, false);
    widgets::text_field(ui, "Address line 2", &mut state.support_address2, None, false);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            widgets::text_field(ui, "ZIP / Postal code", &mut state.support_zip, None, false);
        });
        ui.vertical(|ui| {
            widgets::text_field(ui, "City", &mut state.support_city, None, false);
        });
    });
    widgets::text_field(ui, "Country / Region", &mut state.support_country, None, false);

    ui.add_space(12.0);
    ui.separator();
    ui.heading("Review & Legal");
    ui.add_space(4.0);

    widgets::multiline_field(ui, "Certification notes", &mut state.certification_notes, Some(2000), false);
    widgets::multiline_field(ui, "Additional license terms", &mut state.additional_license_terms, Some(10000), false);

    ui.add_space(12.0);
    ui.separator();
    ui.heading("Assets");
    ui.add_space(4.0);

    widgets::path_field(ui, "Poster logo (720x1080 PNG)", &mut state.logo_poster_path);
    widgets::path_field(ui, "Box art logo (1080x1080 PNG)", &mut state.logo_box_art_path);
    widgets::path_field(ui, "Tile icon (300x300 PNG)", &mut state.logo_tile_path);
    widgets::list_field(ui, "Screenshots (1366x768 to 3840x2160 PNG)", &mut state.screenshots, None);

    ui.add_space(12.0);
    ui.separator();
    ui.heading("Installer & Requirements");
    ui.add_space(4.0);

    widgets::choice_field(ui, "Installer type", &mut state.installer_type, INSTALLER_TYPES);
    widgets::bool_field(ui, "Supports silent install", &mut state.silent_install);
    widgets::text_field(ui, "Minimum OS", &mut state.min_os, None, false);
    widgets::choice_field(ui, "Minimum RAM", &mut state.min_ram, RAM_CHOICES);
    widgets::text_field(ui, "Minimum disk space", &mut state.min_disk, None, false);

}

