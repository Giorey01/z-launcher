//! Steam: the user's library from the official Web API, and the public store details of a game.
//!
//! - [`account`] reads what the user typed as their account (a SteamID64, a profile URL or a
//!   custom profile name) and checks the shape of a Web API key.
//! - [`web`] asks the Web API for the owned games (`IPlayerService/GetOwnedGames`) and turns a
//!   custom profile name into a SteamID64 (`ISteamUser/ResolveVanityURL`).
//! - [`store`] reads the public `appdetails` of one app: description, dates, developers,
//!   genres, screenshots, trailers. No key is needed.
//!
//! Everything is synchronous over [`zl_net::HttpClient`], so the tests run on the mock client.
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

pub mod account;
pub mod error;
pub mod store;
pub mod web;

pub use account::{is_web_api_key, parse_account, AccountInput};
pub use error::SteamError;
pub use store::{SteamStore, StoreApp};
pub use web::{OwnedGame, SteamWeb};
