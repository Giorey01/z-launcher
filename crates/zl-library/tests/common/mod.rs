//! Helpers shared by the integration tests.
#![allow(dead_code)]

use std::path::Path;

use zl_library::{ExtensionRules, NoProgress, RootSpec};

/// Result type of the tests: any error fails the test.
pub type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Writes a file (creating folders) with the given content.
pub fn write(root: &Path, rel: &str, content: &str) -> std::io::Result<()> {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)
}

/// A root spec with sub-folders on.
pub fn root(path: &Path) -> RootSpec {
    RootSpec {
        path: path.to_path_buf(),
        scan_subfolders: true,
    }
}

/// Rules for a few platforms used by the behaviour tests.
pub fn basic_rules() -> ExtensionRules {
    let mut rules = ExtensionRules::new();
    rules.insert("gba", &["gba", "zip"]);
    rules.insert("psx", &["cue", "bin", "chd", "m3u"]);
    rules
}

/// Scans without progress reporting.
pub fn quiet_scan(
    db: &zl_db::Db,
    roots: &[RootSpec],
    rules: &ExtensionRules,
) -> Result<zl_library::ScanSummary, zl_library::LibraryError> {
    zl_library::scan(db, roots, rules, &NoProgress)
}
