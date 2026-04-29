use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

use serde_json::json;

use crate::languages::language_name;

pub enum TranslateStatus {
    Translating,
    /// (target_language_code, field_key -> translated_text)
    Done(String, HashMap<String, String>),
    Error(String),
}

pub type TranslateReceiver = mpsc::Receiver<TranslateStatus>;

/// Translate a batch of named text fields from one language to another using xAI Grok.
/// `fields` maps field_key -> source_text. Returned map uses the same keys.
/// Empty values are skipped. Comma-separated lists (e.g. keywords) stay comma-separated.
pub fn translate_fields(
    fields: HashMap<String, String>,
    from_lang: &str,
    to_lang: &str,
) -> TranslateReceiver {
    let (tx, rx) = mpsc::channel();
    let from_lang = from_lang.to_string();
    let to_lang = to_lang.to_string();

    thread::spawn(move || {
        let _ = tx.send(TranslateStatus::Translating);

        let api_key = match std::env::var("XAI_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => {
                let _ = tx.send(TranslateStatus::Error(
                    "XAI_API_KEY not set. Add it to your .zshrc and restart.".to_string(),
                ));
                return;
            }
        };

        let non_empty: HashMap<String, String> = fields
            .into_iter()
            .filter(|(_, v)| !v.trim().is_empty())
            .collect();
        if non_empty.is_empty() {
            let _ = tx.send(TranslateStatus::Error(
                format!("No source text found in {} fields.", language_name(&from_lang)),
            ));
            return;
        }

        let from_name = language_name(&from_lang);
        let to_name = language_name(&to_lang);

        let input_json = match serde_json::to_string(&non_empty) {
            Ok(s) => s,
            Err(e) => {
                let _ = tx.send(TranslateStatus::Error(format!("Encode error: {}", e)));
                return;
            }
        };

        let prompt = format!(
            "Translate the string values in the following JSON object from {} to {}. \
             Preserve every key exactly. Do not translate keys. Keep formatting \
             (comma-separated keyword lists must stay comma-separated; line breaks \
             must be preserved). Return ONLY a single valid JSON object with the \
             translated values — no commentary, no markdown fences.\n\n{}",
            from_name, to_name, input_json
        );

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        let body = json!({
            "model": "grok-3-mini-fast",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a professional translator for app store metadata. \
                                You always return a single valid JSON object with the same \
                                keys as the input."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.2
        });

        let response = client
            .post("https://api.x.ai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send();

        let response = match response {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(TranslateStatus::Error(format!("Request failed: {}", e)));
                return;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            let _ = tx.send(TranslateStatus::Error(format!(
                "API error ({}): {}",
                status, body
            )));
            return;
        }

        let body: serde_json::Value = match response.json() {
            Ok(v) => v,
            Err(e) => {
                let _ = tx.send(TranslateStatus::Error(format!("Parse error: {}", e)));
                return;
            }
        };

        let content = match body["choices"][0]["message"]["content"].as_str() {
            Some(s) => s.to_string(),
            None => {
                let _ = tx.send(TranslateStatus::Error(format!(
                    "Unexpected API response: {}",
                    serde_json::to_string_pretty(&body).unwrap_or_default()
                )));
                return;
            }
        };

        let cleaned = strip_code_fences(&content);

        let translated: HashMap<String, String> = match serde_json::from_str(&cleaned) {
            Ok(v) => v,
            Err(e) => {
                let _ = tx.send(TranslateStatus::Error(format!(
                    "Could not parse translated JSON: {} — got: {}",
                    e, cleaned
                )));
                return;
            }
        };

        let _ = tx.send(TranslateStatus::Done(to_lang.clone(), translated));
    });

    rx
}

fn strip_code_fences(s: &str) -> String {
    let trimmed = s.trim();
    let without_prefix = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed);
    let without_suffix = without_prefix.strip_suffix("```").unwrap_or(without_prefix);
    without_suffix.trim().to_string()
}
