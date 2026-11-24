//! Defaults of the look of the launcher: the built-in theme's colours and the value of every
//! setting a user (or a theme, when the user applies its settings) can change.
//!
//! Colours are ARGB integers (`0xAARRGGBB`), the form Android's `Color` takes. Enum-like settings
//! are the canonical text of the theme format (`"ANIMATED"`), parsed by the core. Ranges are the
//! ones the theme format documents; values outside them are limited when stored.

/// Name of the built-in theme, shown when no installed theme is selected.
pub const BUILTIN_THEME_NAME: &str = "Z-Launcher";

/// Start of the background gradient of the built-in theme: a dark blue-grey.
pub const BACKGROUND_GRADIENT_START: u32 = 0xFF14171F;
/// End of the background gradient of the built-in theme.
pub const BACKGROUND_GRADIENT_END: u32 = 0xFF07080C;
/// Start of the card gradient: slightly lighter than the background so cards read as raised.
pub const CARD_GRADIENT_START: u32 = 0xFF222633;
/// End of the card gradient.
pub const CARD_GRADIENT_END: u32 = 0xFF191C26;
/// Main text colour.
pub const TEXT_PRIMARY: u32 = 0xFFF2F4FA;
/// Secondary text colour.
pub const TEXT_SECONDARY: u32 = 0xFF9AA1B5;
/// Tint of monochrome icons.
pub const ICON_TINT: u32 = 0xFFE6E9F5;
/// Background of a tile.
pub const TILE_BACKGROUND: u32 = 0xFF1C1F2B;
/// Border of a tile.
pub const TILE_BORDER: u32 = 0x33FFFFFF;
/// Start of the gradient of a switched-off toggle.
pub const TOGGLE_OFF_GRADIENT_START: u32 = 0xFF3A3F52;
/// End of the gradient of a switched-off toggle.
pub const TOGGLE_OFF_GRADIENT_END: u32 = 0xFF2B2F3F;
/// Start of the gradient of a toggle thumb.
pub const TOGGLE_THUMB_GRADIENT_START: u32 = 0xFFFFFFFF;
/// End of the gradient of a toggle thumb.
pub const TOGGLE_THUMB_GRADIENT_END: u32 = 0xFFD5D9E6;
/// Outer shadow.
pub const DROP_SHADOW: u32 = 0x80000000;
/// Light inner shadow (a highlight on the top edge of raised surfaces).
pub const INNER_SHADOW_LIGHT: u32 = 0x1FFFFFFF;
/// Dark inner shadow.
pub const INNER_SHADOW_DARK: u32 = 0x66000000;
/// Success colour.
pub const SUCCESS: u32 = 0xFF4CD08A;
/// Warning colour.
pub const WARNING: u32 = 0xFFF2B84B;
/// Divider lines.
pub const DIVIDER: u32 = 0x1FFFFFFF;
/// Start of the accent gradient of the built-in theme: a cyan.
pub const ACCENT_GRADIENT_START: u32 = 0xFF3DD6E8;
/// End of the accent gradient: a blue.
pub const ACCENT_GRADIENT_END: u32 = 0xFF4C7DF0;
/// Glow around accent elements (the accent at about 40% opacity).
pub const ACCENT_GLOW: u32 = 0x663DD6E8;

/// Alpha of the glow derived from an accent colour the user picked (0-255).
pub const ACCENT_OVERRIDE_GLOW_ALPHA: u8 = 0x66;

/// Default corner roundness of icons (range 0.0-0.5): clearly rounded, not a circle.
pub const ICON_ROUNDNESS: f32 = 0.22;
/// Range of `icon_roundness`.
pub const ICON_ROUNDNESS_RANGE: (f32, f32) = (0.0, 0.5);
/// Default scale of icons inside their cell (range 0.5-1.0).
pub const ICON_SCALE: f32 = 0.9;
/// Range of `icon_scale`.
pub const ICON_SCALE_RANGE: (f32, f32) = (0.5, 1.0);
/// Default scale of the item under the cursor. The themes seen use 0.9 and 1.0, so the range of
/// meaningful values is not documented; 1.0 means "no zoom", and the range allows a slight zoom.
pub const HOVER_SCALE: f32 = 1.0;
/// Range accepted for `hover_scale` (our own limit; the format documents none).
pub const HOVER_SCALE_RANGE: (f32, f32) = (0.5, 1.5);
/// Default way the hero image of a game is shown.
pub const HERO_DISPLAY_STYLE: &str = "VIGNETTE";
/// Default scale of the logo over the hero (the themes seen use 0.35 to 0.65).
pub const HERO_LOGO_SCALE: f32 = 0.5;
/// Default scale of the folder logo over the hero.
pub const HERO_FOLDER_LOGO_SCALE: f32 = 0.5;
/// Range accepted for the two hero logo scales (our own limit).
pub const HERO_LOGO_SCALE_RANGE: (f32, f32) = (0.1, 1.0);
/// Default gradient over the hero image.
pub const HERO_GRADIENT_STYLE: &str = "BLACK_GRADIENT";
/// Default size of the hero vignette, as a fraction of the screen.
pub const HERO_VIGNETTE_SIZE: f32 = 0.5;
/// Range accepted for `hero_vignette_size` (our own limit).
pub const HERO_VIGNETTE_SIZE_RANGE: (f32, f32) = (0.0, 1.0);
/// Default icon colouring.
pub const ICON_STYLE: &str = "COLORED";
/// Default surface material.
pub const SURFACE_MATERIAL: &str = "SOLID";
/// Default blur of glass surfaces (range 0-10).
pub const GLASS_BLUR: f32 = 4.0;
/// Range of `glass_blur`.
pub const GLASS_BLUR_RANGE: (f32, f32) = (0.0, 10.0);
/// Default glass refraction (0-1; the format documents no range).
pub const GLASS_REFRACTION: f32 = 0.5;
/// Default glass highlight (0-1).
pub const GLASS_HIGHLIGHT: f32 = 0.5;
/// Default glass rim strength (0-1).
pub const GLASS_RIM_STRENGTH: f32 = 0.5;
/// Default glass tint (0-1).
pub const GLASS_TINT: f32 = 0.2;
/// Range accepted for the glass numbers the format does not bound (our own limit).
pub const GLASS_UNIT_RANGE: (f32, f32) = (0.0, 1.0);
/// Default for glass on tiles.
pub const GLASS_ON_TILES: bool = false;
/// Default look of the selection cursor.
pub const CURSOR_STYLE: &str = "ANIMATED";
/// Default thickness of the selection highlight (range 0.5-1.0).
pub const HIGHLIGHT_THICKNESS: f32 = 0.75;
/// Range of `highlight_thickness`.
pub const HIGHLIGHT_THICKNESS_RANGE: (f32, f32) = (0.5, 1.0);
/// Default shape of the dock icons.
pub const DOCK_ICON_SHAPE: &str = "CIRCLE";
/// Default for hiding the dock background.
pub const HIDE_DOCK_BACKGROUND: bool = false;
/// Default for the dock at the top of the screen (the layout the launcher is modelled on).
pub const DOCK_ON_TOP: bool = true;
/// Default for always showing the corner hints (controller button glyphs).
pub const CORNER_HINTS: bool = false;
/// Default for opening folders full screen.
pub const FULLSCREEN_FOLDERS: bool = false;

/// Default for interface sound effects.
pub const SFX_ENABLED: bool = true;
/// Default volume of the interface sound effects (0-1).
pub const SFX_VOLUME: f32 = 0.8;
/// Default volume of the background music (0-1).
pub const MUSIC_VOLUME: f32 = 0.5;
/// Default volume of the jingles (0-1).
pub const JINGLE_VOLUME: f32 = 0.8;
/// Default music mode: off, because the built-in theme has no music.
pub const MUSIC_MODE: &str = "DISABLED";
/// Default playback order of a music playlist.
pub const MUSIC_PLAYBACK_MODE: &str = "IN ORDER";
/// Range of the three volumes.
pub const VOLUME_RANGE: (f32, f32) = (0.0, 1.0);

/// Default screen mode: `DUAL` (grid on one display, hero on the other). The app decides at
/// first start whether the device has two displays and stores `SINGLE` when it has not; the core
/// only keeps the choice.
pub const SCREEN_MODE: &str = "DUAL";
/// Default for swapping which display shows the grid and which the hero.
pub const REVERSE_DISPLAY_ROLES: bool = false;
/// Default for docking the Home to an external display: follow the connection.
pub const EXTERNAL_DISPLAY_DOCKING: &str = "AUTOMATIC";
/// Default fraction of the single screen given to the hero area.
pub const HERO_SPLIT: f32 = 0.4;
/// Range of `hero_split`.
pub const HERO_SPLIT_RANGE: (f32, f32) = (0.0, 1.0);
/// Default number of grid rows on a single screen.
pub const SINGLE_SCREEN_ROWS: u8 = 3;
/// Range of `single_screen_rows`.
pub const SINGLE_SCREEN_ROWS_RANGE: (u8, u8) = (1, 5);
/// Default style of the rows on a single screen.
pub const SINGLE_ROW_STYLE: &str = "STANDARD";
