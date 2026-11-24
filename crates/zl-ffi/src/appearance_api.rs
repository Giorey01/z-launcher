//! The theme, settings and media half of the [`Core`] object.

use zl_db::MediaKind as DbMediaKind;

use crate::appearance::{
    AssetCategory, CocoonDataDir, CursorStyle, ExternalDisplayDocking, FolderArt, FolderSlot,
    GameMedia, IconStyle, ImportReport, InstalledPack, InstalledTheme, MediaImportReport,
    MediaKind, MediaSources, MusicMode, OverlayArt, PlatformEmulatorSetting, ResolvedTheme,
    ScreenMode, SingleRowStyle, SurfaceMaterial, ThemeIssue, ThemeSelection, UserSettings,
};
use crate::core_object::Core;
use crate::error::CoreError;

/// Result of a core call.
type Res<T> = Result<T, CoreError>;

#[uniffi::export]
impl Core {
    /// Imports a theme ZIP (also packs, if the archive has `assets/`). `themes_added` holds the
    /// ids now installed; problems inside the themes come back as `issues`. An unreadable or
    /// unsafe archive is `CoreError.Theme` with a code.
    pub fn import_theme_zip(&self, path: String) -> Res<ImportReport> {
        Ok(self.service().import_theme_zip(&path)?.into())
    }

    /// Copies the themes and asset packs of a Cocoon data folder into the app's, skipping those
    /// already installed with the same id and version.
    pub fn import_from_cocoon(&self, cocoon_data_dir: String) -> Res<ImportReport> {
        Ok(self.service().import_from_cocoon(&cocoon_data_dir)?.into())
    }

    /// Cocoon data folders found among the candidate paths of the configuration (needs storage
    /// access; an empty list means none was found or it cannot be read).
    pub fn detect_cocoon_data_dirs(&self) -> Vec<CocoonDataDir> {
        self.service()
            .detect_cocoon_data_dirs()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    /// The installed themes, by name.
    pub fn installed_themes(&self) -> Vec<InstalledTheme> {
        self.service()
            .installed_themes()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    /// The installed asset packs of all layers, by layer and name.
    pub fn installed_packs(&self) -> Vec<InstalledPack> {
        self.service()
            .installed_packs()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    /// The selected theme and the pack in use for each layer.
    pub fn theme_selection(&self) -> ThemeSelection {
        self.service().theme_selection().into()
    }

    /// The problems of an installed theme or pack, as codes and locations.
    pub fn theme_issues(&self, id: String) -> Vec<ThemeIssue> {
        self.service()
            .theme_issues(&id)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    /// Selects a theme (`None` for the built-in one) and clears the layer packs chosen before.
    /// With `apply_settings` the theme's own settings, music mode and volumes are copied into the
    /// user's settings once (the built-in theme restores all defaults). `NotFound` for an id
    /// that is not installed.
    pub fn apply_theme(&self, theme_id: Option<String>, apply_settings: bool) -> Res<()> {
        Ok(self
            .service()
            .apply_theme(theme_id.as_deref(), apply_settings)?)
    }

    /// Chooses the pack of one layer; `None` follows the theme. `NotFound` for an id that is not
    /// installed in that category.
    pub fn set_layer_pack(&self, category: AssetCategory, pack_id: Option<String>) -> Res<()> {
        Ok(self
            .service()
            .set_layer_pack(category.into(), pack_id.as_deref())?)
    }

    /// The active theme with everything to draw: mode, colours, settings, wallpapers, sounds and
    /// music. Cheap to call again after any change.
    pub fn active_theme(&self) -> Res<ResolvedTheme> {
        Ok(self.service().active_theme()?.into())
    }

    /// Icon, hero and logo of the folder of a platform (any id or alias).
    pub fn platform_art(&self, platform_id: String) -> FolderArt {
        self.service().platform_art(&platform_id).into()
    }

    /// Icon, hero and logo of a special folder: `favorites`, `recent`, `lastplayed`,
    /// `emulators`, `androidgames`, `android_app`, `cloud`, ...
    pub fn smart_folder_art(&self, folder_type: String) -> FolderArt {
        self.service().smart_folder_art(&folder_type).into()
    }

    /// Mask and overlay for the covers of a platform, the `global` ones when it has none.
    pub fn icon_overlay(&self, platform_id: String) -> OverlayArt {
        self.service().icon_overlay(&platform_id).into()
    }

    /// Replaces one image of a smart folder (or, with `by_platform`, of a platform folder) with a
    /// picked file, which wins over every theme and pack; `None` removes the replacement.
    pub fn set_user_folder_art(
        &self,
        name: String,
        by_platform: bool,
        slot: FolderSlot,
        source_path: Option<String>,
    ) -> Res<()> {
        Ok(self.service().set_user_folder_art(
            &name,
            by_platform,
            slot.into(),
            source_path.as_deref(),
        )?)
    }

    /// All the look-and-sound settings, defaults applied.
    pub fn user_settings(&self) -> Res<UserSettings> {
        Ok(self.service().user_settings()?.into())
    }

    /// Stores all the settings (numbers are limited to their ranges).
    pub fn set_user_settings(&self, settings: UserSettings) -> Res<()> {
        Ok(self.service().set_user_settings(&settings.into())?)
    }

    /// Sets (or with `None` clears) the accent colour (`0xAARRGGBB`).
    pub fn set_accent_override(&self, argb: Option<u32>) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.accent_override = argb)?;
        Ok(())
    }

    /// Sets the cursor style.
    pub fn set_cursor_style(&self, value: CursorStyle) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.cursor_style = value.into())?;
        Ok(())
    }

    /// Sets the surface material.
    pub fn set_surface_material(&self, value: SurfaceMaterial) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.surface_material = value.into())?;
        Ok(())
    }

    /// Sets the icon style.
    pub fn set_icon_style(&self, value: IconStyle) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.icon_style = value.into())?;
        Ok(())
    }

    /// Sets the thickness of the selection highlight (0.5-1.0).
    pub fn set_highlight_thickness(&self, value: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.highlight_thickness = value)?;
        Ok(())
    }

    /// Switches the interface sound effects on or off.
    pub fn set_sfx_enabled(&self, enabled: bool) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.sfx_enabled = enabled)?;
        Ok(())
    }

    /// Sets the volume of the sound effects (0-1).
    pub fn set_sfx_volume(&self, value: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.sfx_volume = value)?;
        Ok(())
    }

    /// Sets the background music mode.
    pub fn set_music_mode(&self, mode: MusicMode) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.music_mode = mode.into())?;
        Ok(())
    }

    /// Sets the volume of the music (0-1).
    pub fn set_music_volume(&self, value: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.music_volume = value)?;
        Ok(())
    }

    /// Opens folders full screen or not.
    pub fn set_fullscreen_folders(&self, on: bool) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.fullscreen_folders = on)?;
        Ok(())
    }

    /// Always shows the corner hints or not.
    pub fn set_corner_hints(&self, on: bool) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.corner_hints = on)?;
        Ok(())
    }

    /// Puts the dock at the top of the screen or not.
    pub fn set_dock_on_top(&self, on: bool) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.dock_on_top = on)?;
        Ok(())
    }

    /// Chooses one display or two.
    pub fn set_screen_mode(&self, mode: ScreenMode) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.screen_mode = mode.into())?;
        Ok(())
    }

    /// Swaps which display shows the grid and which the hero.
    pub fn set_reverse_display_roles(&self, on: bool) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.reverse_display_roles = on)?;
        Ok(())
    }

    /// Chooses whether the Home moves to an external display.
    pub fn set_external_display_docking(&self, value: ExternalDisplayDocking) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.external_display_docking = value.into())?;
        Ok(())
    }

    /// Single screen: fraction (0-1) of the screen given to the hero area.
    pub fn set_hero_split(&self, fraction: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.hero_split = fraction)?;
        Ok(())
    }

    /// Single screen: number of grid rows (1-5).
    pub fn set_single_screen_rows(&self, rows: u8) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.single_screen_rows = rows)?;
        Ok(())
    }

    /// Single screen: style of the rows.
    pub fn set_single_row_style(&self, style: SingleRowStyle) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.single_row_style = style.into())?;
        Ok(())
    }

    /// Removes a ROM folder and its games from the library (the files stay on disk).
    pub fn remove_root(&self, root_id: i64) -> Res<()> {
        Ok(self.service().remove_root(root_id)?)
    }

    /// Every platform of the library with its default emulator and the installed candidates
    /// (`installed_packages` as for `detect_installed`). Set one with `set_platform_default`.
    pub fn platform_emulators(
        &self,
        installed_packages: Vec<String>,
    ) -> Res<Vec<PlatformEmulatorSetting>> {
        let rows = self.service().platform_emulators(&installed_packages)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Where media is read from.
    pub fn media_sources(&self) -> Res<MediaSources> {
        Ok(self.service().media_sources()?.into())
    }

    /// Sets where media is read from; `None` restores the default of each.
    pub fn set_media_sources(
        &self,
        cocoon_dir: Option<String>,
        esde_root: Option<String>,
    ) -> Res<()> {
        Ok(self
            .service()
            .set_media_sources(cocoon_dir.as_deref(), esde_root.as_deref())?)
    }

    /// Reads the Cocoon and ES-DE media and stores what matches a game. The user's choices are
    /// never touched.
    pub fn import_media(&self) -> Res<MediaImportReport> {
        Ok(self.service().import_media()?.into())
    }

    /// The media of a game: best file of each kind and the full list.
    pub fn game_media(&self, game_id: i64) -> Res<GameMedia> {
        Ok(self.service().game_media(game_id)?.into())
    }

    /// Sets the user's file for one kind of a game (protected from imports); `None` clears it.
    pub fn set_game_media(&self, game_id: i64, kind: MediaKind, path: Option<String>) -> Res<()> {
        let kind: DbMediaKind = kind.into();
        Ok(self
            .service()
            .set_game_media(game_id, kind, path.as_deref())?)
    }
}
