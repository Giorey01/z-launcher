//! The theme, settings and media half of the [`Core`] object.

use zl_db::MediaKind as DbMediaKind;

use crate::appearance::{
    AnimationSpeed, AssetCategory, CocoonDataDir, CursorStyle, DockIconShape,
    ExternalDisplayDocking, FolderArt, FolderSlot, GameMedia, HeroDisplayStyle, HeroGradientStyle,
    IconStyle, ImportReport, InstalledPack, InstalledTheme, MediaImportReport, MediaKind,
    MediaSources, MusicMode, OverlayArt, PlatformEmulatorSetting, ResolvedTheme, ScreenMode,
    SingleRowStyle, SurfaceMaterial, ThemeIssue, ThemeSelection, UserSettings,
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

    /// Sets whether battery percentage is shown in the status pill.
    pub fn set_show_battery_percentage(&self, on: bool) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.show_battery_percentage = on)?;
        Ok(())
    }

    /// Sets discrete paging mode for the grid.
    pub fn set_discrete_paging(&self, on: bool) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.discrete_paging = on)?;
        Ok(())
    }

    /// Sets UI animation speed.
    pub fn set_animation_speed(&self, speed: AnimationSpeed) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.animation_speed = speed.into())?;
        Ok(())
    }

    /// Sets dimming factor for now playing.
    pub fn set_now_playing_dimming(&self, value: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.now_playing_dimming = value)?;
        Ok(())
    }

    /// Sets saturation factor for now playing.
    pub fn set_now_playing_saturation(&self, value: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.now_playing_saturation = value)?;
        Ok(())
    }

    /// Sets the app icon pack id (`None` for default).
    pub fn set_app_icon_pack(&self, pack_id: Option<String>) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.app_icon_pack = pack_id)?;
        Ok(())
    }

    /// Enables or disables RetroAchievements integration.
    pub fn set_retroachievements_enabled(&self, on: bool) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.retroachievements_enabled = on)?;
        Ok(())
    }

    /// Enables or disables Discord Rich Presence integration.
    pub fn set_discord_presence_enabled(&self, on: bool) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.discord_presence_enabled = on)?;
        Ok(())
    }

    /// Sets dock icon shape.
    pub fn set_dock_icon_shape(&self, shape: DockIconShape) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.dock_icon_shape = shape.into())?;
        Ok(())
    }

    /// Sets hide dock background.
    pub fn set_hide_dock_background(&self, on: bool) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.hide_dock_background = on)?;
        Ok(())
    }

    /// Sets icon roundness (0.0 to 0.5).
    pub fn set_icon_roundness(&self, value: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.icon_roundness = value)?;
        Ok(())
    }

    /// Sets icon scale (0.5 to 1.0).
    pub fn set_icon_scale(&self, value: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.icon_scale = value)?;
        Ok(())
    }

    /// Sets hover scale.
    pub fn set_hover_scale(&self, value: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.hover_scale = value)?;
        Ok(())
    }

    /// Sets hero display style.
    pub fn set_hero_display_style(&self, style: HeroDisplayStyle) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.hero_display_style = style.into())?;
        Ok(())
    }

    /// Sets hero logo scale.
    pub fn set_hero_logo_scale(&self, value: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.hero_logo_scale = value)?;
        Ok(())
    }

    /// Sets hero folder logo scale.
    pub fn set_hero_folder_logo_scale(&self, value: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.hero_folder_logo_scale = value)?;
        Ok(())
    }

    /// Sets hero gradient style.
    pub fn set_hero_gradient_style(&self, style: HeroGradientStyle) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.hero_gradient_style = style.into())?;
        Ok(())
    }

    /// Sets hero vignette size.
    pub fn set_hero_vignette_size(&self, value: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.hero_vignette_size = value)?;
        Ok(())
    }

    /// Sets glass blur (0 to 10).
    pub fn set_glass_blur(&self, value: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.glass_blur = value)?;
        Ok(())
    }

    /// Sets glass refraction (0 to 1).
    pub fn set_glass_refraction(&self, value: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.glass_refraction = value)?;
        Ok(())
    }

    /// Sets glass highlight (0 to 1).
    pub fn set_glass_highlight(&self, value: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.glass_highlight = value)?;
        Ok(())
    }

    /// Sets glass rim strength (0 to 1).
    pub fn set_glass_rim_strength(&self, value: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.glass_rim_strength = value)?;
        Ok(())
    }

    /// Sets glass tint (0 to 1).
    pub fn set_glass_tint(&self, value: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.glass_tint = value)?;
        Ok(())
    }

    /// Sets glass on tiles.
    pub fn set_glass_on_tiles(&self, on: bool) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.glass_on_tiles = on)?;
        Ok(())
    }

    /// Sets jingle volume (0 to 1).
    pub fn set_jingle_volume(&self, value: f32) -> Res<()> {
        self.service()
            .update_user_settings(|s| s.jingle_volume = value)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that the newly created core exports the default active theme.
    #[test]
    fn core_active_theme_defaults() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let core = Core::new(dir.path().to_string_lossy().into_owned())?;
        let theme = core.active_theme()?;
        assert_eq!(theme.settings.screen_mode, ScreenMode::Dual);
        assert_eq!(theme.settings.single_screen_rows, 3);
        assert!(theme.settings.discrete_paging);
        Ok(())
    }

    /// Tests setting screen mode and row parameters.
    #[test]
    fn core_set_screen_mode_and_rows() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let core = Core::new(dir.path().to_string_lossy().into_owned())?;
        core.set_screen_mode(ScreenMode::Single)?;
        core.set_single_screen_rows(4)?;
        let theme = core.active_theme()?;
        assert_eq!(theme.settings.screen_mode, ScreenMode::Single);
        assert_eq!(theme.settings.single_screen_rows, 4);
        Ok(())
    }

    /// Tests modifying glass effect parameters.
    #[test]
    fn core_set_glass_parameters() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let core = Core::new(dir.path().to_string_lossy().into_owned())?;
        core.set_glass_blur(7.5)?;
        core.set_glass_refraction(0.8)?;
        core.set_glass_highlight(0.3)?;
        let theme = core.active_theme()?;
        assert!((theme.settings.glass_blur - 7.5).abs() < 1e-4);
        assert!((theme.settings.glass_refraction - 0.8).abs() < 1e-4);
        assert!((theme.settings.glass_highlight - 0.3).abs() < 1e-4);
        Ok(())
    }

    /// Tests toggling dock position, paging and animation speed.
    #[test]
    fn core_set_dock_and_paging() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let core = Core::new(dir.path().to_string_lossy().into_owned())?;
        core.set_dock_on_top(true)?;
        core.set_discrete_paging(false)?;
        core.set_animation_speed(AnimationSpeed::Fast)?;
        let theme = core.active_theme()?;
        assert!(theme.settings.dock_on_top);
        assert!(!theme.settings.discrete_paging);
        assert_eq!(theme.settings.animation_speed, AnimationSpeed::Fast);
        Ok(())
    }

    /// Tests setting and clearing custom accent colour override.
    #[test]
    fn core_set_accent_override() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let core = Core::new(dir.path().to_string_lossy().into_owned())?;
        core.set_accent_override(Some(0xFFFF_5376))?;
        let theme = core.active_theme()?;
        assert_eq!(theme.settings.accent_override, Some(0xFFFF_5376));
        core.set_accent_override(None)?;
        let cleared = core.active_theme()?;
        assert_eq!(cleared.settings.accent_override, None);
        Ok(())
    }
}
