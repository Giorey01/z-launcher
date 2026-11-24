//! The theme store: importing theme ZIPs and Cocoon data folders into the app's data folder,
//! which has Cocoon's layout (`themes/<id>/`, `assets/<category>/<id>/`) so `zl_theme` reads it
//! as it is.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use zl_config::media::COCOON_MEDIA_FOLDER;
use zl_config::theme::{COCOON_DATA_DIR_CANDIDATES, IMPORT_STAGING_DIR_NAME};
use zl_theme::{
    import_zip, list_installed, read_package, validate_pack, validate_package, AssetCategory,
    AssetPack, InstalledContent, Issue,
};

use crate::appearance_types::{CocoonDataDir, ImportReport, PackRef, ThemeIssue};
use crate::error::CoreError;
use crate::Service;

/// Copies a folder tree, skipping symbolic links. Creates `dst` and its parents.
fn copy_tree(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let target = dst.join(entry.file_name());
        if kind.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// Replaces `dst` with a copy of `src`: the copy is made beside it first, so a failure leaves the
/// old folder in place.
fn replace_tree(src: &Path, dst: &Path) -> io::Result<()> {
    let name = dst.file_name().and_then(|n| n.to_str()).unwrap_or("pack");
    let staging = dst.with_file_name(format!(".{name}.copying"));
    let _ = fs::remove_dir_all(&staging);
    copy_tree(src, &staging)?;
    let _ = fs::remove_dir_all(dst);
    fs::rename(&staging, dst)
}

/// Converts the issues of one source.
fn to_theme_issues(source: &str, issues: &[Issue]) -> Vec<ThemeIssue> {
    issues
        .iter()
        .map(|i| ThemeIssue {
            source: source.to_string(),
            severity: i.severity,
            code: i.code,
            location: i.location.clone(),
        })
        .collect()
}

/// A `copy_failed` entry for the report.
fn copy_failed(source: &str) -> ThemeIssue {
    ThemeIssue {
        source: source.to_string(),
        severity: zl_theme::Severity::Error,
        code: zl_theme::IssueCode::InvalidJson,
        location: "copy_failed".to_string(),
    }
}

/// The version text of the theme in `dir`, `None` when absent or unreadable.
fn theme_version(dir: &Path) -> Option<String> {
    read_package(dir).ok()?.manifest?.version
}

/// The versions of the installed packs by (category, id).
fn pack_versions(content: &InstalledContent) -> HashMap<(AssetCategory, String), Option<String>> {
    content
        .packs
        .iter()
        .map(|p| {
            let version = p.meta.as_ref().and_then(|m| m.version.clone());
            ((p.category, p.id.clone()), version)
        })
        .collect()
}

/// Copies the themes of `src_dir/themes/*` into `data/themes/`, skipping those already installed
/// with the same version.
fn import_themes(data: &Path, src: &InstalledContent, report: &mut ImportReport) {
    for dir in &src.themes {
        let Some(id) = dir.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let dest = data.join("themes").join(id);
        let version = theme_version(dir);
        if dest.join("theme.json").is_file() && theme_version(&dest) == version {
            report.themes_skipped.push(id.to_string());
        } else if replace_tree(dir, &dest).is_ok() {
            report.themes_added.push(id.to_string());
        } else {
            report.issues.push(copy_failed(id));
        }
    }
}

/// Copies the packs of `src_dir/assets/*/*` into `data/assets/`, skipping those already
/// installed with the same version.
fn import_packs(data: &Path, src: &InstalledContent, report: &mut ImportReport) {
    let installed = pack_versions(&list_installed(data));
    for pack in &src.packs {
        let reference = PackRef {
            category: pack.category,
            id: pack.id.clone(),
            name: pack
                .meta
                .as_ref()
                .and_then(|m| m.name.clone())
                .unwrap_or_default(),
        };
        let version = pack.meta.as_ref().and_then(|m| m.version.clone());
        let dest = data
            .join("assets")
            .join(pack.category.as_str())
            .join(&pack.id);
        if installed.get(&(pack.category, pack.id.clone())) == Some(&version) {
            report.packs_skipped.push(reference);
        } else if replace_tree(&pack.dir, &dest).is_ok() {
            report.packs_added.push(reference);
        } else {
            report.issues.push(copy_failed(&pack.id));
        }
    }
}

/// Validates what an import added, against everything installed afterwards.
fn validate_added(data: &Path, report: &mut ImportReport) {
    let installed = list_installed(data);
    let ids = installed.pack_ids();
    for id in &report.themes_added.clone() {
        if let Ok(pkg) = read_package(&data.join("themes").join(id)) {
            report
                .issues
                .extend(to_theme_issues(id, &validate_package(&pkg, &ids)));
        }
    }
    for added in &report.packs_added.clone() {
        let found: Option<&AssetPack> = installed
            .packs
            .iter()
            .find(|p| p.id == added.id && p.category == added.category);
        if let Some(pack) = found {
            report
                .issues
                .extend(to_theme_issues(&pack.id, &validate_pack(pack)));
        }
    }
}

/// Imports a data-folder-shaped `src` (with `themes/` and `assets/`) into `data`.
fn import_data_dir(data: &Path, src: &Path) -> ImportReport {
    let mut report = ImportReport::default();
    let found = list_installed(src);
    import_themes(data, &found, &mut report);
    import_packs(data, &found, &mut report);
    validate_added(data, &mut report);
    report
}

/// 32-bit FNV-1a hash, for ids made from a name.
fn fnv(text: &str) -> u32 {
    text.bytes().fold(0x811c_9dc5, |h, b| {
        (h ^ u32::from(b)).wrapping_mul(0x0100_0193)
    })
}

/// An id for a theme that came without a folder of its own: the name as a slug and a short hash,
/// like the ids Cocoon makes (`wiive-95ab15`).
fn id_from_name(name: &str) -> String {
    let slug: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let slug = slug
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let slug = if slug.is_empty() {
        "theme".to_string()
    } else {
        slug
    };
    format!("{slug}-{:06x}", fnv(name) & 0x00ff_ffff)
}

/// Arranges the themes an archive held as `<norm>/themes/<id>/`, the layout the importer reads.
/// An archive that already has `themes/` is used as it is.
fn normalise(stage: &Path, roots: &[PathBuf], norm: &Path) -> io::Result<PathBuf> {
    if roots.iter().all(|r| r.starts_with(stage.join("themes"))) {
        return Ok(stage.to_path_buf());
    }
    for root in roots {
        let id = if root == stage {
            let name = read_package(root)
                .ok()
                .and_then(|p| p.manifest)
                .map_or_else(String::new, |m| m.name);
            id_from_name(&name)
        } else {
            root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("theme")
                .to_string()
        };
        copy_tree(root, &norm.join("themes").join(id))?;
    }
    Ok(norm.to_path_buf())
}

impl Service {
    /// Imports a theme ZIP: extracts it safely to a staging folder, copies each theme (and each
    /// pack, if the archive has `assets/`) into the data folder and validates what was added.
    /// An archive that cannot be read is an error code; problems inside the themes are
    /// issues in the report. `themes_added` holds the ids of the themes now installed.
    pub fn import_theme_zip(&self, zip_path: &str) -> Result<ImportReport, CoreError> {
        let staging = self.data_dir.join(IMPORT_STAGING_DIR_NAME);
        let _ = fs::remove_dir_all(&staging);
        let stage = staging.join("zip");
        let outcome = import_zip(Path::new(zip_path), &stage)
            .map_err(CoreError::from)
            .and_then(|roots| {
                let source = normalise(&stage, &roots, &staging.join("norm")).map_err(|_| {
                    CoreError::Theme {
                        code: "io".to_string(),
                    }
                })?;
                Ok(import_data_dir(&self.data_dir, &source))
            });
        let _ = fs::remove_dir_all(&staging);
        self.invalidate_theme();
        outcome
    }

    /// Copies the themes and asset packs of a Cocoon data folder into the app's, skipping those
    /// already installed with the same id and version. Media is not copied: it is read in place
    /// by the media import.
    pub fn import_from_cocoon(&self, cocoon_dir: &str) -> Result<ImportReport, CoreError> {
        let src = Path::new(cocoon_dir);
        if !src.is_dir() {
            return Err(CoreError::NotFound);
        }
        let report = import_data_dir(&self.data_dir, src);
        self.invalidate_theme();
        Ok(report)
    }

    /// Looks for Cocoon data folders among the candidate paths of the configuration: a folder
    /// counts when it has a theme (`themes/<id>/theme.json`) or a pack
    /// (`assets/<category>/<id>/metadata.json`).
    pub fn detect_cocoon_data_dirs(&self) -> Vec<CocoonDataDir> {
        COCOON_DATA_DIR_CANDIDATES
            .iter()
            .filter_map(|candidate| describe_cocoon_dir(Path::new(candidate)))
            .collect()
    }
}

/// Describes a candidate Cocoon folder, `None` when it holds no theme and no pack.
pub(crate) fn describe_cocoon_dir(dir: &Path) -> Option<CocoonDataDir> {
    let found = list_installed(dir);
    let with_metadata = found.packs.iter().filter(|p| p.meta.is_some()).count();
    if found.themes.is_empty() && with_metadata == 0 {
        return None;
    }
    Some(CocoonDataDir {
        path: dir.to_string_lossy().into_owned(),
        themes: u32::try_from(found.themes.len()).unwrap_or(u32::MAX),
        packs: u32::try_from(found.packs.len()).unwrap_or(u32::MAX),
        has_media: dir.join(COCOON_MEDIA_FOLDER).is_dir(),
    })
}

#[cfg(test)]
mod tests {
    use super::{describe_cocoon_dir, id_from_name};

    /// The shared pack fixture counts as a Cocoon data folder (1 theme, 9 packs); an empty
    /// folder does not.
    #[test]
    fn describes_a_cocoon_folder() {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/themes/cocoon/data_dir_packs");
        let found = describe_cocoon_dir(&fixture);
        assert_eq!(
            found.map(|d| (d.themes, d.packs, d.has_media)),
            Some((1, 9, false))
        );
        assert!(describe_cocoon_dir(std::path::Path::new("/nonexistent/zl")).is_none());
    }

    /// An id made from a name is a slug plus six hex digits, stable for the same name.
    #[test]
    fn ids_from_names() {
        let id = id_from_name("My Cool Theme!");
        assert!(id.starts_with("my-cool-theme-"), "{id}");
        assert_eq!(id.len(), "my-cool-theme-".len() + 6);
        assert_eq!(id, id_from_name("My Cool Theme!"));
        assert!(id_from_name("???").starts_with("theme-"));
    }
}
