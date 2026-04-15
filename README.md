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

The GUI opens with store checkboxes and language selection at the top. Select your target stores, fill out the Common tab (shared across all stores), then fill in store-specific tabs. Click "Save" to export to `{app_name}.json` and auto-generate a `.github/workflows/release.yml` with build jobs for all selected stores.

Form state is auto-saved to the `json/` directory and restored on next launch.

## What's Covered

Each template includes the maximum metadata supported by each store's API:

- **Common** (filled once, shared by all stores): app name, bundle/package ID (auto-suggested), descriptions (multi-language), keywords, URLs, contact, pricing, age rating, AI icon generation
- **Apple**: SKU (auto-suggested, with link to App Store Connect), subtitle, promotional text, categories, screenshots per device type
- **Google Play**: package name (with link to Google Play Console), category, feature graphic, IARC content rating, release track
- **Microsoft Store**: App ID (with link to Partner Center), "What's new", product features, search terms, store logos, installer config, system requirements
- **GitHub**: tag pattern, release notes template, draft/prerelease flags, build AppImage option, asset patterns

## Building

Requires Rust toolchain (rustup.rs).

```bash
cargo build --release
```

The binary is at `target/release/storetemplate.exe` (Windows) or `target/release/storetemplate` (macOS/Linux).

## Releases

Releases are automated via GitHub Actions. Push a tag to trigger a build:

```bash
git tag v1.2.0 && git push origin v1.2.0
```

The CI pipeline produces:

- **macOS** — Universal binary (arm64 + x86_64), signed with Developer ID, notarized and stapled DMG, plus App Store .pkg uploaded to App Store Connect
- **Windows** — Portable ZIP
- **Linux** — AppImage

### macOS Code Signing & Notarization

The macOS build is signed with `Developer ID Application: ywesee GmbH` and notarized via Apple's notary service. The App Store package is signed with `Apple Distribution` and `3rd Party Mac Developer Installer` certificates and uploaded to App Store Connect via the API.

Required GitHub Secrets:

| Secret | Description |
|---|---|
| `APPLE_API_KEY_ID` | App Store Connect API Key ID |
| `APPLE_API_KEY_P8` | App Store Connect API key (.p8, base64) |
| `APPLE_API_ISSUER_ID` | App Store Connect Issuer ID |
| `APPLE_TEAM_ID` | Apple Developer Team ID |
| `MACOS_CERTIFICATE` | Mac App Distribution cert (.p12, base64) |
| `MACOS_CERTIFICATE_PASSWORD` | Password for above |
| `MACOS_INSTALLER_CERTIFICATE` | Mac Installer Distribution cert (.p12, base64) |
| `MACOS_INSTALLER_CERTIFICATE_PASSWORD` | Password for above |
| `MACOS_DEVELOPER_ID_CERTIFICATE` | Developer ID Application cert (.p12, base64) |
| `MACOS_DEVELOPER_ID_CERTIFICATE_PASSWORD` | Password for above |

### Setting up a new Mac

Import the signing certificates into your Keychain:

```bash
security import /path/to/mac_app_distribution.p12 -k ~/Library/Keychains/login.keychain-db -P "PASSWORD"
security import /path/to/mac_installer_distribution.p12 -k ~/Library/Keychains/login.keychain-db -P "PASSWORD"
```

For Developer ID, use Xcode: Settings > Accounts > ywesee GmbH > Manage Certificates.

## Generated Workflow

When saving a template, a `.github/workflows/release.yml` is generated alongside the JSON. It includes build jobs for each selected store:

- **macOS** — `cargo build --release` + DMG creation
- **iOS** — `xcodebuild` archive + IPA export
- **Windows** — `cargo build --release` + .exe artifact
- **Android** — Gradle `assembleRelease` + APK artifact
- **AppImage** (optional) — `cargo build --release` + `appimagetool` packaging
- **GitHub Release** — `softprops/action-gh-release`, collects all build artifacts

## Deploy (API Integration)

The **Deploy** tab lets you push metadata directly to store APIs — no manual form-filling in App Store Connect or Partner Center needed.

### Auto-fill Credentials

Click **"Auto-fill Credentials"** to load credentials from:
- `~/.apple/credentials.json` — Apple API Key ID, Issuer ID, .p8 key path, Azure AD Tenant/Client/Secret
- `~/.config/gh/hosts.yml` — GitHub PAT (from `gh` CLI)

### Apple App Store Connect

Reads from Common + Apple tabs and via the App Store Connect API:
- Registers Bundle ID (or finds existing)
- Creates/updates app info localizations per language (name, subtitle, privacy policy URL)
- Creates/updates version localizations per language (description, keywords, support URL, marketing URL)
- Sets copyright and primary locale
- Creates Mac App Store provisioning profile

### Microsoft Partner Center

Reads from Common + Windows tabs and via the Partner Center API:
- Authenticates via Azure AD OAuth2
- Creates/updates submission with per-language listings (title, description, keywords, features, search terms, release notes, URLs)
- Commits submission for review

### GitHub Secrets & Workflow

Uses `gh` CLI to:
- Set all Apple and Azure secrets in the target repository
- Generate and push `release.yml` workflow

### Credentials File (`~/.apple/credentials.json`)

```json
{
  "apple": {
    "api_key_id": "YOUR_KEY_ID",
    "api_key_path": "~/.apple/AuthKey_XXXX.p8",
    "api_issuer_id": "YOUR_ISSUER_UUID"
  },
  "azure": {
    "tenant_id": "YOUR_TENANT_UUID",
    "client_id": "YOUR_CLIENT_UUID",
    "client_secret": "YOUR_SECRET"
  }
}
```

## AI Icon Generation

Generate app icons directly from a text description using the xAI Grok API. Set your API key:

```bash
export XAI_API_KEY="your-key-here"
```

Features:
- **Generate New Icon** — creates a fresh icon from your description
- **Iterate on Icon** — sends the current icon + description to refine the design
- Background is automatically made transparent via post-processing
- All generated icons are saved in the `png/` directory with timestamps
- Icon preview displayed inline in the GUI

## Dependencies

- `eframe` / `egui` — GUI framework
- `serde` / `serde_json` — JSON serialization
- `chrono` — timestamps
- `rfd` — native file dialogs
- `open` — open URLs in default browser
- `reqwest` — HTTP client for Grok API
- `image` — image processing and background removal
- `base64` — base64 encoding/decoding
- `jsonwebtoken` — ES256 JWT signing for Apple App Store Connect API

## License

GNU General Public License v3.0
