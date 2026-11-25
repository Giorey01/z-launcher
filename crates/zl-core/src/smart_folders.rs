//! Smart Folders (RF-13): Favourites, Recent, Never played and Hidden, built from the user's
//! flags and the play sessions. Their ids start with `@`, which no platform id can.

use std::collections::HashMap;

use zl_config::sessions::RECENT_LIMIT;
use zl_db::{GameFlags, PlayStats};

use crate::error::CoreError;
use crate::types::{FolderKind, GameEntry, PlatformEntry};
use crate::Service;

/// Id and Smart Folder type (the theme's `smart_folders/<type>/` folder) of each kind.
const SMART: [(FolderKind, &str, &str); 4] = [
    (FolderKind::Favorites, "@favorites", "favorites"),
    (FolderKind::Recent, "@recent", "recent"),
    (FolderKind::NeverPlayed, "@neverplayed", "neverplayed"),
    (FolderKind::Hidden, "@hidden", "hidden"),
];

/// The kind of a Smart Folder id.
fn kind_of(id: &str) -> Option<FolderKind> {
    SMART.iter().find(|(_, i, _)| *i == id).map(|(k, _, _)| *k)
}

/// The id of a Smart Folder kind.
fn id_of(kind: FolderKind) -> &'static str {
    SMART
        .iter()
        .find(|(k, _, _)| *k == kind)
        .map_or("", |(_, i, _)| i)
}

/// The theme folder type of a Smart Folder id (`@recent` → `recent`).
pub(crate) fn theme_type_of(id: &str) -> Option<&'static str> {
    SMART.iter().find(|(_, i, _)| *i == id).map(|(_, _, t)| *t)
}

/// Whether a game was ever played, on this device or on Steam.
fn played(stats: Option<&PlayStats>) -> bool {
    stats.is_some_and(|s| s.device_seconds > 0 || s.steam_seconds > 0)
}

impl Service {
    /// The games of a Home folder: a Smart Folder id, a platform or group id (any alias), or
    /// every platform with `None`. Hidden games appear only in the Hidden folder.
    pub fn games(&self, folder: Option<&str>) -> Result<Vec<GameEntry>, CoreError> {
        let flags = self.db().all_game_flags()?;
        if let Some(kind) = folder.and_then(kind_of) {
            return self.smart_games(kind, &flags);
        }
        let mut games = self.listed_games(folder)?;
        mark_and_drop_hidden(&mut games, &flags);
        Ok(games)
    }

    /// The games of one Smart Folder.
    fn smart_games(
        &self,
        kind: FolderKind,
        flags: &HashMap<i64, GameFlags>,
    ) -> Result<Vec<GameEntry>, CoreError> {
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
        let stats = self.db().all_play_stats()?;
        match kind {
            FolderKind::Favorites => all.retain(|g| g.favorite),
            FolderKind::NeverPlayed => all.retain(|g| !played(stats.get(&g.id))),
            FolderKind::Recent => {
                let last = |g: &GameEntry| stats.get(&g.id).map_or(0, |s| s.last_played);
                all.retain(|g| last(g) > 0);
                all.sort_by_key(|g| std::cmp::Reverse(last(g)));
                all.truncate(RECENT_LIMIT);
            }
            FolderKind::Platform | FolderKind::Hidden => {}
        }
        Ok(all)
    }

    /// The Favourites, Recent and Never played entries that have games, from the visible games
    /// (Never played only when it is not the whole library).
    pub(crate) fn smart_entries(
        &self,
        visible: &[GameEntry],
    ) -> Result<Vec<PlatformEntry>, CoreError> {
        let stats = self.db().all_play_stats()?;
        let count = |pred: &dyn Fn(&GameEntry) -> bool| visible.iter().filter(|g| pred(g)).count();
        let favorites = count(&|g| g.favorite);
        let recent =
            count(&|g| stats.get(&g.id).is_some_and(|s| s.last_played > 0)).min(RECENT_LIMIT);
        // Never played is shown once something was played: before that it is the whole library.
        let never = count(&|g| !played(stats.get(&g.id)));
        let never = if never == visible.len() { 0 } else { never };
        let entries = [
            (FolderKind::Favorites, favorites),
            (FolderKind::Recent, recent),
            (FolderKind::NeverPlayed, never),
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

/// A Smart Folder entry with `n` games.
fn smart_entry(kind: FolderKind, n: usize) -> PlatformEntry {
    let id = id_of(kind);
    PlatformEntry {
        id: id.to_string(),
        name: theme_type_of(id).unwrap_or_default().to_string(),
        games: u32::try_from(n).unwrap_or(u32::MAX),
        kind,
    }
}

/// Sets the favourite mark of each game and drops the hidden ones.
fn mark_and_drop_hidden(games: &mut Vec<GameEntry>, flags: &HashMap<i64, GameFlags>) {
    games.retain(|g| !flags.get(&g.id).is_some_and(|f| f.hidden));
    for g in games.iter_mut() {
        g.favorite = flags.get(&g.id).is_some_and(|f| f.favorite);
    }
}
