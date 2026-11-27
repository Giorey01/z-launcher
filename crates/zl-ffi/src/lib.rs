//! The foreign interface of the Z-Launcher core: what the Android app (through UniFFI) can call.
//!
//! A thin adapter: the logic is in `zl-core` (and the crates below it); this crate only turns its
//! types into UniFFI records and enums and exposes the [`Core`] object. Errors are codes.
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

#[macro_use]
mod mirror;
mod appearance;
mod appearance_api;
mod core_object;
mod error;
mod hash_api;
mod meta_api;
mod pods_api;
mod scan;
mod secrets_api;
mod sessions_api;
mod steam_api;
mod transfer_api;
mod types;

uniffi::setup_scaffolding!();

pub use appearance::*;
pub use core_object::Core;
pub use error::CoreError;
pub use hash_api::{HashEvent, HashListener, HashReport, HashRun, HashStatus, RomHash};
pub use meta_api::{GameDetails, ImageScrapeReport, KeyCheck, ScrapeListener, ScrapeStop};
pub use pods_api::{IndexEntryView, IndexStoreView, InstalledPodView, PodPreview};
pub use scan::{ScanEvent, ScanIssue, ScanListener, ScanSummary};
pub use secrets_api::SecretStore;
pub use sessions_api::{PendingSession, PlayStats, SessionEnd, UsageEvent, UsageKind};
pub use steam_api::{SteamAccount, SteamImport, SteamProblem};
pub use transfer_api::{
    download_file, extract_archive, http_get, http_post, TransferError, TransferListener,
};
pub use types::{
    Candidate, ClipData, Detection, Extra, ExtraValue, FolderKind, GameEntry, GameKind,
    InstalledEmulator, LaunchIntent, LibraryMode, Plan, PlatformEntry, ProfileRef, ProposedDefault,
    RawLaunchRequest, ResolvedBy, RomRoot, UriGrant,
};

/// The version of the core library, taken from the workspace package version.
#[uniffi::export]
pub fn core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// The name of the folder the app keeps its data in, from the shared configuration.
#[uniffi::export]
pub fn data_dir_name() -> String {
    zl_config::app::DATA_DIR_NAME.to_string()
}

/// The folder suggested as ROM root at onboarding, from the shared configuration.
#[uniffi::export]
pub fn suggested_rom_folder() -> String {
    zl_config::app::SUGGESTED_ROM_FOLDER.to_string()
}

/// Number of dock slots (the "All apps" button included), from the shared configuration.
#[uniffi::export]
pub fn dock_slots() -> u32 {
    u32::try_from(zl_config::app::DOCK_SLOTS).unwrap_or(u32::MAX)
}

/// The tracks of the theme's music to play now, in order (empty when music is off):
/// `minute_of_day` is the local time, `seed` varies a shuffled playlist between sessions.
#[uniffi::export]
pub fn music_queue(music: MusicSetup, minute_of_day: u32, seed: u64) -> Vec<String> {
    zl_core::music_queue(&music.into(), minute_of_day, seed)
}

/// The folder of the data directory the app copies its built-in music into.
#[uniffi::export]
pub fn default_music_dir_name() -> String {
    zl_config::theme::DEFAULT_MUSIC_DIR_NAME.to_string()
}

/// The default URL for the public Pod store index.
#[uniffi::export]
pub fn default_pod_index_url() -> String {
    zl_config::pods::DEFAULT_INDEX_URL.to_string()
}

/// The tree URI the system folder picker should start at for a folder path, if it is on a
/// storage volume.
#[uniffi::export]
pub fn saf_tree_for_path(path: String) -> Option<String> {
    zl_platforms::saf::tree_uri_for_path(&path)
}

/// The folder path a granted tree URI stands for, if it can be told.
#[uniffi::export]
pub fn path_of_saf_tree(tree_uri: String) -> Option<String> {
    zl_platforms::saf::tree_path(&tree_uri)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exported version is the workspace version, not a placeholder.
    #[test]
    fn version_is_the_package_version() {
        assert_eq!(core_version(), env!("CARGO_PKG_VERSION"));
        assert!(!core_version().is_empty());
    }

    /// The exported folder name is the one defined in the configuration crate.
    #[test]
    fn data_dir_name_comes_from_config() {
        assert_eq!(data_dir_name(), zl_config::app::DATA_DIR_NAME);
    }

    /// A picked tree URI maps to the folder the core scans, and back.
    #[test]
    fn saf_tree_maps_to_the_scanned_folder() {
        let tree = saf_tree_for_path("/storage/emulated/0/Games".into());
        assert_eq!(
            tree.as_deref(),
            Some("content://com.android.externalstorage.documents/tree/primary%3AGames")
        );
        assert_eq!(
            path_of_saf_tree(tree.unwrap_or_default()).as_deref(),
            Some("/storage/emulated/0/Games")
        );
    }

    /// The dock size crosses the boundary unchanged.
    #[test]
    fn dock_slots_come_from_config() {
        assert_eq!(dock_slots() as usize, zl_config::app::DOCK_SLOTS);
    }

    /// The default pod index URL comes from the configuration crate.
    #[test]
    fn default_pod_index_url_comes_from_config() {
        assert_eq!(default_pod_index_url(), zl_config::pods::DEFAULT_INDEX_URL);
    }
}
