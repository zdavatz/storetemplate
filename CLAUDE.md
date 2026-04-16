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

- `src/main.rs` — eframe entry point, `StoreTemplateApp` struct, top-level render loop with header (store/language checkboxes), tab bar, central scroll area, and footer (save/load/clear buttons). Handles icon texture loading, auto-save polling, and app icon for taskbar
- `src/icon_gen.rs` — AI icon generation via xAI Grok API (`grok-imagine-image` model). Supports new generation and iteration on existing icons via `/images/generations` and `/images/edits` endpoints. Post-processes images to make background transparent. Saves icons to `png/` directory
- `src/state.rs` — all form state: `AppState` (top-level), `CommonState`, `AppleState`, `GooglePlayState`, `MicrosoftState`, `GithubState`. Per-language fields use `HashMap<String, String>`. `SavedState` for JSON serialization. Auto-save/load functions for `json/` directory
- `src/widgets.rs` — reusable form widget helpers: `text_field`, `multiline_field`, `choice_field`, `bool_field`, `list_field`, `path_field`, `dir_field`, `url_field`, `email_field`, `per_language_text`, `per_language_multiline`, `per_language_list`
- `src/languages.rs` — `LANGUAGES` constant (20 ISO codes with display names)
- `src/json_output.rs` — `build_json()` assembles JSON from state, `validate()` checks required fields, `save_to_file()` opens native save dialog and also generates `.github/workflows/release.yml`
- `src/workflow.rs` — `build_workflow()` generates GitHub Actions release workflow YAML based on selected stores (build jobs for macOS/iOS/Windows/Android/AppImage + create-release job)
- `src/stores/mod.rs` — module registry
- `src/stores/common.rs` — shared fields UI (app name, descriptions, URLs, pricing, age rating, icon description field, generate/iterate icon buttons, icon preview). Bundle/Package ID auto-suggested from app name as `com.example.appname`
- `src/stores/apple.rs` — Apple-specific UI (SKU with auto-suggest and App Store Connect link, subtitle, categories, screenshots per device type for macOS/iOS)
- `src/stores/google_play.rs` — Android-specific UI (package name with Google Play Console link, category, IARC content rating, assets)
- `src/stores/microsoft.rs` — Windows Store UI (App ID with Partner Center link, category, support info/phone/address for Properties page, "what's new", product features, search terms, logos, installer config)
- `src/stores/github.rs` — GitHub Releases UI (tag pattern, branch, draft/prerelease, build AppImage option, asset patterns)
- `src/deploy.rs` — Store API integration for one-click deployment:
  - `autofill_credentials()` — reads `~/.apple/credentials.json` + `~/.config/gh/hosts.yml` to populate all credential fields
  - `deploy_apple()` — App Store Connect API: JWT auth (ES256), bundle ID registration, app info/version localizations (per-language), provisioning profile creation
  - `deploy_microsoft()` — Microsoft Store Submission API **v2** (`api.store.microsoft.com/submission/v1/product/{productId}`). Entra ID OAuth2 token with `api.store.microsoft.com/.default` scope. PATCH `/metadata` with Properties module (privacyPolicyUrl, website, supportContactInfo, certificationNotes, category, subcategory, productDeclarations) and per-language Listings (description, shortDescription, whatsNew, productFeatures, searchTerms, additionalLicenseTerms, copyright, contactInfo). Requires `X-Seller-Account-Id` header. Metadata-only — binary upload is delegated to the GitHub Actions release workflow. Note: phone/company address are NOT settable via the API and must be entered in Partner Center account settings.
  - `deploy_github()` — sets secrets via `gh` CLI, generates and pushes release.yml workflow
  - All deploy functions run in background threads with `mpsc` channel (same pattern as `icon_gen.rs`)
  - `DeployState` in `state.rs` holds credentials (Apple .p8 path/key ID/issuer ID, Azure tenant/client/secret, **MS Store seller ID**, GitHub PAT/repo), persisted with auto-save. Product ID for the v2 Microsoft API is reused from `MicrosoftState.msstore_app_id`.

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

## Key Design Decisions

- Common tab holds all shared fields (name, descriptions, keywords, URLs) — store tabs only have store-unique fields to avoid duplicate entry
- Per-language fields render side-by-side language groups using `HashMap<String, String>` keyed by ISO code
- egui immediate mode: conditional rendering based on which stores are checked — no dynamic widget tree needed
- Validation runs on save, not per-keystroke; character counts shown inline with red/gray coloring
- Light theme (egui::Visuals::light()) with white tab bar background
- Store-specific fields include direct links to open the relevant store console in the browser (App Store Connect, Google Play Console, Partner Center)
- SKU auto-suggested from app name (lowercase, special chars replaced with underscores)
- Bundle/Package ID auto-suggested as `com.example.<app_name>` — user replaces `com.example` with their domain
- Save status shows "Saved to: ..." for 3 seconds then auto-clears; cancelled save dialog shows nothing
- Save generates both JSON template and `.github/workflows/release.yml` with build jobs matching selected stores
- Auto-save to `json/<app_name>.json` every ~2 seconds and on exit; auto-loads most recent on startup
- Load button opens file picker for `json/` directory to restore any saved state
- AI icon generation via xAI Grok API with background transparency post-processing; icons saved to `png/` with timestamps
- Iterate on existing icon by sending current image to the Grok edit endpoint
- App icon loaded from `png/Storetemplate_icon_1775851683.png` for taskbar/dock display
- Deploy tab reads all metadata from existing form state (Common + store tabs), so user fills form once and deploys to all stores
- Credentials auto-filled from `~/.apple/credentials.json` (Apple + Azure) and `~/.config/gh/hosts.yml` (GitHub PAT)
- Widget ID clashes resolved via `ui.push_id()` for macOS/iOS sections and `from_id_salt(label)` for ComboBoxes

## License

GNU General Public License v3.0
