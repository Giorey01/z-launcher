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
