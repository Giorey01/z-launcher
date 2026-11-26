//! Limits applied when importing a theme ZIP, so a hostile or broken archive cannot exhaust the
//! device's storage or memory.

/// Maximum number of entries (files and folders) a theme ZIP may contain. A full theme with
/// sounds, overlays and smart folders stays well under a thousand; the margin is for packs that
/// bundle several themes.
pub const MAX_ZIP_ENTRIES: usize = 4000;

/// Maximum total size of everything a theme ZIP expands to, in bytes (512 MiB). Wallpaper videos
/// are the bulk of a real theme; this is several times larger than any exported theme seen.
pub const MAX_ZIP_TOTAL_UNCOMPRESSED_BYTES: u64 = 512 * 1024 * 1024;

/// Maximum expanded size of a single ZIP entry, in bytes (256 MiB). Stops one giant file from
/// using the whole total budget.
pub const MAX_ZIP_ENTRY_UNCOMPRESSED_BYTES: u64 = 256 * 1024 * 1024;

/// Maximum ratio between an entry's expanded size and its compressed size. Real media is already
/// compressed (ratio near 1) and JSON compresses about 10 to 1; a "zip bomb" is orders of magnitude
/// above this.
pub const MAX_ZIP_COMPRESSION_RATIO: u64 = 200;

/// Entries smaller than this many expanded bytes are exempt from the ratio check, because tiny
/// files (a few hundred zero bytes) can legitimately compress by a large factor.
pub const ZIP_RATIO_CHECK_MIN_BYTES: u64 = 1024 * 1024;

/// How many folder levels below the content root of an installed asset pack are searched for
/// music files. Real packs keep tracks in `music/` or `music/music/`; a small depth also stops a
/// malformed pack with a deep tree from making the scan slow.
pub const PACK_MUSIC_SCAN_DEPTH: usize = 4;

/// Folders where Cocoon's data folder (its `themes/`, `assets/` and `downloaded_media/`) may be
/// on a device, tried in order by the "find the Cocoon data" helper. The first entry is the
/// folder Cocoon asks its users to pick on a handheld; add the others seen in the field.
pub const COCOON_DATA_DIR_CANDIDATES: [&str; 1] = ["/storage/emulated/0/Documents"];

/// Name of the folder (inside the app data folder) that holds the user's own artwork for smart
/// folders and icon overlays. It has the layout of a theme folder (`smart_folders/<type>/icon.png`,
/// `smart_folders/by_platform/<id>/hero.png`, ...), so the same reader serves it, and it wins
/// over every pack and theme.
pub const USER_ART_DIR_NAME: &str = "user";

/// Folder of the data directory with the launcher's own music (the app copies its built-in
/// tracks there): it plays when the theme and its packs have none.
pub const DEFAULT_MUSIC_DIR_NAME: &str = "default-music";

/// Name of the folder (inside the app data folder) where a theme ZIP is extracted before its
/// themes are moved to `themes/`; deleted right after.
pub const IMPORT_STAGING_DIR_NAME: &str = "import-staging";
