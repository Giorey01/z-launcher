//! Walking a platform folder: lists every visible file with its size and modification time.
//!
//! Hidden entries (a leading dot: `.nomedia`, macOS `._` files, `.git`) are skipped. Symlinks are
//! followed, but every folder is canonicalised and entered only once, so a link back to an
//! ancestor cannot loop.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::events::{IssueCode, ScanIssue};

/// A file found by the walk.
#[derive(Debug, Clone)]
pub(crate) struct FileEntry {
    /// Full path.
    pub path: PathBuf,
    /// File name (last component), valid UTF-8.
    pub name: String,
    /// Size in bytes.
    pub size: u64,
    /// Modification time, Unix milliseconds.
    pub mtime_ms: i64,
}

/// Result of walking one platform folder.
#[derive(Debug, Default)]
pub(crate) struct WalkResult {
    /// All visible files, sorted by path.
    pub files: Vec<FileEntry>,
    /// False when something could not be read: the games of this platform must not be flagged
    /// missing on this scan, because the list may be partial.
    pub complete: bool,
    /// Problems found.
    pub issues: Vec<ScanIssue>,
}

/// Converts a modification time to Unix milliseconds (0 for times before the epoch).
fn mtime_ms(meta: &fs::Metadata) -> i64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}

/// State of one walk.
struct Walker<'a> {
    /// Canonical paths of folders already entered (shared across the platforms of a root).
    visited: &'a mut HashSet<PathBuf>,
    /// Called once per file seen.
    on_file: &'a mut dyn FnMut(),
    /// Maximum depth below the platform folder.
    max_depth: usize,
    /// What has been collected.
    out: WalkResult,
}

impl Walker<'_> {
    /// Records a problem and marks the listing incomplete.
    fn problem(&mut self, code: IssueCode, path: &Path) {
        self.out.complete = false;
        self.out.issues.push(ScanIssue {
            code,
            path: path.to_string_lossy().into_owned(),
        });
    }

    /// Handles one directory entry: queues a folder or records a file.
    fn visit(&mut self, entry: &fs::DirEntry, depth: usize, stack: &mut Vec<(PathBuf, usize)>) {
        let path = entry.path();
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            self.problem(IssueCode::NonUtf8Name, &path);
            return;
        };
        if name.starts_with('.') {
            return;
        }
        let Ok(kind) = entry.file_type() else {
            self.problem(IssueCode::StatFailed, &path);
            return;
        };
        // Symlinks: follow them; a broken link is simply skipped.
        let meta = if kind.is_symlink() {
            match fs::metadata(&path) {
                Ok(meta) => meta,
                Err(_) => return,
            }
        } else {
            match entry.metadata() {
                Ok(meta) => meta,
                Err(_) => return self.problem(IssueCode::StatFailed, &path),
            }
        };
        if meta.is_dir() {
            let excluded = zl_config::library::EXCLUDED_SUBFOLDER_NAMES
                .contains(&name.to_ascii_lowercase().as_str());
            if depth < self.max_depth && !excluded {
                stack.push((path, depth + 1));
            }
        } else if meta.is_file() {
            (self.on_file)();
            self.out.files.push(FileEntry {
                path,
                name,
                size: meta.len(),
                mtime_ms: mtime_ms(&meta),
            });
        }
    }

    /// Lists one folder, unless it was already entered through another path.
    fn list(&mut self, dir: &Path, depth: usize, stack: &mut Vec<(PathBuf, usize)>) {
        let canonical = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
        if !self.visited.insert(canonical) {
            return;
        }
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return self.problem(IssueCode::ReadDirFailed, dir),
        };
        for entry in entries {
            match entry {
                Ok(entry) => self.visit(&entry, depth, stack),
                Err(_) => self.problem(IssueCode::ReadDirFailed, dir),
            }
        }
    }
}

/// Walks `platform_dir`. `max_depth` is 0 to read only the folder itself. `visited` is shared by
/// the platforms of one root to stop symlink loops and duplicates. `on_file` is called per file.
pub(crate) fn walk_platform(
    platform_dir: &Path,
    max_depth: usize,
    visited: &mut HashSet<PathBuf>,
    on_file: &mut dyn FnMut(),
) -> WalkResult {
    let mut walker = Walker {
        visited,
        on_file,
        max_depth,
        out: WalkResult {
            complete: true,
            ..WalkResult::default()
        },
    };
    let mut stack = vec![(platform_dir.to_path_buf(), 0)];
    while let Some((dir, depth)) = stack.pop() {
        walker.list(&dir, depth, &mut stack);
    }
    walker.out.files.sort_by(|a, b| a.path.cmp(&b.path));
    walker.out
}
