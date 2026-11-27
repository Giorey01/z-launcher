//! Manual folders (RF-15, RF-26): folders the user makes, the same game in several of them, and
//! the user's own order inside each. Their Home id is `@folder:<id>`.

use std::collections::HashMap;

use zl_config::organization::{MAX_FOLDER_NAME_CHARS, MAX_MANUAL_FOLDERS};
use zl_db::GameFlags;

use crate::error::CoreError;
use crate::smart_folders::mark_and_drop_hidden;
use crate::types::{CollectionInfo, FolderKind, GameEntry, PlatformEntry};
use crate::Service;

/// Prefix of the id of a manual folder.
const PREFIX: &str = "@folder:";

/// The Home id of manual folder `id`.
pub(crate) fn folder_id(id: i64) -> String {
    format!("{PREFIX}{id}")
}

/// The manual folder a Home id names, `None` for any other id.
pub(crate) fn collection_id_of(folder: &str) -> Option<i64> {
    folder.strip_prefix(PREFIX)?.parse().ok()
}

/// The name cleaned up: trimmed, inner spaces kept, cut to the longest allowed. `None` when
/// nothing is left.
fn clean_name(name: &str) -> Option<String> {
    let name: String = name
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .take(MAX_FOLDER_NAME_CHARS)
        .collect();
    let name = name.trim().to_string();
    (!name.is_empty()).then_some(name)
}

impl Service {
    /// The manual folders in creation order, with the count of their visible games.
    pub fn collections(&self) -> Result<Vec<CollectionInfo>, CoreError> {
        let flags = self.db().all_game_flags()?;
        let db = self.db();
        let counts = db.collection_counts()?;
        let mut out = Vec::new();
        for c in db.list_collections()? {
            let hidden = hidden_in(&db.collection_game_ids(c.id)?, &flags);
            let total = counts.get(&c.id).copied().unwrap_or(0) - hidden;
            out.push(CollectionInfo {
                id: c.id,
                folder_id: folder_id(c.id),
                name: c.name,
                games: u32::try_from(total.max(0)).unwrap_or(u32::MAX),
            });
        }
        Ok(out)
    }

    /// The Home entries of the manual folders, shown even when empty so they can be filled.
    pub(crate) fn collection_entries(&self) -> Result<Vec<PlatformEntry>, CoreError> {
        Ok(self
            .collections()?
            .into_iter()
            .map(|c| PlatformEntry {
                id: c.folder_id,
                name: c.name,
                games: c.games,
                kind: FolderKind::Collection,
            })
            .collect())
    }

    /// Makes a manual folder and returns it. `InvalidInput` for a name with nothing in it, or
    /// one that is already used (any case), or when there are too many folders.
    pub fn create_collection(&self, name: &str) -> Result<CollectionInfo, CoreError> {
        let name = clean_name(name).ok_or(CoreError::InvalidInput)?;
        let db = self.db();
        let existing = db.list_collections()?;
        if existing.len() >= MAX_MANUAL_FOLDERS || taken(&existing, &name, None) {
            return Err(CoreError::InvalidInput);
        }
        let id = db.create_collection(&name)?;
        Ok(CollectionInfo {
            id,
            folder_id: folder_id(id),
            name,
            games: 0,
        })
    }

    /// Renames a manual folder. `NotFound` for an unknown folder, `InvalidInput` for an empty
    /// name or one used by another folder.
    pub fn rename_collection(&self, id: i64, name: &str) -> Result<(), CoreError> {
        let name = clean_name(name).ok_or(CoreError::InvalidInput)?;
        let db = self.db();
        if taken(&db.list_collections()?, &name, Some(id)) {
            return Err(CoreError::InvalidInput);
        }
        db.rename_collection(id, &name)?
            .then_some(())
            .ok_or(CoreError::NotFound)
    }

    /// Deletes a manual folder; its games stay in the library and in other folders.
    pub fn delete_collection(&self, id: i64) -> Result<(), CoreError> {
        let db = self.db();
        db.delete_collection(id)?
            .then_some(())
            .ok_or(CoreError::NotFound)?;
        db.set_setting(&format!("sort.{}", folder_id(id)), "")?;
        Ok(())
    }

    /// The manual folders that hold game `game_id`, by folder id.
    pub fn game_collections(&self, game_id: i64) -> Result<Vec<i64>, CoreError> {
        Ok(self.db().collections_of_game(game_id)?)
    }

    /// Puts games in a manual folder (at its end); those already in it stay where they are.
    /// `NotFound` for an unknown folder or game.
    pub fn add_to_collection(&self, id: i64, game_ids: &[i64]) -> Result<(), CoreError> {
        let db = self.db();
        self.require_collection(&db, id)?;
        for game in game_ids {
            db.game(*game)?.ok_or(CoreError::NotFound)?;
        }
        db.transaction::<(), CoreError>(|db| Ok(db.add_to_collection(id, game_ids)?))
    }

    /// Takes games out of a manual folder (they stay in the library). `NotFound` for an unknown
    /// folder.
    pub fn remove_from_collection(&self, id: i64, game_ids: &[i64]) -> Result<(), CoreError> {
        let db = self.db();
        self.require_collection(&db, id)?;
        db.transaction::<(), CoreError>(|db| Ok(db.remove_from_collection(id, game_ids)?))
    }

    /// Moves games from manual folder `from` to `to`, in one step: they leave `from` and join the
    /// end of `to`.
    pub fn move_between_collections(
        &self,
        from: i64,
        to: i64,
        game_ids: &[i64],
    ) -> Result<(), CoreError> {
        let db = self.db();
        self.require_collection(&db, from)?;
        self.require_collection(&db, to)?;
        db.transaction::<(), CoreError>(|db| {
            db.add_to_collection(to, game_ids)?;
            if from != to {
                db.remove_from_collection(from, game_ids)?;
            }
            Ok(())
        })
    }

    /// Moves `game_id` to place `to` (counted from 0, among the folder's games) in manual folder
    /// `id`, the others shifting. A place past the end means the end. `NotFound` when the game
    /// is not in the folder.
    pub fn move_in_collection(&self, id: i64, game_id: i64, to: usize) -> Result<(), CoreError> {
        let db = self.db();
        self.require_collection(&db, id)?;
        let mut ids = db.collection_game_ids(id)?;
        let from = ids
            .iter()
            .position(|g| *g == game_id)
            .ok_or(CoreError::NotFound)?;
        ids.remove(from);
        ids.insert(to.min(ids.len()), game_id);
        db.transaction::<(), CoreError>(|db| Ok(db.set_collection_order(id, &ids)?))
    }

    /// The games of manual folder `id` in the order the user set, without hidden ones. The order
    /// in force (see `sort_order`) applies on top when the user chose another one.
    pub(crate) fn collection_games(
        &self,
        id: i64,
        flags: &HashMap<i64, GameFlags>,
    ) -> Result<Vec<GameEntry>, CoreError> {
        let ids = {
            let db = self.db();
            self.require_collection(&db, id)?;
            db.collection_game_ids(id)?
        };
        let mut by_id: HashMap<i64, GameEntry> = self
            .listed_games(None)?
            .into_iter()
            .map(|g| (g.id, g))
            .collect();
        let mut games: Vec<GameEntry> = ids.iter().filter_map(|g| by_id.remove(g)).collect();
        mark_and_drop_hidden(&mut games, flags);
        self.apply_sort(&mut games, Some(&folder_id(id)))?;
        Ok(games)
    }

    /// `NotFound` unless manual folder `id` exists.
    fn require_collection(&self, db: &zl_db::Db, id: i64) -> Result<(), CoreError> {
        let found = db.list_collections()?.iter().any(|c| c.id == id);
        found.then_some(()).ok_or(CoreError::NotFound)
    }
}

/// How many of the games `ids` are hidden (and so do not count).
fn hidden_in(ids: &[i64], flags: &HashMap<i64, GameFlags>) -> i64 {
    let hidden = ids
        .iter()
        .filter(|g| flags.get(g).is_some_and(|f| f.hidden))
        .count();
    i64::try_from(hidden).unwrap_or(0)
}

/// Whether `name` is the name of a folder other than `except` (any case).
fn taken(folders: &[zl_db::Collection], name: &str, except: Option<i64>) -> bool {
    folders
        .iter()
        .any(|c| Some(c.id) != except && c.name.to_lowercase() == name.to_lowercase())
}
