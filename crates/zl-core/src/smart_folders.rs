//! Smart Folders (RF-13): Favourites, Recent, Most played, New, Never played and Hidden, built
//! from the user's flags, the play sessions and the time a game was added; Apps and Emulators
//! are built from the installed apps (see `app_folders`). Their ids start with `@`, which no
//! platform id can.

use std::collections::HashMap;

use zl_config::organization::{MOST_PLAYED_LIMIT, NEW_GAME_DAYS, SECONDS_PER_DAY};
use zl_config::sessions::RECENT_LIMIT;
use zl_db::{GameFlags, PlayStats};

use crate::error::CoreError;
use crate::types::{FolderKind, GameEntry, PlatformEntry};
use crate::Service;

/// Id and Smart Folder type (the theme's `smart_folders/<type>/` folder) of each kind.
const SMART: [(FolderKind, &str, &str); 8] = [
    (FolderKind::Favorites, "@favorites", "favorites"),
    (FolderKind::Recent, "@recent", "recent"),
    (FolderKind::MostPlayed, "@mostplayed", "mostplayed"),
    (FolderKind::New, "@new", "new"),
    (FolderKind::NeverPlayed, "@neverplayed", "neverplayed"),
    (FolderKind::Hidden, "@hidden", "hidden"),
    (FolderKind::Apps, "@apps", "apps"),
    (FolderKind::Emulators, "@emulators", "emulators"),
];

/// The kind of a Smart Folder id.
pub(crate) fn kind_of(id: &str) -> Option<FolderKind> {
    SMART.iter().find(|(_, i, _)| *i == id).map(|(k, _, _)| *k)
}

/// The id of a Smart Folder kind.
pub(crate) fn id_of(kind: FolderKind) -> &'static str {
    SMART
        .iter()
        .find(|(k, _, _)| *k == kind)
        .map_or("", |(_, i, _)| i)
}

/// The theme folder type of a Smart Folder id (`@recent` → `recent`).
pub(crate) fn theme_type_of(id: &str) -> Option<&'static str> {
    SMART.iter().find(|(_, i, _)| *i == id).map(|(_, _, t)| *t)
}

/// Seconds since the Unix epoch now (0 when the clock is before it).
fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

/// Total play time of a game: measured on the device plus the Steam total.
fn play_seconds(stats: Option<&PlayStats>) -> i64 {
    stats.map_or(0, |s| s.device_seconds + s.steam_seconds)
}

/// Whether a game was ever played, on this device or on Steam.
fn played(stats: Option<&PlayStats>) -> bool {
    stats.is_some_and(|s| s.device_seconds > 0 || s.steam_seconds > 0)
}

impl Service {
    /// The games of a Home folder: a Smart Folder id, a manual folder id (`@folder:<id>`), a
    /// platform or group id (any alias), or every platform with `None`, in the order the user
    /// chose for it. Hidden games appear only in the Hidden folder. Apps and Emulators hold no
    /// games (see `app_folders`).
    pub fn games(&self, folder: Option<&str>) -> Result<Vec<GameEntry>, CoreError> {
        self.games_at(folder, now_secs())
    }

    /// Like [`Service::games`] with the clock given (Unix seconds), which decides what is new.
    pub fn games_at(&self, folder: Option<&str>, now: i64) -> Result<Vec<GameEntry>, CoreError> {
        let flags = self.db().all_game_flags()?;
        if let Some(kind) = folder.and_then(kind_of) {
            return self.smart_games(kind, &flags, now);
        }
        if let Some(id) = folder.and_then(crate::collections::collection_id_of) {
            return self.collection_games(id, &flags);
        }
        let mut games = self.listed_games(folder)?;
        mark_and_drop_hidden(&mut games, &flags);
        self.apply_sort(&mut games, folder)?;
        Ok(games)
    }

    /// Searches games matching `query` using typo-tolerant fuzzy matching (RF-16).
    /// If `query` is blank, returns the first `limit` games in alphabetical order.
    pub fn search_games(&self, query: &str, limit: usize) -> Result<Vec<GameEntry>, CoreError> {
        let all = self.games(None)?;
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(all.into_iter().take(limit).collect());
        }
        let titles: Vec<String> = all.iter().map(|g| g.display_title.clone()).collect();
        let fuzzy = crate::search::FuzzySearch::new(&titles);
        let matches = fuzzy.search(trimmed, limit);
        let mut results = Vec::with_capacity(matches.len());
        for idx in matches {
            if let Some(game) = all.get(idx) {
                results.push(game.clone());
            }
        }
        Ok(results)
    }

    /// The games of one Smart Folder.
    fn smart_games(
        &self,
        kind: FolderKind,
        flags: &HashMap<i64, GameFlags>,
        now: i64,
    ) -> Result<Vec<GameEntry>, CoreError> {
        if matches!(kind, FolderKind::Apps | FolderKind::Emulators) {
            return Ok(Vec::new());
        }
        let mut all = self.listed_games(None)?;
        let hidden = |g: &GameEntry| flags.get(&g.id).is_some_and(|f| f.hidden);
        if kind == FolderKind::Hidden {
            all.retain(hidden);
            for g in &mut all {
                g.favorite = flags.get(&g.id).is_some_and(|f| f.favorite);
            }
            return Ok(all);
        }
        mark_and_drop_hidden(&mut all, flags);
        self.filter_smart(kind, &mut all, now)?;
        self.apply_sort(&mut all, Some(id_of(kind)))?;
        Ok(all)
    }

    /// Keeps the games of a Smart Folder in `all` (already without the hidden ones), in the
    /// folder's own order.
    fn filter_smart(
        &self,
        kind: FolderKind,
        all: &mut Vec<GameEntry>,
        now: i64,
    ) -> Result<(), CoreError> {
        let stats = self.db().all_play_stats()?;
        let seconds = |g: &GameEntry| play_seconds(stats.get(&g.id));
        let last = |g: &GameEntry| stats.get(&g.id).map_or(0, |s| s.last_played);
        match kind {
            FolderKind::Favorites => all.retain(|g| g.favorite),
            FolderKind::NeverPlayed => all.retain(|g| !played(stats.get(&g.id))),
            FolderKind::Recent => {
                all.retain(|g| last(g) > 0);
                all.sort_by_key(|g| std::cmp::Reverse(last(g)));
                all.truncate(RECENT_LIMIT);
            }
            FolderKind::MostPlayed => {
                all.retain(|g| seconds(g) > 0);
                all.sort_by_key(|g| std::cmp::Reverse(seconds(g)));
                all.truncate(MOST_PLAYED_LIMIT);
            }
            FolderKind::New => {
                let added = self.db().all_added_at()?;
                let when = |g: &GameEntry| added.get(&g.id).copied().unwrap_or(0);
                all.retain(|g| when(g) >= new_cutoff(now));
                all.sort_by_key(|g| std::cmp::Reverse(when(g)));
            }
            _ => {}
        }
        Ok(())
    }

    /// The Smart Folders of the library that have games, in Home order: Favourites, Recent, Most
    /// played, New, Never played. New and Never played are left out while they would be the whole
    /// library (nothing older than the new games, nothing played yet).
    pub(crate) fn smart_entries(
        &self,
        visible: &[GameEntry],
    ) -> Result<Vec<PlatformEntry>, CoreError> {
        let stats = self.db().all_play_stats()?;
        let added = self.db().all_added_at()?;
        let cutoff = new_cutoff(now_secs());
        let count = |pred: &dyn Fn(&GameEntry) -> bool| visible.iter().filter(|g| pred(g)).count();
        let favorites = count(&|g| g.favorite);
        let recent =
            count(&|g| stats.get(&g.id).is_some_and(|s| s.last_played > 0)).min(RECENT_LIMIT);
        let most = count(&|g| play_seconds(stats.get(&g.id)) > 0).min(MOST_PLAYED_LIMIT);
        let new = count(&|g| added.get(&g.id).is_some_and(|a| *a >= cutoff));
        let never = count(&|g| !played(stats.get(&g.id)));
        let not_all = |n: usize| if n == visible.len() { 0 } else { n };
        let entries = [
            (FolderKind::Favorites, favorites),
            (FolderKind::Recent, recent),
            (FolderKind::MostPlayed, most),
            (FolderKind::New, not_all(new)),
            (FolderKind::NeverPlayed, not_all(never)),
        ];
        Ok(entries
            .into_iter()
            .filter(|(_, n)| *n > 0)
            .map(|(kind, n)| smart_entry(kind, n))
            .collect())
    }

    /// The Hidden entry, when some game is hidden.
    pub(crate) fn hidden_entry(&self) -> Result<Option<PlatformEntry>, CoreError> {
        let flags = self.db().all_game_flags()?;
        let n = self
            .listed_games(None)?
            .iter()
            .filter(|g| flags.get(&g.id).is_some_and(|f| f.hidden))
            .count();
        Ok((n > 0).then(|| smart_entry(FolderKind::Hidden, n)))
    }
}

/// The oldest `added_at` (Unix seconds) that still counts as new at time `now`.
fn new_cutoff(now: i64) -> i64 {
    now - NEW_GAME_DAYS * SECONDS_PER_DAY
}

/// A Smart Folder entry with `n` games.
pub(crate) fn smart_entry(kind: FolderKind, n: usize) -> PlatformEntry {
    let id = id_of(kind);
    PlatformEntry {
        id: id.to_string(),
        name: theme_type_of(id).unwrap_or_default().to_string(),
        games: u32::try_from(n).unwrap_or(u32::MAX),
        kind,
    }
}

/// Sets the favourite mark of each game and drops the hidden ones.
pub(crate) fn mark_and_drop_hidden(games: &mut Vec<GameEntry>, flags: &HashMap<i64, GameFlags>) {
    games.retain(|g| !flags.get(&g.id).is_some_and(|f| f.hidden));
    for g in games.iter_mut() {
        g.favorite = flags.get(&g.id).is_some_and(|f| f.favorite);
    }
}
