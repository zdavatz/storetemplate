use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

use serde_json::json;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Auto-fill credentials from ~/.apple/credentials.json
// ---------------------------------------------------------------------------

/// Read ~/.apple/credentials.json and fill in deploy credentials.
/// Returns a log message describing what was filled.
pub fn autofill_credentials(state: &mut AppState) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let cred_path = std::path::Path::new(&home).join(".apple").join("credentials.json");

    if !cred_path.exists() {
        return format!("Not found: {}", cred_path.display());
    }

    let content = match std::fs::read_to_string(&cred_path) {
        Ok(c) => c,
        Err(e) => return format!("Error reading {}: {}", cred_path.display(), e),
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => return format!("Error parsing JSON: {}", e),
    };

    let mut filled = Vec::new();

    // Apple credentials
    if let Some(apple) = json.get("apple") {
        if let Some(key_id) = apple.get("api_key_id").and_then(|v| v.as_str()) {
            if !key_id.is_empty() {
                state.deploy.apple_api_key_id = key_id.to_string();
                filled.push("Apple Key ID");
            }
        }
        if let Some(issuer_id) = apple.get("api_issuer_id").and_then(|v| v.as_str()) {
            if !issuer_id.is_empty() {
                state.deploy.apple_api_issuer_id = issuer_id.to_string();
                filled.push("Apple Issuer ID");
            }
        }
        if let Some(key_path) = apple.get("api_key_path").and_then(|v| v.as_str()) {
            if !key_path.is_empty() {
                // Expand ~ to home dir
                let expanded = key_path.replace("~", &home);
                state.deploy.apple_api_key_path = expanded;
                filled.push("Apple Key Path");
            }
        }
    }

    // Azure credentials
    if let Some(azure) = json.get("azure") {
        if let Some(tenant_id) = azure.get("tenant_id").and_then(|v| v.as_str()) {
            if !tenant_id.is_empty() {
                state.deploy.azure_tenant_id = tenant_id.to_string();
                filled.push("Azure Tenant ID");
            }
        }
        if let Some(client_id) = azure.get("client_id").and_then(|v| v.as_str()) {
            if !client_id.is_empty() {
                state.deploy.azure_client_id = client_id.to_string();
                filled.push("Azure Client ID");
            }
        }
        if let Some(client_secret) = azure.get("client_secret").and_then(|v| v.as_str()) {
            if !client_secret.is_empty() {
                state.deploy.azure_client_secret = client_secret.to_string();
                filled.push("Azure Client Secret");
            }
        }
    }

    // GitHub PAT from gh CLI config
    let gh_hosts = std::path::Path::new(&home).join(".config").join("gh").join("hosts.yml");
    if gh_hosts.exists() && state.deploy.github_pat.is_empty() {
        if let Ok(gh_content) = std::fs::read_to_string(&gh_hosts) {
            for line in gh_content.lines() {
                if line.trim().starts_with("oauth_token:") {
                    let token = line.trim().strip_prefix("oauth_token:").unwrap_or("").trim();
                    if !token.is_empty() {
                        state.deploy.github_pat = token.to_string();
                        filled.push("GitHub PAT (from gh CLI)");
                    }
                    break;
                }
            }
        }
    }

    if filled.is_empty() {
        "No credentials found to fill.".to_string()
    } else {
        format!("Filled: {}", filled.join(", "))
    }
}

/// Messages sent from deploy background threads to the UI.
pub enum DeployMsg {
    Log(String),
    Done,
    Error(String),
}

pub type DeployReceiver = mpsc::Receiver<DeployMsg>;

// ---------------------------------------------------------------------------
// Apple App Store Connect helpers
// ---------------------------------------------------------------------------

/// Map our short language codes (en, de, fr, ...) to Apple locale strings.
fn apple_locale(lang: &str) -> &'static str {
    match lang {
        "en" => "en-US",
        "de" => "de-DE",
        "fr" => "fr-FR",
        "it" => "it-IT",
        "es" => "es-ES",
        "pt" => "pt-PT",
        "nl" => "nl-NL",
        "ja" => "ja",
        "ko" => "ko",
        "zh" => "zh-Hans",
        "ru" => "ru",
        "ar" => "ar-SA",
        "sv" => "sv",
        "da" => "da",
        "fi" => "fi",
        "nb" => "nb",
        "pl" => "pl",
        "tr" => "tr",
        "cs" => "cs",
        "el" => "el",
        _ => "en-US",
    }
}

/// Build an Apple App Store Connect JWT (ES256, 20 min expiry).
fn build_apple_jwt(key_path: &str, key_id: &str, issuer_id: &str) -> Result<String, String> {
    let key_pem = std::fs::read_to_string(key_path)
        .map_err(|e| format!("Cannot read .p8 key file: {}", e))?;

    let encoding_key = jsonwebtoken::EncodingKey::from_ec_pem(key_pem.as_bytes())
        .map_err(|e| format!("Invalid .p8 key: {}", e))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = json!({
        "iss": issuer_id,
        "iat": now,
        "exp": now + 1200,
        "aud": "appstoreconnect-v1",
    });

    let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256);
    header.kid = Some(key_id.to_string());
    header.typ = Some("JWT".to_string());

    jsonwebtoken::encode(&header, &claims, &encoding_key)
        .map_err(|e| format!("JWT encode error: {}", e))
}

/// Deploy metadata to Apple App Store Connect.
pub fn deploy_apple(state: &AppState) -> DeployReceiver {
    let (tx, rx) = mpsc::channel();

    let deploy = state.deploy.clone();
    let bundle_id = state.common.bundle_id.clone();
    let app_name = state.common.app_name.clone();
    let display_name = state.common.display_name.clone();
    let version = state.common.version.clone();
    let copyright = state.common.copyright.clone();
    let privacy_url = state.common.privacy_policy_url.clone();
    let support_url = state.common.support_url.clone();
    let marketing_url = crate::state::resolved_apple_marketing_url(&state.common);
    let _short_desc = state.common.short_description.clone();
    let full_desc = state.common.full_description.clone();
    let keywords = state.common.keywords.clone();
    let subtitle = crate::state::resolved_apple_subtitle(&state.common, &state.apple);
    let languages = state.active_languages.clone();

    thread::spawn(move || {
        let _ = tx.send(DeployMsg::Log("Starting Apple App Store Connect deploy...".into()));

        // Validate credentials
        if deploy.apple_api_key_path.is_empty() || deploy.apple_api_key_id.is_empty() || deploy.apple_api_issuer_id.is_empty() {
            let _ = tx.send(DeployMsg::Error("Apple API credentials not set. Provide .p8 key path, Key ID, and Issuer ID.".into()));
            return;
        }

        let token = match build_apple_jwt(&deploy.apple_api_key_path, &deploy.apple_api_key_id, &deploy.apple_api_issuer_id) {
            Ok(t) => t,
            Err(e) => { let _ = tx.send(DeployMsg::Error(e)); return; }
        };
        let _ = tx.send(DeployMsg::Log("JWT token generated.".into()));

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        let base = "https://api.appstoreconnect.apple.com/v1";
        let auth = format!("Bearer {}", token);

        // 1. Register Bundle ID (or find existing)
        let _ = tx.send(DeployMsg::Log(format!("Registering bundle ID: {}", bundle_id)));
        let bid_body = json!({
            "data": {
                "type": "bundleIds",
                "attributes": {
                    "identifier": bundle_id,
                    "name": app_name,
                    "platform": "UNIVERSAL"
                }
            }
        });
        let bid_resp = client.post(format!("{}/bundleIds", base))
            .header("Authorization", &auth)
            .header("Content-Type", "application/json")
            .json(&bid_body)
            .send();

        let bundle_id_resource_id = match bid_resp {
            Ok(r) => {
                let status = r.status();
                let body: serde_json::Value = r.json().unwrap_or_default();
                if status.is_success() {
                    let id = body["data"]["id"].as_str().unwrap_or("").to_string();
                    let _ = tx.send(DeployMsg::Log(format!("Bundle ID registered: {}", id)));
                    id
                } else if status.as_u16() == 409 {
                    let _ = tx.send(DeployMsg::Log("Bundle ID already exists, looking it up...".into()));
                    let filter = format!("{}/bundleIds?filter%5Bidentifier%5D={}", base, bundle_id);
                    match client.get(&filter).header("Authorization", &auth).send() {
                        Ok(r2) => {
                            let b2: serde_json::Value = r2.json().unwrap_or_default();
                            b2["data"][0]["id"].as_str().unwrap_or("").to_string()
                        }
                        Err(e) => { let _ = tx.send(DeployMsg::Error(format!("Lookup failed: {}", e))); return; }
                    }
                } else {
                    let _ = tx.send(DeployMsg::Error(format!("Register bundle ID failed ({}): {}", status, body)));
                    return;
                }
            }
            Err(e) => { let _ = tx.send(DeployMsg::Error(format!("Request failed: {}", e))); return; }
        };

        if bundle_id_resource_id.is_empty() {
            let _ = tx.send(DeployMsg::Error("Could not find or create bundle ID.".into()));
            return;
        }

        // 2. Find the app
        let _ = tx.send(DeployMsg::Log("Looking up app...".into()));
        let app_filter = format!("{}/apps?filter%5BbundleId%5D={}", base, bundle_id);
        let app_id = match client.get(&app_filter).header("Authorization", &auth).send() {
            Ok(r) => {
                let body: serde_json::Value = r.json().unwrap_or_default();
                let id = body["data"][0]["id"].as_str().unwrap_or("").to_string();
                if id.is_empty() {
                    let _ = tx.send(DeployMsg::Log("App not found. The app must be created in App Store Connect first. Bundle ID was registered.".into()));
                    let _ = tx.send(DeployMsg::Done);
                    return;
                }
                let _ = tx.send(DeployMsg::Log(format!("Found app: {}", id)));
                id
            }
            Err(e) => { let _ = tx.send(DeployMsg::Error(format!("App lookup failed: {}", e))); return; }
        };

        // 3. Find or create app store version
        let _ = tx.send(DeployMsg::Log("Looking up app store versions...".into()));
        let versions_url = format!("{}/apps/{}/appStoreVersions", base, app_id);
        let version_id = match client.get(&versions_url).header("Authorization", &auth).send() {
            Ok(r) => {
                let body: serde_json::Value = r.json().unwrap_or_default();
                let mut found_id = String::new();
                if let Some(arr) = body["data"].as_array() {
                    for v in arr {
                        let vs = v["attributes"]["versionString"].as_str().unwrap_or("");
                        let state_str = v["attributes"]["appStoreState"].as_str().unwrap_or("");
                        if vs == version || state_str == "PREPARE_FOR_SUBMISSION" {
                            found_id = v["id"].as_str().unwrap_or("").to_string();
                            break;
                        }
                    }
                }
                if found_id.is_empty() {
                    let _ = tx.send(DeployMsg::Log(format!("Creating new version {}...", version)));
                    let ver_body = json!({
                        "data": {
                            "type": "appStoreVersions",
                            "attributes": {
                                "versionString": version,
                                "platform": "MAC_OS"
                            },
                            "relationships": {
                                "app": {
                                    "data": { "type": "apps", "id": app_id }
                                }
                            }
                        }
                    });
                    match client.post(format!("{}/appStoreVersions", base))
                        .header("Authorization", &auth)
                        .header("Content-Type", "application/json")
                        .json(&ver_body)
                        .send()
                    {
                        Ok(r2) => {
                            let b2: serde_json::Value = r2.json().unwrap_or_default();
                            b2["data"]["id"].as_str().unwrap_or("").to_string()
                        }
                        Err(e) => { let _ = tx.send(DeployMsg::Error(format!("Create version failed: {}", e))); return; }
                    }
                } else {
                    let _ = tx.send(DeployMsg::Log(format!("Using existing version: {}", found_id)));
                    found_id
                }
            }
            Err(e) => { let _ = tx.send(DeployMsg::Error(format!("Versions lookup failed: {}", e))); return; }
        };

        if version_id.is_empty() {
            let _ = tx.send(DeployMsg::Error("Could not find or create app store version.".into()));
            return;
        }

        // 4. Update app info (name, subtitle, privacy URL)
        let _ = tx.send(DeployMsg::Log("Updating app info...".into()));
        let app_infos_url = format!("{}/apps/{}/appInfos", base, app_id);
        if let Ok(r) = client.get(&app_infos_url).header("Authorization", &auth).send() {
            let body: serde_json::Value = r.json().unwrap_or_default();
            if let Some(info_id) = body["data"][0]["id"].as_str() {
                let locs_url = format!("{}/appInfos/{}/appInfoLocalizations", base, info_id);
                if let Ok(r2) = client.get(&locs_url).header("Authorization", &auth).send() {
                    let locs_body: serde_json::Value = r2.json().unwrap_or_default();
                    let existing: HashMap<String, String> = locs_body["data"].as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|l| {
                            let locale = l["attributes"]["locale"].as_str()?.to_string();
                            let id = l["id"].as_str()?.to_string();
                            Some((locale, id))
                        })
                        .collect();

                    for lang in &languages {
                        let locale = apple_locale(lang);
                        let name_val = if !display_name.is_empty() { &display_name } else { &app_name };
                        let sub_val = subtitle.get(lang).map(|s| s.as_str()).unwrap_or("");

                        if let Some(loc_id) = existing.get(locale) {
                            let patch_body = json!({
                                "data": {
                                    "type": "appInfoLocalizations",
                                    "id": loc_id,
                                    "attributes": {
                                        "name": name_val,
                                        "subtitle": sub_val,
                                        "privacyPolicyUrl": privacy_url
                                    }
                                }
                            });
                            let _ = client.patch(format!("{}/appInfoLocalizations/{}", base, loc_id))
                                .header("Authorization", &auth)
                                .header("Content-Type", "application/json")
                                .json(&patch_body)
                                .send();
                            let _ = tx.send(DeployMsg::Log(format!("  Updated app info for {}", locale)));
                        } else {
                            let create_body = json!({
                                "data": {
                                    "type": "appInfoLocalizations",
                                    "attributes": {
                                        "locale": locale,
                                        "name": name_val,
                                        "subtitle": sub_val,
                                        "privacyPolicyUrl": privacy_url
                                    },
                                    "relationships": {
                                        "appInfo": {
                                            "data": { "type": "appInfos", "id": info_id }
                                        }
                                    }
                                }
                            });
                            let _ = client.post(format!("{}/appInfoLocalizations", base))
                                .header("Authorization", &auth)
                                .header("Content-Type", "application/json")
                                .json(&create_body)
                                .send();
                            let _ = tx.send(DeployMsg::Log(format!("  Created app info for {}", locale)));
                        }
                    }
                }
            }
        }

        // 5. Update version localizations (description, keywords, support/marketing URLs)
        let _ = tx.send(DeployMsg::Log("Updating version localizations...".into()));
        let ver_locs_url = format!("{}/appStoreVersions/{}/appStoreVersionLocalizations", base, version_id);
        if let Ok(r) = client.get(&ver_locs_url).header("Authorization", &auth).send() {
            let body: serde_json::Value = r.json().unwrap_or_default();
            let existing: HashMap<String, String> = body["data"].as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|l| {
                    let locale = l["attributes"]["locale"].as_str()?.to_string();
                    let id = l["id"].as_str()?.to_string();
                    Some((locale, id))
                })
                .collect();

            for lang in &languages {
                let locale = apple_locale(lang);
                let desc = full_desc.get(lang).map(|s| s.as_str()).unwrap_or("");
                let kw = keywords.get(lang).map(|s| s.as_str()).unwrap_or("");

                if let Some(loc_id) = existing.get(locale) {
                    let patch_body = json!({
                        "data": {
                            "type": "appStoreVersionLocalizations",
                            "id": loc_id,
                            "attributes": {
                                "description": desc,
                                "keywords": kw,
                                "supportUrl": support_url,
                                "marketingUrl": marketing_url
                            }
                        }
                    });
                    let _ = client.patch(format!("{}/appStoreVersionLocalizations/{}", base, loc_id))
                        .header("Authorization", &auth)
                        .header("Content-Type", "application/json")
                        .json(&patch_body)
                        .send();
                    let _ = tx.send(DeployMsg::Log(format!("  Updated version localization for {}", locale)));
                } else {
                    let create_body = json!({
                        "data": {
                            "type": "appStoreVersionLocalizations",
                            "attributes": {
                                "locale": locale,
                                "description": desc,
                                "keywords": kw,
                                "supportUrl": support_url,
                                "marketingUrl": marketing_url
                            },
                            "relationships": {
                                "appStoreVersion": {
                                    "data": { "type": "appStoreVersions", "id": version_id }
                                }
                            }
                        }
                    });
                    let _ = client.post(format!("{}/appStoreVersionLocalizations", base))
                        .header("Authorization", &auth)
                        .header("Content-Type", "application/json")
                        .json(&create_body)
                        .send();
                    let _ = tx.send(DeployMsg::Log(format!("  Created version localization for {}", locale)));
                }
            }
        }

        // 6. Update copyright on the app-level
        if !copyright.is_empty() {
            let _ = tx.send(DeployMsg::Log("Updating primary locale...".into()));
            let patch_body = json!({
                "data": {
                    "type": "apps",
                    "id": app_id,
                    "attributes": {
                        "primaryLocale": apple_locale(languages.first().map(|s| s.as_str()).unwrap_or("en"))
                    }
                }
            });
            let _ = client.patch(format!("{}/apps/{}", base, app_id))
                .header("Authorization", &auth)
                .header("Content-Type", "application/json")
                .json(&patch_body)
                .send();
        }

        // 7. Set pricing to Free (USD $0)
        let _ = tx.send(DeployMsg::Log("Setting price to Free...".into()));
        // First, find the Free price point for USA
        let pp_url = format!("{}/apps/{}/appPricePoints?filter%5Bterritory%5D=USA&include=territory&limit=1", base, app_id);
        let free_price_point_id = match client.get(&pp_url).header("Authorization", &auth).send() {
            Ok(r) => {
                let body: serde_json::Value = r.json().unwrap_or_default();
                // The first price point with customerPrice "0.0" is the free tier
                body["data"].as_array()
                    .and_then(|arr| arr.iter().find(|pp| {
                        pp["attributes"]["customerPrice"].as_str().map(|p| p == "0.0" || p == "0").unwrap_or(false)
                    }))
                    .and_then(|pp| pp["id"].as_str())
                    .map(|s| s.to_string())
                    .or_else(|| body["data"][0]["id"].as_str().map(|s| s.to_string()))
            }
            Err(_) => None,
        };

        if let Some(pp_id) = free_price_point_id {
            let _ = tx.send(DeployMsg::Log(format!("Found free price point: {}", pp_id)));
            let price_body = json!({
                "data": {
                    "type": "appPriceSchedules",
                    "relationships": {
                        "app": {
                            "data": { "type": "apps", "id": app_id }
                        },
                        "manualPrices": {
                            "data": [{
                                "type": "appPrices",
                                "id": "${price1}"
                            }]
                        },
                        "baseTerritory": {
                            "data": { "type": "territories", "id": "USA" }
                        }
                    }
                },
                "included": [{
                    "type": "appPrices",
                    "id": "${price1}",
                    "attributes": {
                        "startDate": serde_json::Value::Null
                    },
                    "relationships": {
                        "appPricePoint": {
                            "data": { "type": "appPricePoints", "id": pp_id }
                        }
                    }
                }]
            });
            match client.post(format!("{}/appPriceSchedules", base))
                .header("Authorization", &auth)
                .header("Content-Type", "application/json")
                .json(&price_body)
                .send()
            {
                Ok(r) => {
                    let status = r.status();
                    if status.is_success() {
                        let _ = tx.send(DeployMsg::Log("Price set to Free.".into()));
                    } else {
                        let body: serde_json::Value = r.json().unwrap_or_default();
                        let _ = tx.send(DeployMsg::Log(format!("Pricing note: {} - {}", status, body)));
                    }
                }
                Err(e) => {
                    let _ = tx.send(DeployMsg::Log(format!("Pricing request failed: {}", e)));
                }
            }
        } else {
            let _ = tx.send(DeployMsg::Log("Could not find free price point. Set pricing manually.".into()));
        }

        // 8. Create provisioning profile
        let _ = tx.send(DeployMsg::Log("Creating provisioning profile...".into()));
        let cert_url = format!("{}/certificates?filter%5BcertificateType%5D=MAC_APP_DISTRIBUTION", base);
        if let Ok(r) = client.get(&cert_url).header("Authorization", &auth).send() {
            let body: serde_json::Value = r.json().unwrap_or_default();
            if let Some(cert_id) = body["data"][0]["id"].as_str() {
                let profile_body = json!({
                    "data": {
                        "type": "profiles",
                        "attributes": {
                            "name": format!("{}_AppStore", app_name),
                            "profileType": "MAC_APP_STORE"
                        },
                        "relationships": {
                            "bundleId": {
                                "data": { "type": "bundleIds", "id": bundle_id_resource_id }
                            },
                            "certificates": {
                                "data": [{ "type": "certificates", "id": cert_id }]
                            }
                        }
                    }
                });
                match client.post(format!("{}/profiles", base))
                    .header("Authorization", &auth)
                    .header("Content-Type", "application/json")
                    .json(&profile_body)
                    .send()
                {
                    Ok(r2) => {
                        let status = r2.status();
                        if status.is_success() {
                            let _ = tx.send(DeployMsg::Log("Provisioning profile created.".into()));
                        } else if status.as_u16() == 409 {
                            let _ = tx.send(DeployMsg::Log("Provisioning profile already exists.".into()));
                        } else {
                            let body: serde_json::Value = r2.json().unwrap_or_default();
                            let _ = tx.send(DeployMsg::Log(format!("Profile creation note: {} - {}", status, body)));
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(DeployMsg::Log(format!("Profile creation failed: {}", e)));
                    }
                }
            } else {
                let _ = tx.send(DeployMsg::Log("No distribution certificate found. Skipping profile creation.".into()));
            }
        }

        let _ = tx.send(DeployMsg::Log("Apple deploy complete.".into()));
        let _ = tx.send(DeployMsg::Done);
    });

    rx
}

// ---------------------------------------------------------------------------
// Microsoft Store (Partner Center) helpers
// ---------------------------------------------------------------------------

/// Map our language codes to Microsoft locale codes.
fn microsoft_locale(lang: &str) -> &'static str {
    match lang {
        "en" => "en-us",
        "de" => "de-de",
        "fr" => "fr-fr",
        "it" => "it-it",
        "es" => "es-es",
        "pt" => "pt-pt",
        "nl" => "nl-nl",
        "ja" => "ja-jp",
        "ko" => "ko-kr",
        "zh" => "zh-cn",
        "ru" => "ru-ru",
        "ar" => "ar-sa",
        "sv" => "sv-se",
        "da" => "da-dk",
        "fi" => "fi-fi",
        "nb" => "nb-no",
        "pl" => "pl-pl",
        "tr" => "tr-tr",
        "cs" => "cs-cz",
        "el" => "el-gr",
        _ => "en-us",
    }
}

/// Deploy text metadata to Microsoft Partner Center via the v2 Store Submission API
/// (api.store.microsoft.com). Sets Properties (privacy URL, support contact, website,
/// category, productDeclarations) and per-language Listings (description, whatsNew,
/// productFeatures, searchTerms, etc.). Does NOT upload a binary — the GitHub Actions
/// release workflow handles package upload.
pub fn deploy_microsoft(state: &AppState) -> DeployReceiver {
    let (tx, rx) = mpsc::channel();

    let deploy = state.deploy.clone();
    let product_id = state.microsoft.msstore_app_id.clone();
    let short_desc = state.common.short_description.clone();
    let full_desc = state.common.full_description.clone();
    let support_url = state.common.support_url.clone();
    let privacy_url = state.common.privacy_policy_url.clone();
    let copyright = state.common.copyright.clone();
    let website_url = state.common.website_url.clone();
    let contact_email = state.common.contact_email.clone();
    let whats_new = crate::state::resolved_microsoft_whats_new(&state.common, &state.microsoft);
    let product_features = crate::state::resolved_microsoft_product_features(&state.common, &state.microsoft);
    let search_terms = crate::state::resolved_microsoft_search_terms(&state.common);
    let certification_notes = state.microsoft.certification_notes.clone();
    let additional_license_terms = state.microsoft.additional_license_terms.clone();
    let languages = state.active_languages.clone();
    let category_idx = state.microsoft.category;
    let subcategory = state.microsoft.subcategory.clone();

    thread::spawn(move || {
        let _ = tx.send(DeployMsg::Log("Starting Microsoft Store deploy (v2 API, metadata only)...".into()));

        if deploy.azure_tenant_id.is_empty() || deploy.azure_client_id.is_empty() || deploy.azure_client_secret.is_empty() {
            let _ = tx.send(DeployMsg::Error("Azure AD credentials not set. Provide Tenant ID, Client ID, and Client Secret.".into()));
            return;
        }
        if deploy.msstore_seller_id.is_empty() {
            let _ = tx.send(DeployMsg::Error("Seller / Account ID not set. Provide it in the Deploy tab.".into()));
            return;
        }
        if product_id.is_empty() {
            let _ = tx.send(DeployMsg::Error("Product ID (MS Store App ID) not set. Set it in the Windows tab — format: 9PXXXXXXXXXX.".into()));
            return;
        }

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        // 1. Acquire Microsoft Entra ID access token (v2 endpoint + scope).
        let _ = tx.send(DeployMsg::Log("Acquiring Microsoft Entra ID token...".into()));
        let token_url = format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", deploy.azure_tenant_id);
        let token_resp = client.post(&token_url)
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &deploy.azure_client_id),
                ("client_secret", &deploy.azure_client_secret),
                ("scope", "https://api.store.microsoft.com/.default"),
            ])
            .send();

        let access_token = match token_resp {
            Ok(r) => {
                let status = r.status();
                let body: serde_json::Value = r.json().unwrap_or_default();
                match body["access_token"].as_str() {
                    Some(t) => {
                        let _ = tx.send(DeployMsg::Log("Token acquired.".into()));
                        t.to_string()
                    }
                    None => {
                        let _ = tx.send(DeployMsg::Error(format!("Token error ({}): {}", status, body)));
                        return;
                    }
                }
            }
            Err(e) => { let _ = tx.send(DeployMsg::Error(format!("Token request failed: {}", e))); return; }
        };

        let api_base = format!("https://api.store.microsoft.com/submission/v1/product/{}", product_id);
        let auth = format!("Bearer {}", access_token);
        let seller_id = deploy.msstore_seller_id.clone();

        // 2. Build Properties module.
        let categories = crate::stores::microsoft::CATEGORIES;
        let category_name = categories.get(category_idx).copied().unwrap_or("UtilitiesAndTools");
        let _ = tx.send(DeployMsg::Log(format!("Category: {} / {}", category_name, if subcategory.is_empty() { "<none>" } else { &subcategory })));

        let support_contact_val = if !contact_email.is_empty() { contact_email.clone() } else { support_url.clone() };

        let mut properties = serde_json::Map::new();
        let has_privacy = !privacy_url.is_empty();
        properties.insert("isPrivacyPolicyRequired".into(), json!(has_privacy));
        if has_privacy {
            properties.insert("privacyPolicyUrl".into(), json!(privacy_url));
        }
        if !website_url.is_empty() {
            properties.insert("website".into(), json!(website_url));
        }
        if !support_contact_val.is_empty() {
            properties.insert("supportContactInfo".into(), json!(support_contact_val));
        }
        if !certification_notes.is_empty() {
            properties.insert("certificationNotes".into(), json!(certification_notes));
        }
        properties.insert("category".into(), json!(category_name));
        if !subcategory.is_empty() {
            properties.insert("subcategory".into(), json!(subcategory));
        }
        properties.insert("productDeclarations".into(), json!({
            "dependsOnDriversOrNT": false,
            "accessibilitySupport": false,
            "penAndInkSupport": false
        }));

        // 3. PATCH properties module.
        let _ = tx.send(DeployMsg::Log("Updating properties...".into()));
        let patch_props = client.patch(format!("{}/metadata", api_base))
            .header("Authorization", &auth)
            .header("X-Seller-Account-Id", &seller_id)
            .header("Content-Type", "application/json")
            .json(&json!({ "properties": properties }))
            .send();

        match patch_props {
            Ok(r) => {
                let status = r.status();
                let body: serde_json::Value = r.json().unwrap_or_default();
                if status.is_success() && body["isSuccess"].as_bool().unwrap_or(false) {
                    let _ = tx.send(DeployMsg::Log("Properties updated.".into()));
                } else {
                    let _ = tx.send(DeployMsg::Error(format!("Properties update failed ({}): {}", status, body)));
                    return;
                }
            }
            Err(e) => { let _ = tx.send(DeployMsg::Error(format!("Properties PATCH failed: {}", e))); return; }
        }

        // 4. PATCH listings, one per language.
        for lang in &languages {
            let locale = microsoft_locale(lang);
            let desc = full_desc.get(lang).cloned().unwrap_or_default();
            let short = short_desc.get(lang).cloned().unwrap_or_default();
            let wn = whats_new.get(lang).cloned().unwrap_or_default();
            let feat_list: Vec<String> = product_features.get(lang).cloned().unwrap_or_default()
                .split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).take(20).collect();
            let st_list: Vec<String> = search_terms.get(lang).cloned().unwrap_or_default()
                .split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).take(7).collect();

            let mut listing = serde_json::Map::new();
            listing.insert("language".into(), json!(locale));
            if !desc.is_empty() {
                listing.insert("description".into(), json!(desc));
            }
            if !short.is_empty() {
                listing.insert("shortDescription".into(), json!(short));
            }
            if !wn.is_empty() {
                listing.insert("whatsNew".into(), json!(wn));
            }
            if !feat_list.is_empty() {
                listing.insert("productFeatures".into(), json!(feat_list));
            }
            if !st_list.is_empty() {
                listing.insert("searchTerms".into(), json!(st_list));
            }
            if !additional_license_terms.is_empty() {
                listing.insert("additionalLicenseTerms".into(), json!(additional_license_terms));
            }
            if !copyright.is_empty() {
                listing.insert("copyright".into(), json!(copyright));
            }
            if !contact_email.is_empty() {
                listing.insert("contactInfo".into(), json!(contact_email));
            }

            let _ = tx.send(DeployMsg::Log(format!("Updating listing for {}...", locale)));
            let patch_list = client.patch(format!("{}/metadata", api_base))
                .header("Authorization", &auth)
                .header("X-Seller-Account-Id", &seller_id)
                .header("Content-Type", "application/json")
                .json(&json!({ "listings": listing }))
                .send();

            match patch_list {
                Ok(r) => {
                    let status = r.status();
                    let body: serde_json::Value = r.json().unwrap_or_default();
                    if status.is_success() && body["isSuccess"].as_bool().unwrap_or(false) {
                        let _ = tx.send(DeployMsg::Log(format!("  Listing updated for {}.", locale)));
                    } else {
                        let _ = tx.send(DeployMsg::Log(format!("  Listing update note for {} ({}): {}", locale, status, body)));
                    }
                }
                Err(e) => { let _ = tx.send(DeployMsg::Log(format!("  Listing PATCH failed for {}: {}", locale, e))); }
            }
        }

        let _ = tx.send(DeployMsg::Log("Metadata deploy complete. Binary upload is handled by the GitHub Actions release workflow.".into()));
        let _ = tx.send(DeployMsg::Log("Phone number and company address cannot be set via the Microsoft Store API — set them once in Partner Center account settings.".into()));
        let _ = tx.send(DeployMsg::Done);
    });

    rx
}

// ---------------------------------------------------------------------------
// GitHub secrets + workflow via `gh` CLI
// ---------------------------------------------------------------------------

/// Set up GitHub secrets and push release workflow using the `gh` CLI.
pub fn deploy_github(state: &AppState) -> DeployReceiver {
    let (tx, rx) = mpsc::channel();

    let deploy = state.deploy.clone();
    let workflow_yaml = crate::workflow::build_workflow(state);

    // Collect secrets to set
    let mut secrets: Vec<(String, String)> = Vec::new();

    // Apple secrets
    if !deploy.apple_api_key_path.is_empty() {
        if let Ok(key_data) = std::fs::read_to_string(&deploy.apple_api_key_path) {
            secrets.push(("APPLE_API_KEY".into(), key_data));
        }
    }
    if !deploy.apple_api_key_id.is_empty() {
        secrets.push(("APPLE_API_KEY_ID".into(), deploy.apple_api_key_id.clone()));
    }
    if !deploy.apple_api_issuer_id.is_empty() {
        secrets.push(("APPLE_API_ISSUER_ID".into(), deploy.apple_api_issuer_id.clone()));
    }

    // Azure / Microsoft secrets
    if !deploy.azure_tenant_id.is_empty() {
        secrets.push(("AZURE_TENANT_ID".into(), deploy.azure_tenant_id.clone()));
    }
    if !deploy.azure_client_id.is_empty() {
        secrets.push(("AZURE_CLIENT_ID".into(), deploy.azure_client_id.clone()));
    }
    if !deploy.azure_client_secret.is_empty() {
        secrets.push(("AZURE_CLIENT_SECRET".into(), deploy.azure_client_secret.clone()));
    }

    let repo = deploy.github_repo.clone();
    let pat = deploy.github_pat.clone();

    thread::spawn(move || {
        let _ = tx.send(DeployMsg::Log("Starting GitHub setup...".into()));

        if repo.is_empty() {
            let _ = tx.send(DeployMsg::Error("GitHub repo not set (e.g. owner/repo).".into()));
            return;
        }

        // Check if gh CLI is available
        let gh_check = std::process::Command::new("gh")
            .arg("--version")
            .output();
        if gh_check.is_err() {
            let _ = tx.send(DeployMsg::Error("gh CLI not found. Install it: https://cli.github.com/".into()));
            return;
        }

        // If PAT is provided, set GH_TOKEN env var for auth
        let env_token: Option<(&str, String)> = if !pat.is_empty() {
            Some(("GH_TOKEN", pat.clone()))
        } else {
            None
        };

        // 1. Set secrets
        let _ = tx.send(DeployMsg::Log(format!("Setting {} secrets on {}...", secrets.len(), repo)));
        for (name, value) in &secrets {
            let _ = tx.send(DeployMsg::Log(format!("  Setting secret: {}", name)));
            let mut cmd = std::process::Command::new("gh");
            cmd.args(["secret", "set", name, "--repo", &repo, "--body", value]);
            if let Some((k, ref v)) = env_token {
                cmd.env(k, v);
            }
            match cmd.output() {
                Ok(output) => {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let _ = tx.send(DeployMsg::Log(format!("    Warning: {}", stderr.trim())));
                    }
                }
                Err(e) => {
                    let _ = tx.send(DeployMsg::Log(format!("    Failed: {}", e)));
                }
            }
        }

        // 2. Push release workflow
        let _ = tx.send(DeployMsg::Log("Writing release workflow...".into()));
        let workflow_dir = std::path::Path::new(".github/workflows");
        if !workflow_dir.exists() {
            let _ = std::fs::create_dir_all(workflow_dir);
        }
        let workflow_path = workflow_dir.join("release.yml");
        match std::fs::write(&workflow_path, &workflow_yaml) {
            Ok(_) => {
                let _ = tx.send(DeployMsg::Log(format!("Workflow written to {}", workflow_path.display())));
            }
            Err(e) => {
                let _ = tx.send(DeployMsg::Log(format!("Could not write workflow file: {}", e)));
                let _ = tx.send(DeployMsg::Log("You can manually create .github/workflows/release.yml with the generated content.".into()));
            }
        }

        // 3. Try to commit and push
        let _ = tx.send(DeployMsg::Log("Committing and pushing workflow...".into()));
        let git_add = std::process::Command::new("git")
            .args(["add", ".github/workflows/release.yml"])
            .output();
        if git_add.is_ok() {
            let git_commit = std::process::Command::new("git")
                .args(["commit", "-m", "Add release workflow"])
                .output();
            match git_commit {
                Ok(output) => {
                    if output.status.success() {
                        let git_push = std::process::Command::new("git")
                            .args(["push"])
                            .output();
                        match git_push {
                            Ok(po) => {
                                if po.status.success() {
                                    let _ = tx.send(DeployMsg::Log("Workflow pushed to remote.".into()));
                                } else {
                                    let stderr = String::from_utf8_lossy(&po.stderr);
                                    let _ = tx.send(DeployMsg::Log(format!("Push failed: {}", stderr.trim())));
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(DeployMsg::Log(format!("Push failed: {}", e)));
                            }
                        }
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let _ = tx.send(DeployMsg::Log(format!("Commit note: {}", stderr.trim())));
                    }
                }
                Err(e) => {
                    let _ = tx.send(DeployMsg::Log(format!("Commit failed: {}", e)));
                }
            }
        }

        let _ = tx.send(DeployMsg::Log("GitHub setup complete.".into()));
        let _ = tx.send(DeployMsg::Done);
    });

    rx
}
