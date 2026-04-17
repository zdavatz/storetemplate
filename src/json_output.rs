use std::collections::HashMap;
use std::path::PathBuf;

use serde_json::{json, Value};

use crate::state::AppState;
use crate::stores::{apple, common, google_play, microsoft};

fn split_list(s: &str) -> Vec<String> {
    if s.trim().is_empty() {
        return Vec::new();
    }
    s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
}

fn lang_map_to_json(map: &HashMap<String, String>) -> Value {
    let obj: serde_json::Map<String, Value> = map
        .iter()
        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
        .collect();
    Value::Object(obj)
}

fn lang_map_list_to_json(map: &HashMap<String, String>) -> Value {
    let obj: serde_json::Map<String, Value> = map
        .iter()
        .map(|(k, v)| (k.clone(), json!(split_list(v))))
        .collect();
    Value::Object(obj)
}

pub fn build_json(state: &AppState) -> Value {
    let mut root = serde_json::Map::new();

    // _meta
    root.insert("_meta".to_string(), json!({
        "generator": "storetemplate",
        "version": "1.0.0",
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "stores": state.selected_store_names(),
    }));

    // common
    let c = &state.common;
    root.insert("common".to_string(), json!({
        "app_name": c.app_name,
        "display_name": c.display_name,
        "bundle_id": c.bundle_id,
        "version": c.version,
        "languages": state.active_languages,
        "short_description": lang_map_to_json(&c.short_description),
        "full_description": lang_map_to_json(&c.full_description),
        "keywords": lang_map_list_to_json(&c.keywords),
        "privacy_policy_url": c.privacy_policy_url,
        "support_url": c.support_url,
        "website_url": c.website_url,
        "contact_email": c.contact_email,
        "copyright": c.copyright,
        "pricing": common::PRICING_CHOICES.get(c.pricing).copied().unwrap_or("free"),
        "age_rating": common::AGE_RATING_CHOICES.get(c.age_rating).copied().unwrap_or("4+"),
        "app_icon_path": c.app_icon_path,
    }));

    // Apple - macOS
    if state.store_macos {
        let a = &state.apple;
        root.insert("macos".to_string(), json!({
            "sku": a.sku,
            "subtitle": lang_map_to_json(&a.subtitle),
            "promotional_text": lang_map_to_json(&a.promotional_text),
            "marketing_url": a.marketing_url,
            "primary_category": apple::MACOS_CATEGORIES.get(a.macos_primary_category).copied().unwrap_or(""),
            "secondary_category": apple::MACOS_CATEGORIES.get(a.macos_secondary_category).copied().unwrap_or(""),
            "screenshots": split_list(&a.macos_screenshots),
            "preview_video": a.macos_preview_video,
        }));
    }

    // Apple - iOS
    if state.store_ios {
        let a = &state.apple;
        root.insert("ios".to_string(), json!({
            "sku": a.sku,
            "subtitle": lang_map_to_json(&a.subtitle),
            "promotional_text": lang_map_to_json(&a.promotional_text),
            "marketing_url": a.marketing_url,
            "primary_category": apple::IOS_CATEGORIES.get(a.ios_primary_category).copied().unwrap_or(""),
            "secondary_category": apple::IOS_CATEGORIES.get(a.ios_secondary_category).copied().unwrap_or(""),
            "screenshots": {
                "iphone_6_9": split_list(&a.ios_screenshots_iphone_6_9),
                "iphone_6_5": split_list(&a.ios_screenshots_iphone_6_5),
                "ipad_13": split_list(&a.ios_screenshots_ipad_13),
            },
            "preview_video": a.ios_preview_video,
        }));
    }

    // Google Play
    if state.store_android {
        let g = &state.google_play;
        root.insert("android".to_string(), json!({
            "package_name": g.package_name,
            "category": google_play::CATEGORIES.get(g.category).copied().unwrap_or(""),
            "feature_graphic_path": g.feature_graphic_path,
            "screenshots": {
                "phone": split_list(&g.screenshots_phone),
                "tablet_7": split_list(&g.screenshots_tablet_7),
                "tablet_10": split_list(&g.screenshots_tablet_10),
            },
            "content_rating": {
                "violence": g.content_rating_violence,
                "sexual_content": g.content_rating_sexual,
                "strong_language": g.content_rating_language,
                "drugs_alcohol_tobacco": g.content_rating_drugs,
                "gambling": g.content_rating_gambling,
                "user_generated_content": g.content_rating_user_generated,
            },
            "release_track": google_play::RELEASE_TRACKS.get(g.release_track).copied().unwrap_or("production"),
            "video_url": g.video_url,
        }));
    }

    // Microsoft Store
    if state.store_windows {
        let m = &state.microsoft;
        root.insert("windows".to_string(), json!({
            "msstore_app_id": m.msstore_app_id,
            "category": microsoft::CATEGORIES.get(m.category).copied().unwrap_or(""),
            "subcategory": m.subcategory,
            "whats_new": lang_map_to_json(&m.whats_new),
            "product_features": lang_map_list_to_json(&m.product_features),
            "search_terms": lang_map_list_to_json(&m.search_terms),
            "certification_notes": m.certification_notes,
            "additional_license_terms": m.additional_license_terms,
            "store_logos": {
                "poster_720x1080": m.logo_poster_path,
                "box_art_1080x1080": m.logo_box_art_path,
                "tile_300x300": m.logo_tile_path,
            },
            "screenshots": split_list(&m.screenshots),
            "installer": {
                "type": microsoft::INSTALLER_TYPES.get(m.installer_type).copied().unwrap_or("msix"),
                "silent_install": m.silent_install,
            },
            "system_requirements": {
                "min_os": m.min_os,
                "min_ram": microsoft::RAM_CHOICES.get(m.min_ram).copied().unwrap_or(""),
                "min_disk": m.min_disk,
            },
        }));
    }

    // GitHub
    if state.store_github {
        let gh = &state.github;
        root.insert("github".to_string(), json!({
            "tag_pattern": gh.tag_pattern,
            "target_branch": gh.target_branch,
            "release_name_template": gh.release_name_template,
            "release_notes_template": gh.release_notes_template,
            "draft": gh.draft,
            "prerelease": gh.prerelease,
            "generate_release_notes": gh.generate_release_notes,
            "build_appimage": gh.build_appimage,
            "asset_patterns": split_list(&gh.asset_patterns),
        }));
    }

    Value::Object(root)
}

pub fn validate(state: &AppState) -> Vec<String> {
    let mut errors = Vec::new();

    if !state.any_store_selected() {
        errors.push("Select at least one store.".to_string());
        return errors;
    }

    if state.active_languages.is_empty() {
        errors.push("Select at least one language.".to_string());
    }

    let c = &state.common;
    if c.app_name.is_empty() {
        errors.push("App name is required.".to_string());
    }
    if c.display_name.is_empty() {
        errors.push("Display name is required.".to_string());
    }
    if c.bundle_id.is_empty() {
        errors.push("Bundle/Package ID is required.".to_string());
    }
    if c.contact_email.is_empty() {
        errors.push("Contact email is required.".to_string());
    }
    if !c.contact_email.is_empty() && !c.contact_email.contains('@') {
        errors.push("Contact email is invalid.".to_string());
    }

    errors
}

pub fn save_to_file(state: &AppState) -> Result<PathBuf, String> {
    let default_name = if state.common.app_name.is_empty() {
        "template.json".to_string()
    } else {
        format!("{}.json", state.common.app_name)
    };

    let path = rfd::FileDialog::new()
        .set_file_name(&default_name)
        .add_filter("JSON", &["json"])
        .save_file()
        .ok_or("Save cancelled.".to_string())?;

    let json = build_json(state);
    let content = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())?;

    // Also save GitHub Actions workflow YAML next to the JSON
    let workflow_content = crate::workflow::build_workflow(state);
    let parent = path.parent().ok_or("Cannot determine parent directory")?;
    let workflow_dir = parent.join(".github").join("workflows");
    std::fs::create_dir_all(&workflow_dir).map_err(|e| format!("Failed to create .github/workflows: {}", e))?;
    let workflow_path = workflow_dir.join("release.yml");
    std::fs::write(&workflow_path, workflow_content)
        .map_err(|e| format!("Failed to write workflow: {}", e))?;

    Ok(path)
}
