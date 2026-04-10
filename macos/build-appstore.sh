#!/bin/bash
set -euo pipefail

# Build and package StoreTemplate for Mac App Store submission
# Usage: ./macos/build-appstore.sh [version]
#
# Required environment variables:
#   APP_CERT_P12       - Path to mac_app_distribution.p12
#   INSTALLER_CERT_P12 - Path to mac_installer_distribution.p12
#   P12_PASSWORD       - Password for p12 files
#   ASC_KEY_PATH       - Path to AuthKey_7B9HFNP99B.p8
#   ASC_KEY_ID         - App Store Connect API Key ID (7B9HFNP99B)
#   ASC_ISSUER_ID      - App Store Connect Issuer ID (UUID)

VERSION="${1:-1.0.0}"
BUILD_NUMBER="${2:-1}"
APP_NAME="StoreTemplate"
BUNDLE_ID="com.ywesee.storetemplate"
TEAM_ID="4B37356EGR"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/build/appstore"
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"

# Signing identities (from the p12 files)
APP_SIGNING_IDENTITY="3rd Party Mac Developer Application: ywesee GmbH ($TEAM_ID)"
INSTALLER_SIGNING_IDENTITY="3rd Party Mac Developer Installer: ywesee GmbH ($TEAM_ID)"

echo "==> Building StoreTemplate v$VERSION ($BUILD_NUMBER) for Mac App Store"

# Clean build directory
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

# Import certificates if p12 paths are set
if [ -n "${APP_CERT_P12:-}" ] && [ -n "${INSTALLER_CERT_P12:-}" ]; then
    echo "==> Importing certificates..."
    KEYCHAIN="build.keychain"
    KEYCHAIN_PASSWORD="build"
    security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN" 2>/dev/null || true
    security default-keychain -s "$KEYCHAIN"
    security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
    security set-keychain-settings -t 3600 -u "$KEYCHAIN"

    security import "$APP_CERT_P12" -k "$KEYCHAIN" -P "${P12_PASSWORD:-}" -T /usr/bin/codesign -T /usr/bin/productbuild
    security import "$INSTALLER_CERT_P12" -k "$KEYCHAIN" -P "${P12_PASSWORD:-}" -T /usr/bin/codesign -T /usr/bin/productbuild
    security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN"

    # Add the keychain to the search list
    security list-keychains -d user -s "$KEYCHAIN" $(security list-keychains -d user | tr -d '"')
fi

# Build release binary
echo "==> Building release binary..."
cd "$PROJECT_DIR"
cargo build --release

# Create app bundle
echo "==> Creating app bundle..."
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

cp "target/release/storetemplate" "$APP_BUNDLE/Contents/MacOS/"

# Generate Info.plist with version
sed -e "s|<string>1.0.0</string>|<string>$VERSION</string>|" \
    -e "s|<string>1</string>|<string>$BUILD_NUMBER</string>|" \
    "$SCRIPT_DIR/Info.plist" > "$APP_BUNDLE/Contents/Info.plist"

# Copy icon if exists
if [ -f "$SCRIPT_DIR/AppIcon.icns" ]; then
    cp "$SCRIPT_DIR/AppIcon.icns" "$APP_BUNDLE/Contents/Resources/"
    /usr/libexec/PlistBuddy -c "Add :CFBundleIconFile string AppIcon" "$APP_BUNDLE/Contents/Info.plist" 2>/dev/null || \
    /usr/libexec/PlistBuddy -c "Set :CFBundleIconFile AppIcon" "$APP_BUNDLE/Contents/Info.plist"
fi

# Sign the app bundle
echo "==> Signing app bundle with: $APP_SIGNING_IDENTITY"
codesign --force --deep --sign "$APP_SIGNING_IDENTITY" \
    --entitlements "$SCRIPT_DIR/entitlements-appstore.plist" \
    --options runtime \
    "$APP_BUNDLE"

# Verify signature
echo "==> Verifying signature..."
codesign --verify --deep --strict "$APP_BUNDLE"

# Create installer package
echo "==> Creating installer package..."
PKG_PATH="$BUILD_DIR/$APP_NAME-$VERSION.pkg"
productbuild \
    --component "$APP_BUNDLE" /Applications \
    --sign "$INSTALLER_SIGNING_IDENTITY" \
    "$PKG_PATH"

echo "==> Package created: $PKG_PATH"

# Upload to App Store Connect
if [ -n "${ASC_KEY_PATH:-}" ] && [ -n "${ASC_KEY_ID:-}" ] && [ -n "${ASC_ISSUER_ID:-}" ]; then
    echo "==> Uploading to App Store Connect..."
    xcrun altool --upload-app \
        --file "$PKG_PATH" \
        --type macos \
        --apiKey "$ASC_KEY_ID" \
        --apiIssuer "$ASC_ISSUER_ID" \
        --show-progress
    echo "==> Upload complete! Check App Store Connect for processing status."
else
    echo ""
    echo "==> Skipping upload (ASC_KEY_PATH, ASC_KEY_ID, ASC_ISSUER_ID not all set)"
    echo "    To upload manually:"
    echo "    xcrun altool --upload-app --file '$PKG_PATH' --type macos --apiKey 7B9HFNP99B --apiIssuer YOUR_ISSUER_ID"
fi

# Cleanup temp keychain if we created one
if [ -n "${APP_CERT_P12:-}" ]; then
    security default-keychain -s login.keychain
fi

echo "==> Done!"
