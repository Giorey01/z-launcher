//! The `settings` object of a theme (spec 3.5): typed, with ranges enforced.

use serde_json::{Map, Value};

use crate::issue::IssueCode;
use crate::reader::Reader;

string_enum! {
    /// How the hero image of a game is shown.
    HeroDisplayStyle {
        /// Faded into the background at the edges.
        Vignette => "VIGNETTE",
        /// Over the whole screen.
        Fullscreen => "FULLSCREEN",
        /// Not shown.
        None => "NONE",
    }
}

string_enum! {
    /// Gradient laid over the hero image.
    HeroGradientStyle {
        /// No gradient.
        None => "NONE",
        /// A black gradient.
        BlackGradient => "BLACK_GRADIENT",
    }
}

string_enum! {
    /// How icons are coloured.
    IconStyle {
        /// Original colours.
        Colored => "COLORED",
        /// Tinted with the accent.
        Accent => "ACCENT",
        /// Single flat colour.
        Flat => "FLAT",
    }
}

string_enum! {
    /// Material of the surfaces.
    SurfaceMaterial {
        /// Opaque.
        Solid => "SOLID",
        /// Frosted glass.
        Glass => "GLASS",
    }
}

string_enum! {
    /// Look of the selection cursor.
    CursorStyle {
        /// Animated.
        Animated => "ANIMATED",
        /// Outline only.
        Outlined => "OUTLINED",
        /// Alternative animated.
        AlternativeAnimated => "ALTERNATIVE_ANIMATED",
        /// Alternative.
        Alternative => "ALTERNATIVE",
    }
}

string_enum! {
    /// Shape of the dock icons.
    DockIconShape {
        /// Circle.
        Circle => "CIRCLE",
        /// Squircle.
        Squircle => "SQUIRCLE",
        /// Square with rounded corners.
        RoundedSquare => "ROUNDED_SQUARE",
        /// Square.
        Square => "SQUARE",
    }
}

/// Every key `settings` may contain.
const KNOWN_KEYS: &[&str] = &[
    "icon_roundness",
    "icon_scale",
    "hover_scale",
    "hero_display_style",
    "hero_logo_scale",
    "hero_folder_logo_scale",
    "hero_gradient_style",
    "hero_vignette_size",
    "icon_style",
    "surface_material",
    "glass_blur",
    "glass_refraction",
    "glass_highlight",
    "glass_rim_strength",
    "glass_tint",
    "glass_on_tiles",
    "cursor_style",
    "highlight_thickness",
    "dock_icon_shape",
    "hide_dock_background",
    "dock_on_top",
    "corner_hints",
];

/// Documented range of `icon_roundness`.
const ICON_ROUNDNESS: (f64, f64) = (0.0, 0.5);
/// Documented range of `icon_scale`.
const ICON_SCALE: (f64, f64) = (0.5, 1.0);
/// Documented range of `glass_blur`.
const GLASS_BLUR: (f64, f64) = (0.0, 10.0);
/// Documented range of `highlight_thickness`.
const HIGHLIGHT_THICKNESS: (f64, f64) = (0.5, 1.0);

/// Settings a theme can apply. `None` means "leave the user's setting unchanged".
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Settings {
    /// Corner roundness of icons, 0.0-0.5.
    pub icon_roundness: Option<f64>,
    /// Scale of icons, 0.5-1.0.
    pub icon_scale: Option<f64>,
    /// Scale of the item under the cursor (range not documented).
    pub hover_scale: Option<f64>,
    /// Hero image display.
    pub hero_display_style: Option<HeroDisplayStyle>,
    /// Scale of the logo over the hero (range not documented).
    pub hero_logo_scale: Option<f64>,
    /// Scale of the folder logo over the hero (range not documented).
    pub hero_folder_logo_scale: Option<f64>,
    /// Gradient over the hero image.
    pub hero_gradient_style: Option<HeroGradientStyle>,
    /// Size of the hero vignette (range not documented).
    pub hero_vignette_size: Option<f64>,
    /// Icon colouring.
    pub icon_style: Option<IconStyle>,
    /// Surface material.
    pub surface_material: Option<SurfaceMaterial>,
    /// Blur of glass surfaces, 0-10.
    pub glass_blur: Option<f64>,
    /// Glass refraction (range not documented).
    pub glass_refraction: Option<f64>,
    /// Glass highlight (range not documented).
    pub glass_highlight: Option<f64>,
    /// Glass rim strength (range not documented).
    pub glass_rim_strength: Option<f64>,
    /// Glass tint (range not documented).
    pub glass_tint: Option<f64>,
    /// Whether tiles are glass too.
    pub glass_on_tiles: Option<bool>,
    /// Cursor look.
    pub cursor_style: Option<CursorStyle>,
    /// Thickness of the selection highlight, 0.5-1.0.
    pub highlight_thickness: Option<f64>,
    /// Shape of the dock icons.
    pub dock_icon_shape: Option<DockIconShape>,
    /// Hide the dock background.
    pub hide_dock_background: Option<bool>,
    /// Put the dock at the top.
    pub dock_on_top: Option<bool>,
    /// Show the corner hints.
    pub corner_hints: Option<bool>,
    /// Keys the format does not define, kept as found.
    pub extras: Map<String, Value>,
}

/// Reads the `settings` value found at `path`; a non-object is a `WrongType` warning.
pub(crate) fn parse_settings(value: &Value, path: &str, reader: &mut Reader) -> Option<Settings> {
    let Some(obj) = value.as_object() else {
        reader.warn(IssueCode::WrongType, path.to_string());
        return None;
    };
    reader.unknown_keys(obj, path, KNOWN_KEYS);
    let mut s = Settings::default();
    read_numbers(obj, path, reader, &mut s);
    read_enums(obj, path, reader, &mut s);
    read_bools(obj, path, reader, &mut s);
    s.extras = obj
        .iter()
        .filter(|(k, _)| !KNOWN_KEYS.contains(&k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    Some(s)
}

/// Reads the numeric settings.
fn read_numbers(o: &Map<String, Value>, p: &str, r: &mut Reader, s: &mut Settings) {
    s.icon_roundness = r.number(o, p, "icon_roundness", Some(ICON_ROUNDNESS));
    s.icon_scale = r.number(o, p, "icon_scale", Some(ICON_SCALE));
    s.hover_scale = r.number(o, p, "hover_scale", None);
    s.hero_logo_scale = r.number(o, p, "hero_logo_scale", None);
    s.hero_folder_logo_scale = r.number(o, p, "hero_folder_logo_scale", None);
    s.hero_vignette_size = r.number(o, p, "hero_vignette_size", None);
    s.glass_blur = r.number(o, p, "glass_blur", Some(GLASS_BLUR));
    s.glass_refraction = r.number(o, p, "glass_refraction", None);
    s.glass_highlight = r.number(o, p, "glass_highlight", None);
    s.glass_rim_strength = r.number(o, p, "glass_rim_strength", None);
    s.glass_tint = r.number(o, p, "glass_tint", None);
    s.highlight_thickness = r.number(o, p, "highlight_thickness", Some(HIGHLIGHT_THICKNESS));
}

/// Reads the enum settings.
fn read_enums(o: &Map<String, Value>, p: &str, r: &mut Reader, s: &mut Settings) {
    s.hero_display_style = r.enum_value(o, p, "hero_display_style");
    s.hero_gradient_style = r.enum_value(o, p, "hero_gradient_style");
    s.icon_style = r.enum_value(o, p, "icon_style");
    s.surface_material = r.enum_value(o, p, "surface_material");
    s.cursor_style = r.enum_value(o, p, "cursor_style");
    s.dock_icon_shape = r.enum_value(o, p, "dock_icon_shape");
}

/// Reads the boolean settings.
fn read_bools(o: &Map<String, Value>, p: &str, r: &mut Reader, s: &mut Settings) {
    s.glass_on_tiles = r.boolean(o, p, "glass_on_tiles");
    s.hide_dock_background = r.boolean(o, p, "hide_dock_background");
    s.dock_on_top = r.boolean(o, p, "dock_on_top");
    s.corner_hints = r.boolean(o, p, "corner_hints");
}
