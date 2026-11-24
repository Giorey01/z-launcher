//! The library scanner: walks the platform folders under the user's ROM roots and keeps the
//! `zl-db` game list in step with the disk.
//!
//! How it works:
//! 1. A root holds one folder per platform; the folder name (case-insensitive) is the platform id
//!    and the caller's [`ExtensionRules`] say which extensions count as games there.
//! 2. Each platform folder is walked ([`walk`]) and its files are collapsed into games
//!    ([`group`]): tag files, `.m3u` playlists (which hide the files they list), `.cue`/`.gdi`
//!    sheets (which hide their tracks), `(Disc N)` siblings and bare-disc folders.
//! 3. The result is compared with the database by fingerprint ([`scan`]): unchanged games are
//!    only touched, changed ones are rewritten, games no longer found are flagged missing (never
//!    deleted) and come back when they reappear.
//!
//! The scanner never depends on the platform database: the caller passes the rules in.
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

mod discs;
mod events;
mod fingerprint;
mod group;
mod rules;
mod scan;
mod sheets;
mod walk;

pub use events::{IssueCode, NoProgress, ScanEvent, ScanIssue, ScanProgress, ScanSummary};
pub use rules::{ExtensionRules, FolderGameRule};
pub use scan::{scan, LibraryError, RootSpec};
