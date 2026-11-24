//! Finding the wallpaper files of a theme (spec 3.4).

use std::path::{Path, PathBuf};

use crate::fsutil::{list_dir, Entry};

/// Extensions a wallpaper may have (the ones seen in themes, plus jpg as tolerance).
pub(crate) const WALLPAPER_EXTENSIONS: &[&str] =
    &["png", "webp", "gif", "mp4", "webm", "jpg", "jpeg"];

/// Names tried for the main wallpaper when `wallpaper_main` is absent.
pub(crate) const MAIN_FALLBACKS: &[&str] = &["main", "bottom"];
/// Names tried for the external wallpaper when `wallpaper_external` is absent.
pub(crate) const EXTERNAL_FALLBACKS: &[&str] = &["external", "top"];

/// Resolves a wallpaper inside `<theme>/wallpapers/`.
///
/// With a `declared` name: the file with exactly that name, else that name plus a supported
/// extension; a declared name that matches nothing gives `None` (the validator reports it).
/// Without one, each of `fallbacks` is tried as a stem with any supported extension.
pub(crate) fn resolve(
    theme_dir: &Path,
    declared: Option<&str>,
    fallbacks: &[&str],
) -> Option<PathBuf> {
    let entries = list_dir(&theme_dir.join("wallpapers"));
    match declared {
        Some(name) => by_name(&entries, name.trim()),
        None => fallbacks.iter().find_map(|stem| by_stem(&entries, stem)),
    }
}

/// Matches a declared name with or without extension.
fn by_name(entries: &[Entry], name: &str) -> Option<PathBuf> {
    entries
        .iter()
        .find(|e| !e.is_dir && e.name.eq_ignore_ascii_case(name))
        .map(|e| e.path.clone())
        .or_else(|| by_stem(entries, name))
}

/// Matches `stem.<supported extension>`, ignoring case.
fn by_stem(entries: &[Entry], stem: &str) -> Option<PathBuf> {
    entries
        .iter()
        .filter(|e| !e.is_dir)
        .find(|e| match e.name.rsplit_once('.') {
            Some((s, ext)) => {
                s.eq_ignore_ascii_case(stem)
                    && WALLPAPER_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str())
            }
            None => false,
        })
        .map(|e| e.path.clone())
}
