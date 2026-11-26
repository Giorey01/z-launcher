//! The application service of Z-Launcher: everything the apps do with the library, in one place
//! and without any Android type. It joins the database (`zl-db`), the scanner (`zl-library`) and
//! the platform and emulator data (`zl-platforms`): roots (with their SAF tree), scanning,
//! platforms and games to show, the emulator assignment, launch planning for a stored game, and
//! the small choices of the Home (library mode, onboarding, dock).
//!
//! `zl-ffi` only turns these types into UniFFI records; Kotlin keeps the screens, the `Intent`
//! and the permissions. Errors are codes ([`CoreError`]), never sentences.
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

mod app_games;
mod appearance_types;
mod default_sounds;
mod detect;
mod error;
mod launch;
mod library;
mod media;
mod meta;
mod platform_root;
mod platform_settings;
mod pods;
mod prefs;
mod scrape;
pub mod search;
mod secrets;
mod sessions;
mod smart_folders;
mod steam_details;
mod steam_library;
mod theme_files;
mod theme_resolve;
mod theme_select;
mod theme_store;
mod types;
mod user_settings;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use zl_config::app::DATABASE_FILE_NAME;
use zl_db::Db;
use zl_platforms::PlatformDb;

pub use appearance_types::{
    argb, AnimationSpeed, CocoonDataDir, ExternalDisplayDocking, FolderArt, FolderSlot,
    ImportReport, InstalledPack, InstalledTheme, LaunchScreen, LayerChoice, MusicSetup, OverlayArt,
    PackRef, ResolvedTheme, ScheduleTrack, ScreenMode, SingleRowStyle, SoundEntry, ThemeColors,
    ThemeIssue, ThemeSelection, UserSettings, Wallpaper, WallpaperKind,
};
pub use error::CoreError;
pub use media::{GameMedia, MediaCount, MediaImportReport, MediaItem, MediaSources};
pub use platform_root::PlatformRoot;
pub use pods::{IndexEntryView, IndexStoreView, InstalledPodView, PodPreview};
pub use scrape::{ImageScrapeReport, KeyCheck, ScrapeProgress, ScrapeStop, ScrapeTarget};
pub use search::FuzzySearch;
pub use secrets::{MemorySecrets, SecretStore};
pub use sessions::{PendingSession, SessionEnd};
pub use steam_library::{SteamAccount, SteamImport, SteamProblem};
pub use types::{
    Candidate, Detection, FolderKind, GameDetails, GameEntry, InstalledView, LibraryMode, Plan,
    PlatformEmulatorSetting, PlatformEntry, ProposedDefault, RootInfo, ScraperConfig,
};

/// The service: an open library database and the platform data, shared by every call.
pub struct Service {
    /// Where the database file lives (a scan opens its own connection to it).
    db_path: PathBuf,
    /// The connection for reads and small writes.
    db: Mutex<Db>,
    /// Platforms, emulators and profiles (embedded copy, or a newer one found in the data folder).
    platforms: PlatformDb,
    /// The app data folder: database, `themes/`, `assets/` and the user's own art.
    data_dir: PathBuf,
    /// The files of the selected theme, loaded on first use and dropped when the selection or
    /// the installed themes change.
    theme_cache: Mutex<Option<Arc<theme_files::ThemeFiles>>>,
    /// Where the API keys are kept (in memory until the app sets its own store).
    secrets: Mutex<Arc<dyn secrets::SecretStore>>,
}

impl Service {
    /// Opens the service with its data in `data_dir` (created if needed). A newer platform or
    /// emulator list in that folder wins over the embedded one when it is valid; a broken one
    /// is ignored and the embedded copy is used.
    pub fn open(data_dir: &Path) -> Result<Service, CoreError> {
        std::fs::create_dir_all(data_dir).map_err(|_| CoreError::Database)?;
        let db_path = data_dir.join(DATABASE_FILE_NAME);
        let db = Db::open(&db_path)?;
        let platforms = match PlatformDb::load_with_updates(data_dir) {
            Ok(updated) => updated,
            Err(_) => PlatformDb::embedded()?,
        };
        Ok(Service {
            db_path,
            db: Mutex::new(db),
            platforms,
            data_dir: data_dir.to_path_buf(),
            theme_cache: Mutex::new(None),
            secrets: Mutex::new(Arc::new(secrets::MemorySecrets::default())),
        })
    }

    /// Locks the shared connection (a panic in another call does not make the data unusable).
    fn db(&self) -> MutexGuard<'_, Db> {
        self.db.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The platform and emulator data in use.
    pub fn platform_db(&self) -> &PlatformDb {
        &self.platforms
    }
}
