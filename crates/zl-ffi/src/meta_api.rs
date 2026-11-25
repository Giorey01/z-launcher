//! The metadata half of the [`Core`] object: SteamGridDB key and art downloads, ES-DE gamelists,
//! and the details of one game.

use std::sync::Arc;
use std::time::Duration;

use zl_config::media as cfg;
use zl_core as core;
use zl_net::UreqClient;

use crate::core_object::Core;
use crate::error::CoreError;

/// Result of a core call.
type Res<T> = Result<T, CoreError>;

mirror_enum! {
    /// Why an art download ended before the last game.
    ScrapeStop <=> core::ScrapeStop { MissingKey, KeyRefused, Offline, Cancelled }
}
mirror_enum! {
    /// The answer of SteamGridDB to a key.
    KeyCheck <=> core::KeyCheck { Valid, Refused, Offline }
}
mirror_record! {
    /// The numbers of an art download.
    ImageScrapeReport <=> core::ImageScrapeReport {
        /// Games covered.
        plain games: u32,
        /// Games that already had cover, hero and logo.
        plain complete: u32,
        /// Games that got at least one file.
        plain updated: u32,
        /// Files downloaded.
        plain images: u32,
        /// Games SteamGridDB had nothing for.
        plain not_found: u32,
        /// Games that failed.
        plain failed: u32,
        /// Why the run ended early.
        opt stopped: ScrapeStop,
    }
}
mirror_record! {
    /// Everything known about one game, for the game info card.
    GameDetails <=> core::GameDetails {
        /// Row id.
        plain id: i64,
        /// Canonical platform id.
        plain platform_id: String,
        /// Platform name.
        plain platform_name: String,
        /// Stored title.
        plain title: String,
        /// Title to show.
        plain display_title: String,
        /// Synopsis.
        opt description: String,
        /// Release date (`YYYY-MM-DD`, or as the source wrote it).
        opt release_date: String,
        /// Developer.
        opt developer: String,
        /// Publisher.
        opt publisher: String,
        /// Genres.
        opt genres: String,
        /// Maximum players.
        opt players: u32,
        /// Rating 0-5.
        opt rating: f32,
        /// Cover.
        opt icon: String,
        /// Hero image.
        opt hero: String,
        /// Logo.
        opt logo: String,
        /// Screenshots.
        list screenshots: String,
        /// Video.
        opt video: String,
    }
}

/// Receiver of an art download's progress, implemented in Kotlin; it can cancel the run.
#[uniffi::export(with_foreign)]
pub trait ScrapeListener: Send + Sync {
    /// `done` of `total` games are finished.
    fn on_game(&self, done: u32, total: u32);
    /// Whether the user asked to stop.
    fn is_cancelled(&self) -> bool;
}

/// Adapts the foreign listener to the core's progress trait.
struct Bridge(Arc<dyn ScrapeListener>);

impl core::ScrapeProgress for Bridge {
    /// Forwards to Kotlin.
    fn on_game(&self, done: u32, total: u32) {
        self.0.on_game(done, total);
    }
    /// Asks Kotlin.
    fn is_cancelled(&self) -> bool {
        self.0.is_cancelled()
    }
}

#[uniffi::export]
impl Core {
    /// Whether a SteamGridDB key is stored (the key itself never goes back to the UI).
    pub fn has_steamgriddb_key(&self) -> Res<bool> {
        Ok(self.service().scraper_config()?.steamgriddb_key.is_some())
    }

    /// Stores the SteamGridDB key; `None` removes it. A malformed key is `InvalidInput`.
    pub fn set_steamgriddb_key(&self, key: Option<String>) -> Res<()> {
        Ok(self.service().set_steamgriddb_key(key.as_deref())?)
    }

    /// Asks SteamGridDB whether it accepts `key` (network call; nothing stored).
    pub fn check_steamgriddb_key(&self, key: String) -> KeyCheck {
        self.service()
            .check_steamgriddb_key(&key, &UreqClient::new())
            .into()
    }

    /// Downloads the missing cover, hero and logo of one game (`game_id`), of one platform, or
    /// of the whole library when both are `None`. Blocks: call it off the main thread.
    pub fn scrape_images(
        &self,
        platform: Option<String>,
        game_id: Option<i64>,
        listener: Arc<dyn ScrapeListener>,
    ) -> Res<ImageScrapeReport> {
        let target = match (game_id, platform) {
            (Some(id), _) => core::ScrapeTarget::Game(id),
            (None, Some(p)) => core::ScrapeTarget::Platform(p),
            (None, None) => core::ScrapeTarget::All,
        };
        let pause = Duration::from_millis(cfg::SGDB_PAUSE_MS);
        let report =
            self.service()
                .scrape_images(&target, &UreqClient::new(), &Bridge(listener), pause)?;
        Ok(report.into())
    }

    /// Imports the metadata of every ES-DE gamelist of the ES-DE folder; returns the games
    /// that got metadata.
    pub fn import_esde_gamelists(&self) -> Res<u32> {
        Ok(self.service().import_esde_gamelists()?)
    }

    /// The details of one game (metadata with the user's overrides, and its best art).
    pub fn game_details(&self, game_id: i64) -> Res<GameDetails> {
        Ok(self.service().game_details(game_id)?.into())
    }
}
