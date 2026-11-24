//! Safe import of a theme ZIP (spec section 2).
//!
//! Every entry is checked before anything is written: no absolute paths, no `..`, no symbolic
//! links, and limits on entry count, expanded size and compression ratio. While extracting, the
//! bytes really written are counted too, because the sizes a ZIP declares can lie.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use zip::ZipArchive;

use crate::error::ThemeError;
use crate::package::find_themes;

/// Limits applied to an import. The default comes from `zl_config::theme`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZipLimits {
    /// Maximum number of entries.
    pub max_entries: usize,
    /// Maximum expanded bytes in total.
    pub max_total_bytes: u64,
    /// Maximum expanded bytes of one entry.
    pub max_entry_bytes: u64,
    /// Maximum expanded/compressed ratio of an entry.
    pub max_ratio: u64,
    /// Entries expanding to fewer bytes than this skip the ratio check.
    pub ratio_check_min_bytes: u64,
}

impl Default for ZipLimits {
    /// The tunables of `zl_config::theme`.
    fn default() -> Self {
        use zl_config::theme as cfg;
        ZipLimits {
            max_entries: cfg::MAX_ZIP_ENTRIES,
            max_total_bytes: cfg::MAX_ZIP_TOTAL_UNCOMPRESSED_BYTES,
            max_entry_bytes: cfg::MAX_ZIP_ENTRY_UNCOMPRESSED_BYTES,
            max_ratio: cfg::MAX_ZIP_COMPRESSION_RATIO,
            ratio_check_min_bytes: cfg::ZIP_RATIO_CHECK_MIN_BYTES,
        }
    }
}

/// An entry that passed the checks: its index in the archive, where it goes, its declared size.
struct Planned {
    /// Index in the archive.
    index: usize,
    /// Path relative to the destination.
    path: PathBuf,
    /// Whether it is a directory.
    is_dir: bool,
    /// Declared expanded size.
    size: u64,
}

/// Imports `zip_path` into `dest` (a new or empty folder) with the default limits and returns
/// the theme folders found: `themes/<id>/`, or the root, or a single top folder.
pub fn import_zip(zip_path: &Path, dest: &Path) -> Result<Vec<PathBuf>, ThemeError> {
    import_zip_with_limits(zip_path, dest, &ZipLimits::default())
}

/// Like [`import_zip`] with explicit limits. On failure `dest` may hold partial output and the
/// caller should delete it.
pub fn import_zip_with_limits(
    zip_path: &Path,
    dest: &Path,
    limits: &ZipLimits,
) -> Result<Vec<PathBuf>, ThemeError> {
    let mut archive = ZipArchive::new(File::open(zip_path)?)?;
    let plan = check_entries(&mut archive, limits)?;
    std::fs::create_dir_all(dest)?;
    extract(&mut archive, &plan, dest, limits)?;
    let roots = find_themes(dest);
    if roots.is_empty() {
        return Err(ThemeError::NoThemeInArchive);
    }
    Ok(roots)
}

/// Turns an entry name into a safe relative path, or `None` if it is absolute, has a drive,
/// contains `..` or a NUL. Both `/` and `\` separate components.
fn safe_relative(name: &str) -> Option<PathBuf> {
    if name.contains('\0') || name.starts_with('/') || name.starts_with('\\') {
        return None;
    }
    let mut path = PathBuf::new();
    for part in name
        .split(['/', '\\'])
        .filter(|p| !p.is_empty() && *p != ".")
    {
        if part == ".." || part.contains(':') {
            return None;
        }
        path.push(part);
    }
    let all_normal = path.components().all(|c| matches!(c, Component::Normal(_)));
    (all_normal && path.components().next().is_some()).then_some(path)
}

/// Checks every entry against the limits and returns the extraction plan.
fn check_entries(
    archive: &mut ZipArchive<File>,
    limits: &ZipLimits,
) -> Result<Vec<Planned>, ThemeError> {
    if archive.len() > limits.max_entries {
        return Err(ThemeError::TooManyEntries);
    }
    let mut plan = Vec::new();
    let mut total: u64 = 0;
    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        let name = entry.name().to_string();
        let path = safe_relative(&name).ok_or_else(|| ThemeError::UnsafePath(name.clone()))?;
        if entry
            .unix_mode()
            .is_some_and(|m| m & 0o170_000 == 0o120_000)
        {
            return Err(ThemeError::SymlinkEntry(name));
        }
        let size = entry.size();
        total = total.saturating_add(size);
        if size > limits.max_entry_bytes || total > limits.max_total_bytes {
            return Err(ThemeError::TooLarge);
        }
        if size >= limits.ratio_check_min_bytes
            && size / entry.compressed_size().max(1) > limits.max_ratio
        {
            return Err(ThemeError::CompressionRatio(name));
        }
        plan.push(Planned {
            index,
            path,
            is_dir: entry.is_dir(),
            size,
        });
    }
    Ok(plan)
}

/// Writes the planned entries under `dest`, counting the bytes actually produced.
fn extract(
    archive: &mut ZipArchive<File>,
    plan: &[Planned],
    dest: &Path,
    limits: &ZipLimits,
) -> Result<(), ThemeError> {
    let mut written_total: u64 = 0;
    for item in plan {
        let target = dest.join(&item.path);
        if item.is_dir {
            std::fs::create_dir_all(&target)?;
            continue;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let entry = archive.by_index(item.index)?;
        let written = copy_bounded(entry, &target, item.size)?;
        written_total = written_total.saturating_add(written);
        if written_total > limits.max_total_bytes {
            return Err(ThemeError::TooLarge);
        }
    }
    Ok(())
}

/// Copies `reader` into a new file at `target`, failing if it yields more than `declared` bytes.
fn copy_bounded(reader: impl Read, target: &Path, declared: u64) -> Result<u64, ThemeError> {
    let mut out = File::create(target)?;
    let mut limited = reader.take(declared.saturating_add(1));
    let written = std::io::copy(&mut limited, &mut out)?;
    out.flush()?;
    if written > declared {
        return Err(ThemeError::TooLarge);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unsafe names are refused, ordinary ones normalised.
    #[test]
    fn safe_relative_rules() {
        assert_eq!(
            safe_relative("themes/a/theme.json"),
            Some(PathBuf::from("themes/a/theme.json"))
        );
        assert_eq!(safe_relative("./a//b"), Some(PathBuf::from("a/b")));
        for bad in [
            "../evil",
            "a/../../evil",
            "/abs/x",
            "\\abs",
            "C:/x",
            "a\\..\\b",
            "",
        ] {
            assert_eq!(safe_relative(bad), None, "should refuse {bad:?}");
        }
    }
}
