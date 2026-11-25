//! Plain data of the look of the launcher: the resolved theme, the user's settings, the lists of
//! installed themes and packs, and the reports of imports.

use zl_config::appearance as defaults;
use zl_theme::{
    AssetCategory, ColorKey, ColorScheme, CursorStyle, DockIconShape, HeroDisplayStyle,
    HeroGradientStyle, IconStyle, IssueCode, MusicMode, PlaybackMode, Rgba, Severity, SoundAction,
    StringEnum, SurfaceMaterial, ThemeMode,
};

/// Packs an 8-bit colour into the `0xAARRGGBB` integer Android's `Color` uses.
pub fn argb(c: Rgba) -> u32 {
    (u32::from(c.a) << 24) | (u32::from(c.r) << 16) | (u32::from(c.g) << 8) | u32::from(c.b)
}

/// Declares [`ThemeColors`]: one `u32` per colour of the format, its default in
/// `zl_config::appearance`, and the code that reads it from a `color_scheme`.
macro_rules! theme_colors {
    ($($field:ident => $key:ident, $default:ident;)+) => {
        /// Every colour of a theme as `0xAARRGGBB`. A colour the theme does not set has the
        /// default of the built-in theme, except the selection gradient, which follows the
        /// accent gradient (spec section 3.3).
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct ThemeColors {
            $(
                #[doc = concat!("The `", stringify!($field), "` colour.")]
                pub $field: u32,
            )+
        }

        impl ThemeColors {
            /// The colours of the built-in theme.
            pub fn defaults() -> Self {
                ThemeColors { $($field: defaults::$default,)+ }
            }

            /// The colours of `scheme`, the default for each one it lacks.
            pub(crate) fn read(scheme: Option<&ColorScheme>) -> Self {
                let get = |key: ColorKey, default: u32| {
                    scheme.and_then(|s| s.get(key)).map_or(default, argb)
                };
                ThemeColors { $($field: get(ColorKey::$key, defaults::$default),)+ }
            }
        }
    };
}

theme_colors! {
    background_gradient_start => BackgroundGradientStart, BACKGROUND_GRADIENT_START;
    background_gradient_end => BackgroundGradientEnd, BACKGROUND_GRADIENT_END;
    card_gradient_start => CardGradientStart, CARD_GRADIENT_START;
    card_gradient_end => CardGradientEnd, CARD_GRADIENT_END;
    text_primary => TextPrimary, TEXT_PRIMARY;
    text_secondary => TextSecondary, TEXT_SECONDARY;
    icon_tint => IconTint, ICON_TINT;
    tile_background => TileBackground, TILE_BACKGROUND;
    tile_border => TileBorder, TILE_BORDER;
    toggle_off_gradient_start => ToggleOffGradientStart, TOGGLE_OFF_GRADIENT_START;
    toggle_off_gradient_end => ToggleOffGradientEnd, TOGGLE_OFF_GRADIENT_END;
    toggle_thumb_gradient_start => ToggleThumbGradientStart, TOGGLE_THUMB_GRADIENT_START;
    toggle_thumb_gradient_end => ToggleThumbGradientEnd, TOGGLE_THUMB_GRADIENT_END;
    drop_shadow => DropShadow, DROP_SHADOW;
    inner_shadow_light => InnerShadowLight, INNER_SHADOW_LIGHT;
    inner_shadow_dark => InnerShadowDark, INNER_SHADOW_DARK;
    success => Success, SUCCESS;
    warning => Warning, WARNING;
    divider => Divider, DIVIDER;
    accent_gradient_start => AccentGradientStart, ACCENT_GRADIENT_START;
    accent_gradient_end => AccentGradientEnd, ACCENT_GRADIENT_END;
    accent_glow => AccentGlow, ACCENT_GLOW;
    selection_gradient_start => SelectionGradientStart, ACCENT_GRADIENT_START;
    selection_gradient_end => SelectionGradientEnd, ACCENT_GRADIENT_END;
}

/// Declares a settings enum spelled as upper-case text in the database.
macro_rules! text_enum {
    ($(#[$m:meta])* $name:ident { $($(#[$vm:meta])* $v:ident => $t:literal),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $($(#[$vm])* $v),+
        }
        impl $name {
            /// The text stored in the database.
            pub fn as_str(self) -> &'static str {
                match self { $($name::$v => $t),+ }
            }
            /// Reads the stored text back (ignoring case).
            pub fn parse_text(text: &str) -> Option<Self> {
                $(if text.trim().eq_ignore_ascii_case($t) { return Some($name::$v); })+
                None
            }
        }
    };
}

text_enum! {
    /// One display or two.
    ScreenMode {
        /// Grid on one display, hero on the other.
        Dual => "DUAL",
        /// Everything on one display, hero above the grid.
        Single => "SINGLE",
    }
}

text_enum! {
    /// Whether the Home moves to an external display when one is connected.
    ExternalDisplayDocking {
        /// Follow the connection.
        Automatic => "AUTOMATIC",
        /// Always use the external display when present.
        AlwaysDock => "ALWAYS_DOCK",
        /// Never use it.
        NeverDock => "NEVER_DOCK",
    }
}

text_enum! {
    /// Look of the rows of the single-screen grid.
    SingleRowStyle {
        /// Plain rows.
        Standard => "STANDARD",
        /// Rows that scroll as a carousel.
        Carousel => "CAROUSEL",
        /// Rows drawn as cover flow.
        CoverFlow => "COVER_FLOW",
    }
}

text_enum! {
    /// Which display a game starts on in the dual-screen mode.
    LaunchScreen {
        /// The display that shows the hero (the big screen of a dual-screen handheld).
        Hero => "HERO",
        /// The display that shows the grid.
        Grid => "GRID",
    }
}

text_enum! {
    /// Animation speed of the launcher UI.
    AnimationSpeed {
        /// Normal animation speed.
        Normal => "NORMAL",
        /// Fast animation speed.
        Fast => "FAST",
        /// Animations turned off.
        Off => "OFF",
    }
}

/// The settings of the look and the sound that the user can change, with defaults applied. A
/// theme's own values reach this record only when the user applies the theme's settings.
#[derive(Debug, Clone, PartialEq)]
pub struct UserSettings {
    /// Corner roundness of icons, 0.0-0.5.
    pub icon_roundness: f32,
    /// Scale of icons in their cell, 0.5-1.0.
    pub icon_scale: f32,
    /// Scale of the item under the cursor.
    pub hover_scale: f32,
    /// How the hero image is shown.
    pub hero_display_style: HeroDisplayStyle,
    /// Scale of the game logo over the hero.
    pub hero_logo_scale: f32,
    /// Scale of the folder logo over the hero.
    pub hero_folder_logo_scale: f32,
    /// Gradient over the hero image.
    pub hero_gradient_style: HeroGradientStyle,
    /// Size of the hero vignette.
    pub hero_vignette_size: f32,
    /// How icons are coloured.
    pub icon_style: IconStyle,
    /// Material of the surfaces.
    pub surface_material: SurfaceMaterial,
    /// Blur of glass surfaces, 0-10.
    pub glass_blur: f32,
    /// Glass refraction, 0-1.
    pub glass_refraction: f32,
    /// Glass highlight, 0-1.
    pub glass_highlight: f32,
    /// Glass rim strength, 0-1.
    pub glass_rim_strength: f32,
    /// Glass tint, 0-1.
    pub glass_tint: f32,
    /// Whether tiles are glass too.
    pub glass_on_tiles: bool,
    /// Look of the selection cursor.
    pub cursor_style: CursorStyle,
    /// Thickness of the selection highlight, 0.5-1.0.
    pub highlight_thickness: f32,
    /// Shape of the dock icons.
    pub dock_icon_shape: DockIconShape,
    /// Hide the dock background.
    pub hide_dock_background: bool,
    /// Dock at the top of the screen.
    pub dock_on_top: bool,
    /// Always show the corner hints.
    pub corner_hints: bool,
    /// Open folders full screen.
    pub fullscreen_folders: bool,
    /// Accent colour picked by the user (`0xAARRGGBB`); replaces the theme's accent and
    /// selection colours when set.
    pub accent_override: Option<u32>,
    /// One display or two.
    pub screen_mode: ScreenMode,
    /// Swap which display shows the grid and which the hero.
    pub reverse_display_roles: bool,
    /// Whether the Home moves to an external display.
    pub external_display_docking: ExternalDisplayDocking,
    /// Single screen: fraction (0-1) of the screen given to the hero area.
    pub hero_split: f32,
    /// Single screen: number of grid rows, 1-5.
    pub single_screen_rows: u8,
    /// Single screen: style of the rows.
    pub single_row_style: SingleRowStyle,
    /// Interface sound effects on.
    pub sfx_enabled: bool,
    /// Volume of the sound effects, 0-1.
    pub sfx_volume: f32,
    /// Background music mode.
    pub music_mode: MusicMode,
    /// Volume of the music, 0-1.
    pub music_volume: f32,
    /// Volume of the jingles, 0-1.
    pub jingle_volume: f32,
    /// Whether to show the battery percentage in the status pill.
    pub show_battery_percentage: bool,
    /// Show the home grid as discrete swipeable pages.
    pub discrete_paging: bool,
    /// Animation speed of the UI.
    pub animation_speed: AnimationSpeed,
    /// Dimming factor of the now playing screen, 0-1.
    pub now_playing_dimming: f32,
    /// Saturation factor of the now playing screen, 0-1.
    pub now_playing_saturation: f32,
    /// Seconds without a touch before the now playing screen dims.
    pub now_playing_idle_seconds: u16,
    /// Which display a game starts on in the dual-screen mode.
    pub launch_screen: LaunchScreen,
    /// Selected app icon pack id (`None` for Default).
    pub app_icon_pack: Option<String>,
    /// RetroAchievements account integration enabled.
    pub retroachievements_enabled: bool,
    /// Discord Rich Presence enabled.
    pub discord_presence_enabled: bool,
}

/// Parses the canonical text of a default (they are constants, checked by a test).
fn text_default<T: StringEnum>(text: &str, fallback: T) -> T {
    T::parse_text(text).unwrap_or(fallback)
}

impl UserSettings {
    /// Fills hero, icon and glass visual defaults.
    fn with_visual_defaults(mut self) -> Self {
        self.icon_roundness = defaults::ICON_ROUNDNESS;
        self.icon_scale = defaults::ICON_SCALE;
        self.hover_scale = defaults::HOVER_SCALE;
        self.hero_display_style =
            text_default(defaults::HERO_DISPLAY_STYLE, HeroDisplayStyle::Vignette);
        self.hero_logo_scale = defaults::HERO_LOGO_SCALE;
        self.hero_folder_logo_scale = defaults::HERO_FOLDER_LOGO_SCALE;
        self.hero_gradient_style = text_default(
            defaults::HERO_GRADIENT_STYLE,
            HeroGradientStyle::BlackGradient,
        );
        self.hero_vignette_size = defaults::HERO_VIGNETTE_SIZE;
        self.icon_style = text_default(defaults::ICON_STYLE, IconStyle::Colored);
        self.surface_material = text_default(defaults::SURFACE_MATERIAL, SurfaceMaterial::Solid);
        self.glass_blur = defaults::GLASS_BLUR;
        self.glass_refraction = defaults::GLASS_REFRACTION;
        self.glass_highlight = defaults::GLASS_HIGHLIGHT;
        self.glass_rim_strength = defaults::GLASS_RIM_STRENGTH;
        self.glass_tint = defaults::GLASS_TINT;
        self.glass_on_tiles = defaults::GLASS_ON_TILES;
        self.cursor_style = text_default(defaults::CURSOR_STYLE, CursorStyle::Animated);
        self.highlight_thickness = defaults::HIGHLIGHT_THICKNESS;
        self
    }

    /// Fills dock, screen and system defaults.
    fn with_system_defaults(mut self) -> Self {
        self.dock_icon_shape = text_default(defaults::DOCK_ICON_SHAPE, DockIconShape::Circle);
        self.hide_dock_background = defaults::HIDE_DOCK_BACKGROUND;
        self.dock_on_top = defaults::DOCK_ON_TOP;
        self.corner_hints = defaults::CORNER_HINTS;
        self.fullscreen_folders = defaults::FULLSCREEN_FOLDERS;
        self.screen_mode =
            ScreenMode::parse_text(defaults::SCREEN_MODE).unwrap_or(ScreenMode::Dual);
        self.reverse_display_roles = defaults::REVERSE_DISPLAY_ROLES;
        self.external_display_docking =
            ExternalDisplayDocking::parse_text(defaults::EXTERNAL_DISPLAY_DOCKING)
                .unwrap_or(ExternalDisplayDocking::Automatic);
        self.hero_split = defaults::HERO_SPLIT;
        self.single_screen_rows = defaults::SINGLE_SCREEN_ROWS;
        self.single_row_style = SingleRowStyle::parse_text(defaults::SINGLE_ROW_STYLE)
            .unwrap_or(SingleRowStyle::Standard);
        self.show_battery_percentage = defaults::SHOW_BATTERY_PERCENTAGE;
        self.discrete_paging = defaults::DISCRETE_PAGING;
        self.animation_speed =
            AnimationSpeed::parse_text(defaults::ANIMATION_SPEED).unwrap_or(AnimationSpeed::Normal);
        self.now_playing_dimming = defaults::NOW_PLAYING_DIMMING;
        self.now_playing_saturation = defaults::NOW_PLAYING_SATURATION;
        self.now_playing_idle_seconds = defaults::NOW_PLAYING_IDLE_SECONDS;
        self.launch_screen =
            LaunchScreen::parse_text(defaults::LAUNCH_SCREEN).unwrap_or(LaunchScreen::Hero);
        self.retroachievements_enabled = defaults::RETROACHIEVEMENTS_ENABLED;
        self.discord_presence_enabled = defaults::DISCORD_PRESENCE_ENABLED;
        self
    }

    /// Fills sound and music defaults.
    fn with_audio_defaults(mut self) -> Self {
        self.sfx_enabled = defaults::SFX_ENABLED;
        self.sfx_volume = defaults::SFX_VOLUME;
        self.music_mode = text_default(defaults::MUSIC_MODE, MusicMode::Disabled);
        self.music_volume = defaults::MUSIC_VOLUME;
        self.jingle_volume = defaults::JINGLE_VOLUME;
        self
    }

    /// An empty settings struct with dummy zeros.
    fn empty() -> Self {
        UserSettings {
            icon_roundness: 0.0,
            icon_scale: 0.0,
            hover_scale: 0.0,
            hero_display_style: HeroDisplayStyle::Vignette,
            hero_logo_scale: 0.0,
            hero_folder_logo_scale: 0.0,
            hero_gradient_style: HeroGradientStyle::None,
            hero_vignette_size: 0.0,
            icon_style: IconStyle::Colored,
            surface_material: SurfaceMaterial::Solid,
            glass_blur: 0.0,
            glass_refraction: 0.0,
            glass_highlight: 0.0,
            glass_rim_strength: 0.0,
            glass_tint: 0.0,
            glass_on_tiles: false,
            cursor_style: CursorStyle::Animated,
            highlight_thickness: 0.0,
            dock_icon_shape: DockIconShape::Circle,
            hide_dock_background: false,
            dock_on_top: false,
            corner_hints: false,
            fullscreen_folders: false,
            accent_override: None,
            screen_mode: ScreenMode::Dual,
            reverse_display_roles: false,
            external_display_docking: ExternalDisplayDocking::Automatic,
            hero_split: 0.0,
            single_screen_rows: 1,
            single_row_style: SingleRowStyle::Standard,
            sfx_enabled: false,
            sfx_volume: 0.0,
            music_mode: MusicMode::Disabled,
            music_volume: 0.0,
            jingle_volume: 0.0,
            show_battery_percentage: true,
            discrete_paging: true,
            animation_speed: AnimationSpeed::Normal,
            now_playing_dimming: 0.0,
            now_playing_saturation: 0.0,
            now_playing_idle_seconds: 0,
            launch_screen: LaunchScreen::Hero,
            app_icon_pack: None,
            retroachievements_enabled: false,
            discord_presence_enabled: false,
        }
    }
}

impl Default for UserSettings {
    /// The defaults of `zl_config::appearance`.
    fn default() -> Self {
        UserSettings::empty()
            .with_visual_defaults()
            .with_system_defaults()
            .with_audio_defaults()
    }
}

/// What kind of file a wallpaper is, from its extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallpaperKind {
    /// A still image (`png`, `jpg`, `webp`, ...).
    Image,
    /// An animated GIF.
    Gif,
    /// A video (`mp4`, `webm`, ...), played in a loop without sound.
    Video,
}

/// A wallpaper file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wallpaper {
    /// Absolute path.
    pub path: String,
    /// What the file is.
    pub kind: WallpaperKind,
}

/// The file of one interface sound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundEntry {
    /// The action the sound plays for.
    pub action: SoundAction,
    /// Absolute path of the file, `None` when no layer has one (the app plays its own default).
    pub path: Option<String>,
}

/// A track of a time-of-day schedule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleTrack {
    /// Hour the track starts, 0-23.
    pub hour: u8,
    /// Minute the track starts, 0-59.
    pub minute: u8,
    /// Absolute path of the file.
    pub path: String,
}

/// Background music of the active theme, with the files found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MusicSetup {
    /// How music is chosen (the user's setting).
    pub mode: MusicMode,
    /// Order of a playlist.
    pub playback_mode: PlaybackMode,
    /// Playlist files in order; entries whose file is missing are left out.
    pub playlist: Vec<String>,
    /// Schedule in the order of the theme; entries whose file is missing are left out.
    pub schedule: Vec<ScheduleTrack>,
}

/// Everything the UI needs to draw the launcher in the active theme.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedTheme {
    /// Folder name of the theme, `None` for the built-in one.
    pub theme_id: Option<String>,
    /// Display name.
    pub name: String,
    /// The selected theme is not installed (or unreadable) and the built-in one is shown.
    pub fell_back: bool,
    /// Light, dark, OLED or follow the system.
    pub mode: ThemeMode,
    /// All colours.
    pub colors: ThemeColors,
    /// The user's settings, defaults applied.
    pub settings: UserSettings,
    /// Wallpaper of the main (small, bottom) screen; `None` means draw the background gradient.
    pub wallpaper_main: Option<Wallpaper>,
    /// Wallpaper of the external (large, top) screen.
    pub wallpaper_external: Option<Wallpaper>,
    /// One entry per sound action, in declaration order.
    pub sounds: Vec<SoundEntry>,
    /// Music.
    pub music: MusicSetup,
}

/// Images of a smart folder or platform folder (absolute paths).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FolderArt {
    /// Icon, possibly an animated GIF.
    pub icon: Option<String>,
    /// Hero image.
    pub hero: Option<String>,
    /// Logo.
    pub logo: Option<String>,
}

/// Mask and overlay of an icon overlay (absolute paths).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OverlayArt {
    /// White-on-transparent mask: the cover shows where its alpha is opaque.
    pub mask: Option<String>,
    /// Frame drawn over the masked cover.
    pub overlay: Option<String>,
}

/// Which image of a folder the user is replacing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderSlot {
    /// `icon`.
    Icon,
    /// `hero`.
    Hero,
    /// `logo`.
    Logo,
}

/// A pack of another layer, as a theme refers to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackRef {
    /// Layer the pack provides.
    pub category: AssetCategory,
    /// Pack id.
    pub id: String,
    /// Display name given in the reference.
    pub name: String,
}

/// An installed theme, for the list of themes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledTheme {
    /// Folder name.
    pub id: String,
    /// Display name (the id when `theme.json` is unreadable).
    pub name: String,
    /// Author.
    pub author: Option<String>,
    /// Version text.
    pub version: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Declared mode.
    pub mode: Option<ThemeMode>,
    /// `preview.png`, if the theme has one.
    pub preview: Option<String>,
    /// Main wallpaper file, usable as a thumbnail when there is no preview.
    pub wallpaper_main: Option<String>,
    /// The theme carries `settings` the user can apply.
    pub has_settings: bool,
    /// Packs the theme refers to that are not installed.
    pub missing_packs: Vec<PackRef>,
    /// Number of errors in the theme (it may not load when above zero).
    pub error_count: u32,
    /// Number of warnings in the theme.
    pub warning_count: u32,
    /// This is the selected theme.
    pub active: bool,
}

/// An installed asset pack, for the list of packs of a layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledPack {
    /// Folder name.
    pub id: String,
    /// Layer.
    pub category: AssetCategory,
    /// Display name.
    pub name: String,
    /// Author.
    pub author: Option<String>,
    /// Version text.
    pub version: Option<String>,
    /// `preview.png`, if the pack has one.
    pub preview: Option<String>,
    /// The pack is the one in use for its layer (chosen by the user or referenced by the theme).
    pub in_use: bool,
    /// The user chose this pack explicitly for its layer.
    pub explicit: bool,
}

/// The layer choice for one category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerChoice {
    /// The layer.
    pub category: AssetCategory,
    /// The pack the user chose, `None` to follow the theme.
    pub explicit_pack: Option<String>,
    /// The pack in use: the explicit one if installed, else the theme's, else `None`.
    pub effective_pack: Option<String>,
}

/// The selected theme and layer packs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeSelection {
    /// Selected theme, `None` for the built-in one.
    pub theme_id: Option<String>,
    /// One entry for each of the four layers.
    pub layers: Vec<LayerChoice>,
}

/// A problem found in a theme or pack: a code and where, never a sentence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeIssue {
    /// Theme or pack the problem is about.
    pub source: String,
    /// Error or warning.
    pub severity: Severity,
    /// What is wrong.
    pub code: IssueCode,
    /// JSON path or file path inside the package.
    pub location: String,
}

/// What an import did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportReport {
    /// Ids of the themes copied (new or a different version).
    pub themes_added: Vec<String>,
    /// Ids of the themes skipped because the same id and version were installed.
    pub themes_skipped: Vec<String>,
    /// Packs copied.
    pub packs_added: Vec<PackRef>,
    /// Packs skipped because the same id and version were installed.
    pub packs_skipped: Vec<PackRef>,
    /// Problems of the themes and packs read, plus copy failures (`copy_failed`).
    pub issues: Vec<ThemeIssue>,
}

/// A Cocoon data folder found on the device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CocoonDataDir {
    /// Absolute path.
    pub path: String,
    /// Themes in it.
    pub themes: u32,
    /// Asset packs in it.
    pub packs: u32,
    /// It also has `downloaded_media/`.
    pub has_media: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use zl_theme::Rgba;

    /// Tests that 8-bit rgba channels are packed into 0xAARRGGBB format.
    #[test]
    fn argb_packing() {
        let red = Rgba {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        };
        assert_eq!(argb(red), 0xFFFF_0000);
        let semi_green = Rgba {
            r: 0,
            g: 255,
            b: 0,
            a: 128,
        };
        assert_eq!(argb(semi_green), 0x8000_FF00);
    }

    /// Tests that default colors match config constants.
    #[test]
    fn theme_colors_defaults() {
        let defaults = ThemeColors::defaults();
        assert_eq!(defaults.text_primary, defaults::TEXT_PRIMARY);
        assert_eq!(defaults.text_secondary, defaults::TEXT_SECONDARY);
        assert_eq!(
            defaults.background_gradient_start,
            defaults::BACKGROUND_GRADIENT_START
        );
    }

    /// Tests text parsing and roundtrip for ScreenMode.
    #[test]
    fn screen_mode_roundtrip() {
        assert_eq!(ScreenMode::parse_text("DUAL"), Some(ScreenMode::Dual));
        assert_eq!(ScreenMode::parse_text("single"), Some(ScreenMode::Single));
        assert_eq!(ScreenMode::parse_text("  Dual  "), Some(ScreenMode::Dual));
        assert_eq!(ScreenMode::parse_text("invalid"), None);
        assert_eq!(ScreenMode::Dual.as_str(), "DUAL");
        assert_eq!(ScreenMode::Single.as_str(), "SINGLE");
    }

    /// Tests text parsing and roundtrip for ExternalDisplayDocking.
    #[test]
    fn external_display_docking_roundtrip() {
        assert_eq!(
            ExternalDisplayDocking::parse_text("AUTOMATIC"),
            Some(ExternalDisplayDocking::Automatic)
        );
        assert_eq!(
            ExternalDisplayDocking::parse_text("ALWAYS_DOCK"),
            Some(ExternalDisplayDocking::AlwaysDock)
        );
        assert_eq!(
            ExternalDisplayDocking::parse_text("NEVER_DOCK"),
            Some(ExternalDisplayDocking::NeverDock)
        );
        assert_eq!(ExternalDisplayDocking::parse_text("unknown"), None);
        assert_eq!(ExternalDisplayDocking::Automatic.as_str(), "AUTOMATIC");
    }

    /// Tests text parsing and roundtrip for SingleRowStyle.
    #[test]
    fn single_row_style_roundtrip() {
        assert_eq!(
            SingleRowStyle::parse_text("STANDARD"),
            Some(SingleRowStyle::Standard)
        );
        assert_eq!(
            SingleRowStyle::parse_text("CAROUSEL"),
            Some(SingleRowStyle::Carousel)
        );
        assert_eq!(
            SingleRowStyle::parse_text("COVER_FLOW"),
            Some(SingleRowStyle::CoverFlow)
        );
        assert_eq!(SingleRowStyle::parse_text("bad"), None);
        assert_eq!(SingleRowStyle::Standard.as_str(), "STANDARD");
    }

    /// Tests text parsing and roundtrip for AnimationSpeed.
    #[test]
    fn animation_speed_roundtrip() {
        assert_eq!(
            AnimationSpeed::parse_text("NORMAL"),
            Some(AnimationSpeed::Normal)
        );
        assert_eq!(
            AnimationSpeed::parse_text("FAST"),
            Some(AnimationSpeed::Fast)
        );
        assert_eq!(AnimationSpeed::parse_text("OFF"), Some(AnimationSpeed::Off));
        assert_eq!(AnimationSpeed::parse_text("none"), None);
        assert_eq!(AnimationSpeed::Normal.as_str(), "NORMAL");
    }
}
