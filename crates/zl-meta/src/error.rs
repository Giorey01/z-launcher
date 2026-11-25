//! Error types for metadata scraping and resolution.

use thiserror::Error;

/// Errors produced during metadata scraping and orchestration.
#[derive(Debug, Error)]
pub enum MetaError {
    /// Network connection or HTTP request failed.
    #[error("Network error: {0}")]
    Network(String),

    /// Missing or invalid API credentials for source.
    #[error("Authentication required for {scraper_id}")]
    AuthRequired {
        /// The scraper source identifier.
        scraper_id: String,
    },

    /// No matching metadata found for game query.
    #[error("No metadata found for game")]
    NotFound,

    /// Failed to parse or serialize metadata payload.
    #[error("Data parsing error: {0}")]
    Parse(String),

    /// Underlying database error.
    #[error("Database error: {0}")]
    Db(#[from] zl_db::DbError),
}
