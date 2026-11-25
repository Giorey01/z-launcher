//! Online game art and metadata. Today one source: SteamGridDB, for covers, heroes and logos
//! (`sources::steamgriddb`). The text metadata comes from ES-DE `gamelist.xml` files, read by
//! `zl-media`.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

pub mod error;
pub mod sources;

pub use error::MetaError;
pub use sources::steamgriddb::{pick_game, pick_image, SgdbGame, SgdbImage, SteamGridDb};
use zl_db::ScraperSource;

/// The online sources in the order they are asked.
pub fn default_source_priority() -> Vec<ScraperSource> {
    vec![ScraperSource::SteamGridDb]
}
