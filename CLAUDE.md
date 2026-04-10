# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**storetemplate** is a Rust GUI application (egui/eframe) for generating app store submission metadata templates. Users select target stores, fill out a tabbed form, and save a single JSON file covering all selected stores.

## Build & Run

```bash
cargo build          # debug build
cargo build --release  # release build
cargo run            # build and launch GUI
```

## Architecture

- `src/main.rs` — eframe entry point, `StoreTemplateApp` struct, top-level render loop with header (store/language checkboxes), tab bar, central scroll area, and footer (save/clear buttons)
- `src/state.rs` — all form state: `AppState` (top-level), `CommonState`, `AppleState`, `GooglePlayState`, `MicrosoftState`, `GithubState`. Per-language fields use `HashMap<String, String>`
- `src/widgets.rs` — reusable form widget helpers: `text_field`, `multiline_field`, `choice_field`, `bool_field`, `list_field`, `path_field`, `url_field`, `email_field`, `per_language_text`, `per_language_multiline`, `per_language_list`
- `src/languages.rs` — `LANGUAGES` constant (20 ISO codes with display names)
- `src/json_output.rs` — `build_json()` assembles JSON from state, `validate()` checks required fields, `save_to_file()` opens native save dialog and also generates `.github/workflows/release.yml`
- `src/workflow.rs` — `build_workflow()` generates GitHub Actions release workflow YAML based on selected stores (build jobs for macOS/iOS/Windows/Android/AppImage + create-release job)
- `src/stores/mod.rs` — module registry
- `src/stores/common.rs` — shared fields UI (app name, descriptions, URLs, pricing, age rating)
- `src/stores/apple.rs` — Apple-specific UI (SKU with auto-suggest and App Store Connect link, subtitle, categories, screenshots per device type for macOS/iOS)
- `src/stores/google_play.rs` — Android-specific UI (package name with Google Play Console link, category, IARC content rating, assets)
- `src/stores/microsoft.rs` — Windows Store UI (App ID with Partner Center link, category, "what's new", product features, search terms, logos, installer config)
- `src/stores/github.rs` — GitHub Releases UI (tag pattern, branch, draft/prerelease, build AppImage option, asset patterns)

## macOS Build & Release Infrastructure

- `macos/Info.plist` — App bundle metadata (bundle ID: `com.ywesee.storetemplate`, team: `4B37356EGR`)
- `macos/entitlements-appstore.plist` — App Store entitlements (sandbox, file access, network)
- `macos/entitlements-devid.plist` — Developer ID entitlements (JIT, unsigned memory, library validation)
- `macos/build-appstore.sh` — Local script for App Store .pkg build and upload
- `macos/build-notarized-dmg.sh` — Local script for notarized DMG build
- `.github/workflows/release.yml` — CI pipeline: universal binary, signing, notarization, App Store upload, Windows ZIP, Linux AppImage

### Signing Identities

- `Developer ID Application: ywesee GmbH (4B37356EGR)` — GitHub DMG notarization
- `Apple Distribution: ywesee GmbH (4B37356EGR)` — App Store app signing
- `3rd Party Mac Developer Application: ywesee GmbH (4B37356EGR)` — App Store app signing (legacy name)
- `3rd Party Mac Developer Installer: ywesee GmbH (4B37356EGR)` — App Store .pkg signing

### Certificate Setup

The p12 files are created by combining `.cer` files from the Apple Developer Portal with `mac_dist.key` using openssl:
```bash
openssl x509 -in mac_app.cer -inform DER -out mac_app.pem
openssl pkcs12 -export -out mac_app.p12 -inkey mac_dist.key -in mac_app.pem -passout pass:PASSWORD -legacy
```
The `-legacy` flag is required for macOS `security import` compatibility.

### App Store Connect API

- Key ID: `7B9HFNP99B`
- Issuer ID: `69a6de70-0490-47e3-e053-5b8c7c11a4d1`
- Key file: `AuthKey_7B9HFNP99B.p8` in iCloud `ywesee/p8/`

## Key Design Decisions

- Common tab holds all shared fields (name, descriptions, keywords, URLs) — store tabs only have store-unique fields to avoid duplicate entry
- Per-language fields render side-by-side language groups using `HashMap<String, String>` keyed by ISO code
- egui immediate mode: conditional rendering based on which stores are checked — no dynamic widget tree needed
- Validation runs on save, not per-keystroke; character counts shown inline with red/gray coloring
- Light theme (egui::Visuals::light()) with white tab bar background
- Store-specific fields include direct links to open the relevant store console in the browser (App Store Connect, Google Play Console, Partner Center)
- SKU auto-suggested from app name (lowercase, special chars replaced with underscores)
- Save generates both JSON template and `.github/workflows/release.yml` with build jobs matching selected stores
- Widget ID clashes resolved via `ui.push_id()` for macOS/iOS sections and `from_id_salt(label)` for ComboBoxes

## License

GNU General Public License v3.0
