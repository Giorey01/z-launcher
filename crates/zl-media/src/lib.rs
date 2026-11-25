//! Readers of the media other apps keep next to the library (covers, fan art, logos,
//! screenshots, videos) and the rules that tie a file to a game.
//!
//! Two layouts are read: ES-DE's `downloaded_media/<system>/<folder>/<rom stem>.<ext>` and
//! Cocoon's `downloaded_media/<platform>/<kind>/<rom stem>.<ext>` (with `__cocoon_edit_` files for
//! the user's own edits). The readers only list files; they know nothing about the database. A
//! caller matches the result to games with [`game_keys`] and stores it with `zl-db`.
//!
//! [`display_title`] also lives here: the title shown on screen, without the tags in the file
//! name.
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

mod cocoon;
mod esde;
mod found;
pub mod gamelist;
mod keys;
mod title;
mod walk;

pub use cocoon::{parse_cocoon_name, scan_cocoon};
pub use esde::{esde_media_dir, scan_esde};
pub use found::FoundMedia;
pub use gamelist::{export_gamelist, parse_gamelist, GamelistEntry};
pub use keys::game_keys;
pub use title::{display_title, match_key, normalize, slugify, strip_accents, title_key};
