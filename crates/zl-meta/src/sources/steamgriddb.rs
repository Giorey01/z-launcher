//! SteamGridDB API client and artwork parser.

use serde::Deserialize;

use crate::error::MetaError;
use crate::model::{ScrapeQuery, ScrapedGame, ScrapedMedia};
use crate::sources::MetaSource;
use zl_db::{MediaKind, ScraperSource};

/// SteamGridDB scraper source.
#[derive(Debug, Default)]
pub struct SteamGridDbSource {
    /// API authentication key.
    pub api_key: Option<String>,
}

/// Generic SteamGridDB API response wrapper.
#[derive(Debug, Deserialize)]
struct SgdbResponse<T> {
    /// Whether the request succeeded.
    success: bool,
    /// Response payload data.
    data: Option<T>,
}

/// An individual artwork item returned by SteamGridDB.
#[derive(Debug, Deserialize)]
struct SgdbImageItem {
    /// Direct image URL.
    url: Option<String>,
}

impl MetaSource for SteamGridDbSource {
    fn source_id(&self) -> ScraperSource {
        ScraperSource::SteamGridDb
    }

    fn query(&self, query: &ScrapeQuery) -> Result<Option<ScrapedGame>, MetaError> {
        if query.title.is_empty() {
            return Ok(None);
        }
        Ok(None)
    }
}

/// Parses SteamGridDB grids JSON into a list of cover [`ScrapedMedia`].
pub fn parse_sgdb_grids_json(json_str: &str) -> Result<Vec<ScrapedMedia>, MetaError> {
    let parsed: SgdbResponse<Vec<SgdbImageItem>> =
        serde_json::from_str(json_str).map_err(|e| MetaError::Parse(e.to_string()))?;
    if !parsed.success {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    if let Some(items) = parsed.data {
        for item in items {
            if let Some(url) = item.url {
                out.push(ScrapedMedia {
                    kind: MediaKind::Icon,
                    url,
                });
            }
        }
    }
    Ok(out)
}

/// Parses SteamGridDB heroes or logos JSON into [`ScrapedMedia`].
pub fn parse_sgdb_art_json(
    json_str: &str,
    kind: MediaKind,
) -> Result<Vec<ScrapedMedia>, MetaError> {
    let parsed: SgdbResponse<Vec<SgdbImageItem>> =
        serde_json::from_str(json_str).map_err(|e| MetaError::Parse(e.to_string()))?;
    if !parsed.success {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    if let Some(items) = parsed.data {
        for item in items {
            if let Some(url) = item.url {
                out.push(ScrapedMedia { kind, url });
            }
        }
    }
    Ok(out)
}
