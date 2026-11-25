//! Scraper source implementations.

pub mod steamgriddb;

use crate::error::MetaError;
use crate::model::{ScrapeQuery, ScrapedGame};
use zl_db::ScraperSource;

/// Common interface implemented by every metadata source.
pub trait MetaSource: Send + Sync {
    /// Identifier of this source.
    fn source_id(&self) -> ScraperSource;

    /// Attempts to query metadata for the given game.
    fn query(&self, query: &ScrapeQuery) -> Result<Option<ScrapedGame>, MetaError>;
}
