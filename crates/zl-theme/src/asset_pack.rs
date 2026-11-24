//! Installed asset packs and the data folder that holds them (spec section 3.8).
//!
//! A data folder has `themes/<id>/` for installed themes and `assets/<category>/<id>/` for
//! packs. A pack has a `metadata.json`, a `preview.png` and its content, either in a
//! `<category>/` subfolder (layout A) or directly in the pack folder (layout B).

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::enums::StringEnum;
use crate::fsutil::{find_file, find_path_ci, list_dir};
use crate::issue::{Issue, IssueCode, Severity};
use crate::layer_set::{read_pack_layers, LayerSet};
use crate::manifest::AssetCategory;
use crate::music::{parse_music, MusicConfig};
use crate::reader::Reader;

/// The `metadata.json` of an installed pack. Absent keys are `None`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PackMeta {
    /// `id`.
    pub id: Option<String>,
    /// `name`.
    pub name: Option<String>,
    /// `author`.
    pub author: Option<String>,
    /// `category`, when it is one the format knows.
    pub category: Option<AssetCategory>,
    /// `version`, as text (numbers are accepted and converted).
    pub version: Option<String>,
    /// `description`.
    pub description: Option<String>,
    /// `file_count`.
    pub file_count: Option<u64>,
    /// `settings` of a music pack; `None` when it is `null` or absent.
    pub settings: Option<MusicConfig>,
}

/// One pack found in `assets/<category>/<id>/`.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetPack {
    /// Pack id: the folder name.
    pub id: String,
    /// The pack folder itself (`assets/<category>/<id>/`).
    pub dir: PathBuf,
    /// The category, taken from the `assets/<category>/` folder name.
    pub category: AssetCategory,
    /// The parsed `metadata.json`, `None` when absent or not a JSON object.
    pub meta: Option<PackMeta>,
    /// Folder holding the pack's files: `<pack>/<category>/` (layout A) when it exists, else
    /// the pack folder itself (layout B).
    pub root_of_content: PathBuf,
    /// Problems found while reading `metadata.json`; locations start with `metadata.json:`.
    pub issues: Vec<Issue>,
}

/// Everything installed in a data folder.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct InstalledContent {
    /// Theme folders (`themes/<id>/` holding a `theme.json`), sorted by id.
    pub themes: Vec<PathBuf>,
    /// Asset packs, sorted by category then id.
    pub packs: Vec<AssetPack>,
}

impl InstalledContent {
    /// The ids of all packs, ready to pass as `installed_pack_ids` to
    /// [`validate_package`](crate::validate_package).
    pub fn pack_ids(&self) -> Vec<String> {
        self.packs.iter().map(|p| p.id.clone()).collect()
    }
}

/// Lists the themes and asset packs installed under `data_dir`. Missing folders list as empty;
/// folders of an unknown category and loose files are ignored.
pub fn list_installed(data_dir: &Path) -> InstalledContent {
    let themes = list_dir(&data_dir.join("themes"))
        .into_iter()
        .filter(|e| e.is_dir && e.path.join("theme.json").is_file())
        .map(|e| e.path)
        .collect();
    let mut packs = Vec::new();
    for cat in list_dir(&data_dir.join("assets"))
        .into_iter()
        .filter(|e| e.is_dir)
    {
        let Some(category) = AssetCategory::parse_text(&cat.name) else {
            continue;
        };
        for dir in list_dir(&cat.path).into_iter().filter(|e| e.is_dir) {
            packs.push(read_pack(&dir.path, &dir.name, category));
        }
    }
    InstalledContent { themes, packs }
}

/// Reads one pack folder: its metadata and where its content is.
fn read_pack(dir: &Path, id: &str, category: AssetCategory) -> AssetPack {
    let files = list_dir(dir);
    let (meta, issues) = match find_file(&files, "metadata.json") {
        Some(path) => read_meta(&path),
        None => (None, Vec::new()),
    };
    let sub = files
        .iter()
        .find(|e| e.is_dir && e.name.eq_ignore_ascii_case(category.as_str()));
    AssetPack {
        id: id.to_string(),
        dir: dir.to_path_buf(),
        category,
        meta,
        root_of_content: sub.map_or_else(|| dir.to_path_buf(), |e| e.path.clone()),
        issues,
    }
}

/// Reads and parses a `metadata.json`; unreadable or invalid files give `None` and an error.
fn read_meta(path: &Path) -> (Option<PackMeta>, Vec<Issue>) {
    let bytes = std::fs::read(path).unwrap_or_default();
    let text = String::from_utf8_lossy(&bytes);
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    let mut reader = Reader::default();
    let Ok(Value::Object(obj)) = serde_json::from_str::<Value>(text) else {
        reader.error(IssueCode::InvalidJson, "$".to_string());
        return (None, prefixed(reader.issues));
    };
    let meta = parse_meta(&obj, &mut reader);
    (Some(meta), prefixed(reader.issues))
}

/// Marks issue locations as belonging to `metadata.json`.
fn prefixed(issues: Vec<Issue>) -> Vec<Issue> {
    issues
        .into_iter()
        .map(|i| Issue::new(i.severity, i.code, format!("metadata.json:{}", i.location)))
        .collect()
}

/// Reads the fields of a `metadata.json` object.
fn parse_meta(obj: &Map<String, Value>, reader: &mut Reader) -> PackMeta {
    let version = match obj.get("version") {
        Some(Value::Number(n)) => Some(n.to_string()),
        _ => reader.string(obj, "$", "version"),
    };
    PackMeta {
        id: reader.string(obj, "$", "id"),
        name: reader.string(obj, "$", "name"),
        author: reader.string(obj, "$", "author"),
        category: reader.enum_value(obj, "$", "category"),
        version,
        description: reader.string(obj, "$", "description"),
        file_count: obj.get("file_count").and_then(Value::as_u64),
        settings: match obj.get("settings") {
            Some(Value::Object(s)) => Some(parse_music(s, reader)),
            _ => None,
        },
    }
}

/// Reads the files of a pack as the layer it provides.
pub fn read_pack_content(pack: &AssetPack) -> LayerSet {
    read_pack_layers(pack.category, &pack.root_of_content)
}

/// Finds the file a playlist or schedule entry refers to, inside the pack. The entry usually
/// starts with `music/`; it is tried as written and without that prefix, and every path
/// component matches without regard to case. `None` when nothing matches.
pub fn resolve_music_file(pack: &AssetPack, reference: &str) -> Option<PathBuf> {
    let reference = reference.trim().replace('\\', "/");
    let stripped = reference
        .get(..6)
        .filter(|p| p.eq_ignore_ascii_case("music/"))
        .map(|_| &reference[6..]);
    let found = [Some(reference.as_str()), stripped]
        .into_iter()
        .flatten()
        .filter_map(|rel| find_path_ci(&pack.root_of_content, rel))
        .find(|p| p.is_file());
    found
}

/// Validates a pack: the issues of its `metadata.json`, a warning for every playlist or
/// schedule file that is not in the pack, and one for each sound file with no action name.
pub fn validate_pack(pack: &AssetPack) -> Vec<Issue> {
    let mut issues = pack.issues.clone();
    if let Some(music) = pack.meta.as_ref().and_then(|m| m.settings.as_ref()) {
        let playlist = music.playlist.iter().flatten().enumerate();
        let schedule = music.schedule.iter().flatten().map(|e| &e.file).enumerate();
        for (i, file) in playlist {
            missing_music(
                &mut issues,
                pack,
                file,
                format!("$.settings.music_playlist[{i}]"),
            );
        }
        for (i, file) in schedule {
            missing_music(
                &mut issues,
                pack,
                file,
                format!("$.settings.music_time_schedule[{i}]"),
            );
        }
    }
    for name in &read_pack_content(pack).unknown_sounds {
        issues.push(Issue::new(
            Severity::Warning,
            IssueCode::UnknownSoundName,
            format!("sounds/{name}"),
        ));
    }
    issues
}

/// Adds a `MissingMusicFile` warning when `file` is not in the pack.
fn missing_music(issues: &mut Vec<Issue>, pack: &AssetPack, file: &str, location: String) {
    if resolve_music_file(pack, file).is_none() {
        issues.push(Issue::new(
            Severity::Warning,
            IssueCode::MissingMusicFile,
            location,
        ));
    }
}
