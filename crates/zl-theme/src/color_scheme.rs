//! The `color_scheme` object of a theme (spec 3.3).

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::colour::Rgba;
use crate::enums::StringEnum;
use crate::issue::IssueCode;
use crate::reader::{join, Reader};

string_enum! {
    /// A colour the theme format defines.
    ColorKey {
        /// Start of the background gradient.
        BackgroundGradientStart => "background_gradient_start",
        /// End of the background gradient.
        BackgroundGradientEnd => "background_gradient_end",
        /// Start of the card gradient.
        CardGradientStart => "card_gradient_start",
        /// End of the card gradient.
        CardGradientEnd => "card_gradient_end",
        /// Main text.
        TextPrimary => "text_primary",
        /// Secondary text.
        TextSecondary => "text_secondary",
        /// Tint of monochrome icons.
        IconTint => "icon_tint",
        /// Background of a tile.
        TileBackground => "tile_background",
        /// Border of a tile.
        TileBorder => "tile_border",
        /// Start of the gradient of a switched-off toggle.
        ToggleOffGradientStart => "toggle_off_gradient_start",
        /// End of the gradient of a switched-off toggle.
        ToggleOffGradientEnd => "toggle_off_gradient_end",
        /// Start of the gradient of a toggle thumb.
        ToggleThumbGradientStart => "toggle_thumb_gradient_start",
        /// End of the gradient of a toggle thumb.
        ToggleThumbGradientEnd => "toggle_thumb_gradient_end",
        /// Outer shadow.
        DropShadow => "drop_shadow",
        /// Light inner shadow.
        InnerShadowLight => "inner_shadow_light",
        /// Dark inner shadow.
        InnerShadowDark => "inner_shadow_dark",
        /// Success colour.
        Success => "success",
        /// Warning colour.
        Warning => "warning",
        /// Divider lines.
        Divider => "divider",
        /// Start of the accent gradient.
        AccentGradientStart => "accent_gradient_start",
        /// End of the accent gradient.
        AccentGradientEnd => "accent_gradient_end",
        /// Glow around accent elements.
        AccentGlow => "accent_glow",
        /// Start of the selection gradient (falls back to the accent when absent).
        SelectionGradientStart => "selection_gradient_start",
        /// End of the selection gradient (falls back to the accent when absent).
        SelectionGradientEnd => "selection_gradient_end",
    }
}

/// The colours of a theme: the known ones parsed, the unknown ones kept as found.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ColorScheme {
    /// Known colours that were present and readable.
    pub colors: BTreeMap<ColorKey, Rgba>,
    /// Keys the format does not define, with their raw JSON value, preserved for round trips.
    pub unknown: BTreeMap<String, Value>,
}

impl ColorScheme {
    /// Returns the colour for `key`, if the theme sets it.
    pub fn get(&self, key: ColorKey) -> Option<Rgba> {
        self.colors.get(&key).copied()
    }
}

/// Reads the `color_scheme` value found at `path`; a non-object is a `WrongType` warning.
pub(crate) fn parse_color_scheme(
    value: &Value,
    path: &str,
    reader: &mut Reader,
) -> Option<ColorScheme> {
    let Some(obj) = value.as_object() else {
        reader.warn(IssueCode::WrongType, path.to_string());
        return None;
    };
    Some(read_entries(obj, path, reader))
}

/// Sorts each entry of the object into known colours and unknown keys.
fn read_entries(obj: &Map<String, Value>, path: &str, reader: &mut Reader) -> ColorScheme {
    let mut scheme = ColorScheme::default();
    for (key, value) in obj {
        match ColorKey::parse_text(key).filter(|k| k.as_str() == key) {
            Some(known) => {
                if let Some(colour) = reader.colour(value, join(path, key)) {
                    scheme.colors.insert(known, colour);
                }
            }
            None => {
                reader.warn(IssueCode::UnknownKey, join(path, key));
                scheme.unknown.insert(key.clone(), value.clone());
            }
        }
    }
    scheme
}
