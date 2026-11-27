//! Tunables of the ROM hashing (CRC32, MD5, SHA1 of the library's files, in the background).

/// Largest plain file hashed, in bytes. A PS2 DVD is under 8.5 GB and a Wii disc under 8 GB; a
/// bigger file (a Switch dump, a raw disk image) is recorded as "too large" and left alone, so a
/// phone does not spend minutes on one game.
pub const MAX_FILE_BYTES: u64 = 8 * 1024 * 1024 * 1024;

/// Largest archive file (zip or 7z) opened, in bytes on disk.
pub const MAX_ARCHIVE_BYTES: u64 = 8 * 1024 * 1024 * 1024;

/// Largest single archive entry hashed, once decompressed, in bytes. Bounds what a zip bomb can
/// make the device decompress; real ROMs and disc images fit well under it.
pub const MAX_ENTRY_BYTES: u64 = 8 * 1024 * 1024 * 1024;

/// Most entries hashed from one archive. A ROM set zipped as one file can hold thousands of
/// small files; past this number the rest is skipped and the archive is marked as truncated.
pub const MAX_ARCHIVE_ENTRIES: usize = 2000;

/// Bytes read at a time while hashing. Large enough to keep the three digests fast, small enough
/// to leave the stop request answered within a few milliseconds.
pub const READ_CHUNK_BYTES: usize = 256 * 1024;

/// Extensions (lower case, no dot) of member files that are not worth hashing: playlists and
/// sheets that only describe other files, and the tag files of Android and Steam entries.
pub const SKIPPED_EXTENSIONS: [&str; 5] = ["m3u", "cue", "gdi", "ccd", "localgameid"];

/// Pause between two games, in milliseconds, so a long run does not keep the storage and the CPU
/// saturated while the user is browsing the library.
pub const PAUSE_BETWEEN_GAMES_MS: u64 = 5;

/// Memory a 7z decoder may reserve for its dictionary, in KiB. A 7z made with the largest
/// dictionary asks for more than a phone can spare; such an archive is reported as unsupported
/// instead of risking the process.
pub const MAX_7Z_MEMORY_KB: usize = 256 * 1024;
