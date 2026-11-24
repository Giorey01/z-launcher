//! Structured problems found in a theme. The core never builds sentences: apps translate the
//! [`IssueCode`] and show the location.

string_enum! {
    /// How serious a problem is.
    Severity {
        /// The theme cannot be loaded.
        Error => "error",
        /// The theme loads, something was ignored or adjusted.
        Warning => "warning",
    }
}

string_enum! {
    /// What is wrong, as a stable code.
    IssueCode {
        /// `theme.json` is not valid JSON (or not a JSON object).
        InvalidJson => "invalid_json",
        /// The required `name` is missing or empty.
        MissingName => "missing_name",
        /// A wallpaper named in `theme.json` has no file in `wallpapers/`.
        MissingWallpaperFile => "missing_wallpaper_file",
        /// A key the format does not define; kept and ignored.
        UnknownKey => "unknown_key",
        /// A colour that cannot be read; the key is ignored.
        BadColor => "bad_color",
        /// A number outside its documented range; it was limited to the range.
        ValueClamped => "value_clamped",
        /// An enum value the format does not define; the key is ignored.
        UnknownEnumValue => "unknown_enum_value",
        /// A value of the wrong JSON type; the key is ignored.
        WrongType => "wrong_type",
        /// A `music_time_schedule` entry that is not `HH:MM|file`; it is skipped.
        BadSchedule => "bad_schedule",
        /// An `asset_refs` package that is not installed.
        MissingAssetPack => "missing_asset_pack",
        /// A file in `sounds/` that does not match an action name and extension.
        UnknownSoundName => "unknown_sound_name",
    }
}

/// One problem: how bad, what, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    /// How serious it is.
    pub severity: Severity,
    /// What is wrong.
    pub code: IssueCode,
    /// A JSON path (`$.settings.icon_scale`) or a path relative to the theme folder.
    pub location: String,
}

impl Issue {
    /// Builds an issue.
    pub fn new(severity: Severity, code: IssueCode, location: impl Into<String>) -> Self {
        Issue {
            severity,
            code,
            location: location.into(),
        }
    }
}
