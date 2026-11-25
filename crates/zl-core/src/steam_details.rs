//! Details of Steam games from the public store (no key): description, release date,
//! developers, publishers, genres, Metacritic score, screenshots saved next to the other
//! downloaded media, and the first trailer kept as an address to stream.
//!
//! Only games with a `steamappid` tag are asked, and only those with no description yet, so
//! metadata from ES-DE and the user's own edits (kept as overrides) are never replaced. The
//! store limits how fast it may be asked: the run waits between games, stops on cancel, and stops
//! as offline after a few failures in a row. The network is never used while the database is
//! locked.

use std::path::PathBuf;
use std::time::Duration;

use zl_config::steam as cfg;
use zl_db::{GameMeta, MediaKind, MediaSource, NewMedia};
use zl_net::HttpClient;
use zl_steam::{SteamError, SteamStore, StoreApp};

use crate::error::CoreError;
use crate::scrape::{ImageScrapeReport, ScrapeProgress, ScrapeStop, ScrapeTarget};
use crate::types::GameEntry;
use crate::Service;

/// Tag of a Steam tag file. A format fact.
const STEAM_TAG: &str = "steamappid";

/// Failed games in a row after which the run stops as offline.
const OFFLINE_AFTER_FAILURES: u32 = 3;

/// What happened to one game.
enum Outcome {
    /// It is not a Steam game, or it has a description already.
    Skipped,
    /// The store has no page for it.
    NotFound,
    /// Details saved, with this many screenshots.
    Saved(u32),
}

/// Whether `url` is an HTTPS address on the store's media hosts.
fn on_steam_cdn(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://") else {
        return false;
    };
    let host = rest.split('/').next().unwrap_or_default();
    host.ends_with(cfg::STEAM_MEDIA_HOST_SUFFIX)
}

/// The stored metadata of a store page.
fn meta_of(game_id: i64, app: &StoreApp) -> GameMeta {
    GameMeta {
        game_id,
        description: app.description.clone(),
        release_date: app.release_date.clone(),
        developer: app.developer.clone(),
        publisher: app.publisher.clone(),
        genres: app.genres.clone(),
        players: None,
        rating: app.rating,
        source: "steam".to_string(),
        scraped_at: 0,
    }
}

impl Service {
    /// Fetches the store details of the Steam games of `target` that have no description,
    /// waiting `pause` between games. `complete` counts the games skipped (no Steam id, or a
    /// description already there), `images` the screenshots saved.
    pub fn fetch_steam_details(
        &self,
        target: &ScrapeTarget,
        http: &dyn HttpClient,
        progress: &dyn ScrapeProgress,
        pause: Duration,
    ) -> Result<ImageScrapeReport, CoreError> {
        let games = self.target_games(target)?;
        let mut report = ImageScrapeReport {
            games: u32::try_from(games.len()).unwrap_or(u32::MAX),
            ..ImageScrapeReport::default()
        };
        let store = SteamStore::new(http);
        let mut failures_in_row = 0;
        for (index, game) in games.iter().enumerate() {
            if progress.is_cancelled() {
                report.stopped = Some(ScrapeStop::Cancelled);
                break;
            }
            let asked = match self.details_of(&store, http, game) {
                Ok(outcome) => {
                    failures_in_row = 0;
                    count(&mut report, &outcome);
                    !matches!(outcome, Outcome::Skipped)
                }
                Err(_) => {
                    report.failed += 1;
                    failures_in_row += 1;
                    true
                }
            };
            if failures_in_row >= OFFLINE_AFTER_FAILURES {
                report.stopped = Some(ScrapeStop::Offline);
                break;
            }
            let done = u32::try_from(index + 1).unwrap_or(u32::MAX);
            progress.on_game(done, report.games);
            if asked && index + 1 < games.len() && !pause.is_zero() {
                std::thread::sleep(pause);
            }
        }
        Ok(report)
    }

    /// The Steam app id of a game and whether it still lacks a description.
    fn wants_details(&self, game: &GameEntry) -> Result<Option<u32>, CoreError> {
        let db = self.db();
        let tags = db.game_tags(game.id)?;
        let Some((_, value)) = tags.into_iter().find(|(k, _)| k == STEAM_TAG) else {
            return Ok(None);
        };
        let described = db
            .game_meta(game.id)?
            .and_then(|m| m.description)
            .is_some_and(|d| !d.trim().is_empty());
        let wanted = !described;
        Ok(wanted.then(|| value.trim().parse().ok()).flatten())
    }

    /// Keeps the first trailer of a page that is on the store's hosts, as an address to stream
    /// (trailers are not downloaded). A local video imported from ES-DE stays first.
    fn save_trailer(&self, game_id: i64, app: &StoreApp) {
        let Some(url) = app.trailers.iter().find(|u| on_steam_cdn(u)) else {
            return;
        };
        let row = NewMedia {
            game_id,
            kind: MediaKind::Video,
            path: url.clone(),
            source: MediaSource::Steam,
            edit: None,
        };
        // A failed write only means the game has no trailer.
        let _ = self.db().add_media(&row);
    }

    /// Asks the store about one game and saves what it says.
    fn details_of(
        &self,
        store: &SteamStore<'_>,
        http: &dyn HttpClient,
        game: &GameEntry,
    ) -> Result<Outcome, SteamError> {
        let appid = match self.wants_details(game) {
            Ok(Some(appid)) => appid,
            Ok(None) => return Ok(Outcome::Skipped),
            Err(_) => return Err(SteamError::Parse("database".into())),
        };
        let Some(app) = store.app(appid)? else {
            return Ok(Outcome::NotFound);
        };
        self.db()
            .save_game_meta(&meta_of(game.id, &app), false)
            .map_err(|_| SteamError::Parse("database".into()))?;
        self.save_trailer(game.id, &app);
        Ok(Outcome::Saved(
            self.save_screenshots(http, game.id, appid, &app),
        ))
    }

    /// Downloads the screenshots of a page that are on the store's hosts; returns how many
    /// were saved. A screenshot that fails is skipped, the others are still saved.
    fn save_screenshots(
        &self,
        http: &dyn HttpClient,
        game_id: i64,
        appid: u32,
        app: &StoreApp,
    ) -> u32 {
        let dir = self.steam_media_dir(appid);
        if std::fs::create_dir_all(&dir).is_err() {
            return 0;
        }
        let mut saved = 0;
        for (n, url) in app
            .screenshots
            .iter()
            .filter(|u| on_steam_cdn(u))
            .enumerate()
        {
            let Ok(bytes) =
                http.get_bytes(url, &[("User-Agent", zl_config::media::HTTP_USER_AGENT)])
            else {
                continue;
            };
            let path = dir.join(format!("{n}.jpg"));
            if bytes.is_empty() || std::fs::write(&path, &bytes).is_err() {
                continue;
            }
            let row = NewMedia {
                game_id,
                kind: MediaKind::Screenshot,
                path: path.to_string_lossy().into_owned(),
                source: MediaSource::Steam,
                edit: None,
            };
            if self.db().add_media(&row).is_ok() {
                saved += 1;
            }
        }
        saved
    }

    /// The folder of the downloaded store media of one app.
    fn steam_media_dir(&self, appid: u32) -> PathBuf {
        self.data_dir
            .join(cfg::STEAM_MEDIA_FOLDER)
            .join(appid.to_string())
    }
}

/// Adds one game's outcome to the report.
fn count(report: &mut ImageScrapeReport, outcome: &Outcome) {
    match outcome {
        Outcome::Skipped => report.complete += 1,
        Outcome::NotFound => report.not_found += 1,
        Outcome::Saved(n) => {
            report.updated += 1;
            report.images += n;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::on_steam_cdn;

    /// Only HTTPS on the store's own hosts, checked on the host and not on the path.
    #[test]
    fn media_must_come_from_the_store() {
        assert!(on_steam_cdn("https://shared.akamai.steamstatic.com/a.jpg"));
        assert!(!on_steam_cdn("http://shared.akamai.steamstatic.com/a.jpg"));
        assert!(!on_steam_cdn(
            "https://evil.example/x.steamstatic.com/a.jpg"
        ));
        assert!(!on_steam_cdn("https://steamstatic.com.evil.example/a.jpg"));
    }
}
