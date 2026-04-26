use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::icon_gen::IconReceiver;
use crate::deploy::DeployReceiver;

fn empty_lang_map() -> HashMap<String, String> {
    HashMap::new()
}

pub struct AppState {
    // Store selection
    pub store_macos: bool,
    pub store_ios: bool,
    pub store_windows: bool,
    pub store_android: bool,
    pub store_github: bool,

    // Language selection
    pub lang_selected: Vec<bool>,
    pub active_languages: Vec<String>,

    // Sections
    pub common: CommonState,
    pub apple: AppleState,
    pub google_play: GooglePlayState,
    pub microsoft: MicrosoftState,
    pub github: GithubState,
    pub deploy: DeployState,

    // UI state
    pub active_tab: Tab,
    pub save_status: Option<String>,
    pub save_status_time: Option<std::time::Instant>,
    pub validation_errors: Vec<String>,

    // Icon generation
    pub icon_gen_receiver: Option<IconReceiver>,
    pub icon_gen_status: Option<String>,

    // Deploy
    pub deploy_log: Vec<String>,
    pub deploy_running: bool,
    pub deploy_receiver: Option<DeployReceiver>,

    // Tracks the last app name we auto-saved under
    pub last_saved_name: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            store_macos: false,
            store_ios: false,
            store_windows: false,
            store_android: false,
            store_github: false,
            lang_selected: Vec::new(),
            active_languages: Vec::new(),
            common: CommonState::default(),
            apple: AppleState::default(),
            google_play: GooglePlayState::default(),
            microsoft: MicrosoftState::default(),
            github: GithubState::default(),
            deploy: DeployState::default(),
            active_tab: Tab::default(),
            save_status: None,
            save_status_time: None,
            validation_errors: Vec::new(),
            icon_gen_receiver: None,
            icon_gen_status: None,
            deploy_log: Vec::new(),
            deploy_running: false,
            deploy_receiver: None,
            last_saved_name: String::new(),
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum Tab {
    #[default]
    Common,
    Apple,
    Android,
    Windows,
    GitHub,
    Deploy,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct CommonState {
    pub app_name: String,
    pub display_name: String,
    pub bundle_id: String,
    pub version: String,
    pub short_description: HashMap<String, String>,
    pub full_description: HashMap<String, String>,
    pub keywords: HashMap<String, String>,
    pub privacy_policy_url: String,
    pub support_url: String,
    pub website_url: String,
    pub contact_email: String,
    pub copyright: String,
    pub pricing: usize,
    pub age_rating: usize,
    pub app_icon_path: String,
    pub icon_description: String,
}

impl Default for CommonState {
    fn default() -> Self {
        Self {
            app_name: String::new(),
            display_name: String::new(),
            bundle_id: String::new(),
            version: "1.0.0".to_string(),
            short_description: empty_lang_map(),
            full_description: empty_lang_map(),
            keywords: empty_lang_map(),
            privacy_policy_url: String::new(),
            support_url: String::new(),
            website_url: String::new(),
            contact_email: String::new(),
            copyright: String::new(),
            pricing: 0,
            age_rating: 0,
            app_icon_path: String::new(),
            icon_description: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct AppleState {
    pub sku: String,
    pub subtitle: HashMap<String, String>,
    pub promotional_text: HashMap<String, String>,
    pub marketing_url: String,
    // macOS
    pub macos_primary_category: usize,
    pub macos_secondary_category: usize,
    pub macos_screenshots: String,
    pub macos_preview_video: String,
    // iOS
    pub ios_primary_category: usize,
    pub ios_secondary_category: usize,
    pub ios_screenshots_iphone_6_9: String,
    pub ios_screenshots_iphone_6_5: String,
    pub ios_screenshots_ipad_13: String,
    pub ios_preview_video: String,
}

impl Default for AppleState {
    fn default() -> Self {
        Self {
            sku: String::new(),
            subtitle: empty_lang_map(),
            promotional_text: empty_lang_map(),
            marketing_url: String::new(),
            macos_primary_category: 0,
            macos_secondary_category: 0,
            macos_screenshots: String::new(),
            macos_preview_video: String::new(),
            ios_primary_category: 0,
            ios_secondary_category: 0,
            ios_screenshots_iphone_6_9: String::new(),
            ios_screenshots_iphone_6_5: String::new(),
            ios_screenshots_ipad_13: String::new(),
            ios_preview_video: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct GooglePlayState {
    pub package_name: String,
    pub category: usize,
    pub feature_graphic_path: String,
    pub screenshots_phone: String,
    pub screenshots_tablet_7: String,
    pub screenshots_tablet_10: String,
    pub content_rating_violence: bool,
    pub content_rating_sexual: bool,
    pub content_rating_language: bool,
    pub content_rating_drugs: bool,
    pub content_rating_gambling: bool,
    pub content_rating_user_generated: bool,
    pub release_track: usize,
    pub video_url: String,
}

impl Default for GooglePlayState {
    fn default() -> Self {
        Self {
            package_name: String::new(),
            category: 0,
            feature_graphic_path: String::new(),
            screenshots_phone: String::new(),
            screenshots_tablet_7: String::new(),
            screenshots_tablet_10: String::new(),
            content_rating_violence: false,
            content_rating_sexual: false,
            content_rating_language: false,
            content_rating_drugs: false,
            content_rating_gambling: false,
            content_rating_user_generated: false,
            release_track: 3, // production
            video_url: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct MicrosoftState {
    pub msstore_app_id: String,
    pub category: usize,
    pub subcategory: String,
    pub whats_new: HashMap<String, String>,
    pub product_features: HashMap<String, String>,
    pub search_terms: HashMap<String, String>,
    pub certification_notes: String,
    pub additional_license_terms: String,
    pub logo_poster_path: String,
    pub logo_box_art_path: String,
    pub logo_tile_path: String,
    pub screenshots: String,
    pub installer_type: usize,
    pub silent_install: bool,
    pub min_os: String,
    pub min_ram: usize,
    pub min_disk: String,
    // Support info (Properties page)
    pub contact_phone: String,
    pub support_address1: String,
    pub support_address2: String,
    pub support_zip: String,
    pub support_city: String,
    pub support_country: String,
}

impl Default for MicrosoftState {
    fn default() -> Self {
        Self {
            msstore_app_id: String::new(),
            category: 0,
            subcategory: String::new(),
            whats_new: empty_lang_map(),
            product_features: empty_lang_map(),
            search_terms: empty_lang_map(),
            certification_notes: String::new(),
            additional_license_terms: String::new(),
            logo_poster_path: String::new(),
            logo_box_art_path: String::new(),
            logo_tile_path: String::new(),
            screenshots: String::new(),
            installer_type: 2, // msix
            silent_install: true,
            min_os: "Windows 10".to_string(),
            min_ram: 3, // 2GB
            min_disk: String::new(),
            contact_phone: String::new(),
            support_address1: String::new(),
            support_address2: String::new(),
            support_zip: String::new(),
            support_city: String::new(),
            support_country: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct GithubState {
    pub tag_pattern: String,
    pub target_branch: String,
    pub release_name_template: String,
    pub release_notes_template: String,
    pub draft: bool,
    pub prerelease: bool,
    pub generate_release_notes: bool,
    pub build_appimage: bool,
    pub asset_patterns: String,
}

impl Default for GithubState {
    fn default() -> Self {
        Self {
            tag_pattern: "v{version}".to_string(),
            target_branch: "main".to_string(),
            release_name_template: "{display_name} v{version}".to_string(),
            release_notes_template: String::new(),
            draft: false,
            prerelease: false,
            generate_release_notes: true,
            build_appimage: false,
            asset_patterns: String::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DeployState {
    // Apple App Store Connect
    pub apple_api_key_path: String,
    pub apple_api_key_id: String,
    pub apple_api_issuer_id: String,
    // Microsoft Partner Center (Azure AD)
    pub azure_tenant_id: String,
    pub azure_client_id: String,
    pub azure_client_secret: String,
    pub msstore_seller_id: String,
    // GitHub
    pub github_pat: String,
    pub github_repo: String,
    // Package source directory (for binary upload)
    pub source_dir: String,
}

impl Default for DeployState {
    fn default() -> Self {
        Self {
            apple_api_key_path: String::new(),
            apple_api_key_id: String::new(),
            apple_api_issuer_id: String::new(),
            azure_tenant_id: String::new(),
            azure_client_id: String::new(),
            azure_client_secret: String::new(),
            msstore_seller_id: String::new(),
            github_pat: String::new(),
            github_repo: String::new(),
            source_dir: String::new(),
        }
    }
}

// --- Field resolution helpers ---
//
// Several per-store fields default to a value from Common when left empty,
// so the user only has to fill the information once. These helpers return the
// effective value used for JSON output and store-API deploys.

fn lang_map_is_empty(m: &HashMap<String, String>) -> bool {
    m.values().all(|v| v.trim().is_empty())
}

/// Apple marketing URL is always Common.website_url (no per-store override).
pub fn resolved_apple_marketing_url(c: &CommonState) -> String {
    c.website_url.clone()
}

/// Microsoft search terms are always Common.keywords (no per-store override).
pub fn resolved_microsoft_search_terms(c: &CommonState) -> HashMap<String, String> {
    c.keywords.clone()
}

/// Apple subtitle defaults to Common.short_description (truncated to 30 chars
/// per the App Store limit) when no per-store override is set.
pub fn resolved_apple_subtitle(c: &CommonState, a: &AppleState) -> HashMap<String, String> {
    if !lang_map_is_empty(&a.subtitle) {
        a.subtitle.clone()
    } else {
        c.short_description
            .iter()
            .map(|(k, v)| (k.clone(), v.chars().take(30).collect::<String>()))
            .collect()
    }
}

/// Apple promotional text defaults to Common.short_description when empty.
pub fn resolved_apple_promotional_text(c: &CommonState, a: &AppleState) -> HashMap<String, String> {
    if !lang_map_is_empty(&a.promotional_text) {
        a.promotional_text.clone()
    } else {
        c.short_description.clone()
    }
}

/// Microsoft "what's new" defaults to Common.full_description when empty.
pub fn resolved_microsoft_whats_new(c: &CommonState, m: &MicrosoftState) -> HashMap<String, String> {
    if !lang_map_is_empty(&m.whats_new) {
        m.whats_new.clone()
    } else {
        c.full_description.clone()
    }
}

/// Microsoft product features default to Common.short_description when empty.
pub fn resolved_microsoft_product_features(c: &CommonState, m: &MicrosoftState) -> HashMap<String, String> {
    if !lang_map_is_empty(&m.product_features) {
        m.product_features.clone()
    } else {
        c.short_description.clone()
    }
}

/// Serializable snapshot of the full app state (excludes transient UI fields).
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SavedState {
    pub store_macos: bool,
    pub store_ios: bool,
    pub store_windows: bool,
    pub store_android: bool,
    pub store_github: bool,
    pub lang_selected: Vec<bool>,
    pub common: CommonState,
    pub apple: AppleState,
    pub google_play: GooglePlayState,
    pub microsoft: MicrosoftState,
    pub github: GithubState,
    pub deploy: DeployState,
}

impl AppState {
    pub fn new() -> Self {
        let mut state = Self::default();
        // Default: en and de selected
        state.lang_selected = vec![true, true, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false];
        state.update_active_languages();
        state
    }

    pub fn update_active_languages(&mut self) {
        self.active_languages = crate::languages::LANGUAGES
            .iter()
            .zip(self.lang_selected.iter())
            .filter(|(_, selected)| **selected)
            .map(|((code, _), _)| code.to_string())
            .collect();
    }

    pub fn any_store_selected(&self) -> bool {
        self.store_macos || self.store_ios || self.store_windows || self.store_android || self.store_github
    }

    pub fn selected_store_names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        if self.store_macos { names.push("macos"); }
        if self.store_ios { names.push("ios"); }
        if self.store_windows { names.push("windows"); }
        if self.store_android { names.push("android"); }
        if self.store_github { names.push("github"); }
        names
    }

    pub fn has_apple(&self) -> bool {
        self.store_macos || self.store_ios
    }

    pub fn to_saved(&self) -> SavedState {
        let mut apple: AppleState = serde_json::from_str(&serde_json::to_string(&self.apple).unwrap()).unwrap();
        apple.marketing_url = resolved_apple_marketing_url(&self.common);
        apple.subtitle = resolved_apple_subtitle(&self.common, &self.apple);
        apple.promotional_text = resolved_apple_promotional_text(&self.common, &self.apple);

        let mut microsoft: MicrosoftState = serde_json::from_str(&serde_json::to_string(&self.microsoft).unwrap()).unwrap();
        microsoft.search_terms = resolved_microsoft_search_terms(&self.common);
        microsoft.whats_new = resolved_microsoft_whats_new(&self.common, &self.microsoft);
        microsoft.product_features = resolved_microsoft_product_features(&self.common, &self.microsoft);

        SavedState {
            store_macos: self.store_macos,
            store_ios: self.store_ios,
            store_windows: self.store_windows,
            store_android: self.store_android,
            store_github: self.store_github,
            lang_selected: self.lang_selected.clone(),
            common: serde_json::from_str(&serde_json::to_string(&self.common).unwrap()).unwrap(),
            apple,
            google_play: serde_json::from_str(&serde_json::to_string(&self.google_play).unwrap()).unwrap(),
            microsoft,
            github: serde_json::from_str(&serde_json::to_string(&self.github).unwrap()).unwrap(),
            deploy: serde_json::from_str(&serde_json::to_string(&self.deploy).unwrap()).unwrap(),
        }
    }

    pub fn load_from_saved(&mut self, saved: SavedState) {
        self.store_macos = saved.store_macos;
        self.store_ios = saved.store_ios;
        self.store_windows = saved.store_windows;
        self.store_android = saved.store_android;
        self.store_github = saved.store_github;
        self.lang_selected = saved.lang_selected;
        self.common = saved.common;
        self.apple = saved.apple;
        self.google_play = saved.google_play;
        self.microsoft = saved.microsoft;
        self.github = saved.github;
        self.deploy = saved.deploy;
        self.update_active_languages();
    }
}

/// Get the path for the JSON save dir
pub fn json_dir() -> std::path::PathBuf {
    let dir = std::env::current_dir().unwrap_or_default().join("json");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

/// Build a safe filename from the app name
fn safe_filename(app_name: &str) -> String {
    let safe: String = app_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    if safe.is_empty() {
        "untitled".to_string()
    } else {
        safe.to_lowercase()
    }
}

/// Auto-save the state to json/<app_name>.json
pub fn auto_save(state: &AppState) {
    let name = safe_filename(&state.common.app_name);
    let path = json_dir().join(format!("{}.json", name));
    let saved = state.to_saved();
    if let Ok(json) = serde_json::to_string_pretty(&saved) {
        let _ = std::fs::write(&path, json);
    }
}

/// Load state from a user-chosen JSON file
pub fn load_from_file_dialog() -> Option<SavedState> {
    let start_dir = json_dir();
    let path = rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .set_directory(&start_dir)
        .pick_file()?;
    let data = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Try to load the most recently modified JSON file from the json/ dir
pub fn auto_load_latest() -> Option<SavedState> {
    let dir = json_dir();
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            let modified = meta.modified().ok()?;
            Some((e.path(), modified))
        })
        .collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));
    let path = entries.first()?.0.clone();
    let data = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}
