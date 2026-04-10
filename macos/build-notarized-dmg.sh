#!/bin/bash
set -euo pipefail

# Build, sign, notarize, and staple a DMG for GitHub distribution
# Usage: ./macos/build-notarized-dmg.sh [version]
#
# Required environment variables:
#   ASC_KEY_PATH  - Path to AuthKey_7B9HFNP99B.p8
#   ASC_KEY_ID    - App Store Connect API Key ID (7B9HFNP99B)
#   ASC_ISSUER_ID - App Store Connect Issuer ID (UUID)
#
# Optional:
#   SIGNING_IDENTITY - Developer ID identity (default: auto-detected)

VERSION="${1:-1.0.0}"
APP_NAME="StoreTemplate"
BUNDLE_ID="com.ywesee.storetemplate"
TEAM_ID="4B37356EGR"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/build/github"
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"

SIGNING_IDENTITY="${SIGNING_IDENTITY:-Developer ID Application: ywesee GmbH ($TEAM_ID)}"

echo "==> Building StoreTemplate v$VERSION for GitHub (notarized DMG)"

# Clean build directory
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

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
sed "s|<string>1.0.0</string>|<string>$VERSION</string>|" \
    "$SCRIPT_DIR/Info.plist" > "$APP_BUNDLE/Contents/Info.plist"

# Copy icon if exists
if [ -f "$SCRIPT_DIR/AppIcon.icns" ]; then
    cp "$SCRIPT_DIR/AppIcon.icns" "$APP_BUNDLE/Contents/Resources/"
    /usr/libexec/PlistBuddy -c "Add :CFBundleIconFile string AppIcon" "$APP_BUNDLE/Contents/Info.plist" 2>/dev/null || \
    /usr/libexec/PlistBuddy -c "Set :CFBundleIconFile AppIcon" "$APP_BUNDLE/Contents/Info.plist"
fi

# Sign the app bundle with Developer ID
echo "==> Signing app bundle with: $SIGNING_IDENTITY"
codesign --force --deep --sign "$SIGNING_IDENTITY" \
    --entitlements "$SCRIPT_DIR/entitlements-devid.plist" \
    --options runtime \
    --timestamp \
    "$APP_BUNDLE"

# Verify signature
echo "==> Verifying signature..."
codesign --verify --deep --strict --verbose=2 "$APP_BUNDLE"
spctl --assess --type exec --verbose=2 "$APP_BUNDLE" || echo "(spctl may fail before notarization)"

# Create DMG
echo "==> Creating DMG..."
DMG_PATH="$BUILD_DIR/storetemplate-macos-v$VERSION.dmg"
hdiutil create -volname "$APP_NAME" \
    -srcfolder "$APP_BUNDLE" \
    -ov -format UDZO \
    "$DMG_PATH"

# Sign the DMG
codesign --force --sign "$SIGNING_IDENTITY" --timestamp "$DMG_PATH"

# Notarize
if [ -n "${ASC_KEY_PATH:-}" ] && [ -n "${ASC_KEY_ID:-}" ] && [ -n "${ASC_ISSUER_ID:-}" ]; then
    echo "==> Submitting to Apple notary service..."
    xcrun notarytool submit "$DMG_PATH" \
        --key "$ASC_KEY_PATH" \
        --key-id "$ASC_KEY_ID" \
        --issuer "$ASC_ISSUER_ID" \
        --wait

    echo "==> Stapling notarization ticket..."
    xcrun stapler staple "$DMG_PATH"

    echo "==> Verifying notarization..."
    spctl --assess --type open --context context:primary-signature --verbose=2 "$DMG_PATH"
else
    echo ""
    echo "==> Skipping notarization (ASC_KEY_PATH, ASC_KEY_ID, ASC_ISSUER_ID not all set)"
    echo "    To notarize manually:"
    echo "    xcrun notarytool submit '$DMG_PATH' --key PATH_TO_P8 --key-id 7B9HFNP99B --issuer YOUR_ISSUER_ID --wait"
    echo "    xcrun stapler staple '$DMG_PATH'"
fi

echo "==> Done! DMG: $DMG_PATH"
