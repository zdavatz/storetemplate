# storetemplate

GUI application for generating app store submission templates. Select your target stores, fill out the form once, and save a single JSON file with all metadata needed to submit your app via store APIs.

Built with Rust and [egui](https://github.com/emilk/egui).

## Supported Stores

- **macOS** — Apple App Store (macOS)
- **iOS** — Apple App Store (iOS)
- **Windows** — Microsoft Store
- **Android** — Google Play
- **GitHub** — GitHub Releases

## Usage

```bash
cargo run
```

The GUI opens with store checkboxes and language selection at the top. Select your target stores, fill out the Common tab (shared across all stores), then fill in store-specific tabs. Click "Save Template" to export to `{app_name}.json`.

## What's Covered

Each template includes the maximum metadata supported by each store's API:

- **Common** (filled once, shared by all stores): app name, descriptions (multi-language), keywords, URLs, contact, pricing, age rating
- **Apple**: SKU, subtitle, promotional text, categories, screenshots per device type
- **Google Play**: package name, category, feature graphic, IARC content rating, release track
- **Microsoft Store**: "What's new", product features, search terms, store logos, installer config, system requirements
- **GitHub**: tag pattern, release notes template, draft/prerelease flags, asset patterns

## Building

Requires Rust toolchain (rustup.rs).

```bash
cargo build --release
```

The binary is at `target/release/storetemplate.exe` (Windows) or `target/release/storetemplate` (macOS/Linux).

## Dependencies

- `eframe` / `egui` — GUI framework
- `serde` / `serde_json` — JSON serialization
- `chrono` — timestamps
- `rfd` — native file dialogs

## License

GNU General Public License v3.0
