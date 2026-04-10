use eframe::egui;

use crate::state::GithubState;
use crate::widgets;

pub fn ui_section(ui: &mut egui::Ui, state: &mut GithubState) {
    ui.heading("GitHub Releases");
    ui.add_space(8.0);

    widgets::text_field(ui, "Tag pattern ({version} replaced)", &mut state.tag_pattern, None, true);
    widgets::text_field(ui, "Target branch", &mut state.target_branch, None, true);
    widgets::text_field(ui, "Release name template", &mut state.release_name_template, None, false);

    ui.add_space(8.0);
    widgets::multiline_field(ui, "Release notes template", &mut state.release_notes_template, None, false);

    ui.add_space(8.0);
    widgets::bool_field(ui, "Create as draft", &mut state.draft);
    widgets::bool_field(ui, "Mark as pre-release", &mut state.prerelease);
    widgets::bool_field(ui, "Auto-generate release notes from PRs", &mut state.generate_release_notes);
    widgets::bool_field(ui, "Build AppImage (Linux)", &mut state.build_appimage);

    ui.add_space(8.0);
    widgets::list_field(ui, "Asset file patterns (e.g. *.dmg, *.zip)", &mut state.asset_patterns, None);
}
