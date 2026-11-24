//! Small filesystem helpers for reading theme folders.

use std::path::{Path, PathBuf};

/// An entry of a directory listing.
#[derive(Debug, Clone)]
pub(crate) struct Entry {
    /// File or folder name.
    pub name: String,
    /// Full path.
    pub path: PathBuf,
    /// Whether it is a directory.
    pub is_dir: bool,
}

/// Lists a directory sorted by name; a missing or unreadable directory lists as empty.
pub(crate) fn list_dir(dir: &Path) -> Vec<Entry> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut entries: Vec<Entry> = read
        .filter_map(Result::ok)
        .filter_map(|e| {
            let name = e.file_name().to_str()?.to_string();
            let path = e.path();
            Some(Entry {
                is_dir: path.is_dir(),
                name,
                path,
            })
        })
        .collect();
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

/// Finds the file in `entries` whose name equals `wanted`, ignoring case.
pub(crate) fn find_file(entries: &[Entry], wanted: &str) -> Option<PathBuf> {
    entries
        .iter()
        .find(|e| !e.is_dir && e.name.eq_ignore_ascii_case(wanted))
        .map(|e| e.path.clone())
}

/// Finds the entry of `entries` whose name is `<stem>.<ext>` for one of `exts` (ignoring case),
/// preferring the order of `exts` when several exist.
pub(crate) fn find_stem(entries: &[Entry], stem: &str, exts: &[&str]) -> Option<PathBuf> {
    exts.iter()
        .find_map(|ext| find_file(entries, &format!("{stem}.{ext}")))
}

/// Resolves a `/`-separated relative path below `base`, comparing every component without
/// regard to case (an exact match is preferred when both spellings exist). `..` and empty
/// components make the lookup fail, so a reference cannot escape `base`.
pub(crate) fn find_path_ci(base: &Path, relative: &str) -> Option<PathBuf> {
    let mut current = base.to_path_buf();
    let parts: Vec<&str> = relative.split('/').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() || parts.contains(&"..") {
        return None;
    }
    for part in parts {
        let entries = list_dir(&current);
        let wanted = part.to_lowercase();
        let hit = entries
            .iter()
            .find(|e| e.name == part)
            .or_else(|| entries.iter().find(|e| e.name.to_lowercase() == wanted))?;
        current = hit.path.clone();
    }
    Some(current)
}

/// Lists the regular files below `dir` down to `depth` folder levels (0 = only `dir`), sorted
/// by full path.
pub(crate) fn list_files_deep(dir: &Path, depth: usize) -> Vec<Entry> {
    let mut out = Vec::new();
    for e in list_dir(dir) {
        if !e.is_dir {
            out.push(e);
        } else if depth > 0 {
            out.extend(list_files_deep(&e.path, depth - 1));
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Components match without regard to case, exact spellings win, `..` is refused.
    #[test]
    fn case_insensitive_paths() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let music = dir.path().join("Music");
        std::fs::create_dir_all(&music)?;
        std::fs::write(music.join("Song One.mp3"), b"x")?;
        std::fs::write(dir.path().join("secret.txt"), b"x")?;
        let found = find_path_ci(dir.path(), "music/song one.MP3");
        assert_eq!(found, Some(music.join("Song One.mp3")));
        assert_eq!(find_path_ci(dir.path(), "music/other.mp3"), None);
        assert_eq!(find_path_ci(&music, "../secret.txt"), None);
        assert_eq!(find_path_ci(&music, ""), None);
        Ok(())
    }

    /// The deep listing stops at the depth limit.
    #[test]
    fn deep_listing_respects_depth() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let deep = dir.path().join("a/b");
        std::fs::create_dir_all(&deep)?;
        std::fs::write(dir.path().join("a/one.mp3"), b"x")?;
        std::fs::write(deep.join("two.mp3"), b"x")?;
        assert_eq!(list_files_deep(dir.path(), 1).len(), 1);
        assert_eq!(list_files_deep(dir.path(), 2).len(), 2);
        Ok(())
    }
}
