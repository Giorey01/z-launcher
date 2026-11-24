//! The validator: turns a theme text or folder into a list of [`Issue`]s (spec section 7).

use crate::issue::{Issue, IssueCode, Severity};
use crate::manifest::parse_manifest;
use crate::package::ThemePackage;

/// Validates the text of a `theme.json` alone (no files are looked at).
pub fn validate_text(text: &str) -> Vec<Issue> {
    parse_manifest(text).issues
}

/// Validates a theme folder: the issues of `theme.json`, wallpapers that are named but absent
/// (error), `asset_refs` whose id is not in `installed_pack_ids` (warning) and files in
/// `sounds/` that match no action (warning).
pub fn validate_package(pkg: &ThemePackage, installed_pack_ids: &[String]) -> Vec<Issue> {
    let mut issues = pkg.manifest_issues.clone();
    if let Some(manifest) = &pkg.manifest {
        if manifest.wallpaper_main.is_some() && pkg.wallpaper_main.is_none() {
            issues.push(Issue::new(
                Severity::Error,
                IssueCode::MissingWallpaperFile,
                "$.wallpaper_main",
            ));
        }
        if manifest.wallpaper_external.is_some() && pkg.wallpaper_external.is_none() {
            issues.push(Issue::new(
                Severity::Error,
                IssueCode::MissingWallpaperFile,
                "$.wallpaper_external",
            ));
        }
        for (index, asset) in manifest.asset_refs.iter().flatten().enumerate() {
            if !installed_pack_ids.contains(&asset.id) {
                let location = format!("$.asset_refs[{index}]");
                issues.push(Issue::new(
                    Severity::Warning,
                    IssueCode::MissingAssetPack,
                    location,
                ));
            }
        }
    }
    for name in &pkg.unknown_sounds {
        issues.push(Issue::new(
            Severity::Warning,
            IssueCode::UnknownSoundName,
            format!("sounds/{name}"),
        ));
    }
    issues
}
