//! Metadata scraper orchestrator and source integrations.
//!
//! Features:
//! - Multi-source scraper pipeline: Libretro DB, ScreenScraper, SteamGridDB, IGDB, LaunchBox, HLTB.
//! - Three scraping modes: MissingOnly, ChosenFields, ForceAll.
//! - Immune to destructive overwrites: protected user overrides are always preserved.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

pub mod error;
pub mod model;
pub mod orchestrator;
pub mod sources;

pub use error::MetaError;
pub use model::{merge_scraped, ScrapeQuery, ScrapedGame, ScrapedMedia};
pub use orchestrator::{default_source_priority, Orchestrator};
pub use sources::steamgriddb::SteamGridDbSource;
pub use sources::MetaSource;
