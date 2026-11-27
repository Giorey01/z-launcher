//! Sort orders (RF-16): the order of the games of a folder, chosen by the user for one folder or
//! for all of them.
//!
//! A folder has a natural order (Recent by last played, a manual folder by the user's own
//! order, anything else by title). The user's choice for the folder wins; failing that, the
//! global choice applies to the folders whose natural order is the title; the time-based Smart
//! Folders and the manual folders keep their own unless chosen explicitly.

use std::cmp::Reverse;

use crate::collections::collection_id_of;
use crate::error::CoreError;
use crate::smart_folders::kind_of;
use crate::types::{FolderKind, GameEntry, SortOrder};
use crate::Service;

/// Setting key of the order for all folders.
const KEY_GLOBAL: &str = "sort.global";

/// Rating scale factor: ratings are compared in tenths of a star.
const RATING_TENTHS: f32 = 10.0;

/// Stable spelling of an order in the settings.
fn code(order: SortOrder) -> &'static str {
    match order {
        SortOrder::Name => "name",
        SortOrder::LastPlayed => "last_played",
        SortOrder::PlayTime => "play_time",
        SortOrder::Added => "added",
        SortOrder::Rating => "rating",
        SortOrder::Manual => "manual",
    }
}

/// The order spelled `text`, `None` for anything unknown.
fn parse(text: &str) -> Option<SortOrder> {
    [
        SortOrder::Name,
        SortOrder::LastPlayed,
        SortOrder::PlayTime,
        SortOrder::Added,
        SortOrder::Rating,
        SortOrder::Manual,
    ]
    .into_iter()
    .find(|o| code(*o) == text)
}

/// Setting key of the order of one folder, or of all folders with `None`.
fn key_of(folder: Option<&str>) -> String {
    folder.map_or_else(|| KEY_GLOBAL.to_string(), |id| format!("sort.{id}"))
}

/// The order a folder has when the user chose nothing for it.
fn natural_order(folder: Option<&str>) -> SortOrder {
    match folder.and_then(kind_of) {
        Some(FolderKind::Recent) => SortOrder::LastPlayed,
        Some(FolderKind::MostPlayed) => SortOrder::PlayTime,
        Some(FolderKind::New) => SortOrder::Added,
        _ if folder.and_then(collection_id_of).is_some() => SortOrder::Manual,
        _ => SortOrder::Name,
    }
}

impl Service {
    /// The order the user chose for `folder` (`None`: for all folders), if they chose one.
    pub fn sort_override(&self, folder: Option<&str>) -> Result<Option<SortOrder>, CoreError> {
        let stored = self.db().setting(&key_of(folder))?;
        Ok(stored.as_deref().and_then(parse))
    }

    /// Sets the order of `folder` (`None`: of all folders), or clears the choice with `None`.
    /// `InvalidInput` for the manual order anywhere but in a manual folder.
    pub fn set_sort_order(
        &self,
        folder: Option<&str>,
        order: Option<SortOrder>,
    ) -> Result<(), CoreError> {
        let in_collection = folder.and_then(collection_id_of).is_some();
        if order == Some(SortOrder::Manual) && !in_collection {
            return Err(CoreError::InvalidInput);
        }
        let key = key_of(folder);
        match order {
            Some(order) => self.db().set_setting(&key, code(order))?,
            None => self.db().set_setting(&key, "")?,
        }
        Ok(())
    }

    /// The order in force for `folder` (`None`: for the list of every game).
    pub fn sort_order(&self, folder: Option<&str>) -> Result<SortOrder, CoreError> {
        let natural = natural_order(folder);
        if let Some(own) = self.sort_override(folder)? {
            return Ok(own);
        }
        if natural != SortOrder::Name {
            return Ok(natural);
        }
        let global = self.sort_override(None)?;
        Ok(global
            .filter(|o| *o != SortOrder::Manual)
            .unwrap_or(natural))
    }

    /// Puts `games` (in the natural order of `folder`) in the order in force for it.
    pub(crate) fn apply_sort(
        &self,
        games: &mut [GameEntry],
        folder: Option<&str>,
    ) -> Result<(), CoreError> {
        let order = self.sort_order(folder)?;
        if order == natural_order(folder) {
            return Ok(());
        }
        self.sort_by_order(games, order)
    }

    /// Sorts `games` by `order`; games that tie keep their places.
    fn sort_by_order(&self, games: &mut [GameEntry], order: SortOrder) -> Result<(), CoreError> {
        match order {
            SortOrder::Name | SortOrder::Manual => {
                games.sort_by_cached_key(|g| g.title.to_lowercase());
            }
            SortOrder::LastPlayed => {
                let stats = self.db().all_play_stats()?;
                games.sort_by_key(|g| Reverse(stats.get(&g.id).map_or(0, |s| s.last_played)));
            }
            SortOrder::PlayTime => {
                let stats = self.db().all_play_stats()?;
                let total = |g: &GameEntry| {
                    stats
                        .get(&g.id)
                        .map_or(0, |s| s.device_seconds + s.steam_seconds)
                };
                games.sort_by_key(|g| Reverse(total(g)));
            }
            SortOrder::Added => {
                let added = self.db().all_added_at()?;
                games.sort_by_key(|g| Reverse(added.get(&g.id).copied().unwrap_or(0)));
            }
            SortOrder::Rating => {
                let ratings = self.db().all_ratings()?;
                // Unrated games get -1, below the lowest rating, so they come last.
                let tenths = |g: &GameEntry| {
                    ratings
                        .get(&g.id)
                        .map_or(-1, |r| (r * RATING_TENTHS).round() as i64)
                };
                games.sort_by_key(|g| Reverse(tenths(g)));
            }
        }
        Ok(())
    }
}
