use std::collections::HashMap;

fn empty_lang_map() -> HashMap<String, String> {
    HashMap::new()
}

#[derive(Default)]
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

    // UI state
    pub active_tab: Tab,
    pub save_status: Option<String>,
    pub validation_errors: Vec<String>,
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum Tab {
    #[default]
    Common,
    Apple,
    Android,
    Windows,
    GitHub,
}

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
        }
    }
}

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
        }
    }
}

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
}
