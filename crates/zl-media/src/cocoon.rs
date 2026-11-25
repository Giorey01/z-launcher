//! Cocoon's media folder: `<data>/downloaded_media/<platform>/<kind>/<name>`.
//!
//! `<kind>` is `icon`, `hero`, `logo`, `screenshot_gameplay` or `screenshot_title`. A name is the
//! ROM's file stem plus the extension, or, for a file the user edited,
//! `<stem>__cocoon_edit_<n>_<32 hex>.<ext>`; the edit with the highest `n` is the newest.

use std::path::Path;

use zl_config::media::{
    COCOON_EDIT_HASH_LEN, COCOON_EDIT_MARKER, COCOON_MEDIA_FOLDER, IMAGE_EXTENSIONS,
    MEDIA_SCAN_DEPTH,
};
use zl_db::{MediaKind, MediaSource};

use crate::found::FoundMedia;
use crate::keys::stem_key;
use crate::walk::{extension_of, files_below, stem_of, sub_dirs};

/// The media kind of a Cocoon kind folder, `None` for a folder the reader does not know.
fn kind_of_folder(name: &str) -> Option<MediaKind> {
    match name.to_ascii_lowercase().as_str() {
        "icon" => Some(MediaKind::Icon),
        "hero" => Some(MediaKind::Hero),
        "logo" => Some(MediaKind::Logo),
        "screenshot_gameplay" => Some(MediaKind::Screenshot),
        "screenshot_title" => Some(MediaKind::TitleScreen),
        _ => None,
    }
}

/// Splits a file stem into the ROM stem and the edit number: `Game__cocoon_edit_3_<hex>` gives
/// `("Game", Some(3))`; a stem without a well-formed edit suffix comes back whole with `None`.
pub fn parse_cocoon_name(stem: &str) -> (String, Option<u32>) {
    let Some(at) = stem.rfind(COCOON_EDIT_MARKER) else {
        return (stem.to_string(), None);
    };
    let tail = &stem[at + COCOON_EDIT_MARKER.len()..];
    let parsed = tail.split_once('_').and_then(|(n, hash)| {
        let well_formed =
            hash.len() == COCOON_EDIT_HASH_LEN && hash.chars().all(|c| c.is_ascii_hexdigit());
        // `parse` alone would take `+3`: the number is plain digits.
        let digits = !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit());
        (well_formed && digits)
            .then(|| n.parse::<u32>().ok())
            .flatten()
    });
    match parsed {
        Some(n) => (stem[..at].to_string(), Some(n)),
        None => (stem.to_string(), None),
    }
}

/// Lists the media of a Cocoon data folder. A missing `downloaded_media/` gives an empty list;
/// files that are not images, kind folders the reader does not know and platforms without files
/// are skipped.
pub fn scan_cocoon(data_dir: &Path) -> Vec<FoundMedia> {
    let mut out = Vec::new();
    for (platform, platform_dir) in sub_dirs(&data_dir.join(COCOON_MEDIA_FOLDER)) {
        for (folder, kind_dir) in sub_dirs(&platform_dir) {
            let Some(kind) = kind_of_folder(&folder) else {
                continue;
            };
            for path in files_below(&kind_dir, MEDIA_SCAN_DEPTH) {
                let is_image =
                    extension_of(&path).is_some_and(|e| IMAGE_EXTENSIONS.contains(&e.as_str()));
                let Some(stem) = stem_of(&path).filter(|_| is_image) else {
                    continue;
                };
                let (rom_stem, edit) = parse_cocoon_name(&stem);
                out.push(FoundMedia {
                    platform: platform.to_ascii_lowercase(),
                    stem: stem_key(&rom_stem),
                    kind,
                    path,
                    source: MediaSource::Cocoon,
                    edit,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::parse_cocoon_name;

    /// A well-formed edit name gives the ROM stem and the number.
    #[test]
    fn edit_name_is_split() {
        let hash = "0123456789abcdef0123456789ABCDEF";
        assert_eq!(
            parse_cocoon_name(&format!("Sonic (USA)__cocoon_edit_12_{hash}")),
            ("Sonic (USA)".to_string(), Some(12))
        );
    }

    /// A malformed suffix (short hash, no number) is part of the name.
    #[test]
    fn malformed_suffix_is_kept() {
        for stem in [
            "Game__cocoon_edit_1_abc",
            "Game__cocoon_edit_x_0123456789abcdef0123456789abcdef",
            "Game__cocoon_edit_",
            "Game",
        ] {
            assert_eq!(parse_cocoon_name(stem), (stem.to_string(), None), "{stem}");
        }
    }
}
