//! The organisation half of the [`Core`] object: manual folders, sort orders and the Apps and
//! Emulators Smart Folders.

use zl_core as core;

use crate::core_object::Core;
use crate::error::CoreError;
use crate::types::PlatformEntry;

/// Result of a core call.
type Res<T> = Result<T, CoreError>;

mirror_enum! {
    /// How a list of games is ordered.
    SortOrder <=> core::SortOrder { Name, LastPlayed, PlayTime, Added, Rating, Manual }
}
mirror_record! {
    /// A manual folder with the games it holds.
    CollectionInfo <=> core::CollectionInfo {
        /// Row id.
        plain id: i64,
        /// Folder id as the Home uses it (`@folder:<id>`).
        plain folder_id: String,
        /// The name the user gave.
        plain name: String,
        /// Games in it that are not missing or hidden.
        plain games: u32,
    }
}
mirror_record! {
    /// An installed launcher app, for the Apps and Emulators folders.
    InstalledApp <=> core::InstalledApp {
        /// Android package name.
        plain package: String,
        /// The app declares itself a game.
        plain is_game: bool,
    }
}

/// Converts the apps the app reports.
fn apps_of(apps: Vec<InstalledApp>) -> Vec<core::InstalledApp> {
    apps.into_iter().map(Into::into).collect()
}

#[uniffi::export]
impl Core {
    /// The Home entries of Apps and Emulators among the installed `apps` (only those with
    /// something in them).
    pub fn app_folder_entries(&self, apps: Vec<InstalledApp>) -> Res<Vec<PlatformEntry>> {
        let entries = self.service().app_folder_entries(&apps_of(apps))?;
        Ok(entries.into_iter().map(Into::into).collect())
    }

    /// The packages of the Apps or Emulators folder (`folder` is `@apps` or `@emulators`).
    pub fn app_folder_packages(&self, folder: String, apps: Vec<InstalledApp>) -> Res<Vec<String>> {
        Ok(self
            .service()
            .app_folder_packages(&folder, &apps_of(apps))?)
    }

    /// The manual folders in creation order.
    pub fn collections(&self) -> Res<Vec<CollectionInfo>> {
        Ok(self
            .service()
            .collections()?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    /// Makes a manual folder.
    pub fn create_collection(&self, name: String) -> Res<CollectionInfo> {
        Ok(self.service().create_collection(&name)?.into())
    }

    /// Renames a manual folder.
    pub fn rename_collection(&self, id: i64, name: String) -> Res<()> {
        Ok(self.service().rename_collection(id, &name)?)
    }

    /// Deletes a manual folder; its games stay in the library.
    pub fn delete_collection(&self, id: i64) -> Res<()> {
        Ok(self.service().delete_collection(id)?)
    }

    /// The ids of the manual folders that hold a game.
    pub fn game_collections(&self, game_id: i64) -> Res<Vec<i64>> {
        Ok(self.service().game_collections(game_id)?)
    }

    /// Puts games in a manual folder.
    pub fn add_to_collection(&self, id: i64, game_ids: Vec<i64>) -> Res<()> {
        Ok(self.service().add_to_collection(id, &game_ids)?)
    }

    /// Takes games out of a manual folder.
    pub fn remove_from_collection(&self, id: i64, game_ids: Vec<i64>) -> Res<()> {
        Ok(self.service().remove_from_collection(id, &game_ids)?)
    }

    /// Moves games from one manual folder to another.
    pub fn move_between_collections(&self, from: i64, to: i64, game_ids: Vec<i64>) -> Res<()> {
        Ok(self
            .service()
            .move_between_collections(from, to, &game_ids)?)
    }

    /// Moves a game to place `to` (from 0) in a manual folder.
    pub fn move_in_collection(&self, id: i64, game_id: i64, to: u32) -> Res<()> {
        Ok(self
            .service()
            .move_in_collection(id, game_id, to as usize)?)
    }

    /// The order in force for a folder (`None`: the list of every game).
    pub fn sort_order(&self, folder: Option<String>) -> Res<SortOrder> {
        Ok(self.service().sort_order(folder.as_deref())?.into())
    }

    /// The order the user chose for a folder (`None`: for all folders), if they chose one.
    pub fn sort_override(&self, folder: Option<String>) -> Res<Option<SortOrder>> {
        Ok(self
            .service()
            .sort_override(folder.as_deref())?
            .map(Into::into))
    }

    /// Sets the order of a folder (`None`: of all folders), or clears the choice with `None`.
    pub fn set_sort_order(&self, folder: Option<String>, order: Option<SortOrder>) -> Res<()> {
        Ok(self
            .service()
            .set_sort_order(folder.as_deref(), order.map(Into::into))?)
    }
}
