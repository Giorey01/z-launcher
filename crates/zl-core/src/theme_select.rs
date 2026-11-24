//! Choosing the theme and the layer packs, applying a theme's settings, and listing what is
//! installed.

use std::fs;
use std::path::Path;

use zl_config::theme::USER_ART_DIR_NAME;
use zl_theme::{
    list_installed, read_package, validate_pack, validate_package, AssetCategory, Severity,
    ThemePackage, FOLDER_ART_EXTENSIONS,
};

use crate::appearance_types::{
    FolderSlot, InstalledPack, InstalledTheme, LayerChoice, PackRef, ThemeIssue, ThemeSelection,
    UserSettings,
};
use crate::error::CoreError;
use crate::theme_files::{pack_key, path_text, KEY_ACTIVE_THEME, LAYERS};
use crate::Service;

/// Sets `target` when the theme has a value.
fn set<T>(target: &mut T, value: Option<T>) {
    if let Some(v) = value {
        *target = v;
    }
}

/// Sets a number from the theme's double-precision value.
fn set_num(target: &mut f32, value: Option<f64>) {
    set(target, value.map(|v| v as f32));
}

/// Copies the numbers of the theme's `settings`.
fn merge_numbers(s: &mut UserSettings, t: &zl_theme::Settings) {
    set_num(&mut s.icon_roundness, t.icon_roundness);
    set_num(&mut s.icon_scale, t.icon_scale);
    set_num(&mut s.hover_scale, t.hover_scale);
    set_num(&mut s.hero_logo_scale, t.hero_logo_scale);
    set_num(&mut s.hero_folder_logo_scale, t.hero_folder_logo_scale);
    set_num(&mut s.hero_vignette_size, t.hero_vignette_size);
    set_num(&mut s.glass_blur, t.glass_blur);
    set_num(&mut s.glass_refraction, t.glass_refraction);
    set_num(&mut s.glass_highlight, t.glass_highlight);
    set_num(&mut s.glass_rim_strength, t.glass_rim_strength);
    set_num(&mut s.glass_tint, t.glass_tint);
    set_num(&mut s.highlight_thickness, t.highlight_thickness);
}

/// Copies the choices and switches of the theme's `settings`.
fn merge_choices(s: &mut UserSettings, t: &zl_theme::Settings) {
    set(&mut s.hero_display_style, t.hero_display_style);
    set(&mut s.hero_gradient_style, t.hero_gradient_style);
    set(&mut s.icon_style, t.icon_style);
    set(&mut s.surface_material, t.surface_material);
    set(&mut s.glass_on_tiles, t.glass_on_tiles);
    set(&mut s.cursor_style, t.cursor_style);
    set(&mut s.dock_icon_shape, t.dock_icon_shape);
    set(&mut s.hide_dock_background, t.hide_dock_background);
    set(&mut s.dock_on_top, t.dock_on_top);
    set(&mut s.corner_hints, t.corner_hints);
}

/// Overwrites the settings the theme sets (an absent key leaves the user's value): its
/// `settings` object, its volumes and its music mode.
fn merge_theme_settings(
    s: &mut UserSettings,
    pkg: &ThemePackage,
    music_mode: Option<zl_theme::MusicMode>,
) {
    let Some(m) = &pkg.manifest else { return };
    if let Some(t) = &m.settings {
        merge_numbers(s, t);
        merge_choices(s, t);
    }
    set_num(&mut s.sfx_volume, m.music.sfx_volume);
    set_num(&mut s.music_volume, m.music.music_volume);
    set_num(&mut s.jingle_volume, m.music.jingle_volume);
    set(&mut s.music_mode, music_mode);
}

impl Service {
    /// Selects a theme (`None` for the built-in one). The layer packs the user had chosen are
    /// cleared, so the new theme's own packs apply. With `apply_settings` the theme's values for
    /// the settings it defines (and its music mode and volumes) are copied into the user's
    /// settings, once; the built-in theme with `apply_settings` restores every setting to its
    /// default. Without it only the look changes (colours, wallpapers, art, sounds, music files).
    pub fn apply_theme(
        &self,
        theme_id: Option<&str>,
        apply_settings: bool,
    ) -> Result<(), CoreError> {
        let package = match theme_id {
            Some(id) => Some(self.read_installed_theme(id)?),
            None => None,
        };
        self.db()
            .set_setting(KEY_ACTIVE_THEME, theme_id.unwrap_or(""))?;
        for category in LAYERS {
            self.db().set_setting(&pack_key(category), "")?;
        }
        self.invalidate_theme();
        if apply_settings {
            let mut settings = UserSettings::default();
            if let Some(pkg) = &package {
                settings = self.user_settings()?;
                let mode = self.theme_files().declared_music_mode;
                merge_theme_settings(&mut settings, pkg, mode);
            }
            self.set_user_settings(&settings)?;
        }
        Ok(())
    }

    /// Reads an installed theme by id; a missing or invalid one is `NotFound`.
    fn read_installed_theme(&self, id: &str) -> Result<ThemePackage, CoreError> {
        let dir = self.data_dir.join("themes").join(id);
        if id.is_empty() || id.contains('/') || id.contains("..") {
            return Err(CoreError::InvalidInput);
        }
        read_package(&dir)
            .ok()
            .filter(|p| p.manifest.is_some())
            .ok_or(CoreError::NotFound)
    }

    /// Chooses the pack of one layer, or with `None` goes back to the theme's own. An id that is
    /// not installed in that category is `NotFound`.
    pub fn set_layer_pack(
        &self,
        category: AssetCategory,
        pack_id: Option<&str>,
    ) -> Result<(), CoreError> {
        if let Some(id) = pack_id {
            let installed = list_installed(&self.data_dir);
            if !installed
                .packs
                .iter()
                .any(|p| p.category == category && p.id == id)
            {
                return Err(CoreError::NotFound);
            }
        }
        self.db()
            .set_setting(&pack_key(category), pack_id.unwrap_or(""))?;
        self.invalidate_theme();
        Ok(())
    }

    /// The selected theme and, for each layer, the explicit and the effective pack.
    pub fn theme_selection(&self) -> ThemeSelection {
        let files = self.theme_files();
        let installed = list_installed(&self.data_dir);
        let refs = self
            .theme_package()
            .and_then(|p| p.manifest)
            .and_then(|m| m.asset_refs)
            .unwrap_or_default();
        let has = |c: AssetCategory, id: &str| {
            installed
                .packs
                .iter()
                .any(|p| p.category == c && p.id == id)
        };
        let layers = LAYERS
            .iter()
            .map(|&category| {
                let explicit_pack = self
                    .theme_setting(&pack_key(category))
                    .filter(|id| has(category, id));
                let from_theme = refs
                    .iter()
                    .find(|r| r.category == category && has(category, &r.id))
                    .map(|r| r.id.clone());
                LayerChoice {
                    category,
                    effective_pack: explicit_pack.clone().or(from_theme),
                    explicit_pack,
                }
            })
            .collect();
        ThemeSelection {
            theme_id: files.id.clone(),
            layers,
        }
    }

    /// The package of the selected theme, if one is selected and readable.
    fn theme_package(&self) -> Option<ThemePackage> {
        let id = self.theme_files().id.clone()?;
        read_package(&self.data_dir.join("themes").join(id)).ok()
    }

    /// The installed themes, by name.
    pub fn installed_themes(&self) -> Vec<InstalledTheme> {
        let content = list_installed(&self.data_dir);
        let ids = content.pack_ids();
        let active = self.theme_files().id.clone();
        let mut out: Vec<InstalledTheme> = content
            .themes
            .iter()
            .filter_map(|dir| read_package(dir).ok())
            .map(|pkg| describe_theme(&pkg, &ids, active.as_deref()))
            .collect();
        out.sort_by_key(|t| t.name.to_lowercase());
        out
    }

    /// The installed asset packs (all layers), by category then name.
    pub fn installed_packs(&self) -> Vec<InstalledPack> {
        let selection = self.theme_selection();
        let mut out: Vec<InstalledPack> = list_installed(&self.data_dir)
            .packs
            .iter()
            .map(|p| {
                let choice = selection.layers.iter().find(|l| l.category == p.category);
                InstalledPack {
                    id: p.id.clone(),
                    category: p.category,
                    name: p
                        .meta
                        .as_ref()
                        .and_then(|m| m.name.clone())
                        .unwrap_or_else(|| p.id.clone()),
                    author: p.meta.as_ref().and_then(|m| m.author.clone()),
                    version: p.meta.as_ref().and_then(|m| m.version.clone()),
                    preview: path_text(preview_of(&p.dir)),
                    in_use: choice.and_then(|c| c.effective_pack.as_deref()) == Some(p.id.as_str()),
                    explicit: choice.and_then(|c| c.explicit_pack.as_deref())
                        == Some(p.id.as_str()),
                }
            })
            .collect();
        out.sort_by_key(|p| (p.category, p.name.to_lowercase()));
        out
    }

    /// The problems of one installed theme or pack (`id` of either), as codes and locations.
    pub fn theme_issues(&self, id: &str) -> Vec<ThemeIssue> {
        let content = list_installed(&self.data_dir);
        let mut out = Vec::new();
        if let Some(dir) = content
            .themes
            .iter()
            .find(|d| d.file_name().and_then(|n| n.to_str()) == Some(id))
        {
            if let Ok(pkg) = read_package(dir) {
                out.extend(issues_of(id, &validate_package(&pkg, &content.pack_ids())));
            }
        }
        for pack in content.packs.iter().filter(|p| p.id == id) {
            out.extend(issues_of(id, &validate_pack(pack)));
        }
        out
    }

    /// Replaces one image (`icon`, `hero` or `logo`) of a smart folder or platform folder with a
    /// file of the user's (copied into the app's data folder), or with `None` removes the
    /// replacement. The user's files win over every pack and theme.
    pub fn set_user_folder_art(
        &self,
        name: &str,
        by_platform: bool,
        slot: FolderSlot,
        source: Option<&str>,
    ) -> Result<(), CoreError> {
        let name = name.trim().to_lowercase();
        if name.is_empty() || name.contains('/') || name.contains("..") {
            return Err(CoreError::InvalidInput);
        }
        let name = if by_platform {
            self.canonical_id(&name)
        } else {
            name
        };
        let mut dir = self.data_dir.join(USER_ART_DIR_NAME).join("smart_folders");
        if by_platform {
            dir.push("by_platform");
        }
        dir.push(&name);
        let stem = match slot {
            FolderSlot::Icon => "icon",
            FolderSlot::Hero => "hero",
            FolderSlot::Logo => "logo",
        };
        for ext in FOLDER_ART_EXTENSIONS {
            let _ = fs::remove_file(dir.join(format!("{stem}.{ext}")));
        }
        if let Some(source) = source {
            copy_user_art(Path::new(source), &dir, stem)?;
        }
        self.invalidate_theme();
        Ok(())
    }
}

/// Copies a picked image to `<dir>/<stem>.<ext>`; an extension the art format does not take is
/// `InvalidInput`.
fn copy_user_art(source: &Path, dir: &Path, stem: &str) -> Result<(), CoreError> {
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .filter(|e| FOLDER_ART_EXTENSIONS.contains(&e.as_str()))
        .ok_or(CoreError::InvalidInput)?;
    fs::create_dir_all(dir).map_err(|_| CoreError::Database)?;
    fs::copy(source, dir.join(format!("{stem}.{ext}"))).map_err(|_| CoreError::NotFound)?;
    Ok(())
}

/// `preview.png` of a folder, any case.
fn preview_of(dir: &Path) -> Option<std::path::PathBuf> {
    fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.eq_ignore_ascii_case("preview.png"))
        })
}

/// Converts issues of one source.
fn issues_of(source: &str, issues: &[zl_theme::Issue]) -> Vec<ThemeIssue> {
    issues
        .iter()
        .map(|i| ThemeIssue {
            source: source.to_string(),
            severity: i.severity,
            code: i.code,
            location: i.location.clone(),
        })
        .collect()
}

/// The list entry of one theme.
fn describe_theme(pkg: &ThemePackage, pack_ids: &[String], active: Option<&str>) -> InstalledTheme {
    let issues = validate_package(pkg, pack_ids);
    let count = |s: Severity| {
        u32::try_from(issues.iter().filter(|i| i.severity == s).count()).unwrap_or(u32::MAX)
    };
    let m = pkg.manifest.as_ref();
    let missing_packs = m
        .and_then(|m| m.asset_refs.as_ref())
        .into_iter()
        .flatten()
        .filter(|r| !pack_ids.contains(&r.id))
        .map(|r| PackRef {
            category: r.category,
            id: r.id.clone(),
            name: r.name.clone(),
        })
        .collect();
    InstalledTheme {
        id: pkg.id.clone(),
        name: m
            .map(|m| m.name.clone())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| pkg.id.clone()),
        author: m.and_then(|m| m.author.clone()),
        version: m.and_then(|m| m.version.clone()),
        description: m.and_then(|m| m.description.clone()),
        mode: m.and_then(|m| m.theme_mode),
        preview: path_text(pkg.preview.clone()),
        wallpaper_main: path_text(pkg.wallpaper_main.clone()),
        has_settings: m.is_some_and(|m| m.settings.is_some()),
        missing_packs,
        error_count: count(Severity::Error),
        warning_count: count(Severity::Warning),
        active: active == Some(pkg.id.as_str()),
    }
}
