use std::sync::mpsc;
use std::thread;

use base64::Engine;
use serde_json::json;

pub enum IconGenStatus {
    Generating,
    Done(String),  // file path
    Error(String),
}

pub type IconReceiver = mpsc::Receiver<IconGenStatus>;

/// Spawn a background thread to generate an app icon via the xAI Grok API.
/// If `existing_icon_path` is provided, sends it to the edit endpoint for iteration.
pub fn generate_icon(description: &str, app_name: &str, existing_icon_path: Option<&str>) -> IconReceiver {
    let (tx, rx) = mpsc::channel();
    let description = description.to_string();
    let app_name = app_name.to_string();
    let existing_icon = existing_icon_path.map(|s| s.to_string());

    thread::spawn(move || {
        let _ = tx.send(IconGenStatus::Generating);

        let api_key = match std::env::var("XAI_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => {
                let _ = tx.send(IconGenStatus::Error(
                    "XAI_API_KEY not set. Add it to your .zshrc and restart.".to_string(),
                ));
                return;
            }
        };

        let base_prompt = format!(
            "Create a professional app icon for an application called \"{}\". \
             {}. \
             Requirements: square icon, modern flat design, no text, vibrant colors, \
             single recognizable symbol. The design must fill the entire square \
             edge-to-edge with no margins or padding. Use a solid WHITE background \
             (not black, not dark). The icon design should extend all the way to \
             the edges of the square, suitable for iOS, macOS, Windows, and Android \
             app stores where the OS applies its own rounding.",
            app_name, description
        );

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        // Check if we have an existing icon to iterate on
        let (endpoint, body) = if let Some(ref icon_path) = existing_icon {
            if let Ok(icon_data) = std::fs::read(icon_path) {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&icon_data);
                let data_uri = format!("data:image/png;base64,{}", b64);
                let prompt = format!(
                    "Iterate and improve on this existing app icon. Keep the same general concept \
                     and style but refine it based on this feedback: {}",
                    base_prompt
                );
                (
                    "https://api.x.ai/v1/images/edits",
                    json!({
                        "model": "grok-imagine-image",
                        "prompt": prompt,
                        "n": 1,
                        "image": {
                            "url": data_uri,
                            "type": "image_url"
                        },
                        "response_format": "b64_json"
                    }),
                )
            } else {
                // Can't read file, fall back to generation
                (
                    "https://api.x.ai/v1/images/generations",
                    json!({
                        "model": "grok-imagine-image",
                        "prompt": base_prompt,
                        "n": 1,
                        "aspect_ratio": "1:1",
                        "response_format": "b64_json"
                    }),
                )
            }
        } else {
            (
                "https://api.x.ai/v1/images/generations",
                json!({
                    "model": "grok-imagine-image",
                    "prompt": base_prompt,
                    "n": 1,
                    "aspect_ratio": "1:1",
                    "response_format": "b64_json"
                }),
            )
        };

        let response = client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send();

        let response = match response {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(IconGenStatus::Error(format!("Request failed: {}", e)));
                return;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            let _ = tx.send(IconGenStatus::Error(format!(
                "API error ({}): {}",
                status, body
            )));
            return;
        }

        let body: serde_json::Value = match response.json() {
            Ok(v) => v,
            Err(e) => {
                let _ = tx.send(IconGenStatus::Error(format!("Parse error: {}", e)));
                return;
            }
        };

        // Extract base64 image data - try both b64_json and url formats
        let image_data = if let Some(b64) = body["data"][0]["b64_json"].as_str() {
            match base64::engine::general_purpose::STANDARD.decode(b64) {
                Ok(data) => data,
                Err(e) => {
                    let _ = tx.send(IconGenStatus::Error(format!("Base64 decode error: {}", e)));
                    return;
                }
            }
        } else if let Some(url) = body["data"][0]["url"].as_str() {
            match client.get(url).send() {
                Ok(r) => match r.bytes() {
                    Ok(bytes) => bytes.to_vec(),
                    Err(e) => {
                        let _ = tx.send(IconGenStatus::Error(format!("Download error: {}", e)));
                        return;
                    }
                },
                Err(e) => {
                    let _ = tx.send(IconGenStatus::Error(format!("Download error: {}", e)));
                    return;
                }
            }
        } else {
            let _ = tx.send(IconGenStatus::Error(format!(
                "Unexpected API response: {}",
                serde_json::to_string_pretty(&body).unwrap_or_default()
            )));
            return;
        };

        // Resize to 512x512 and make background transparent
        let img = match image::load_from_memory(&image_data) {
            Ok(img) => img,
            Err(e) => {
                let _ = tx.send(IconGenStatus::Error(format!("Image decode error: {}", e)));
                return;
            }
        };

        let resized = img.resize_exact(512, 512, image::imageops::FilterType::Lanczos3);
        let mut rgba = resized.to_rgba8();

        // Detect background color from corner pixels
        let bg_color = rgba.get_pixel(0, 0).0;
        let threshold = 60u32;

        for pixel in rgba.pixels_mut() {
            let dr = (pixel[0] as i32 - bg_color[0] as i32).unsigned_abs();
            let dg = (pixel[1] as i32 - bg_color[1] as i32).unsigned_abs();
            let db = (pixel[2] as i32 - bg_color[2] as i32).unsigned_abs();
            let diff = dr + dg + db;
            if diff < threshold {
                // Fully transparent
                pixel[3] = 0;
            } else if diff < threshold * 2 {
                // Feather the edges for smooth anti-aliasing
                let alpha = ((diff - threshold) * 255 / threshold).min(255) as u8;
                pixel[3] = alpha;
            }
        }

        let resized = image::DynamicImage::ImageRgba8(rgba);

        // Save into the png/ directory
        let current_dir = match std::env::current_dir() {
            Ok(d) => d,
            Err(e) => {
                let _ = tx.send(IconGenStatus::Error(format!("Cannot determine working directory: {}", e)));
                return;
            }
        };
        let png_dir = current_dir.join("png");
        if let Err(e) = std::fs::create_dir_all(&png_dir) {
            let _ = tx.send(IconGenStatus::Error(format!("Failed to create png/ directory: {}", e)));
            return;
        }
        let safe_name = app_name
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let filename = format!("{}_icon_{}.png", safe_name, timestamp);
        let save_path = png_dir.join(&filename);

        match resized.save(&save_path) {
            Ok(_) => {
                let _ = tx.send(IconGenStatus::Done(save_path.display().to_string()));
            }
            Err(e) => {
                let _ = tx.send(IconGenStatus::Error(format!("Save error: {}", e)));
            }
        }
    });

    rx
}
