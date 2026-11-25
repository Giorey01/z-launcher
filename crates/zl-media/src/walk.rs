//! Listing the files below a folder, with a depth limit and sorted output.

use std::path::{Path, PathBuf};

/// Whether a file or folder name is hidden (`.DS_Store`, macOS `._Game.png` copies, `.Trashes`):
/// never media, never a platform.
fn is_hidden(name: &std::ffi::OsStr) -> bool {
    name.to_str().is_some_and(|n| n.starts_with('.'))
}

/// Names of the entries of `dir` that are folders, sorted, hidden ones left out. Unreadable
/// folders list as empty.
pub(crate) fn sub_dirs(dir: &Path) -> Vec<(String, PathBuf)> {
    let mut out: Vec<(String, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| !is_hidden(&e.file_name()) && e.path().is_dir())
        .filter_map(|e| Some((e.file_name().to_str()?.to_string(), e.path())))
        .collect();
    out.sort();
    out
}

/// Every visible file below `dir` (at most `depth` folder levels down, 0 = only `dir`), sorted by path.
pub(crate) fn files_below(dir: &Path, depth: usize) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect(dir, depth, &mut out);
    out.sort();
    out
}

/// Adds the files of `dir` to `out` and recurses while `depth` allows.
fn collect(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if is_hidden(&entry.file_name()) {
            continue;
        }
        if path.is_dir() {
            if depth > 0 {
                collect(&path, depth - 1, out);
            }
        } else if path.is_file() {
            out.push(path);
        }
    }
}

/// The extension of a path, lower case, `None` when it has none.
pub(crate) fn extension_of(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
}

/// The file name of a path without its last extension.
pub(crate) fn stem_of(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(str::to_string)
}
