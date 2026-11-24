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
