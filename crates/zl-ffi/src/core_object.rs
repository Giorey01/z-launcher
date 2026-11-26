//! The [`Core`] object Kotlin holds: one per app process, constructed with the data folder.

use std::path::Path;
use std::sync::Arc;

use zl_core::Service;

use crate::error::CoreError;
use crate::scan::{Bridge, ScanListener, ScanSummary};
use crate::types::{
    Detection, GameEntry, LibraryMode, Plan, PlatformEntry, PlatformFolder, ProfileRef,
    RawLaunchRequest, RomRoot,
};

/// Result of a core call.
type Res<T> = Result<T, CoreError>;

/// The library, the emulator data and the choices of the Home.
#[derive(uniffi::Object)]
pub struct Core {
    /// The service every call goes to.
    svc: Service,
}

impl Core {
    /// The service behind the object (for the other halves of the interface).
    pub(crate) fn service(&self) -> &Service {
        &self.svc
    }
}

#[uniffi::export]
impl Core {
    /// Opens the core with its data (the library database, updated platform lists) in `data_dir`.
    #[uniffi::constructor]
    pub fn new(data_dir: String) -> Res<Arc<Core>> {
        let svc = Service::open(Path::new(&data_dir))?;
        Ok(Arc::new(Core { svc }))
    }

    /// Adds a ROM folder, with the SAF tree URI the user granted for it, or updates that tree.
    pub fn add_root(&self, path: String, saf_tree: Option<String>) -> Res<RomRoot> {
        Ok(self.svc.add_root(&path, saf_tree.as_deref())?.into())
    }

    /// The ROM folders added so far.
    pub fn roots(&self) -> Res<Vec<RomRoot>> {
        Ok(self.svc.roots()?.into_iter().map(Into::into).collect())
    }

    /// Includes or excludes a folder from scans.
    pub fn set_root_enabled(&self, root_id: i64, enabled: bool) -> Res<()> {
        Ok(self.svc.set_root_enabled(root_id, enabled)?)
    }

    /// The folder of `platform` (id or alias) in the library, `None` when there is none: the
    /// folder of a ROM root that holds its games, else one named after it.
    pub fn platform_folder(&self, platform: String) -> Res<Option<PlatformFolder>> {
        Ok(self.svc.platform_root(&platform)?.map(Into::into))
    }

    /// Scans the enabled folders; events go to `listener` on the calling thread.
    pub fn scan(&self, listener: Arc<dyn ScanListener>) -> Res<ScanSummary> {
        Ok(self.svc.scan(&Bridge(listener))?.into())
    }

    /// Platforms with games, with their counts.
    pub fn platforms(&self) -> Res<Vec<PlatformEntry>> {
        Ok(self.svc.platforms()?.into_iter().map(Into::into).collect())
    }

    /// Games of one platform, or of all platforms with `None`.
    pub fn games(&self, platform: Option<String>) -> Res<Vec<GameEntry>> {
        let games = self.svc.games(platform.as_deref())?;
        Ok(games.into_iter().map(Into::into).collect())
    }

    /// Searches games matching `query` using typo-tolerant fuzzy matching (RF-16).
    pub fn search_games(&self, query: String, limit: u32) -> Res<Vec<GameEntry>> {
        let games = self.svc.search_games(&query, limit as usize)?;
        Ok(games.into_iter().map(Into::into).collect())
    }

    /// Consoles an Android app can be filed under, those with games first.
    pub fn app_consoles(&self) -> Res<Vec<PlatformEntry>> {
        Ok(self
            .svc
            .app_consoles()?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    /// Files the Android app `package` under the console `platform` as `title` (a
    /// `.localgameid` file in the first enabled folder), rescans, and returns the game id.
    pub fn file_app(
        &self,
        package: String,
        title: String,
        platform: String,
        listener: Arc<dyn ScanListener>,
    ) -> Res<i64> {
        Ok(self
            .svc
            .file_app(&package, &title, &platform, &Bridge(listener))?)
    }

    /// Takes a filed Android app out of its console (deletes its tag file) and rescans.
    pub fn unfile_app(&self, game_id: i64, listener: Arc<dyn ScanListener>) -> Res<()> {
        Ok(self.svc.unfile_app(game_id, &Bridge(listener))?)
    }

    /// Emulators among the installed `packages` and the proposed defaults for the library.
    pub fn detect_installed(&self, packages: Vec<String>) -> Res<Detection> {
        Ok(self.svc.detect(&packages)?.into())
    }

    /// Saves the proposed defaults that are not set yet; returns how many.
    pub fn accept_proposed_defaults(&self, packages: Vec<String>) -> Res<u32> {
        Ok(self.svc.accept_proposed_defaults(&packages)?)
    }

    /// The default emulator profile of a platform.
    pub fn platform_default(&self, platform: String) -> Res<Option<ProfileRef>> {
        Ok(self.svc.platform_default(&platform)?.map(Into::into))
    }

    /// Sets or clears the default emulator profile of a platform.
    pub fn set_platform_default(&self, platform: String, profile: Option<ProfileRef>) -> Res<()> {
        let profile = profile.map(Into::into);
        Ok(self.svc.set_platform_default(&platform, profile.as_ref())?)
    }

    /// The emulator profile set on one game.
    pub fn game_emulator(&self, game_id: i64) -> Res<Option<ProfileRef>> {
        Ok(self.svc.game_emulator(game_id)?.map(Into::into))
    }

    /// Sets or clears the emulator profile of one game.
    pub fn set_game_emulator(&self, game_id: i64, profile: Option<ProfileRef>) -> Res<()> {
        let profile = profile.map(Into::into);
        Ok(self.svc.set_game_emulator(game_id, profile.as_ref())?)
    }

    /// Decides how to launch a stored game, given the installed launcher packages.
    pub fn plan_launch(&self, game_id: i64, installed_packages: Vec<String>) -> Res<Plan> {
        Ok(self.svc.plan_launch(game_id, &installed_packages)?.into())
    }

    /// Decides how to launch an explicit request (no stored game).
    pub fn plan_request(
        &self,
        installed_packages: Vec<String>,
        request: RawLaunchRequest,
    ) -> Res<Plan> {
        Ok(self
            .svc
            .plan_request(&installed_packages, &request.into())?
            .into())
    }

    /// How the Home shows the library.
    pub fn library_mode(&self) -> Res<LibraryMode> {
        Ok(self.svc.library_mode()?.into())
    }

    /// Stores the library mode.
    pub fn set_library_mode(&self, mode: LibraryMode) -> Res<()> {
        Ok(self.svc.set_library_mode(mode.into())?)
    }

    /// Whether the onboarding is finished.
    pub fn onboarding_done(&self) -> Res<bool> {
        Ok(self.svc.onboarding_done()?)
    }

    /// Records that the onboarding is finished.
    pub fn set_onboarding_done(&self, done: bool) -> Res<()> {
        Ok(self.svc.set_onboarding_done(done)?)
    }

    /// Packages pinned to the dock.
    pub fn dock(&self) -> Res<Vec<String>> {
        Ok(self.svc.dock()?)
    }

    /// Pins or unpins a package; returns the new dock.
    pub fn toggle_dock(&self, package: String) -> Res<Vec<String>> {
        Ok(self.svc.toggle_dock(&package)?)
    }
}
