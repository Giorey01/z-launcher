//! ES-DE's media folder: `<media>/<system>/<folder>/<rom stem>.<ext>`.
//!
//! `<media>` is `<esde root>/downloaded_media` unless the `MediaDirectory` entry of
//! `settings/es_settings.xml` is not empty. `<folder>` is one of `covers`, `3dboxes`,
//! `backcovers`, `fanart`, `marquees`, `miximages`, `physicalmedia`, `screenshots`,
//! `titlescreens`, `videos`, `manuals`.

use std::path::{Path, PathBuf};

use zl_config::media::{
    ESDE_MEDIA_DIRECTORY_SETTING, ESDE_MEDIA_FOLDER, ESDE_SETTINGS_FILE, IMAGE_EXTENSIONS,
    MANUAL_EXTENSIONS, MEDIA_SCAN_DEPTH, VIDEO_EXTENSIONS,
};
use zl_db::{MediaKind, MediaSource};

use crate::found::FoundMedia;
use crate::walk::{extension_of, files_below, stem_of, sub_dirs};

/// The media kind of an ES-DE folder and the extensions it holds.
fn kind_of_folder(name: &str) -> Option<(MediaKind, &'static [&'static str])> {
    let images: &'static [&'static str] = &IMAGE_EXTENSIONS;
    Some(match name.to_ascii_lowercase().as_str() {
        "covers" => (MediaKind::Icon, images),
        "3dboxes" => (MediaKind::Box3d, images),
        "backcovers" => (MediaKind::BackCover, images),
        "fanart" => (MediaKind::Hero, images),
        "marquees" => (MediaKind::Logo, images),
        "miximages" => (MediaKind::MixImage, images),
        "physicalmedia" => (MediaKind::PhysicalMedia, images),
        "screenshots" => (MediaKind::Screenshot, images),
        "titlescreens" => (MediaKind::TitleScreen, images),
        "videos" => (MediaKind::Video, &VIDEO_EXTENSIONS),
        "manuals" => (MediaKind::Manual, &MANUAL_EXTENSIONS),
        _ => return None,
    })
}

/// The value of the XML `<string name="NAME" value="..." />` entry in `xml`, if present.
fn xml_string_value(xml: &str, name: &str) -> Option<String> {
    let needle = format!("name=\"{name}\"");
    let at = xml.find(&needle)?;
    let tag_start = xml[..at].rfind('<')?;
    let tag_end = at + xml[at..].find('>')?;
    let tag = &xml[tag_start..tag_end];
    let value_at = tag.find("value=\"")? + "value=\"".len();
    let value = &tag[value_at..];
    Some(value[..value.find('"')?].to_string())
}

/// The folder ES-DE keeps its scraped media in: the `MediaDirectory` setting when it is not
/// empty, else `<root>/downloaded_media`.
pub fn esde_media_dir(root: &Path) -> PathBuf {
    let custom = std::fs::read_to_string(root.join(ESDE_SETTINGS_FILE))
        .ok()
        .and_then(|xml| xml_string_value(&xml, ESDE_MEDIA_DIRECTORY_SETTING))
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    custom.map_or_else(|| root.join(ESDE_MEDIA_FOLDER), PathBuf::from)
}

/// Lists the media of an ES-DE data folder (`root` is the folder that holds `settings/`). A
/// missing media folder gives an empty list; unknown folders and files of the wrong type are
/// skipped.
pub fn scan_esde(root: &Path) -> Vec<FoundMedia> {
    let mut out = Vec::new();
    for (system, system_dir) in sub_dirs(&esde_media_dir(root)) {
        for (folder, kind_dir) in sub_dirs(&system_dir) {
            let Some((kind, exts)) = kind_of_folder(&folder) else {
                continue;
            };
            for path in files_below(&kind_dir, MEDIA_SCAN_DEPTH) {
                let allowed = extension_of(&path).is_some_and(|e| exts.contains(&e.as_str()));
                let Some(stem) = stem_of(&path).filter(|_| allowed) else {
                    continue;
                };
                out.push(FoundMedia {
                    platform: system.to_ascii_lowercase(),
                    stem: stem.to_lowercase(),
                    kind,
                    path,
                    source: MediaSource::Esde,
                    edit: None,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::xml_string_value;

    /// The value is read whatever the attribute order and the other entries.
    #[test]
    fn reads_media_directory() {
        let xml = r#"<?xml version="1.0"?>
<string name="Other" value="x" />
<string name="MediaDirectory" value="/sdcard/media" />"#;
        assert_eq!(
            xml_string_value(xml, "MediaDirectory").as_deref(),
            Some("/sdcard/media")
        );
        let swapped = r#"<string value="" name="MediaDirectory"/>"#;
        assert_eq!(
            xml_string_value(swapped, "MediaDirectory").as_deref(),
            Some("")
        );
        assert_eq!(xml_string_value(xml, "Missing"), None);
    }
}
