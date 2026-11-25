//! Database of platforms, emulators and launch profiles, and everything the launcher decides from
//! it: which extensions belong to a platform, which emulators are installed, which profile a game
//! uses and the exact Android intent to start it.
//!
//! The data is two JSON files (`data/platforms.json`, `data/emulators.json`, format in
//! `docs/spec/platforms.md`) embedded in the crate; a newer copy on disk wins when its `version`
//! is higher, so the lists can be updated without a new APK. Every function is pure: Kotlin only
//! turns a [`LaunchIntent`] into an Android `Intent`.
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

mod db;
mod detect;
mod error;
mod launch;
mod model;
mod plan;
mod resolve;
pub mod saf;
mod template;
mod validate;

pub use db::{PlatformDb, SUPPORTED_SCHEMA};
pub use detect::{Detection, InstalledEmulator};
pub use error::{DataFile, DbError, LaunchError};
pub use launch::{
    build_launch, ClipData, Extra, ExtraValue, LaunchContext, LaunchIntent, RomUris, UriGrant,
};
pub use model::{
    ClipTemplate, Core, Emulator, EmulatorKind, ExtraTemplate, ExtraTemplateValue, Folder,
    GrantTemplate, IntTemplate, IntentTemplate, Package, Platform, PrefEntry, Profile, RomAccess,
    TagFileRule,
};
pub use plan::{
    LaunchRequest, Outcome, ANDROID_APP_EMULATOR, ANDROID_APP_PROFILE, ANDROID_PACKAGE_TAG,
};
pub use resolve::{ProfileRef, Resolution, ResolveInput, ResolvedBy};
pub use validate::KNOWN_FLAGS;
