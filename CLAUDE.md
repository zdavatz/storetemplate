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
- `src/json_output.rs` — `build_json()` assembles JSON from state, `validate()` checks required fields, `save_to_file()` opens native save dialog
- `src/stores/mod.rs` — module registry
- `src/stores/common.rs` — shared fields UI (app name, descriptions, URLs, pricing, age rating)
- `src/stores/apple.rs` — Apple-specific UI (SKU, subtitle, categories, screenshots per device type for macOS/iOS)
- `src/stores/google_play.rs` — Android-specific UI (package name, category, IARC content rating, assets)
- `src/stores/microsoft.rs` — Windows Store UI (category, "what's new", product features, search terms, logos, installer config)
- `src/stores/github.rs` — GitHub Releases UI (tag pattern, branch, draft/prerelease, asset patterns)

## Key Design Decisions

- Common tab holds all shared fields (name, descriptions, keywords, URLs) — store tabs only have store-unique fields to avoid duplicate entry
- Per-language fields render side-by-side language groups using `HashMap<String, String>` keyed by ISO code
- egui immediate mode: conditional rendering based on which stores are checked — no dynamic widget tree needed
- Validation runs on save, not per-keystroke; character counts shown inline with red/gray coloring

## License

GNU General Public License v3.0
