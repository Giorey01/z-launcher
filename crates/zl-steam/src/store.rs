//! The public store API: `appdetails?appids=<id>` answers `{"<id>": {"success": bool, "data":
//! {...}}}` with the store page of one app. No key; the store limits how fast it may be asked
//! (the caller waits between apps, see `zl_config::steam::STEAM_STORE_DELAY_MS`).
//!
//! The description is the store's short one, as plain text. Dates come as `10 Oct, 2007` or
//! `Oct 10, 2007` in English and become `2007-10-10`; any other form is kept as written.
//! Trailers prefer the MP4 file, then WebM, then the HLS playlist newer pages carry.

use std::collections::HashMap;

use serde::Deserialize;
use zl_config::media::HTTP_USER_AGENT;
use zl_config::steam as cfg;
use zl_net::{HttpClient, HttpError};

use crate::error::SteamError;

/// Months as the English store writes them. A format fact.
const MONTHS: [&str; 12] = [
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
];

/// What the store says about one app, ready for the library.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StoreApp {
    /// Name on the store.
    pub name: String,
    /// Short description, plain text.
    pub description: Option<String>,
    /// Release date: `YYYY-MM-DD` when it could be read, else as the store wrote it.
    pub release_date: Option<String>,
    /// Developers, joined with `, `.
    pub developer: Option<String>,
    /// Publishers, joined with `, `.
    pub publisher: Option<String>,
    /// Genres, joined with `, `.
    pub genres: Option<String>,
    /// Metacritic score as a 0 to 5 rating.
    pub rating: Option<f32>,
    /// Full-size screenshot URLs, in store order.
    pub screenshots: Vec<String>,
    /// Trailer URLs, in store order.
    pub trailers: Vec<String>,
    /// The wide header image.
    pub header_image: Option<String>,
}

/// One `{ "description": ... }` entry (genres).
#[derive(Debug, Deserialize)]
struct Named {
    /// The text.
    description: String,
}

/// A screenshot.
#[derive(Debug, Deserialize)]
struct Shot {
    /// Full-size URL.
    path_full: String,
}

/// A trailer, in whichever encodings the page has.
#[derive(Debug, Deserialize)]
struct Movie {
    /// `{"480": url, "max": url}`.
    #[serde(default)]
    mp4: HashMap<String, String>,
    /// Same for WebM.
    #[serde(default)]
    webm: HashMap<String, String>,
    /// HLS playlist of newer pages.
    hls_h264: Option<String>,
}

/// The release date block.
#[derive(Debug, Deserialize)]
struct Release {
    /// The date as written.
    #[serde(default)]
    date: String,
}

/// Metacritic block.
#[derive(Debug, Deserialize)]
struct Metacritic {
    /// Score out of 100.
    score: u32,
}

/// The `data` of an answer, the fields used.
#[derive(Debug, Deserialize)]
struct Data {
    /// Name.
    #[serde(default)]
    name: String,
    /// Short description (HTML entities possible).
    short_description: Option<String>,
    /// Developers.
    #[serde(default)]
    developers: Vec<String>,
    /// Publishers.
    #[serde(default)]
    publishers: Vec<String>,
    /// Release date.
    release_date: Option<Release>,
    /// Genres.
    #[serde(default)]
    genres: Vec<Named>,
    /// Screenshots.
    #[serde(default)]
    screenshots: Vec<Shot>,
    /// Trailers.
    #[serde(default)]
    movies: Vec<Movie>,
    /// Header image.
    header_image: Option<String>,
    /// Metacritic score.
    metacritic: Option<Metacritic>,
}

/// One app's entry of the answer.
#[derive(Debug, Deserialize)]
struct Entry {
    /// `false` for an unknown or region-locked app.
    success: bool,
    /// The page.
    data: Option<Data>,
}

/// A store client over any [`HttpClient`].
pub struct SteamStore<'a> {
    /// Transport.
    http: &'a dyn HttpClient,
}

impl<'a> SteamStore<'a> {
    /// A client over `http`.
    pub fn new(http: &'a dyn HttpClient) -> Self {
        Self { http }
    }

    /// The store page of `appid`; `None` when the store has no page for it.
    pub fn app(&self, appid: u32) -> Result<Option<StoreApp>, SteamError> {
        let url = format!(
            "{}/appdetails?appids={appid}&l={}&cc={}",
            cfg::STEAM_STORE_API_BASE,
            cfg::STEAM_STORE_LANGUAGE,
            cfg::STEAM_STORE_COUNTRY
        );
        let resp = self
            .http
            .get(&url, &[("User-Agent", HTTP_USER_AGENT)])
            .map_err(|e: HttpError| SteamError::Network(e.to_string()))?;
        if !resp.is_success() {
            return Err(SteamError::Network(format!("http {}", resp.status)));
        }
        let mut parsed: HashMap<String, Entry> =
            serde_json::from_str(&resp.body).map_err(|e| SteamError::Parse(e.to_string()))?;
        let entry = parsed.remove(&appid.to_string());
        Ok(entry
            .filter(|e| e.success)
            .and_then(|e| e.data)
            .map(store_app))
    }
}

/// Joins a list with `, `; `None` when empty.
fn joined(items: impl IntoIterator<Item = String>) -> Option<String> {
    let list: Vec<String> = items.into_iter().filter(|s| !s.trim().is_empty()).collect();
    (!list.is_empty()).then(|| list.join(", "))
}

/// The best URL of a trailer.
fn trailer_url(movie: &Movie) -> Option<String> {
    movie
        .mp4
        .get("max")
        .or_else(|| movie.webm.get("max"))
        .cloned()
        .or_else(|| movie.hls_h264.clone())
}

/// Builds the library view of a page.
fn store_app(d: Data) -> StoreApp {
    StoreApp {
        name: d.name,
        description: d
            .short_description
            .as_deref()
            .map(plain_text)
            .filter(|s| !s.is_empty()),
        release_date: d
            .release_date
            .map(|r| iso_date(&r.date).unwrap_or(r.date))
            .filter(|s| !s.is_empty()),
        developer: joined(d.developers),
        publisher: joined(d.publishers),
        genres: joined(d.genres.into_iter().map(|g| g.description)),
        rating: d.metacritic.map(|m| m.score.min(100) as f32 / 20.0),
        screenshots: d
            .screenshots
            .into_iter()
            .map(|s| s.path_full)
            .take(cfg::STEAM_STORE_MAX_SCREENSHOTS)
            .collect(),
        trailers: d.movies.iter().filter_map(trailer_url).collect(),
        header_image: d.header_image,
    }
}

/// `10 Oct, 2007` or `Oct 10, 2007` as `2007-10-10`; `None` for any other form.
pub fn iso_date(text: &str) -> Option<String> {
    let parts: Vec<&str> = text.split([' ', ',']).filter(|p| !p.is_empty()).collect();
    let [a, b, year] = parts.as_slice() else {
        return None;
    };
    let month_of = |s: &str| {
        let lower = s.to_ascii_lowercase();
        MONTHS
            .iter()
            .position(|m| lower.starts_with(m))
            .map(|i| i + 1)
    };
    let (day, month) = match (a.parse::<u32>(), month_of(b)) {
        (Ok(day), Some(month)) => (day, month),
        _ => (b.parse::<u32>().ok()?, month_of(a)?),
    };
    let year: u32 = year.parse().ok()?;
    ((1..=31).contains(&day) && year >= 1970).then(|| format!("{year:04}-{month:02}-{day:02}"))
}

/// HTML of the store as plain text: tags dropped, the common entities decoded, spaces collapsed
/// (and none left before a punctuation mark).
pub fn plain_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    let decoded = out
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&trade;", "™")
        .replace("&reg;", "®")
        .replace("&copy;", "©")
        .replace("&amp;", "&");
    let mut text = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
    for mark in [".", ",", ";", ":", "!", "?"] {
        text = text.replace(&format!(" {mark}"), mark);
    }
    text
}
