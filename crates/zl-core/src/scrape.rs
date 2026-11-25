//! Downloading the art a game lacks (cover, hero, logo) from SteamGridDB.
//!
//! Only the kinds a game has no file for are asked, so art imported from ES-DE or Cocoon and the
//! user's own choices are never replaced. The files go to `<data>/media/sgdb/<platform>/<kind>/`
//! and are recorded with the `sgdb` source, which imports leave alone. The network is never used
//! while the database is locked.

use std::path::PathBuf;
use std::time::Duration;

use zl_config::media as cfg;
use zl_db::{MediaKind, MediaSource, NewMedia};
use zl_meta::{pick_game, MetaError, SteamGridDb};
use zl_net::HttpClient;

use crate::error::CoreError;
use crate::types::GameEntry;
use crate::Service;

/// The kinds of art downloaded, in the order they are asked.
const KINDS: [MediaKind; 3] = [MediaKind::Icon, MediaKind::Hero, MediaKind::Logo];

/// Failed games in a row after which the run stops as offline.
const OFFLINE_AFTER_FAILURES: u32 = 3;

/// Which games a run covers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrapeTarget {
    /// Every game of the library.
    All,
    /// The games of one platform (canonical id).
    Platform(String),
    /// One game.
    Game(i64),
}

/// Why a run ended before the last game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrapeStop {
    /// No SteamGridDB key is set.
    MissingKey,
    /// SteamGridDB refused the key.
    KeyRefused,
    /// Several games in a row failed on the network.
    Offline,
    /// The user cancelled.
    Cancelled,
}

/// The numbers of a run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImageScrapeReport {
    /// Games the run covered.
    pub games: u32,
    /// Games that already had a cover, a hero and a logo.
    pub complete: u32,
    /// Games that got at least one new file.
    pub updated: u32,
    /// Files downloaded.
    pub images: u32,
    /// Games SteamGridDB has no match (or no usable art) for.
    pub not_found: u32,
    /// Games that failed on the network or on disk.
    pub failed: u32,
    /// Why the run ended early, `None` when it went through.
    pub stopped: Option<ScrapeStop>,
}

/// The answer to a key check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCheck {
    /// SteamGridDB accepts the key.
    Valid,
    /// SteamGridDB refuses the key.
    Refused,
    /// SteamGridDB could not be reached.
    Offline,
}

/// Receives the progress of a run and can cancel it. Called on the running thread.
pub trait ScrapeProgress: Send + Sync {
    /// `done` of `total` games are finished.
    fn on_game(&self, done: u32, total: u32);
    /// Whether the user asked to stop.
    fn is_cancelled(&self) -> bool;
}

/// What happened to one game.
enum Outcome {
    /// It had every kind already.
    Complete,
    /// `n` files were saved (0 means SteamGridDB had nothing usable).
    Saved(u32),
}

impl Service {
    /// Asks SteamGridDB whether it accepts `key` (nothing is stored).
    pub fn check_steamgriddb_key(&self, key: &str, http: &dyn HttpClient) -> KeyCheck {
        match SteamGridDb::new(http, key).verify_key() {
            Ok(()) => KeyCheck::Valid,
            Err(MetaError::AuthRequired { .. }) => KeyCheck::Refused,
            Err(_) => KeyCheck::Offline,
        }
    }

    /// Stores the SteamGridDB key (`None` or blank removes it). A key must be letters and digits
    /// of a plausible length.
    pub fn set_steamgriddb_key(&self, key: Option<&str>) -> Result<(), CoreError> {
        let key = key.map(str::trim).filter(|k| !k.is_empty());
        if let Some(k) = key {
            let len_ok = (cfg::SGDB_MIN_KEY_LEN..=cfg::SGDB_MAX_KEY_LEN).contains(&k.len());
            if !len_ok || !k.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Err(CoreError::InvalidInput);
            }
        }
        let mut config = self.scraper_config()?;
        config.steamgriddb_key = key.map(str::to_string);
        self.set_scraper_config(&config)
    }

    /// Downloads the missing cover, hero and logo of the games of `target`, pausing `pause`
    /// between games (SteamGridDB asks clients to go slowly).
    pub fn scrape_images(
        &self,
        target: &ScrapeTarget,
        http: &dyn HttpClient,
        progress: &dyn ScrapeProgress,
        pause: Duration,
    ) -> Result<ImageScrapeReport, CoreError> {
        let games = self.target_games(target)?;
        let mut report = ImageScrapeReport {
            games: games.len() as u32,
            ..ImageScrapeReport::default()
        };
        let Some(key) = self.scraper_config()?.steamgriddb_key else {
            report.stopped = Some(ScrapeStop::MissingKey);
            return Ok(report);
        };
        let sgdb = SteamGridDb::new(http, &key);
        self.run(&sgdb, &games, progress, pause, &mut report);
        Ok(report)
    }

    /// The loop over the games; stops on a refused key, on cancel, or when offline.
    fn run(
        &self,
        sgdb: &SteamGridDb<'_>,
        games: &[GameEntry],
        progress: &dyn ScrapeProgress,
        pause: Duration,
        report: &mut ImageScrapeReport,
    ) {
        let mut failures_in_row = 0;
        for (index, game) in games.iter().enumerate() {
            if progress.is_cancelled() {
                report.stopped = Some(ScrapeStop::Cancelled);
                return;
            }
            match self.scrape_game(sgdb, game) {
                Ok(outcome) => {
                    failures_in_row = 0;
                    count(report, outcome);
                }
                Err(MetaError::AuthRequired { .. }) => {
                    report.stopped = Some(ScrapeStop::KeyRefused);
                    return;
                }
                Err(_) => {
                    report.failed += 1;
                    failures_in_row += 1;
                    if failures_in_row >= OFFLINE_AFTER_FAILURES {
                        report.stopped = Some(ScrapeStop::Offline);
                        return;
                    }
                }
            }
            progress.on_game(index as u32 + 1, games.len() as u32);
            if index + 1 < games.len() && !pause.is_zero() {
                std::thread::sleep(pause);
            }
        }
    }

    /// The games of a target, missing ones left out.
    pub(crate) fn target_games(&self, target: &ScrapeTarget) -> Result<Vec<GameEntry>, CoreError> {
        Ok(match target {
            ScrapeTarget::All => self.games(None)?,
            ScrapeTarget::Platform(id) => self.games(Some(id))?,
            ScrapeTarget::Game(id) => self
                .games(None)?
                .into_iter()
                .filter(|g| g.id == *id)
                .collect(),
        })
    }

    /// Finds one game on SteamGridDB and saves the kinds it lacks.
    fn scrape_game(&self, sgdb: &SteamGridDb<'_>, game: &GameEntry) -> Result<Outcome, MetaError> {
        let have = [&game.icon, &game.hero, &game.logo];
        let missing: Vec<MediaKind> = KINDS
            .iter()
            .zip(have)
            .filter(|(_, path)| path.is_none())
            .map(|(kind, _)| *kind)
            .collect();
        if missing.is_empty() {
            return Ok(Outcome::Complete);
        }
        let found = sgdb.search(&game.display_title)?;
        let Some(sgdb_id) = pick_game(&found, &game.display_title) else {
            return Ok(Outcome::Saved(0));
        };
        let mut saved = 0;
        for kind in missing {
            if let Some(url) = sgdb.best_image(sgdb_id, kind)? {
                let bytes = sgdb.download(&url)?;
                self.save_art(game, kind, &url, &bytes)
                    .map_err(|_| MetaError::Parse("write".into()))?;
                saved += 1;
            }
        }
        Ok(Outcome::Saved(saved))
    }

    /// Writes a downloaded file and records it for the game.
    fn save_art(
        &self,
        game: &GameEntry,
        kind: MediaKind,
        url: &str,
        bytes: &[u8],
    ) -> Result<(), CoreError> {
        if bytes.is_empty() {
            return Err(CoreError::Io);
        }
        let dir = self.art_dir(&game.platform_id, kind);
        std::fs::create_dir_all(&dir).map_err(|_| CoreError::Io)?;
        let path = dir.join(format!("{}.{}", game.id, extension_of(url)));
        std::fs::write(&path, bytes).map_err(|_| CoreError::Io)?;
        self.db().add_media(&NewMedia {
            game_id: game.id,
            kind,
            path: path.to_string_lossy().into_owned(),
            source: MediaSource::SteamGridDb,
            edit: None,
        })?;
        Ok(())
    }

    /// The folder of the downloaded files of one platform and kind.
    fn art_dir(&self, platform: &str, kind: MediaKind) -> PathBuf {
        self.data_dir
            .join(cfg::SGDB_MEDIA_FOLDER)
            .join(platform)
            .join(kind.as_str())
    }
}

/// Adds one game's outcome to the report.
fn count(report: &mut ImageScrapeReport, outcome: Outcome) {
    match outcome {
        Outcome::Complete => report.complete += 1,
        Outcome::Saved(0) => report.not_found += 1,
        Outcome::Saved(n) => {
            report.updated += 1;
            report.images += n;
        }
    }
}

/// The image extension of a URL, `png` when it has none that is known.
fn extension_of(url: &str) -> String {
    let last = url.rsplit('/').next().unwrap_or_default();
    let ext = last
        .split('?')
        .next()
        .and_then(|name| name.rsplit_once('.'))
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    if cfg::IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        ext
    } else {
        "png".into()
    }
}

#[cfg(test)]
mod tests {
    use super::extension_of;

    /// The extension comes from the file name of the URL, query ignored, `png` otherwise.
    #[test]
    fn extension_from_url() {
        assert_eq!(
            extension_of("https://cdn2.steamgriddb.com/grid/a.WEBP"),
            "webp"
        );
        assert_eq!(
            extension_of("https://cdn2.steamgriddb.com/grid/a.jpg?x=1"),
            "jpg"
        );
        assert_eq!(extension_of("https://cdn2.steamgriddb.com/grid/a"), "png");
        assert_eq!(
            extension_of("https://cdn2.steamgriddb.com/grid/a.exe"),
            "png"
        );
    }
}
