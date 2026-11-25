//! The official Steam Web API with the user's own key: owned games and custom profile names.
//!
//! `GetOwnedGames` answers `{"response": {"game_count": n, "games": [...]}}`; a profile whose
//! game details are private answers `{"response": {}}`, which is told apart from an empty
//! library (`game_count: 0`). A refused key is HTTP 401 or 403.

use serde::de::DeserializeOwned;
use serde::Deserialize;
use zl_config::media::HTTP_USER_AGENT;
use zl_config::steam::STEAM_WEB_API_BASE;
use zl_net::{HttpClient, HttpError};

use crate::account::AccountInput;
use crate::error::SteamError;

/// `success` value of `ResolveVanityURL` when the name was found. A format fact.
const VANITY_FOUND: i64 = 1;

/// One game of the library.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OwnedGame {
    /// Steam app id.
    pub appid: u32,
    /// Name as Steam lists it.
    #[serde(default)]
    pub name: String,
    /// Minutes played in total.
    #[serde(default, rename = "playtime_forever")]
    pub playtime_minutes: u64,
    /// Unix time of the last session, 0 when never played.
    #[serde(default, rename = "rtime_last_played")]
    pub last_played: i64,
}

/// The `response` of `GetOwnedGames`.
#[derive(Debug, Deserialize)]
struct OwnedResponse {
    /// Absent for a private profile.
    game_count: Option<u32>,
    /// The games.
    #[serde(default)]
    games: Vec<OwnedGame>,
}

/// The `response` of `ResolveVanityURL`.
#[derive(Debug, Deserialize)]
struct VanityResponse {
    /// 1 when found.
    success: i64,
    /// The SteamID64, as text.
    steamid: Option<String>,
}

/// The `{"response": ...}` envelope of both calls.
#[derive(Debug, Deserialize)]
struct Envelope<T> {
    /// The payload.
    response: T,
}

/// A Web API client over any [`HttpClient`].
pub struct SteamWeb<'a> {
    /// Transport.
    http: &'a dyn HttpClient,
    /// The user's key.
    key: String,
}

impl<'a> SteamWeb<'a> {
    /// A client that sends `key` with every call.
    pub fn new(http: &'a dyn HttpClient, key: &str) -> Self {
        Self {
            http,
            key: key.trim().to_string(),
        }
    }

    /// The SteamID64 of an account: as given, or resolved from its custom name.
    pub fn steam_id(&self, account: &AccountInput) -> Result<u64, SteamError> {
        match account {
            AccountInput::SteamId(id) => Ok(*id),
            AccountInput::Vanity(name) => self.resolve_vanity(name),
        }
    }

    /// The SteamID64 of the custom profile name `name`.
    pub fn resolve_vanity(&self, name: &str) -> Result<u64, SteamError> {
        let url = format!(
            "{STEAM_WEB_API_BASE}/ISteamUser/ResolveVanityURL/v1/?key={}&vanityurl={name}",
            self.key
        );
        let found: VanityResponse = self.fetch(&url)?;
        let id = found.steamid.and_then(|s| s.parse::<u64>().ok());
        match id {
            Some(id) if found.success == VANITY_FOUND => Ok(id),
            _ => Err(SteamError::UnknownProfile),
        }
    }

    /// The games `steam_id` owns, free games they played included, with names.
    pub fn owned_games(&self, steam_id: u64) -> Result<Vec<OwnedGame>, SteamError> {
        let url = format!(
            "{STEAM_WEB_API_BASE}/IPlayerService/GetOwnedGames/v1/?key={}&steamid={steam_id}\
             &include_appinfo=1&include_played_free_games=1&format=json",
            self.key
        );
        let owned: OwnedResponse = self.fetch(&url)?;
        match owned.game_count {
            None => Err(SteamError::PrivateProfile),
            Some(_) => Ok(owned.games),
        }
    }

    /// GETs `url` and reads the `response` of the envelope.
    fn fetch<T: DeserializeOwned>(&self, url: &str) -> Result<T, SteamError> {
        let resp = self
            .http
            .get(url, &[("User-Agent", HTTP_USER_AGENT)])
            .map_err(net_error)?;
        match resp.status {
            401 | 403 => return Err(SteamError::KeyRefused),
            s if !(200..300).contains(&s) => return Err(SteamError::Network(format!("http {s}"))),
            _ => {}
        }
        let parsed: Envelope<T> =
            serde_json::from_str(&resp.body).map_err(|e| SteamError::Parse(e.to_string()))?;
        Ok(parsed.response)
    }
}

/// Maps a transport error; a refused status is a refused key.
fn net_error(e: HttpError) -> SteamError {
    match e {
        HttpError::Status(401 | 403) => SteamError::KeyRefused,
        other => SteamError::Network(other.to_string()),
    }
}
