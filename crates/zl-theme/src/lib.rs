//! Reader and validator for themes in the Cocoon format (see `docs/spec/cocoon-theme.md`).
//!
//! The crate turns a `theme.json` into a typed [`ThemeManifest`], reads an unpacked theme folder
//! into a [`ThemePackage`], imports ZIP files safely, validates everything into a list of
//! structured [`Issue`]s (codes, never sentences) and resolves which layer serves each asset.
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

#[macro_use]
mod enums;
mod color_scheme;
mod colour;
mod error;
mod fsutil;
mod issue;
mod layers;
mod manifest;
mod music;
mod package;
mod reader;
mod settings;
mod sounds;
mod validate;
mod wallpaper;
mod zip_import;

pub use color_scheme::{ColorKey, ColorScheme};
pub use colour::{ColourError, Rgba};
pub use enums::StringEnum;
pub use error::ThemeError;
pub use issue::{Issue, IssueCode, Severity};
pub use layers::{resolve_slot, slots_of_package, LayerSource, Resolution, SourceLevel};
pub use manifest::{
    parse_manifest, AssetCategory, AssetRef, ManifestOutcome, ThemeManifest, ThemeMode,
};
pub use music::{MusicConfig, MusicMode, PlaybackMode, ScheduleEntry};
pub use package::{find_themes, read_package, FolderArt, OverlayArt, ThemePackage};
pub use settings::{
    CursorStyle, DockIconShape, HeroDisplayStyle, HeroGradientStyle, IconStyle, Settings,
    SurfaceMaterial,
};
pub use sounds::{SoundAction, SOUND_EXTENSIONS};
pub use validate::{validate_package, validate_text};
pub use zip_import::{import_zip, import_zip_with_limits, ZipLimits};
