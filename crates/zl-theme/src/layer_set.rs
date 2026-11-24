//! The folder-based layers shared by themes and installed asset packs: icon overlays, smart
//! folders, sounds and music (spec sections 4, 5, 6 and 3.8).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use zl_config::theme::PACK_MUSIC_SCAN_DEPTH;

use crate::fsutil::{find_file, find_stem, list_dir, list_files_deep};
use crate::manifest::AssetCategory;
use crate::package::{FolderArt, OverlayArt};
use crate::sounds::{action_of_file, is_sound_extension, SoundAction};

/// Folder name of the overlays that apply to platforms without their own folder.
pub const GLOBAL_OVERLAY_ID: &str = "global";

/// Extensions of smart folder art seen in real packs, in order of preference.
pub const FOLDER_ART_EXTENSIONS: &[&str] = &["png", "gif", "webp", "jpg", "jpeg"];

/// Smart folder types that are not platforms. Some authors wrongly put them under
/// `by_platform/`; the reader accepts them in both places.
pub const SPECIAL_SMART_FOLDER_TYPES: &[&str] = &[
    "favorites",
    "recent",
    "lastplayed",
    "emulators",
    "androidgames",
    "android_app",
    "cloud",
];

/// Extensions of music files in an installed pack.
const MUSIC_EXTENSIONS: &[&str] = &["mp3", "ogg", "wav", "m4a", "flac", "opus", "aac"];

/// The layers found in one theme folder or one asset pack. Paths are full paths.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LayerSet {
    /// Overlay art by platform id (lower-case folder name), without the `global` folder.
    pub icon_overlays: BTreeMap<String, OverlayArt>,
    /// The `global` overlay folder, used by platforms without a folder of their own.
    pub icon_overlay_global: Option<OverlayArt>,
    /// Smart folder art by type name (any folder name is accepted).
    pub smart_folders: BTreeMap<String, FolderArt>,
    /// Smart folder art by platform id.
    pub smart_folders_by_platform: BTreeMap<String, FolderArt>,
    /// Sound files by action.
    pub sounds: BTreeMap<SoundAction, PathBuf>,
    /// Names of sound files that match no action.
    pub unknown_sounds: Vec<String>,
    /// Music files.
    pub music: Vec<PathBuf>,
}

/// Reads the layers of a theme-shaped folder: `icon_overlays/`, `smart_folders/`, `sounds/`,
/// `music/`. A theme folder, and a folder of the user's own files, both have this layout.
pub fn read_theme_layers(root: &Path) -> LayerSet {
    let mut set = LayerSet::default();
    read_icon_overlays(&mut set, &root.join("icon_overlays"));
    read_smart_folders(&mut set, &root.join("smart_folders"));
    read_sounds(&mut set, &root.join("sounds"), true);
    set.music = list_dir(&root.join("music"))
        .into_iter()
        .filter(|e| !e.is_dir)
        .map(|e| e.path)
        .collect();
    set
}

/// Reads the layer of one installed pack from its content root.
pub(crate) fn read_pack_layers(category: AssetCategory, content: &Path) -> LayerSet {
    let mut set = LayerSet::default();
    match category {
        AssetCategory::IconOverlays => read_icon_overlays(&mut set, content),
        AssetCategory::SmartFolders => read_smart_folders(&mut set, content),
        AssetCategory::Sounds => read_sounds(&mut set, content, false),
        AssetCategory::Music => {
            set.music = list_files_deep(content, PACK_MUSIC_SCAN_DEPTH)
                .into_iter()
                .filter(|e| has_extension(&e.name, MUSIC_EXTENSIONS))
                .map(|e| e.path)
                .collect();
        }
    }
    set
}

/// Whether `file_name` ends in one of `exts`, ignoring case.
fn has_extension(file_name: &str, exts: &[&str]) -> bool {
    file_name
        .rsplit_once('.')
        .is_some_and(|(_, ext)| exts.contains(&ext.to_ascii_lowercase().as_str()))
}

/// Reads `<dir>/<platform>/{mask,overlay}.png`; the `global` folder is kept apart.
fn read_icon_overlays(set: &mut LayerSet, dir: &Path) {
    for e in list_dir(dir).into_iter().filter(|e| e.is_dir) {
        let files = list_dir(&e.path);
        let art = OverlayArt {
            mask: find_file(&files, "mask.png"),
            overlay: find_file(&files, "overlay.png"),
        };
        let id = e.name.to_ascii_lowercase();
        if id == GLOBAL_OVERLAY_ID {
            set.icon_overlay_global = Some(art);
        } else {
            set.icon_overlays.insert(id, art);
        }
    }
}

/// Reads one folder of `{icon,hero,logo}.<ext>`.
fn read_folder_art(dir: &Path) -> FolderArt {
    let files = list_dir(dir);
    let art = |stem: &str| find_stem(&files, stem, FOLDER_ART_EXTENSIONS);
    FolderArt {
        icon: art("icon"),
        hero: art("hero"),
        logo: art("logo"),
    }
}

/// Reads `<dir>/<type>/` and `<dir>/by_platform/<platform>/`. Special types found under
/// `by_platform/` move to the type map unless the same type exists outside it (that one wins).
fn read_smart_folders(set: &mut LayerSet, dir: &Path) {
    let entries: Vec<_> = list_dir(dir).into_iter().filter(|e| e.is_dir).collect();
    for e in entries.iter().filter(|e| !is_by_platform(&e.name)) {
        set.smart_folders
            .insert(e.name.to_ascii_lowercase(), read_folder_art(&e.path));
    }
    for e in entries.iter().filter(|e| is_by_platform(&e.name)) {
        for p in list_dir(&e.path).into_iter().filter(|p| p.is_dir) {
            let id = p.name.to_ascii_lowercase();
            let art = read_folder_art(&p.path);
            if SPECIAL_SMART_FOLDER_TYPES.contains(&id.as_str()) {
                set.smart_folders.entry(id).or_insert(art);
            } else {
                set.smart_folders_by_platform.insert(id, art);
            }
        }
    }
}

/// Whether a smart folder name is the `by_platform` container.
fn is_by_platform(name: &str) -> bool {
    name.eq_ignore_ascii_case("by_platform")
}

/// Reads sound files from `dir`: files named after an action go in the map (the first by name
/// wins), the rest are listed as unknown. With `report_all` every non-action file is listed
/// (a theme's `sounds/`); without it only files with a sound extension are, because the folder
/// may be a pack root that also holds `metadata.json` and `preview.png`.
fn read_sounds(set: &mut LayerSet, dir: &Path, report_all: bool) {
    for e in list_dir(dir).into_iter().filter(|e| !e.is_dir) {
        match action_of_file(&e.name) {
            Some(action) => {
                set.sounds.entry(action).or_insert(e.path);
            }
            None if report_all || is_sound_extension(&e.name) => set.unknown_sounds.push(e.name),
            None => {}
        }
    }
}
