//! The model of `theme.json` and its tolerant reader (spec section 3).

use serde_json::{Map, Value};

use crate::color_scheme::{parse_color_scheme, ColorScheme};
use crate::issue::{Issue, IssueCode};
use crate::music::{parse_music, MusicConfig};
use crate::reader::{join, Reader};
use crate::settings::{parse_settings, Settings};

string_enum! {
    /// The `theme_mode` of a theme. It does not constrain the colours.
    ThemeMode {
        /// Light.
        Light => "light",
        /// Dark.
        Dark => "dark",
        /// Pure black for OLED screens.
        Oled => "oled",
        /// Follows the system.
        System => "system",
    }
}

string_enum! {
    /// The layer an `asset_refs` package belongs to.
    AssetCategory {
        /// Icon masks and overlays.
        IconOverlays => "icon_overlays",
        /// Smart folder artwork.
        SmartFolders => "smart_folders",
        /// Sound effects.
        Sounds => "sounds",
        /// Music tracks.
        Music => "music",
    }
}

/// A reference from a theme to a package of another layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetRef {
    /// Package id.
    pub id: String,
    /// Display name of the package.
    pub name: String,
    /// Layer the package provides.
    pub category: AssetCategory,
    /// Package version as text (numbers are accepted and converted).
    pub version: Option<String>,
}

/// A parsed `theme.json`. `None` fields were absent: the user's setting stays as it is.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ThemeManifest {
    /// Theme name (required; empty only if the file lacked it, which is reported as an error).
    pub name: String,
    /// Author.
    pub author: Option<String>,
    /// Version text, such as `"1.1"`.
    pub version: Option<String>,
    /// Description; may contain `\r\n`.
    pub description: Option<String>,
    /// Credits.
    pub credits: Option<String>,
    /// Website.
    pub website: Option<String>,
    /// Light/dark mode.
    pub theme_mode: Option<ThemeMode>,
    /// Colours.
    pub color_scheme: Option<ColorScheme>,
    /// `wallpaper_main`: file name (with or without extension) in `wallpapers/`.
    pub wallpaper_main: Option<String>,
    /// `wallpaper_external`: file name (with or without extension) in `wallpapers/`.
    pub wallpaper_external: Option<String>,
    /// Settings to apply.
    pub settings: Option<Settings>,
    /// Music and volumes.
    pub music: MusicConfig,
    /// Packages of other layers this theme uses.
    pub asset_refs: Option<Vec<AssetRef>>,
    /// The Z-Launcher extension object (spec section 8), untouched.
    pub zl: Option<Value>,
    /// Top-level keys the format does not define, preserved.
    pub extras: Map<String, Value>,
}

/// Result of reading a `theme.json`.
#[derive(Debug, Clone, PartialEq)]
pub struct ManifestOutcome {
    /// The manifest, or `None` when the text was not a JSON object.
    pub manifest: Option<ThemeManifest>,
    /// Everything that was wrong or adjusted.
    pub issues: Vec<Issue>,
}

/// Top-level keys the format defines.
const KNOWN_KEYS: &[&str] = &[
    "name",
    "author",
    "version",
    "description",
    "credits",
    "website",
    "theme_mode",
    "color_scheme",
    "wallpaper_main",
    "wallpaper_external",
    "settings",
    "music_mode",
    "music_playlist",
    "music_time_schedule",
    "music_playback_mode",
    "sfx_volume",
    "music_volume",
    "jingle_volume",
    "asset_refs",
    "zl",
];

/// Reads the text of a `theme.json`. A leading BOM is ignored. Invalid JSON gives
/// `manifest: None` and an `InvalidJson` error; every other problem is reported and reading goes on.
pub fn parse_manifest(text: &str) -> ManifestOutcome {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut reader = Reader::default();
    let Ok(Value::Object(obj)) = serde_json::from_str::<Value>(text) else {
        reader.error(IssueCode::InvalidJson, "$".to_string());
        return ManifestOutcome {
            manifest: None,
            issues: reader.issues,
        };
    };
    let manifest = read_object(&obj, &mut reader);
    ManifestOutcome {
        manifest: Some(manifest),
        issues: reader.issues,
    }
}

/// Reads all fields of the top-level object.
fn read_object(obj: &Map<String, Value>, reader: &mut Reader) -> ThemeManifest {
    reader.unknown_keys(obj, "$", KNOWN_KEYS);
    let name = reader
        .string(obj, "$", "name")
        .filter(|n| !n.trim().is_empty());
    if name.is_none() {
        reader.error(IssueCode::MissingName, join("$", "name"));
    }
    ThemeManifest {
        name: name.unwrap_or_default(),
        author: reader.string(obj, "$", "author"),
        version: read_version(obj, reader),
        description: reader.string(obj, "$", "description"),
        credits: reader.string(obj, "$", "credits"),
        website: reader.string(obj, "$", "website"),
        theme_mode: reader.enum_value(obj, "$", "theme_mode"),
        color_scheme: obj
            .get("color_scheme")
            .and_then(|v| parse_color_scheme(v, "$.color_scheme", reader)),
        wallpaper_main: reader.string(obj, "$", "wallpaper_main"),
        wallpaper_external: reader.string(obj, "$", "wallpaper_external"),
        settings: obj
            .get("settings")
            .and_then(|v| parse_settings(v, "$.settings", reader)),
        music: parse_music(obj, reader),
        asset_refs: obj.get("asset_refs").map(|v| parse_asset_refs(v, reader)),
        zl: obj.get("zl").cloned(),
        extras: obj
            .iter()
            .filter(|(k, _)| !KNOWN_KEYS.contains(&k.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    }
}

/// Reads `version`, which is a string in real themes but tolerated as a number.
fn read_version(obj: &Map<String, Value>, reader: &mut Reader) -> Option<String> {
    match obj.get("version")? {
        Value::Number(n) => Some(n.to_string()),
        _ => reader.string(obj, "$", "version"),
    }
}

/// Reads the `asset_refs` list, skipping entries that cannot be read.
fn parse_asset_refs(value: &Value, reader: &mut Reader) -> Vec<AssetRef> {
    let Some(items) = value.as_array() else {
        reader.warn(IssueCode::WrongType, "$.asset_refs".to_string());
        return Vec::new();
    };
    let mut refs = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let path = format!("$.asset_refs[{index}]");
        match item.as_object() {
            Some(obj) => refs.extend(parse_asset_ref(obj, &path, reader)),
            None => reader.warn(IssueCode::WrongType, path),
        }
    }
    refs
}

/// Reads one asset reference; `id` and a known `category` are required.
fn parse_asset_ref(obj: &Map<String, Value>, path: &str, reader: &mut Reader) -> Option<AssetRef> {
    reader.unknown_keys(obj, path, &["id", "name", "category", "version"]);
    let id = reader.string(obj, path, "id")?;
    let category = reader.enum_value(obj, path, "category")?;
    let version = match obj.get("version") {
        Some(Value::Number(n)) => Some(n.to_string()),
        _ => reader.string(obj, path, "version"),
    };
    Some(AssetRef {
        name: reader.string(obj, path, "name").unwrap_or_default(),
        id,
        category,
        version,
    })
}
