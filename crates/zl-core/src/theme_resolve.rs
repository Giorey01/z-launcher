//! The resolved theme: what the UI draws with, computed from the selection, the user's settings
//! and the files on disk.

use std::path::Path;

use zl_config::appearance::ACCENT_OVERRIDE_GLOW_ALPHA;
use zl_config::media::VIDEO_WALLPAPER_EXTENSIONS;
use zl_theme::{ColorKey, SoundAction, StringEnum, GLOBAL_OVERLAY_ID};

use crate::appearance_types::{
    FolderArt, OverlayArt, ResolvedTheme, SoundEntry, ThemeColors, Wallpaper, WallpaperKind,
};
use crate::error::CoreError;
use crate::theme_files::{path_text, ThemeFiles};
use crate::Service;

/// The kind of a wallpaper file from its extension.
fn wallpaper_kind(path: &Path) -> WallpaperKind {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if ext == "gif" {
        WallpaperKind::Gif
    } else if VIDEO_WALLPAPER_EXTENSIONS.contains(&ext.as_str()) {
        WallpaperKind::Video
    } else {
        WallpaperKind::Image
    }
}

/// A wallpaper record from an optional file.
fn wallpaper(path: Option<&std::path::PathBuf>) -> Option<Wallpaper> {
    path.map(|p| Wallpaper {
        path: p.to_string_lossy().into_owned(),
        kind: wallpaper_kind(p),
    })
}

/// Reads the colours of the theme: missing ones default, the selection gradient follows the
/// accent when the theme sets none, and the user's accent replaces accent and selection.
fn resolve_colors(files: &ThemeFiles, accent_override: Option<u32>) -> ThemeColors {
    let mut c = ThemeColors::read(files.scheme.as_ref());
    let has = |key: ColorKey| files.scheme.as_ref().is_some_and(|s| s.get(key).is_some());
    if !has(ColorKey::SelectionGradientStart) {
        c.selection_gradient_start = c.accent_gradient_start;
    }
    if !has(ColorKey::SelectionGradientEnd) {
        c.selection_gradient_end = c.accent_gradient_end;
    }
    if let Some(accent) = accent_override {
        let glow = (accent & 0x00FF_FFFF) | (u32::from(ACCENT_OVERRIDE_GLOW_ALPHA) << 24);
        c.accent_gradient_start = accent;
        c.accent_gradient_end = accent;
        c.accent_glow = glow;
        c.selection_gradient_start = accent;
        c.selection_gradient_end = accent;
    }
    c
}

/// The three images of a folder at `<prefix>/<name>/`.
fn folder_art(files: &ThemeFiles, prefix: &str, name: &str) -> FolderArt {
    let get = |slot: &str| path_text(files.resolve(&format!("{prefix}/{name}/{slot}")));
    FolderArt {
        icon: get("icon"),
        hero: get("hero"),
        logo: get("logo"),
    }
}

/// Mask and overlay under `<prefix>` (`icon_overlay/<id>` or `icon_overlay_global`).
fn overlay_art(files: &ThemeFiles, prefix: &str) -> OverlayArt {
    OverlayArt {
        mask: path_text(files.resolve(&format!("{prefix}/mask"))),
        overlay: path_text(files.resolve(&format!("{prefix}/overlay"))),
    }
}

impl Service {
    /// The active theme with everything the UI needs, the defaults applied.
    pub fn active_theme(&self) -> Result<ResolvedTheme, CoreError> {
        let files = self.theme_files();
        let settings = self.user_settings()?;
        let mut music = files.music.clone();
        music.mode = settings.music_mode;
        Ok(ResolvedTheme {
            theme_id: files.id.clone(),
            name: files.name.clone(),
            fell_back: files.fell_back,
            mode: files.mode,
            colors: resolve_colors(&files, settings.accent_override),
            settings,
            wallpaper_main: wallpaper(files.wallpaper_main.as_ref()),
            wallpaper_external: wallpaper(files.wallpaper_external.as_ref()),
            sounds: SoundAction::ALL
                .iter()
                .map(|&action| SoundEntry {
                    action,
                    path: path_text(files.sound(action)),
                })
                .collect(),
            music,
        })
    }

    /// Icon, hero and logo of a platform's folder: the user's file, else the explicit pack, else
    /// the theme (and the packs it refers to), each image on its own. Aliases of the platform
    /// (`megadrive` for `genesis`) are tried by the pack loader.
    pub fn platform_art(&self, platform_id: &str) -> FolderArt {
        let id = self.canonical_id(&platform_id.to_lowercase());
        folder_art(&self.theme_files(), "smart_folder_platform", &id)
    }

    /// Icon, hero and logo of a special folder (`favorites`, `recent`, `lastplayed`,
    /// `emulators`, `androidgames`, `cloud`, ...), through the same precedence.
    pub fn smart_folder_art(&self, folder_type: &str) -> FolderArt {
        folder_art(
            &self.theme_files(),
            "smart_folder",
            &folder_type.to_lowercase(),
        )
    }

    /// Mask and overlay for the covers of a platform. A platform with no art of its own gets the
    /// `global` one.
    pub fn icon_overlay(&self, platform_id: &str) -> OverlayArt {
        let files = self.theme_files();
        let id = self.canonical_id(&platform_id.to_lowercase());
        let own = overlay_art(&files, &format!("icon_overlay/{id}"));
        if own.mask.is_some() || own.overlay.is_some() {
            own
        } else {
            overlay_art(&files, &format!("icon_overlay_{GLOBAL_OVERLAY_ID}"))
        }
    }
}
