//! Data models for querying and exchanging scraped metadata.

use zl_db::{MediaKind, ScraperSource};

/// Search query used to find metadata across sources.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScrapeQuery {
    /// Normalized game display title.
    pub title: String,
    /// Canonical platform id (e.g. "gba", "nds", "snes").
    pub platform: String,
    /// Path to the ROM file if available.
    pub rom_path: Option<String>,
}

/// A media file referenced by an online scraper result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrapedMedia {
    /// Kind of media (cover icon, hero wallpaper, logo, screenshot, video).
    pub kind: MediaKind,
    /// Web URL or local path to download the media from.
    pub url: String,
}

/// The metadata resolved from a single source.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScrapedGame {
    /// Official title from database.
    pub title: Option<String>,
    /// Synopsis or description.
    pub description: Option<String>,
    /// Release date (YYYY-MM-DD or YYYY).
    pub release_date: Option<String>,
    /// Developer studio.
    pub developer: Option<String>,
    /// Publisher company.
    pub publisher: Option<String>,
    /// Comma-separated genres.
    pub genres: Option<String>,
    /// Maximum player count.
    pub players: Option<u32>,
    /// Rating normalized to 0.0 - 5.0.
    pub rating: Option<f32>,
    /// Media items offered by this source.
    pub media: Vec<ScrapedMedia>,
    /// Which source provided this data.
    pub source: ScraperSource,
}

/// Merges fields from `incoming` into `target` where `target` has `None`.
pub fn merge_scraped(target: &mut ScrapedGame, incoming: ScrapedGame) {
    if target.title.is_none() {
        target.title = incoming.title;
    }
    if target.description.is_none() {
        target.description = incoming.description;
    }
    if target.release_date.is_none() {
        target.release_date = incoming.release_date;
    }
    if target.developer.is_none() {
        target.developer = incoming.developer;
    }
    if target.publisher.is_none() {
        target.publisher = incoming.publisher;
    }
    if target.genres.is_none() {
        target.genres = incoming.genres;
    }
    if target.players.is_none() {
        target.players = incoming.players;
    }
    if target.rating.is_none() {
        target.rating = incoming.rating;
    }
    for m in incoming.media {
        if !target.media.iter().any(|existing| existing.kind == m.kind) {
            target.media.push(m);
        }
    }
}
