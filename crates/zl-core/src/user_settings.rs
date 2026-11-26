//! The user's look-and-sound settings, kept one key per value in the database settings table
//! (`ui.<name>`), with the defaults of `zl_config::appearance` for what was never written.

use zl_config::appearance as cfg;
use zl_db::Db;
use zl_theme::StringEnum;

use crate::appearance_types::{
    AnimationSpeed, ExternalDisplayDocking, LaunchScreen, ScreenMode, SingleRowStyle, UserSettings,
};
use crate::error::CoreError;
use crate::Service;

/// Limits `value` to `range`; a NaN becomes the lower bound.
fn clamp(value: f32, range: (f32, f32)) -> f32 {
    if value.is_nan() {
        range.0
    } else {
        value.clamp(range.0, range.1)
    }
}

impl UserSettings {
    /// The same settings with every number limited to its range (a stored or applied value
    /// outside it is a bug of the writer, not a reason to draw a broken screen).
    pub fn clamped(mut self) -> Self {
        self.icon_roundness = clamp(self.icon_roundness, cfg::ICON_ROUNDNESS_RANGE);
        self.icon_scale = clamp(self.icon_scale, cfg::ICON_SCALE_RANGE);
        self.hover_scale = clamp(self.hover_scale, cfg::HOVER_SCALE_RANGE);
        self.hero_logo_scale = clamp(self.hero_logo_scale, cfg::HERO_LOGO_SCALE_RANGE);
        self.hero_folder_logo_scale =
            clamp(self.hero_folder_logo_scale, cfg::HERO_LOGO_SCALE_RANGE);
        self.hero_vignette_size = clamp(self.hero_vignette_size, cfg::HERO_VIGNETTE_SIZE_RANGE);
        self.glass_blur = clamp(self.glass_blur, cfg::GLASS_BLUR_RANGE);
        self.glass_refraction = clamp(self.glass_refraction, cfg::GLASS_UNIT_RANGE);
        self.glass_highlight = clamp(self.glass_highlight, cfg::GLASS_UNIT_RANGE);
        self.glass_rim_strength = clamp(self.glass_rim_strength, cfg::GLASS_UNIT_RANGE);
        self.glass_tint = clamp(self.glass_tint, cfg::GLASS_UNIT_RANGE);
        self.highlight_thickness = clamp(self.highlight_thickness, cfg::HIGHLIGHT_THICKNESS_RANGE);
        self.hero_split = clamp(self.hero_split, cfg::HERO_SPLIT_RANGE);
        let rows = cfg::SINGLE_SCREEN_ROWS_RANGE;
        self.single_screen_rows = self.single_screen_rows.clamp(rows.0, rows.1);
        self.sfx_volume = clamp(self.sfx_volume, cfg::VOLUME_RANGE);
        self.music_volume = clamp(self.music_volume, cfg::VOLUME_RANGE);
        self.jingle_volume = clamp(self.jingle_volume, cfg::VOLUME_RANGE);
        self.now_playing_dimming = clamp(self.now_playing_dimming, cfg::NOW_PLAYING_RANGE);
        self.secondary_brightness =
            clamp(self.secondary_brightness, cfg::SECONDARY_BRIGHTNESS_RANGE);
        self.now_playing_saturation = clamp(self.now_playing_saturation, cfg::NOW_PLAYING_RANGE);
        let idle = cfg::NOW_PLAYING_IDLE_RANGE;
        self.now_playing_idle_seconds = self.now_playing_idle_seconds.clamp(idle.0, idle.1);
        self
    }
}

/// The database key of a setting.
fn key(name: &str) -> String {
    format!("ui.{name}")
}

/// Reads typed values with a default.
struct Reader<'a> {
    /// The database.
    db: &'a Db,
}

impl Reader<'_> {
    /// The stored text of a setting.
    fn text(&self, name: &str) -> Result<Option<String>, CoreError> {
        Ok(self.db.setting(&key(name))?)
    }

    /// A number; an unreadable one is the default.
    fn num(&self, name: &str, default: f32) -> Result<f32, CoreError> {
        Ok(self
            .text(name)?
            .and_then(|t| t.parse().ok())
            .unwrap_or(default))
    }

    /// A boolean stored as `1` or `0`.
    fn flag(&self, name: &str, default: bool) -> Result<bool, CoreError> {
        Ok(self.text(name)?.map_or(default, |t| t == "1"))
    }

    /// A settings enum of this crate, read with its own parser.
    fn text_choice<T>(
        &self,
        name: &str,
        default: T,
        parse: fn(&str) -> Option<T>,
    ) -> Result<T, CoreError> {
        Ok(self.text(name)?.and_then(|t| parse(&t)).unwrap_or(default))
    }

    /// An enum stored as its canonical text.
    fn choice<T: StringEnum>(&self, name: &str, default: T) -> Result<T, CoreError> {
        Ok(self
            .text(name)?
            .and_then(|t| T::parse_text(&t))
            .unwrap_or(default))
    }
}

/// Visual settings as `(name, text)` pairs.
fn look_pairs(s: &UserSettings) -> Vec<(&'static str, String)> {
    let flag = |b: bool| if b { "1" } else { "0" }.to_string();
    vec![
        ("icon_roundness", s.icon_roundness.to_string()),
        ("icon_scale", s.icon_scale.to_string()),
        ("hover_scale", s.hover_scale.to_string()),
        ("hero_display_style", s.hero_display_style.as_str().into()),
        ("hero_logo_scale", s.hero_logo_scale.to_string()),
        (
            "hero_folder_logo_scale",
            s.hero_folder_logo_scale.to_string(),
        ),
        ("hero_gradient_style", s.hero_gradient_style.as_str().into()),
        ("hero_vignette_size", s.hero_vignette_size.to_string()),
        ("icon_style", s.icon_style.as_str().into()),
        ("surface_material", s.surface_material.as_str().into()),
        ("glass_blur", s.glass_blur.to_string()),
        ("glass_refraction", s.glass_refraction.to_string()),
        ("glass_highlight", s.glass_highlight.to_string()),
        ("glass_rim_strength", s.glass_rim_strength.to_string()),
        ("glass_tint", s.glass_tint.to_string()),
        ("glass_on_tiles", flag(s.glass_on_tiles)),
        ("cursor_style", s.cursor_style.as_str().into()),
        ("highlight_thickness", s.highlight_thickness.to_string()),
        ("dock_icon_shape", s.dock_icon_shape.as_str().into()),
        ("hide_dock_background", flag(s.hide_dock_background)),
        ("dock_on_top", flag(s.dock_on_top)),
        ("corner_hints", flag(s.corner_hints)),
        ("fullscreen_folders", flag(s.fullscreen_folders)),
    ]
}

/// System, sound, and integrations settings as `(name, text)` pairs.
fn system_pairs(s: &UserSettings) -> Vec<(&'static str, String)> {
    let flag = |b: bool| if b { "1" } else { "0" }.to_string();
    vec![
        (
            "accent_override",
            s.accent_override
                .map_or_else(String::new, |c| c.to_string()),
        ),
        ("screen_mode", s.screen_mode.as_str().into()),
        ("reverse_display_roles", flag(s.reverse_display_roles)),
        (
            "external_display_docking",
            s.external_display_docking.as_str().into(),
        ),
        ("hero_split", s.hero_split.to_string()),
        ("single_screen_rows", s.single_screen_rows.to_string()),
        ("single_row_style", s.single_row_style.as_str().into()),
        ("sfx_enabled", flag(s.sfx_enabled)),
        ("sfx_volume", s.sfx_volume.to_string()),
        ("music_mode", s.music_mode.as_str().into()),
        ("music_volume", s.music_volume.to_string()),
        ("jingle_volume", s.jingle_volume.to_string()),
        ("show_battery_percentage", flag(s.show_battery_percentage)),
        ("discrete_paging", flag(s.discrete_paging)),
        ("animation_speed", s.animation_speed.as_str().into()),
        ("now_playing_dimming", s.now_playing_dimming.to_string()),
        ("secondary_brightness", s.secondary_brightness.to_string()),
        (
            "now_playing_saturation",
            s.now_playing_saturation.to_string(),
        ),
        (
            "now_playing_idle_seconds",
            s.now_playing_idle_seconds.to_string(),
        ),
        ("launch_screen", s.launch_screen.as_str().into()),
        ("app_icon_pack", s.app_icon_pack.clone().unwrap_or_default()),
        (
            "retroachievements_enabled",
            flag(s.retroachievements_enabled),
        ),
        ("discord_presence_enabled", flag(s.discord_presence_enabled)),
    ]
}

/// The settings as `(name, text)` pairs ready to store.
fn pairs(s: &UserSettings) -> Vec<(&'static str, String)> {
    let mut out = look_pairs(s);
    out.extend(system_pairs(s));
    out
}

/// Reads the look settings (everything up to the dock) over the defaults.
fn read_look(r: &Reader<'_>, d: UserSettings) -> Result<UserSettings, CoreError> {
    Ok(UserSettings {
        icon_roundness: r.num("icon_roundness", d.icon_roundness)?,
        icon_scale: r.num("icon_scale", d.icon_scale)?,
        hover_scale: r.num("hover_scale", d.hover_scale)?,
        hero_display_style: r.choice("hero_display_style", d.hero_display_style)?,
        hero_logo_scale: r.num("hero_logo_scale", d.hero_logo_scale)?,
        hero_folder_logo_scale: r.num("hero_folder_logo_scale", d.hero_folder_logo_scale)?,
        hero_gradient_style: r.choice("hero_gradient_style", d.hero_gradient_style)?,
        hero_vignette_size: r.num("hero_vignette_size", d.hero_vignette_size)?,
        icon_style: r.choice("icon_style", d.icon_style)?,
        surface_material: r.choice("surface_material", d.surface_material)?,
        glass_blur: r.num("glass_blur", d.glass_blur)?,
        glass_refraction: r.num("glass_refraction", d.glass_refraction)?,
        glass_highlight: r.num("glass_highlight", d.glass_highlight)?,
        glass_rim_strength: r.num("glass_rim_strength", d.glass_rim_strength)?,
        glass_tint: r.num("glass_tint", d.glass_tint)?,
        glass_on_tiles: r.flag("glass_on_tiles", d.glass_on_tiles)?,
        cursor_style: r.choice("cursor_style", d.cursor_style)?,
        highlight_thickness: r.num("highlight_thickness", d.highlight_thickness)?,
        ..d
    })
}

/// Reads the dock, accent and sound settings over `base`.
fn read_rest(r: &Reader<'_>, base: UserSettings) -> Result<UserSettings, CoreError> {
    Ok(UserSettings {
        dock_icon_shape: r.choice("dock_icon_shape", base.dock_icon_shape)?,
        hide_dock_background: r.flag("hide_dock_background", base.hide_dock_background)?,
        dock_on_top: r.flag("dock_on_top", base.dock_on_top)?,
        corner_hints: r.flag("corner_hints", base.corner_hints)?,
        fullscreen_folders: r.flag("fullscreen_folders", base.fullscreen_folders)?,
        accent_override: r.text("accent_override")?.and_then(|t| t.parse().ok()),
        screen_mode: r.text_choice("screen_mode", base.screen_mode, ScreenMode::parse_text)?,
        reverse_display_roles: r.flag("reverse_display_roles", base.reverse_display_roles)?,
        external_display_docking: r.text_choice(
            "external_display_docking",
            base.external_display_docking,
            ExternalDisplayDocking::parse_text,
        )?,
        hero_split: r.num("hero_split", base.hero_split)?,
        single_screen_rows: r
            .text("single_screen_rows")?
            .and_then(|t| t.parse().ok())
            .unwrap_or(base.single_screen_rows),
        single_row_style: r.text_choice(
            "single_row_style",
            base.single_row_style,
            SingleRowStyle::parse_text,
        )?,
        sfx_enabled: r.flag("sfx_enabled", base.sfx_enabled)?,
        sfx_volume: r.num("sfx_volume", base.sfx_volume)?,
        music_mode: r.choice("music_mode", base.music_mode)?,
        music_volume: r.num("music_volume", base.music_volume)?,
        jingle_volume: r.num("jingle_volume", base.jingle_volume)?,
        show_battery_percentage: r.flag("show_battery_percentage", base.show_battery_percentage)?,
        discrete_paging: r.flag("discrete_paging", base.discrete_paging)?,
        animation_speed: r.text_choice(
            "animation_speed",
            base.animation_speed,
            AnimationSpeed::parse_text,
        )?,
        app_icon_pack: r.text("app_icon_pack")?.filter(|t| !t.is_empty()),
        retroachievements_enabled: r
            .flag("retroachievements_enabled", base.retroachievements_enabled)?,
        discord_presence_enabled: r
            .flag("discord_presence_enabled", base.discord_presence_enabled)?,
        ..base
    })
}

/// Reads the now playing and launch settings over `base`.
fn read_play(r: &Reader<'_>, base: UserSettings) -> Result<UserSettings, CoreError> {
    Ok(UserSettings {
        now_playing_dimming: r.num("now_playing_dimming", base.now_playing_dimming)?,
        secondary_brightness: r.num("secondary_brightness", base.secondary_brightness)?,
        now_playing_saturation: r.num("now_playing_saturation", base.now_playing_saturation)?,
        now_playing_idle_seconds: r
            .text("now_playing_idle_seconds")?
            .and_then(|t| t.parse().ok())
            .unwrap_or(base.now_playing_idle_seconds),
        launch_screen: r.text_choice(
            "launch_screen",
            base.launch_screen,
            LaunchScreen::parse_text,
        )?,
        ..base
    })
}

impl Service {
    /// The user's settings: what was stored, the default for the rest, numbers limited to
    /// their ranges.
    pub fn user_settings(&self) -> Result<UserSettings, CoreError> {
        let db = self.db();
        let r = Reader { db: &db };
        let look = read_look(&r, UserSettings::default())?;
        let rest = read_rest(&r, look)?;
        Ok(read_play(&r, rest)?.clamped())
    }

    /// Stores all the settings (numbers limited to their ranges first).
    pub fn set_user_settings(&self, settings: &UserSettings) -> Result<(), CoreError> {
        let db = self.db();
        let rows = pairs(&settings.clone().clamped());
        db.transaction::<(), CoreError>(|db| {
            for (name, value) in rows {
                db.set_setting(&key(name), &value)?;
            }
            Ok(())
        })
    }

    /// Reads the settings, lets `change` edit them and stores the result. The convenience
    /// behind the single-value setters of the FFI.
    pub fn update_user_settings(
        &self,
        change: impl FnOnce(&mut UserSettings),
    ) -> Result<UserSettings, CoreError> {
        let mut settings = self.user_settings()?;
        change(&mut settings);
        self.set_user_settings(&settings)?;
        self.user_settings()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every default text of `zl_config::appearance` is a value the format knows, so none of the
    /// fallbacks in `Default` is used.
    #[test]
    fn default_texts_parse() {
        use zl_theme::{
            CursorStyle, DockIconShape, HeroDisplayStyle, HeroGradientStyle, IconStyle, MusicMode,
            PlaybackMode, SurfaceMaterial,
        };
        assert!(HeroDisplayStyle::parse_text(cfg::HERO_DISPLAY_STYLE).is_some());
        assert!(HeroGradientStyle::parse_text(cfg::HERO_GRADIENT_STYLE).is_some());
        assert!(IconStyle::parse_text(cfg::ICON_STYLE).is_some());
        assert!(SurfaceMaterial::parse_text(cfg::SURFACE_MATERIAL).is_some());
        assert!(CursorStyle::parse_text(cfg::CURSOR_STYLE).is_some());
        assert!(DockIconShape::parse_text(cfg::DOCK_ICON_SHAPE).is_some());
        assert!(MusicMode::parse_text(cfg::MUSIC_MODE).is_some());
        assert!(PlaybackMode::parse_text(cfg::MUSIC_PLAYBACK_MODE).is_some());
        assert!(ScreenMode::parse_text(cfg::SCREEN_MODE).is_some());
        assert!(ExternalDisplayDocking::parse_text(cfg::EXTERNAL_DISPLAY_DOCKING).is_some());
        assert!(SingleRowStyle::parse_text(cfg::SINGLE_ROW_STYLE).is_some());
        assert!(AnimationSpeed::parse_text(cfg::ANIMATION_SPEED).is_some());
    }

    /// Defaults sit inside their own ranges, so clamping them changes nothing.
    #[test]
    fn defaults_are_inside_their_ranges() {
        let d = UserSettings::default();
        assert_eq!(d.clone().clamped(), d);
    }

    /// Out-of-range numbers are pulled in and NaN becomes the lower bound.
    #[test]
    fn clamping() {
        let s = UserSettings {
            icon_scale: 7.0,
            sfx_volume: -1.0,
            glass_blur: f32::NAN,
            ..UserSettings::default()
        }
        .clamped();
        assert_eq!(s.icon_scale, 1.0);
        assert_eq!(s.sfx_volume, 0.0);
        assert_eq!(s.glass_blur, 0.0);
        let rows = UserSettings {
            single_screen_rows: 9,
            hero_split: 3.0,
            ..UserSettings::default()
        }
        .clamped();
        assert_eq!((rows.single_screen_rows, rows.hero_split), (5, 1.0));
    }

    /// User settings roundtrip through a real database.
    #[test]
    fn settings_database_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let svc = Service::open(dir.path())?;
        let initial = svc.user_settings()?;
        assert_eq!(initial.screen_mode, ScreenMode::Dual);
        assert!(initial.discrete_paging);

        let modified = UserSettings {
            screen_mode: ScreenMode::Single,
            discrete_paging: false,
            single_screen_rows: 4,
            animation_speed: AnimationSpeed::Fast,
            retroachievements_enabled: true,
            discord_presence_enabled: true,
            app_icon_pack: Some("org.example.pack".to_string()),
            now_playing_dimming: 0.75,
            now_playing_saturation: 0.25,
            secondary_brightness: 0.2,
            ..initial
        };
        svc.set_user_settings(&modified)?;

        let loaded = svc.user_settings()?;
        assert_eq!(loaded.screen_mode, ScreenMode::Single);
        assert!(!loaded.discrete_paging);
        assert_eq!(loaded.single_screen_rows, 4);
        assert_eq!(loaded.animation_speed, AnimationSpeed::Fast);
        assert!(loaded.retroachievements_enabled);
        assert!(loaded.discord_presence_enabled);
        assert_eq!(loaded.app_icon_pack.as_deref(), Some("org.example.pack"));
        assert!((loaded.now_playing_dimming - 0.75).abs() < 1e-4);
        assert!((loaded.now_playing_saturation - 0.25).abs() < 1e-4);
        assert!((loaded.secondary_brightness - 0.2).abs() < 1e-4);
        Ok(())
    }

    /// Games start on the hero display by default; the choice and the idle delay are stored,
    /// and the delay is held to its range.
    #[test]
    fn launch_screen_and_idle_delay_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let svc = Service::open(dir.path())?;
        let initial = svc.user_settings()?;
        assert_eq!(initial.launch_screen, LaunchScreen::Hero);
        assert_eq!(
            initial.now_playing_idle_seconds,
            cfg::NOW_PLAYING_IDLE_SECONDS
        );

        svc.set_user_settings(&UserSettings {
            launch_screen: LaunchScreen::Grid,
            now_playing_idle_seconds: 30,
            ..initial.clone()
        })?;
        let loaded = svc.user_settings()?;
        assert_eq!(loaded.launch_screen, LaunchScreen::Grid);
        assert_eq!(loaded.now_playing_idle_seconds, 30);

        svc.set_user_settings(&UserSettings {
            now_playing_idle_seconds: 1,
            ..loaded
        })?;
        let low = svc.user_settings()?.now_playing_idle_seconds;
        assert_eq!(low, cfg::NOW_PLAYING_IDLE_RANGE.0);
        Ok(())
    }

    /// update_user_settings modifies settings through a closure and persists.
    #[test]
    fn update_user_settings_helper() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let svc = Service::open(dir.path())?;
        let updated = svc.update_user_settings(|s| {
            s.glass_blur = 8.0;
            s.dock_on_top = true;
            s.music_volume = 0.65;
        })?;
        assert_eq!(updated.glass_blur, 8.0);
        assert!(updated.dock_on_top);
        assert!((updated.music_volume - 0.65).abs() < 1e-4);

        let reloaded = svc.user_settings()?;
        assert_eq!(reloaded.glass_blur, 8.0);
        assert!(reloaded.dock_on_top);
        assert!((reloaded.music_volume - 0.65).abs() < 1e-4);
        Ok(())
    }

    /// Empty app icon pack strings are parsed as None.
    #[test]
    fn empty_icon_pack_is_none() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let svc = Service::open(dir.path())?;
        svc.update_user_settings(|s| {
            s.app_icon_pack = Some(String::new());
        })?;
        let loaded = svc.user_settings()?;
        assert_eq!(loaded.app_icon_pack, None);
        Ok(())
    }
}
