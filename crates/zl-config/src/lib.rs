//! Every tunable number and fixed string of Z-Launcher, in one place.
//!
//! A constant lives here when it is a choice someone could revisit: a timeout, a threshold, a
//! budget, a default. Constants that are facts of a file format (magic bytes, the value ranges a
//! theme format defines) stay next to the parser that reads them.
//!
//! Group constants in a module per theme and give each one a doc comment that says what it
//! controls and why it has that value.
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

pub mod app;
