use eframe::egui;

use crate::state::AppleState;
use crate::widgets;

pub const MACOS_CATEGORIES: &[&str] = &[
    "public.app-category.business",
    "public.app-category.developer-tools",
    "public.app-category.education",
    "public.app-category.entertainment",
    "public.app-category.finance",
    "public.app-category.games",
    "public.app-category.graphics-design",
    "public.app-category.healthcare-fitness",
    "public.app-category.lifestyle",
    "public.app-category.medical",
    "public.app-category.music",
    "public.app-category.news",
    "public.app-category.photography",
    "public.app-category.productivity",
    "public.app-category.reference",
    "public.app-category.social-networking",
    "public.app-category.sports",
    "public.app-category.travel",
    "public.app-category.utilities",
    "public.app-category.video",
    "public.app-category.weather",
];

pub const IOS_CATEGORIES: &[&str] = &[
    "Books", "Business", "Developer Tools", "Education", "Entertainment",
    "Finance", "Food & Drink", "Games", "Graphics & Design", "Health & Fitness",
    "Lifestyle", "Medical", "Music", "Navigation", "News", "Photo & Video",
    "Productivity", "Reference", "Shopping", "Social Networking", "Sports",
    "Travel", "Utilities", "Weather",
];

pub fn ui_section(ui: &mut egui::Ui, state: &mut AppleState, languages: &[String], has_macos: bool, has_ios: bool, app_name: &str) {
    ui.heading("Apple App Store");
    ui.add_space(8.0);

    // SKU field with auto-suggest and App Store Connect link
    if state.sku.is_empty() && !app_name.is_empty() {
        state.sku = app_name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>()
            .trim_matches('_')
            .to_string();
    }
    ui.horizontal(|ui| {
        ui.label("SKU*");
        ui.text_edit_singleline(&mut state.sku);
        if ui.link("Open App Store Connect").clicked() {
            let _ = open::that("https://appstoreconnect.apple.com/apps");
        }
    });
    widgets::per_language_text(ui, "apple_subtitle", "Subtitle", &mut state.subtitle, languages, Some(30), false);
    ui.weak("Auto-filled from Common.short_description (truncated to 30 chars) if left empty.");
    ui.add_space(4.0);
    widgets::per_language_text(ui, "apple_promo", "Promotional text", &mut state.promotional_text, languages, Some(170), false);
    ui.weak("Auto-filled from Common.short_description if left empty.");
    ui.add_space(4.0);
    ui.weak("Marketing URL is taken from Common.website_url.");

    if has_macos {
        ui.add_space(12.0);
        ui.separator();
        ui.heading("macOS");
        ui.add_space(4.0);

        ui.push_id("macos", |ui| {
            widgets::choice_field(ui, "Primary category", &mut state.macos_primary_category, MACOS_CATEGORIES);
            widgets::choice_field(ui, "Secondary category", &mut state.macos_secondary_category, MACOS_CATEGORIES);
            widgets::list_field(ui, "Screenshots (16:10, e.g. 2880x1800)", &mut state.macos_screenshots, None);
            widgets::path_field(ui, "Preview video", &mut state.macos_preview_video);
        });
    }

    if has_ios {
        ui.add_space(12.0);
        ui.separator();
        ui.heading("iOS");
        ui.add_space(4.0);

        ui.push_id("ios", |ui| {
            widgets::choice_field(ui, "Primary category", &mut state.ios_primary_category, IOS_CATEGORIES);
            widgets::choice_field(ui, "Secondary category", &mut state.ios_secondary_category, IOS_CATEGORIES);
            widgets::list_field(ui, "iPhone 6.9\" screenshots (1320x2868)", &mut state.ios_screenshots_iphone_6_9, None);
            widgets::list_field(ui, "iPhone 6.5\" screenshots (1284x2778)", &mut state.ios_screenshots_iphone_6_5, None);
            widgets::list_field(ui, "iPad 13\" screenshots (2064x2752)", &mut state.ios_screenshots_ipad_13, None);
            widgets::path_field(ui, "Preview video", &mut state.ios_preview_video);
        });
    }
}
