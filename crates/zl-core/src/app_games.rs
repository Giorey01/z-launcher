//! Android apps filed under a console (recomps and ports): the app becomes a game of that
//! console through a `.localgameid` tag file, the same file a user can write by hand.
//!
//! Filing writes `<root>/<console folder>/<title>.localgameid` with `[localgameid] <package>`
//! and rescans; taking the app out deletes that file and rescans. The file is the only record,
//! so the choice survives a reinstall, a backup and other launchers that read the format.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use zl_db::GameKind;
use zl_library::ScanProgress;
use zl_platforms::ANDROID_PACKAGE_TAG;

use crate::error::CoreError;
use crate::types::PlatformEntry;
use crate::Service;

/// Characters a file name may not hold on Android's shared storage (FAT and exFAT rules).
const FORBIDDEN_IN_NAMES: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

/// The title as a file name: forbidden characters and control characters dropped, spaces
/// trimmed, no leading dot. `None` when nothing is left.
pub(crate) fn file_stem(title: &str) -> Option<String> {
    let cleaned: String = title
        .chars()
        .filter(|c| !FORBIDDEN_IN_NAMES.contains(c) && !c.is_control())
        .collect();
    let stem = cleaned.trim().trim_start_matches('.').trim().to_string();
    (!stem.is_empty()).then_some(stem)
}

/// Content of the tag file for `package`.
fn tag_text(package: &str) -> String {
    format!("[{ANDROID_PACKAGE_TAG}] {package}\n")
}

/// A free path for `stem` in `dir` with extension `ext`: `stem.ext`, else `stem (2).ext`, and
/// so on. A file that already holds `text` (the same tag) is reused.
pub(crate) fn free_path(dir: &Path, stem: &str, ext: &str, text: &str) -> PathBuf {
    let mut n = 1;
    loop {
        let name = if n == 1 {
            format!("{stem}.{ext}")
        } else {
            format!("{stem} ({n}).{ext}")
        };
        let path = dir.join(name);
        match fs::read_to_string(&path) {
            Err(_) => return path,
            Ok(found) if found.trim() == text.trim() => return path,
            Ok(_) => n += 1,
        }
    }
}

impl Service {
    /// The folder of `platform` inside `root`: an existing sub-folder whose name is the
    /// platform's id or one of its aliases (any case), else `<root>/<id>` (not created here).
    pub(crate) fn console_dir(&self, root: &Path, platform: &str) -> PathBuf {
        let existing = fs::read_dir(root).ok().and_then(|entries| {
            entries.flatten().find_map(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                let is_dir = entry.file_type().is_ok_and(|t| t.is_dir());
                (is_dir && self.canonical_id(&name.to_lowercase()) == platform)
                    .then(|| entry.path())
            })
        });
        existing.unwrap_or_else(|| root.join(platform))
    }

    /// Files the Android app `package` under the console `platform` (id or alias) with the
    /// display `title`, in the first enabled folder, then rescans. Returns the game's id.
    /// `InvalidInput` for a title with no usable character, an empty package or a group folder
    /// id; `NotFound` for an unknown platform or no enabled folder; `Io` if the file cannot be
    /// written (no access to shared storage).
    pub fn file_app(
        &self,
        package: &str,
        title: &str,
        platform: &str,
        progress: &dyn ScanProgress,
    ) -> Result<i64, CoreError> {
        let package = package.trim();
        let stem = file_stem(title).ok_or(CoreError::InvalidInput)?;
        if package.is_empty() || package.contains(char::is_whitespace) {
            return Err(CoreError::InvalidInput);
        }
        let platform = self
            .platforms
            .platform(platform)
            .ok_or(CoreError::NotFound)?;
        let root = self.roots()?.into_iter().find(|r| r.enabled);
        let root = PathBuf::from(root.ok_or(CoreError::NotFound)?.path);
        let dir = self.console_dir(&root, &platform.id);
        fs::create_dir_all(&dir).map_err(|_| CoreError::Io)?;
        let text = tag_text(package);
        let path = free_path(&dir, &stem, ANDROID_PACKAGE_TAG, &text);
        fs::write(&path, text).map_err(|_| CoreError::Io)?;
        self.scan(progress)?;
        let path = path.to_string_lossy();
        let game = self.db().game_by_path(&path)?.ok_or(CoreError::NotFound)?;
        Ok(game.id)
    }

    /// The consoles an Android app can be filed under, for a picker: those with games first,
    /// then the others, each part by name. Platforms made only of tag files (Steam, Windows,
    /// Moonlight, xCloud) are left out, and so are group folders.
    pub fn app_consoles(&self) -> Result<Vec<PlatformEntry>, CoreError> {
        let mut counts: HashMap<String, u32> = HashMap::new();
        for c in self.db().platform_counts()? {
            let games = u32::try_from(c.games).unwrap_or(u32::MAX);
            *counts.entry(self.canonical_id(&c.platform_id)).or_default() += games;
        }
        let mut out: Vec<PlatformEntry> = self
            .platforms
            .platforms()
            .iter()
            .filter(|p| {
                !p.extensions
                    .iter()
                    .all(|x| p.tag_files.iter().any(|t| &t.ext == x))
            })
            .map(|p| PlatformEntry {
                id: p.id.clone(),
                name: p.name.clone(),
                games: counts.get(&p.id).copied().unwrap_or(0),
            })
            .collect();
        out.sort_by_key(|p| (p.games == 0, p.name.to_lowercase()));
        Ok(out)
    }

    /// Takes a filed Android app out of its console: deletes its `.localgameid` file and
    /// rescans. `InvalidInput` for a game that is not an app filed under a console (a ROM, or a
    /// Windows tag file, whose tag means something else).
    pub fn unfile_app(&self, game_id: i64, progress: &dyn ScanProgress) -> Result<(), CoreError> {
        let game = self.db().game(game_id)?.ok_or(CoreError::NotFound)?;
        let tags = self.db().game_tags(game_id)?;
        let package = tags.iter().find(|(k, _)| k == ANDROID_PACKAGE_TAG);
        let is_app =
            package.is_some_and(|(_, v)| self.platforms.is_android_app(&game.platform_id, v));
        if game.kind != GameKind::TagFile || !is_app {
            return Err(CoreError::InvalidInput);
        }
        match fs::remove_file(&game.path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(CoreError::Io),
        }
        self.scan(progress)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Forbidden characters go, a dot-only or empty title gives nothing.
    #[test]
    fn titles_become_safe_file_names() {
        assert_eq!(
            file_stem(" Zelda: Majora's Mask / Recomp ").as_deref(),
            Some("Zelda Majora's Mask  Recomp")
        );
        assert_eq!(file_stem("..hidden").as_deref(), Some("hidden"));
        assert_eq!(file_stem(" ?* ").as_deref(), None);
        assert_eq!(file_stem("").as_deref(), None);
    }
}
