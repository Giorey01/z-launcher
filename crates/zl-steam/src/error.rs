//! Errors of the Steam clients: codes for the app to translate, never sentences.

use thiserror::Error;

/// Why a Steam call failed.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SteamError {
    /// The Web API refused the key (HTTP 401 or 403).
    #[error("key_refused")]
    KeyRefused,
    /// The profile hides its game details, so the owned games come back empty.
    #[error("private_profile")]
    PrivateProfile,
    /// No profile has that custom name.
    #[error("unknown_profile")]
    UnknownProfile,
    /// Steam could not be reached, or answered with an error status.
    #[error("network {0}")]
    Network(String),
    /// The answer was not the JSON expected.
    #[error("parse {0}")]
    Parse(String),
}
