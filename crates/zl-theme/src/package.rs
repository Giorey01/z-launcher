//! Reading an unpacked theme folder (spec sections 2, 4, 5 and 6).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::ThemeError;
use crate::fsutil::{find_file, list_dir};
use crate::issue::Issue;
use crate::layer_set::read_theme_layers;
use crate::manifest::{parse_manifest, ThemeManifest};
use crate::sounds::SoundAction;
use crate::wallpaper::{resolve, EXTERNAL_FALLBACKS, MAIN_FALLBACKS};

/// Mask and overlay images of one platform (`icon_overlays/<platform>/`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OverlayArt {
    /// `mask.png`, if present.
    pub mask: Option<PathBuf>,
    /// `overlay.png`, if present.
    pub overlay: Option<PathBuf>,
}

/// Images of one smart folder type or platform (`png`, `gif`, `webp`, `jpg` or `jpeg`; the
/// path keeps its extension).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FolderArt {
    /// `icon.<ext>`, if present.
    pub icon: Option<PathBuf>,
    /// `hero.<ext>`, if present.
    pub hero: Option<PathBuf>,
    /// `logo.<ext>`, if present.
    pub logo: Option<PathBuf>,
}

/// Everything found in one theme folder. Paths are full paths under [`ThemePackage::root`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ThemePackage {
    /// The theme folder.
    pub root: PathBuf,
    /// Package id: the folder name (such as `wiive-95ab15`).
    pub id: String,
    /// The parsed `theme.json`, `None` when it was not valid JSON.
    pub manifest: Option<ThemeManifest>,
    /// Problems found while parsing `theme.json`.
    pub manifest_issues: Vec<Issue>,
    /// Resolved main wallpaper file.
    pub wallpaper_main: Option<PathBuf>,
    /// Resolved external wallpaper file.
    pub wallpaper_external: Option<PathBuf>,
    /// `preview.png`, if present.
    pub preview: Option<PathBuf>,
    /// Overlay art by platform id (lower-case folder name).
    pub icon_overlays: BTreeMap<String, OverlayArt>,
    /// The `global` overlay folder: applies to platforms without a folder of their own.
    pub icon_overlay_global: Option<OverlayArt>,
    /// Smart folder art by type name (any folder name is accepted).
    pub smart_folders: BTreeMap<String, FolderArt>,
    /// Smart folder art by platform id (`smart_folders/by_platform/<platform>/`).
    pub smart_folders_by_platform: BTreeMap<String, FolderArt>,
    /// Sound files by action.
    pub sounds: BTreeMap<SoundAction, PathBuf>,
    /// Names of files in `sounds/` that match no action and allowed extension.
    pub unknown_sounds: Vec<String>,
    /// Files in `music/`, sorted by name.
    pub music: Vec<PathBuf>,
}

/// Finds the theme folders inside an extracted archive or a themes folder: each
/// `<dir>/themes/<id>/` holding a `theme.json`, else `<dir>` itself, else each direct
/// subfolder holding one (a single top folder).
pub fn find_themes(dir: &Path) -> Vec<PathBuf> {
    let with_manifest = |parent: &Path| -> Vec<PathBuf> {
        list_dir(parent)
            .into_iter()
            .filter(|e| e.is_dir && e.path.join("theme.json").is_file())
            .map(|e| e.path)
            .collect()
    };
    let nested = with_manifest(&dir.join("themes"));
    if !nested.is_empty() {
        return nested;
    }
    if dir.join("theme.json").is_file() {
        return vec![dir.to_path_buf()];
    }
    with_manifest(dir)
}

/// Reads a theme folder. Fails only if `theme.json` is missing or unreadable; problems inside
/// it are reported in [`ThemePackage::manifest_issues`].
pub fn read_package(root: &Path) -> Result<ThemePackage, ThemeError> {
    let manifest_path = root.join("theme.json");
    if !manifest_path.is_file() {
        return Err(ThemeError::MissingManifest);
    }
    let text = String::from_utf8_lossy(&std::fs::read(&manifest_path)?).into_owned();
    let outcome = parse_manifest(&text);
    let (main, external) = outcome
        .manifest
        .as_ref()
        .map(|m| (m.wallpaper_main.clone(), m.wallpaper_external.clone()))
        .unwrap_or_default();
    let mut pkg = ThemePackage {
        root: root.to_path_buf(),
        id: root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string(),
        wallpaper_main: resolve(root, main.as_deref(), MAIN_FALLBACKS),
        wallpaper_external: resolve(root, external.as_deref(), EXTERNAL_FALLBACKS),
        preview: find_file(&list_dir(root), "preview.png"),
        manifest: outcome.manifest,
        manifest_issues: outcome.issues,
        ..ThemePackage::default()
    };
    let layers = read_theme_layers(root);
    pkg.icon_overlays = layers.icon_overlays;
    pkg.icon_overlay_global = layers.icon_overlay_global;
    pkg.smart_folders = layers.smart_folders;
    pkg.smart_folders_by_platform = layers.smart_folders_by_platform;
    pkg.sounds = layers.sounds;
    pkg.unknown_sounds = layers.unknown_sounds;
    pkg.music = layers.music;
    Ok(pkg)
}
