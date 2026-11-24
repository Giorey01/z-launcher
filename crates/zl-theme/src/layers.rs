//! Layer precedence (spec section 1): which file serves an asset slot.
//!
//! The order is user file, then the layer the user chose explicitly, then the base theme, then
//! the app default. Each slot falls back on its own, so a layer that lacks `hero.png` of a
//! folder does not lose its `icon.png`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::enums::StringEnum;
use crate::package::{FolderArt, ThemePackage};

string_enum! {
    /// Where a file comes from, in decreasing precedence (the derived order is the precedence).
    SourceLevel {
        /// A file the user picked.
        User => "user",
        /// A layer package chosen explicitly.
        Explicit => "explicit",
        /// The base theme.
        Base => "base",
        /// The app's built-in default.
        Default => "default",
    }
}

/// A source of files, keyed by slot (see [`slots_of_package`] for the key format).
#[derive(Debug, Clone, Copy)]
pub struct LayerSource<'a> {
    /// Precedence level of this source.
    pub level: SourceLevel,
    /// Files this source provides.
    pub files: &'a BTreeMap<String, PathBuf>,
}

/// Which file serves a slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    /// Level of the winning source; [`SourceLevel::Default`] with no path means "use the built-in".
    pub level: SourceLevel,
    /// The file, or `None` when no source has the slot.
    pub path: Option<PathBuf>,
}

/// Picks the file for `slot`: the source with the highest-precedence level that has it (the
/// slice order only breaks ties between sources of the same level, earlier wins).
pub fn resolve_slot(sources: &[LayerSource<'_>], slot: &str) -> Resolution {
    sources
        .iter()
        .filter(|s| s.files.contains_key(slot))
        .min_by_key(|s| s.level)
        .map(|s| Resolution {
            level: s.level,
            path: s.files.get(slot).cloned(),
        })
        .unwrap_or(Resolution {
            level: SourceLevel::Default,
            path: None,
        })
}

/// Flattens a package into slots. Keys: `wallpaper/main`, `wallpaper/external`, `preview`,
/// `icon_overlay/<platform>/{mask,overlay}`, `smart_folder/<type>/{icon,hero,logo}`,
/// `smart_folder_platform/<platform>/{icon,hero,logo}`, `sound/<action>`.
pub fn slots_of_package(pkg: &ThemePackage) -> BTreeMap<String, PathBuf> {
    let mut slots = BTreeMap::new();
    let mut put = |key: String, path: &Option<PathBuf>| {
        if let Some(p) = path {
            slots.insert(key, p.clone());
        }
    };
    put("wallpaper/main".into(), &pkg.wallpaper_main);
    put("wallpaper/external".into(), &pkg.wallpaper_external);
    put("preview".into(), &pkg.preview);
    for (platform, art) in &pkg.icon_overlays {
        put(format!("icon_overlay/{platform}/mask"), &art.mask);
        put(format!("icon_overlay/{platform}/overlay"), &art.overlay);
    }
    for (name, art) in &pkg.smart_folders {
        put_folder(&mut put, "smart_folder", name, art);
    }
    for (name, art) in &pkg.smart_folders_by_platform {
        put_folder(&mut put, "smart_folder_platform", name, art);
    }
    for (action, path) in &pkg.sounds {
        put(format!("sound/{}", action.text()), &Some(path.clone()));
    }
    slots
}

/// Adds the three images of a folder under `<prefix>/<name>/`.
fn put_folder(
    put: &mut impl FnMut(String, &Option<PathBuf>),
    prefix: &str,
    name: &str,
    art: &FolderArt,
) {
    put(format!("{prefix}/{name}/icon"), &art.icon);
    put(format!("{prefix}/{name}/hero"), &art.hero);
    put(format!("{prefix}/{name}/logo"), &art.logo);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a file map from `(slot, path)` pairs.
    fn files(items: &[(&str, &str)]) -> BTreeMap<String, PathBuf> {
        items
            .iter()
            .map(|(k, v)| (k.to_string(), PathBuf::from(v)))
            .collect()
    }

    /// User beats explicit beats base; each slot falls back independently.
    #[test]
    fn precedence_and_independent_fallback() {
        let user = files(&[("smart_folder/favorites/icon", "user-icon.png")]);
        let explicit = files(&[
            ("smart_folder/favorites/icon", "pack-icon.png"),
            ("smart_folder/favorites/hero", "pack-hero.png"),
        ]);
        let base = files(&[
            ("smart_folder/favorites/hero", "base-hero.png"),
            ("smart_folder/favorites/logo", "base-logo.png"),
        ]);
        let sources = [
            LayerSource {
                level: SourceLevel::Base,
                files: &base,
            },
            LayerSource {
                level: SourceLevel::User,
                files: &user,
            },
            LayerSource {
                level: SourceLevel::Explicit,
                files: &explicit,
            },
        ];
        let icon = resolve_slot(&sources, "smart_folder/favorites/icon");
        assert_eq!(
            (icon.level, icon.path),
            (SourceLevel::User, Some("user-icon.png".into()))
        );
        let hero = resolve_slot(&sources, "smart_folder/favorites/hero");
        assert_eq!(
            (hero.level, hero.path),
            (SourceLevel::Explicit, Some("pack-hero.png".into()))
        );
        let logo = resolve_slot(&sources, "smart_folder/favorites/logo");
        assert_eq!(
            (logo.level, logo.path),
            (SourceLevel::Base, Some("base-logo.png".into()))
        );
        let none = resolve_slot(&sources, "sound/pop");
        assert_eq!((none.level, none.path), (SourceLevel::Default, None));
    }
}
