//! Tunables of the library scanner (folder walk, incremental rescan, progress).

/// Deepest level of sub-folders the scanner enters below a platform folder when the root has
/// "scan subfolders" on (0 = only the platform folder itself). Eight covers `Genre/Series/Game/`
/// layouts with room to spare and bounds the walk on a pathological tree.
pub const MAX_SCAN_DEPTH: usize = 8;

/// A "files seen" progress event is sent every this many files, so the UI gets a live counter
/// without being flooded (a 5000-game library produces about ten events).
pub const PROGRESS_EVERY_FILES: u64 = 500;

/// Most per-file problems kept in the scan summary; further ones are only counted. Stops a
/// broken storage with thousands of unreadable files from building a huge list.
pub const MAX_REPORTED_ISSUES: usize = 200;

/// Largest M3U or CUE or GDI file the scanner reads, in bytes. Real playlists and sheets are
/// under a few KiB; a bigger file is not one and is treated as a plain ROM.
pub const MAX_SHEET_BYTES: u64 = 1024 * 1024;

/// Largest tag file the scanner reads, in bytes (they hold a handful of short lines).
pub const MAX_TAG_FILE_BYTES: u64 = 64 * 1024;

/// Sub-folder names (lower-case) the scanner never enters, at any depth below a platform folder.
/// They hold updates and DLC of Switch games and similar add-ons, which are not games to list.
/// Matching ignores case.
pub const EXCLUDED_SUBFOLDER_NAMES: [&str; 3] = ["updates_dlc", "updates", "dlc"];
