//! Metadata orchestrator: prioritizes sources, applies scrape modes and respects user overrides.

use crate::error::MetaError;
use crate::model::{merge_scraped, ScrapeQuery, ScrapedGame};
use crate::sources::MetaSource;
use zl_db::{GameMeta, MetaField, MetaOverride, ScrapeMode, ScraperSource};

/// Metadata scraper orchestrator.
pub struct Orchestrator {
    /// Ordered list of scraper sources by priority.
    sources: Vec<Box<dyn MetaSource>>,
}

impl Orchestrator {
    /// Creates a new orchestrator with the given sources in priority order.
    pub fn new(sources: Vec<Box<dyn MetaSource>>) -> Self {
        Self { sources }
    }

    /// Queries the sources in priority order and merges the results according to `mode`.
    pub fn scrape(
        &self,
        query: &ScrapeQuery,
        existing: Option<&GameMeta>,
        mode: ScrapeMode,
        overrides: &[MetaOverride],
    ) -> Result<ScrapedGame, MetaError> {
        let mut aggregated = ScrapedGame::default();

        for source in &self.sources {
            if is_complete(&aggregated, mode) {
                break;
            }
            if let Ok(Some(data)) = source.query(query) {
                merge_scraped(&mut aggregated, data);
            }
        }

        apply_existing_if_missing_only(&mut aggregated, existing, mode);
        protect_overrides(&mut aggregated, overrides);
        Ok(aggregated)
    }
}

/// Checks whether all requested fields are populated.
fn is_complete(scraped: &ScrapedGame, mode: ScrapeMode) -> bool {
    if mode == ScrapeMode::ChosenFields {
        return false;
    }
    scraped.title.is_some()
        && scraped.description.is_some()
        && scraped.release_date.is_some()
        && scraped.developer.is_some()
        && scraped.publisher.is_some()
        && scraped.genres.is_some()
}

/// Preserves existing metadata when running in [`ScrapeMode::MissingOnly`].
fn apply_existing_if_missing_only(
    scraped: &mut ScrapedGame,
    existing: Option<&GameMeta>,
    mode: ScrapeMode,
) {
    if mode != ScrapeMode::MissingOnly {
        return;
    }
    let Some(ex) = existing else { return };
    if ex.description.is_some() {
        scraped.description = ex.description.clone();
    }
    if ex.release_date.is_some() {
        scraped.release_date = ex.release_date.clone();
    }
    if ex.developer.is_some() {
        scraped.developer = ex.developer.clone();
    }
    if ex.publisher.is_some() {
        scraped.publisher = ex.publisher.clone();
    }
    if ex.genres.is_some() {
        scraped.genres = ex.genres.clone();
    }
    if ex.players.is_some() {
        scraped.players = ex.players;
    }
    if ex.rating.is_some() {
        scraped.rating = ex.rating;
    }
}

/// Overwrites scraped fields with protected user overrides.
fn protect_overrides(scraped: &mut ScrapedGame, overrides: &[MetaOverride]) {
    for ov in overrides {
        match ov.field {
            MetaField::Title => scraped.title = Some(ov.value.clone()),
            MetaField::Description => scraped.description = Some(ov.value.clone()),
            MetaField::ReleaseDate => scraped.release_date = Some(ov.value.clone()),
            MetaField::Developer => scraped.developer = Some(ov.value.clone()),
            MetaField::Publisher => scraped.publisher = Some(ov.value.clone()),
            MetaField::Genres => scraped.genres = Some(ov.value.clone()),
            MetaField::Players => scraped.players = ov.value.parse().ok(),
            MetaField::Rating => scraped.rating = ov.value.parse().ok(),
        }
    }
}

/// Returns the default priority order of metadata sources as recommended.
pub fn default_source_priority() -> Vec<ScraperSource> {
    vec![ScraperSource::SteamGridDb]
}
