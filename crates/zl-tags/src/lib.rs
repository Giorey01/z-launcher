//! Reader for Cocoon-style "file tags": small text files that stand in for a game in a platform
//! folder (a Steam app id, an Android package, a ScummVM game id, ...). The file name without its
//! extension is the game's name and each line is `[key] value`.
//!
//! Parsing is tolerant: a bad line never aborts the file, it is reported as a [`TagIssue`] with
//! its line number and a structured code that the apps translate.
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

mod kind;
mod parse;

pub use kind::TagKind;
pub use parse::{
    parent_folder_name, parse_tag_file, read_tag_file, IssueCode, ParsedTag, TagError, TagFile,
    TagIssue,
};
