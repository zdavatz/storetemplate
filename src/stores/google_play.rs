use eframe::egui;

use crate::state::GooglePlayState;
use crate::widgets;

pub const CATEGORIES: &[&str] = &[
    "Art & Design", "Auto & Vehicles", "Beauty", "Books & Reference", "Business",
    "Comics", "Communication", "Dating", "Education", "Entertainment", "Events",
    "Finance", "Food & Drink", "Health & Fitness", "House & Home",
    "Libraries & Demo", "Lifestyle", "Maps & Navigation", "Medical",
    "Music & Audio", "News & Magazines", "Parenting", "Personalization",
    "Photography", "Productivity", "Shopping", "Social", "Sports", "Tools",
    "Travel & Local", "Video Players & Editors", "Weather",
];

pub const RELEASE_TRACKS: &[&str] = &["internal", "closed", "open", "production"];

pub fn ui_section(ui: &mut egui::Ui, state: &mut GooglePlayState, _languages: &[String]) {
    ui.heading("Google Play (Android)");
    ui.add_space(8.0);

    ui.label("Name, descriptions and keywords are taken from the Common tab.");
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label("Package name*");
        ui.text_edit_singleline(&mut state.package_name);
        if ui.link("Open Google Play Console").clicked() {
            let _ = open::that("https://play.google.com/console/developers");
        }
    });
    widgets::choice_field(ui, "Category", &mut state.category, CATEGORIES);
    widgets::choice_field(ui, "Release track", &mut state.release_track, RELEASE_TRACKS);

    ui.add_space(12.0);
    ui.separator();
    ui.heading("Assets");
    ui.add_space(4.0);

    widgets::path_field(ui, "Feature graphic (1024x500)", &mut state.feature_graphic_path);
    widgets::list_field(ui, "Phone screenshots (2-8)", &mut state.screenshots_phone, Some(8));
    widgets::list_field(ui, "7\" tablet screenshots", &mut state.screenshots_tablet_7, Some(8));
    widgets::list_field(ui, "10\" tablet screenshots", &mut state.screenshots_tablet_10, Some(8));
    widgets::url_field(ui, "Promotional video URL (YouTube)", &mut state.video_url, false);

    ui.add_space(12.0);
    ui.separator();
    ui.heading("Content Rating (IARC)");
    ui.add_space(4.0);

    widgets::bool_field(ui, "Contains violence", &mut state.content_rating_violence);
    widgets::bool_field(ui, "Contains sexual content", &mut state.content_rating_sexual);
    widgets::bool_field(ui, "Contains strong language", &mut state.content_rating_language);
    widgets::bool_field(ui, "References drugs/alcohol/tobacco", &mut state.content_rating_drugs);
    widgets::bool_field(ui, "Contains gambling", &mut state.content_rating_gambling);
    widgets::bool_field(ui, "Contains user-generated content", &mut state.content_rating_user_generated);
}
