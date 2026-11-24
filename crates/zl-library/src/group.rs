//! Collapsing the files of one platform folder into games.
//!
//! Order of the passes (earlier passes hide files from later ones):
//! 0. caller-defined folder games (Wii U `code/*.rpx`) take their whole folder;
//! 1. tag files become `tagfile` games;
//! 2. `.m3u` playlists become `m3u` games and hide the files they list (and, for listed CUE/GDI
//!    sheets, their tracks);
//! 3. `.cue` and `.gdi` sheets become `rom` games that own their tracks;
//! 4. every other file with an allowed extension becomes a `rom`;
//! 5. `rom` entries of one folder that differ only by a `(Disc N)` marker merge into a
//!    `multidisc` game (two or more needed);
//! 6. in a sub-folder, two or more bare disc files (`Disc 1`, `Disc 2`) merge into a `folder`
//!    game named after the folder.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

use zl_db::GameKind;

use crate::discs::{bare_disc_number, disc_key};
use crate::events::{IssueCode, ScanIssue};
use crate::rules::{glob_matches, split_allowed};
use crate::sheets::{cue_files, gdi_files, m3u_entries};
use crate::walk::FileEntry;

/// A game assembled from files, before it is compared with the database.
#[derive(Debug, Clone)]
pub(crate) struct Unit {
    /// Shape of the entry.
    pub kind: GameKind,
    /// Title shown in the library.
    pub title: String,
    /// Index (into the platform's file list) of the primary file.
    pub primary: usize,
    /// Indexes of every member file, the primary included, sorted.
    pub members: Vec<usize>,
    /// Pairs read from a tag file.
    pub tags: Vec<(String, String)>,
}

/// Lower-cased, lexically normalised path string used to match references to files
/// case-insensitively (playlists written on Windows disagree about case).
fn lookup_key(path: &Path) -> String {
    let mut parts: Vec<Component<'_>> = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(parts.last(), Some(Component::Normal(_))) {
                    parts.pop();
                } else {
                    parts.push(component);
                }
            }
            other => parts.push(other),
        }
    }
    let joined: PathBuf = parts.iter().collect();
    joined.to_string_lossy().to_lowercase()
}

/// Whether a file name is a CUE or GDI sheet (by its last extension).
fn is_sheet_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".cue") || lower.ends_with(".gdi")
}

/// Working state of the grouping of one platform folder.
struct Grouper<'a> {
    /// Platform id, for tag-file disambiguation.
    platform: &'a str,
    /// Platform folder.
    platform_dir: &'a Path,
    /// Launch globs of the platform's folder-game rule.
    globs: &'a [String],
    /// Allowed extensions of the platform.
    exts: &'a HashSet<String>,
    /// All files of the platform, sorted by path.
    files: &'a [FileEntry],
    /// Lower-cased path to index.
    index: HashMap<String, usize>,
    /// Files already owned by a game.
    consumed: Vec<bool>,
    /// Problems found while reading sheets and tags.
    issues: Vec<ScanIssue>,
}

impl<'a> Grouper<'a> {
    /// Builds the lookup index.
    fn new(
        platform: &'a str,
        platform_dir: &'a Path,
        globs: &'a [String],
        exts: &'a HashSet<String>,
        files: &'a [FileEntry],
    ) -> Grouper<'a> {
        let index = files
            .iter()
            .enumerate()
            .map(|(i, f)| (lookup_key(&f.path), i))
            .collect();
        Grouper {
            platform,
            platform_dir,
            globs,
            exts,
            files,
            index,
            consumed: vec![false; files.len()],
            issues: Vec::new(),
        }
    }

    /// Extension of a file if it is an allowed one, else `None`.
    fn ext_of(&self, i: usize) -> Option<String> {
        split_allowed(&self.files[i].name, self.exts).map(|(_, ext)| ext)
    }

    /// File name without its allowed extension.
    fn stem_of(&self, i: usize) -> String {
        let name = &self.files[i].name;
        split_allowed(name, self.exts).map_or_else(|| name.clone(), |(stem, _)| stem.to_string())
    }

    /// Records a problem for file `i`.
    fn issue(&mut self, code: IssueCode, i: usize) {
        let path = self.files[i].path.to_string_lossy().into_owned();
        self.issues.push(ScanIssue { code, path });
    }

    /// Reads a small text file, recording a problem when it cannot be read.
    fn read_text(&mut self, i: usize) -> Option<String> {
        if self.files[i].size > zl_config::library::MAX_SHEET_BYTES {
            return None;
        }
        match std::fs::read(&self.files[i].path) {
            Ok(bytes) => Some(String::from_utf8_lossy(&bytes).into_owned()),
            Err(_) => {
                self.issue(IssueCode::SheetUnreadable, i);
                None
            }
        }
    }

    /// Finds the file a reference points to, relative to the folder of file `from`. Falls back
    /// to the bare file name next to `from` for absolute or foreign (`C:\`) paths.
    fn resolve(&self, from: usize, reference: &str) -> Option<usize> {
        let dir = self.files[from].path.parent()?;
        let direct = self.index.get(&lookup_key(&dir.join(reference)));
        let by_name = || {
            let name = reference.rsplit('/').next()?;
            self.index.get(&lookup_key(&dir.join(name)))
        };
        direct.or_else(by_name).copied()
    }

    /// Existing files referenced by the sheet or playlist `i`, in order.
    fn references(&mut self, i: usize, parse: fn(&str) -> Vec<String>) -> Vec<usize> {
        let Some(text) = self.read_text(i) else {
            return Vec::new();
        };
        parse(&text)
            .iter()
            .filter_map(|r| self.resolve(i, r))
            .filter(|&target| target != i)
            .collect()
    }

    /// Tracks of a CUE or GDI sheet.
    fn tracks(&mut self, i: usize) -> Vec<usize> {
        let parse = if self.files[i].name.to_ascii_lowercase().ends_with(".gdi") {
            gdi_files
        } else {
            cue_files
        };
        self.references(i, parse)
    }

    /// Marks files as owned and builds a unit with the given members (primary included).
    fn claim(&mut self, kind: GameKind, primary: usize, mut members: Vec<usize>) -> Unit {
        members.push(primary);
        members.sort_unstable();
        members.dedup();
        for &m in &members {
            self.consumed[m] = true;
        }
        Unit {
            kind,
            title: self.stem_of(primary),
            primary,
            members,
            tags: Vec::new(),
        }
    }

    /// Pass 0: each sub-folder of the platform folder that holds a file matching a launch glob
    /// is one `folder` game; the launch file is the primary one and every file below the folder
    /// is a member.
    fn rule_units(&mut self) -> Vec<Unit> {
        if self.globs.is_empty() {
            return Vec::new();
        }
        let mut folders: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, file) in self.files.iter().enumerate() {
            let Ok(rel) = file.path.strip_prefix(self.platform_dir) else {
                continue;
            };
            let mut parts = rel.components();
            if let (Some(top), Some(_)) = (parts.next(), parts.clone().next()) {
                folders
                    .entry(top.as_os_str().to_string_lossy().into_owned())
                    .or_default()
                    .push(i);
            }
        }
        let mut units = Vec::new();
        for (top, members) in folders {
            let launch = members
                .iter()
                .copied()
                .find(|&i| self.matches_glob(i, &top));
            if let Some(primary) = launch {
                let mut unit = self.claim(GameKind::Folder, primary, members);
                unit.title = top;
                units.push(unit);
            }
        }
        units
    }

    /// Whether file `i`, inside game folder `top`, matches a launch glob.
    fn matches_glob(&self, i: usize, top: &str) -> bool {
        let Ok(rel) = self.files[i].path.strip_prefix(self.platform_dir.join(top)) else {
            return false;
        };
        let rel = rel.to_string_lossy().replace('\\', "/");
        self.globs.iter().any(|g| glob_matches(g, &rel))
    }

    /// Pass 1: tag files.
    fn tag_units(&mut self) -> Vec<Unit> {
        let mut units = Vec::new();
        for i in 0..self.files.len() {
            let Some(ext) = self.ext_of(i) else { continue };
            if !zl_tags::is_tag_extension(&ext) {
                continue;
            }
            // Whatever happens, a tag file is never a plain ROM.
            self.consumed[i] = true;
            match zl_tags::read_tag_file(&self.files[i].path) {
                Err(_) => self.issue(IssueCode::TagFileUnreadable, i),
                Ok(parsed) if parsed.file.kind(Some(self.platform)) == zl_tags::TagKind::Other => {
                    self.issue(IssueCode::InvalidTagFile, i);
                }
                Ok(parsed) => {
                    let mut unit = self.claim(GameKind::TagFile, i, Vec::new());
                    unit.title = parsed.file.name;
                    unit.tags = parsed.file.tags;
                    units.push(unit);
                }
            }
        }
        units
    }

    /// Pass 2: playlists and the files they list (with the tracks of listed sheets).
    fn m3u_units(&mut self) -> Vec<Unit> {
        let mut units = Vec::new();
        for i in 0..self.files.len() {
            if self.consumed[i] || self.ext_of(i).as_deref() != Some("m3u") {
                continue;
            }
            if self.files[i].size > zl_config::library::MAX_SHEET_BYTES {
                continue;
            }
            let listed = self.references(i, m3u_entries);
            let mut members = listed.clone();
            for target in listed {
                if is_sheet_name(&self.files[target].name) {
                    members.extend(self.tracks(target));
                }
            }
            units.push(self.claim(GameKind::M3u, i, members));
        }
        units
    }

    /// Pass 3: CUE and GDI sheets with their tracks.
    fn sheet_units(&mut self) -> Vec<Unit> {
        let mut units = Vec::new();
        for i in 0..self.files.len() {
            let is_sheet = matches!(self.ext_of(i).as_deref(), Some("cue" | "gdi"));
            if self.consumed[i] || !is_sheet {
                continue;
            }
            let tracks = self.tracks(i);
            units.push(self.claim(GameKind::Rom, i, tracks));
        }
        units
    }

    /// Pass 4: every other allowed file is a game on its own.
    fn plain_units(&mut self) -> Vec<Unit> {
        let mut units = Vec::new();
        for i in 0..self.files.len() {
            if !self.consumed[i] && self.ext_of(i).is_some() {
                units.push(self.claim(GameKind::Rom, i, Vec::new()));
            }
        }
        units
    }
}

/// Folder of a unit's primary file.
fn dir_of(files: &[FileEntry], unit: &Unit) -> PathBuf {
    files[unit.primary]
        .path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

/// Merges `members` of several units into one unit with the given kind, title and primary.
fn merge(kind: GameKind, title: String, parts: Vec<(u32, Unit)>) -> Unit {
    let mut parts = parts;
    parts.sort_by_key(|(number, unit)| (*number, unit.primary));
    let primary = parts[0].1.primary;
    let mut members: Vec<usize> = parts.iter().flat_map(|(_, u)| u.members.clone()).collect();
    members.sort_unstable();
    members.dedup();
    Unit {
        kind,
        title,
        primary,
        members,
        tags: Vec::new(),
    }
}

/// Pass 5: `(Disc N)` siblings of one folder become a `multidisc` unit.
fn merge_discs(
    files: &[FileEntry],
    units: Vec<Unit>,
    stems: &dyn Fn(usize) -> String,
) -> Vec<Unit> {
    let mut groups: BTreeMap<(PathBuf, String), Vec<(u32, Unit)>> = BTreeMap::new();
    let mut out = Vec::new();
    for unit in units {
        match disc_key(&stems(unit.primary)).filter(|_| unit.kind == GameKind::Rom) {
            Some((title, number)) => groups
                .entry((dir_of(files, &unit), title.to_lowercase()))
                .or_default()
                .push((number, Unit { title, ..unit })),
            None => out.push(unit),
        }
    }
    for (_, parts) in groups {
        if parts.len() >= 2 {
            let title = parts[0].1.title.clone();
            out.push(merge(GameKind::Multidisc, title, parts));
        } else {
            out.extend(parts.into_iter().map(|(_, unit)| unit.restore_title(stems)));
        }
    }
    out
}

impl Unit {
    /// Puts back the full file-stem title of a disc that stayed alone.
    fn restore_title(mut self, stems: &dyn Fn(usize) -> String) -> Unit {
        self.title = stems(self.primary);
        self
    }
}

/// Pass 6: two or more bare disc files in a sub-folder become a `folder` unit.
fn merge_folders(
    files: &[FileEntry],
    platform_dir: &Path,
    units: Vec<Unit>,
    stems: &dyn Fn(usize) -> String,
) -> Vec<Unit> {
    let mut groups: BTreeMap<PathBuf, Vec<(u32, Unit)>> = BTreeMap::new();
    let mut out = Vec::new();
    for unit in units {
        let dir = dir_of(files, &unit);
        let number = bare_disc_number(&stems(unit.primary));
        match number.filter(|_| unit.kind == GameKind::Rom && dir != platform_dir) {
            Some(n) => groups.entry(dir).or_default().push((n, unit)),
            None => out.push(unit),
        }
    }
    for (dir, parts) in groups {
        if parts.len() >= 2 {
            let title = dir
                .file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
            out.push(merge(GameKind::Folder, title, parts));
        } else {
            out.extend(parts.into_iter().map(|(_, unit)| unit));
        }
    }
    out
}

/// Result of grouping a platform folder.
pub(crate) struct Grouped {
    /// The games, sorted by primary path.
    pub units: Vec<Unit>,
    /// Problems found while reading sheets and tag files.
    pub issues: Vec<ScanIssue>,
}

/// Collapses `files` (sorted, from one platform folder) into games.
pub(crate) fn group_platform(
    platform: &str,
    platform_dir: &Path,
    exts: &HashSet<String>,
    globs: &[String],
    files: &[FileEntry],
) -> Grouped {
    let mut g = Grouper::new(platform, platform_dir, globs, exts, files);
    let mut units = g.rule_units();
    units.extend(g.tag_units());
    units.extend(g.m3u_units());
    units.extend(g.sheet_units());
    units.extend(g.plain_units());
    let stems = |i: usize| g.stem_of(i);
    let units = merge_discs(files, units, &stems);
    let mut units = merge_folders(files, platform_dir, units, &stems);
    units.sort_by_key(|u| u.primary);
    Grouped {
        units,
        issues: g.issues,
    }
}
