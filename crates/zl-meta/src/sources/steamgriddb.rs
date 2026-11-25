//! SteamGridDB (API v2): finds a game by name and picks its best cover, hero and logo.
//!
//! Only images on the SteamGridDB CDN are accepted, never NSFW or humor ones; the best image is the
//! one with the most net votes, the API score breaking a tie. Every request carries the user's key
//! as a bearer token; a refused key stops the caller instead of being retried game after game.

use serde::Deserialize;
use zl_config::media as cfg;
use zl_db::MediaKind;
use zl_net::{HttpClient, HttpError};

use crate::error::MetaError;

/// A game found by the name search.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SgdbGame {
    /// SteamGridDB game id.
    pub id: u64,
    /// Name as listed by SteamGridDB.
    pub name: String,
}

/// One image offered for a game.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub struct SgdbImage {
    /// Full-size URL.
    #[serde(default)]
    pub url: String,
    /// Up votes.
    #[serde(default)]
    pub upvotes: i64,
    /// Down votes.
    #[serde(default)]
    pub downvotes: i64,
    /// The API's own score.
    #[serde(default)]
    pub score: i64,
    /// Marked as not safe for work.
    #[serde(default)]
    pub nsfw: bool,
    /// Marked as a joke image.
    #[serde(default)]
    pub humor: bool,
}

/// The envelope of every answer: `{ "success": bool, "data": ... }`.
#[derive(Debug, Deserialize)]
struct Envelope<T> {
    /// Whether the request succeeded.
    success: bool,
    /// The payload (absent on failure).
    data: Option<T>,
}

/// A SteamGridDB client over any [`HttpClient`] (the real one, or a mock in tests).
pub struct SteamGridDb<'a> {
    /// Transport.
    http: &'a dyn HttpClient,
    /// The user's API key.
    key: String,
}

impl<'a> SteamGridDb<'a> {
    /// A client that authenticates with `key`.
    pub fn new(http: &'a dyn HttpClient, key: &str) -> Self {
        Self {
            http,
            key: key.trim().to_string(),
        }
    }

    /// Whether the key is accepted, checked with a one-word search.
    pub fn verify_key(&self) -> Result<(), MetaError> {
        self.search(cfg::SGDB_KEY_CHECK_TERM).map(|_| ())
    }

    /// The games whose name matches `term`, as ranked by SteamGridDB.
    pub fn search(&self, term: &str) -> Result<Vec<SgdbGame>, MetaError> {
        let url = format!(
            "{}/search/autocomplete/{}",
            cfg::STEAMGRIDDB_API_BASE,
            encode_segment(term)
        );
        Ok(self.fetch::<Vec<SgdbGame>>(&url)?.unwrap_or_default())
    }

    /// The best image of `kind` (cover, hero or logo) for the game `game_id`, if any is usable.
    pub fn best_image(&self, game_id: u64, kind: MediaKind) -> Result<Option<String>, MetaError> {
        for query in queries_for(kind) {
            let url = format!(
                "{}/{}/game/{game_id}?{query}",
                cfg::STEAMGRIDDB_API_BASE,
                endpoint_for(kind)
            );
            let images = self.fetch::<Vec<SgdbImage>>(&url)?.unwrap_or_default();
            if let Some(found) = pick_image(&images) {
                return Ok(Some(found));
            }
        }
        Ok(None)
    }

    /// Downloads an image chosen by [`Self::best_image`]; refuses URLs outside the CDN.
    pub fn download(&self, url: &str) -> Result<Vec<u8>, MetaError> {
        if !on_cdn(url) {
            return Err(MetaError::Parse("image outside the SteamGridDB CDN".into()));
        }
        self.http
            .get_bytes(url, &[("User-Agent", cfg::HTTP_USER_AGENT)])
            .map_err(net_error)
    }

    /// GETs an API URL and returns its `data`; 404 and `success: false` are "nothing".
    fn fetch<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<Option<T>, MetaError> {
        let auth = format!("Bearer {}", self.key);
        let headers = [
            ("Authorization", auth.as_str()),
            ("User-Agent", cfg::HTTP_USER_AGENT),
        ];
        let resp = self.http.get(url, &headers).map_err(net_error)?;
        match resp.status {
            401 | 403 => return Err(key_refused()),
            404 => return Ok(None),
            s if !(200..300).contains(&s) => return Err(MetaError::Network(format!("HTTP {s}"))),
            _ => {}
        }
        let parsed: Envelope<T> =
            serde_json::from_str(&resp.body).map_err(|e| MetaError::Parse(e.to_string()))?;
        Ok(if parsed.success { parsed.data } else { None })
    }
}

/// The error for a key SteamGridDB refuses.
fn key_refused() -> MetaError {
    MetaError::AuthRequired {
        scraper_id: "steamgriddb".into(),
    }
}

/// Maps a transport error; a refused status becomes [`key_refused`].
fn net_error(e: HttpError) -> MetaError {
    match e {
        HttpError::Status(401 | 403) => key_refused(),
        other => MetaError::Network(other.to_string()),
    }
}

/// The API path of a kind of image.
fn endpoint_for(kind: MediaKind) -> &'static str {
    match kind {
        MediaKind::Hero => "heroes",
        MediaKind::Logo => "logos",
        _ => "grids",
    }
}

/// The filters to try in order for a kind: covers square first, then portrait.
fn queries_for(kind: MediaKind) -> &'static [&'static str] {
    match kind {
        MediaKind::Hero => &[cfg::SGDB_HERO_QUERY],
        MediaKind::Logo => &[cfg::SGDB_LOGO_QUERY],
        _ => &[cfg::SGDB_GRID_QUERY, cfg::SGDB_GRID_FALLBACK_QUERY],
    }
}

/// Whether `url` is on the SteamGridDB CDN (a prefix check, so `evil.example/cdn2...` fails).
fn on_cdn(url: &str) -> bool {
    url.starts_with(cfg::SGDB_CDN_PREFIX)
}

/// The comparable form of a name: accents and apostrophes gone, lower case, single spaces.
fn name_key(name: &str) -> String {
    zl_media::title_key(&name.replace(['\'', '’'], ""))
}

/// The game that is `title`: an exact name first (wherever it is listed), else a listed name that
/// is the start of the title up to a word boundary (the longest such). `None` when nothing fits.
pub fn pick_game(results: &[SgdbGame], title: &str) -> Option<u64> {
    let wanted = name_key(title);
    if wanted.is_empty() {
        return None;
    }
    if let Some(exact) = results.iter().find(|g| name_key(&g.name) == wanted) {
        return Some(exact.id);
    }
    results
        .iter()
        .map(|g| (g.id, name_key(&g.name)))
        .filter(|(_, key)| !key.is_empty() && wanted.starts_with(&format!("{key} ")))
        .max_by_key(|(_, key)| key.len())
        .map(|(id, _)| id)
}

/// The URL of the best clean image on the CDN: most net votes, then highest score, then the
/// first listed.
pub fn pick_image(images: &[SgdbImage]) -> Option<String> {
    let net = |i: &SgdbImage| i.upvotes - i.downvotes;
    let mut best: Option<&SgdbImage> = None;
    for image in images
        .iter()
        .filter(|i| !i.nsfw && !i.humor && on_cdn(&i.url))
    {
        let better = best.is_none_or(|b| (net(image), image.score) > (net(b), b.score));
        if better {
            best = Some(image);
        }
    }
    best.map(|i| i.url.clone())
}

/// Percent-encodes one path segment (everything but unreserved characters).
fn encode_segment(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::encode_segment;

    /// Spaces, accents and slashes are escaped so the name stays one path segment.
    #[test]
    fn encodes_a_path_segment() {
        assert_eq!(encode_segment("Pokémon X/Y"), "Pok%C3%A9mon%20X%2FY");
        assert_eq!(encode_segment("Mario-Kart_7.~"), "Mario-Kart_7.~");
    }
}
