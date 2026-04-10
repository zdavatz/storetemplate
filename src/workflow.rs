use crate::state::AppState;

/// Build a GitHub Actions workflow YAML string based on selected stores.
pub fn build_workflow(state: &AppState) -> String {
    let mut lines = Vec::new();

    lines.push("name: Release".to_string());
    lines.push(String::new());
    lines.push("on:".to_string());
    lines.push("  push:".to_string());
    lines.push("    tags:".to_string());
    lines.push("      - 'v*'".to_string());
    lines.push(String::new());
    lines.push("permissions:".to_string());
    lines.push("  contents: write".to_string());
    lines.push(String::new());
    lines.push("jobs:".to_string());

    if state.store_macos {
        lines.push(String::new());
        lines.push("  build-macos:".to_string());
        lines.push("    runs-on: macos-latest".to_string());
        lines.push("    steps:".to_string());
        lines.push("      - uses: actions/checkout@v4".to_string());
        lines.push("      - name: Build macOS app".to_string());
        lines.push("        run: cargo build --release".to_string());
        lines.push("      - name: Create DMG".to_string());
        lines.push("        run: |".to_string());
        lines.push("          mkdir -p dmg_contents".to_string());
        lines.push("          cp target/release/${{ github.event.repository.name }} dmg_contents/".to_string());
        lines.push("          hdiutil create -volname \"${{ github.event.repository.name }}\" -srcfolder dmg_contents -ov -format UDZO ${{ github.event.repository.name }}.dmg".to_string());
        lines.push("      - name: Upload macOS artifact".to_string());
        lines.push("        uses: actions/upload-artifact@v4".to_string());
        lines.push("        with:".to_string());
        lines.push("          name: macos-build".to_string());
        lines.push("          path: '*.dmg'".to_string());
    }

    if state.store_ios {
        lines.push(String::new());
        lines.push("  build-ios:".to_string());
        lines.push("    runs-on: macos-latest".to_string());
        lines.push("    steps:".to_string());
        lines.push("      - uses: actions/checkout@v4".to_string());
        lines.push("      - name: Build iOS app".to_string());
        lines.push("        run: |".to_string());
        lines.push("          xcodebuild -scheme ${{ github.event.repository.name }} -sdk iphoneos -configuration Release archive \\".to_string());
        lines.push("            -archivePath build/${{ github.event.repository.name }}.xcarchive".to_string());
        lines.push("      - name: Export IPA".to_string());
        lines.push("        run: |".to_string());
        lines.push("          xcodebuild -exportArchive \\".to_string());
        lines.push("            -archivePath build/${{ github.event.repository.name }}.xcarchive \\".to_string());
        lines.push("            -exportOptionsPlist ExportOptions.plist \\".to_string());
        lines.push("            -exportPath build/export".to_string());
        lines.push("      - name: Upload iOS artifact".to_string());
        lines.push("        uses: actions/upload-artifact@v4".to_string());
        lines.push("        with:".to_string());
        lines.push("          name: ios-build".to_string());
        lines.push("          path: 'build/export/*.ipa'".to_string());
    }

    if state.store_windows {
        lines.push(String::new());
        lines.push("  build-windows:".to_string());
        lines.push("    runs-on: windows-latest".to_string());
        lines.push("    steps:".to_string());
        lines.push("      - uses: actions/checkout@v4".to_string());
        lines.push("      - name: Build Windows app".to_string());
        lines.push("        run: cargo build --release".to_string());
        lines.push("      - name: Upload Windows artifact".to_string());
        lines.push("        uses: actions/upload-artifact@v4".to_string());
        lines.push("        with:".to_string());
        lines.push("          name: windows-build".to_string());
        lines.push("          path: target/release/*.exe".to_string());
    }

    if state.store_android {
        lines.push(String::new());
        lines.push("  build-android:".to_string());
        lines.push("    runs-on: ubuntu-latest".to_string());
        lines.push("    steps:".to_string());
        lines.push("      - uses: actions/checkout@v4".to_string());
        lines.push("      - name: Set up JDK".to_string());
        lines.push("        uses: actions/setup-java@v4".to_string());
        lines.push("        with:".to_string());
        lines.push("          distribution: temurin".to_string());
        lines.push("          java-version: '17'".to_string());
        lines.push("      - name: Build Android APK".to_string());
        lines.push("        run: ./gradlew assembleRelease".to_string());
        lines.push("      - name: Upload Android artifact".to_string());
        lines.push("        uses: actions/upload-artifact@v4".to_string());
        lines.push("        with:".to_string());
        lines.push("          name: android-build".to_string());
        lines.push("          path: 'app/build/outputs/apk/release/*.apk'".to_string());
    }

    if state.github.build_appimage {
        lines.push(String::new());
        lines.push("  build-appimage:".to_string());
        lines.push("    runs-on: ubuntu-latest".to_string());
        lines.push("    steps:".to_string());
        lines.push("      - uses: actions/checkout@v4".to_string());
        lines.push("      - name: Install dependencies".to_string());
        lines.push("        run: |".to_string());
        lines.push("          sudo apt-get update".to_string());
        lines.push("          sudo apt-get install -y libfuse2 desktop-file-utils".to_string());
        lines.push("      - name: Build release binary".to_string());
        lines.push("        run: cargo build --release".to_string());
        lines.push("      - name: Create AppImage".to_string());
        lines.push("        run: |".to_string());
        lines.push("          wget -q https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage -O appimagetool".to_string());
        lines.push("          chmod +x appimagetool".to_string());
        lines.push("          mkdir -p AppDir/usr/bin AppDir/usr/share/icons/hicolor/256x256/apps".to_string());
        lines.push("          cp target/release/${{ github.event.repository.name }} AppDir/usr/bin/".to_string());
        lines.push("          # Copy your .desktop file and icon to AppDir".to_string());
        lines.push("          # cp assets/app.desktop AppDir/".to_string());
        lines.push("          # cp assets/icon.png AppDir/usr/share/icons/hicolor/256x256/apps/".to_string());
        lines.push("          ARCH=x86_64 ./appimagetool AppDir".to_string());
        lines.push("      - name: Upload AppImage artifact".to_string());
        lines.push("        uses: actions/upload-artifact@v4".to_string());
        lines.push("        with:".to_string());
        lines.push("          name: appimage-build".to_string());
        lines.push("          path: '*.AppImage'".to_string());
    }

    // GitHub Release job
    if state.store_github {
        let mut needs = Vec::new();
        if state.store_macos { needs.push("build-macos"); }
        if state.store_ios { needs.push("build-ios"); }
        if state.store_windows { needs.push("build-windows"); }
        if state.store_android { needs.push("build-android"); }
        if state.github.build_appimage { needs.push("build-appimage"); }

        lines.push(String::new());
        lines.push("  create-release:".to_string());
        lines.push("    runs-on: ubuntu-latest".to_string());
        if !needs.is_empty() {
            lines.push(format!("    needs: [{}]", needs.join(", ")));
        }
        lines.push("    steps:".to_string());
        lines.push("      - uses: actions/checkout@v4".to_string());

        if !needs.is_empty() {
            lines.push("      - name: Download all artifacts".to_string());
            lines.push("        uses: actions/download-artifact@v4".to_string());
            lines.push("        with:".to_string());
            lines.push("          path: artifacts/".to_string());
        }

        let draft = state.github.draft;
        let prerelease = state.github.prerelease;
        let generate_notes = state.github.generate_release_notes;

        lines.push("      - name: Create GitHub Release".to_string());
        lines.push("        uses: softprops/action-gh-release@v2".to_string());
        lines.push("        with:".to_string());
        lines.push(format!("          draft: {}", draft));
        lines.push(format!("          prerelease: {}", prerelease));
        lines.push(format!("          generate_release_notes: {}", generate_notes));

        if !state.github.release_name_template.is_empty() {
            lines.push(format!("          name: '{}'", state.github.release_name_template));
        }

        let has_artifacts = !needs.is_empty();
        let has_patterns = !state.github.asset_patterns.is_empty();
        if has_artifacts || has_patterns {
            lines.push("          files: |".to_string());
            if has_artifacts {
                lines.push("            artifacts/**/*".to_string());
            }
            if has_patterns {
                for pattern in state.github.asset_patterns.split(',') {
                    let pattern = pattern.trim();
                    if !pattern.is_empty() {
                        lines.push(format!("            {}", pattern));
                    }
                }
            }
        }

        lines.push("        env:".to_string());
        lines.push("          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}".to_string());
    }

    lines.push(String::new());
    lines.join("\n")
}
