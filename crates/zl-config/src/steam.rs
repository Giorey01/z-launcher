//! Steam: the official Web API (library import) and the public store API (game details).

/// Base URL of the Steam Web API. The user's own key goes with every call.
pub const STEAM_WEB_API_BASE: &str = "https://api.steampowered.com";

/// Base URL of the public store API (`appdetails`), used without a key.
pub const STEAM_STORE_API_BASE: &str = "https://store.steampowered.com/api";

/// Language asked of the store for descriptions. English is the one every game has; the app
/// does not translate game texts.
pub const STEAM_STORE_LANGUAGE: &str = "english";

/// Country code asked of the store. It only changes prices, which the launcher does not show.
pub const STEAM_STORE_COUNTRY: &str = "us";

/// Wait between two store requests (milliseconds). The store is not documented; the limit
/// commonly reported is about 200 requests every five minutes per address, and 1.6 s keeps a
/// long run under it.
pub const STEAM_STORE_DELAY_MS: u64 = 1_600;

/// Screenshots kept per game from the store (the store lists up to about 20).
pub const STEAM_STORE_MAX_SCREENSHOTS: usize = 8;

/// Name of the folder that receives the `.steamappid` files of an import, when the games folder
/// has no Steam folder yet (Cocoon's folder name).
pub const STEAM_IMPORT_FOLDER: &str = "steam";
