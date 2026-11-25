//! Tunable limits, thresholds, and defaults for the Pod system.
//!
//! Defined in `docs/spec/pods.md` (§2, §3, §5, §7, §10).

/// Maximum number of file entries permitted inside a `.zlpod` package.
pub const MAX_PACKAGE_ENTRIES: usize = 2000;

/// Maximum compressed size of a `.zlpod` archive file in bytes (128 MiB).
pub const MAX_PACKAGE_SIZE_BYTES: u64 = 128 * 1024 * 1024;

/// Maximum total uncompressed size across all files in a `.zlpod` archive in bytes (512 MiB).
pub const MAX_TOTAL_UNCOMPRESSED_SIZE_BYTES: u64 = 512 * 1024 * 1024;

/// Maximum uncompressed size of any single entry in bytes (256 MiB).
pub const MAX_ENTRY_UNCOMPRESSED_SIZE_BYTES: u64 = 256 * 1024 * 1024;

/// Maximum compression ratio (uncompressed size / compressed size) permitted for an entry.
pub const MAX_ENTRY_RATIO: u64 = 200;

/// Compression ratio checks are ignored for entries uncompressing to under this size (1 MiB).
pub const ENTRY_RATIO_THRESHOLD_BYTES: u64 = 1024 * 1024;

/// Maximum allowed size for `manifest.json` in bytes (64 KiB).
pub const MAX_MANIFEST_SIZE_BYTES: u64 = 64 * 1024;

/// Current API version supported by the launcher host.
pub const API_CURRENT: u32 = 1;

/// Minimum API version supported by the launcher host.
pub const API_MIN: u32 = 1;

/// Maximum crashes allowed within [`CRASH_WINDOW_HOURS`] before automatic disabling.
pub const MAX_CRASHES: u32 = 3;

/// Window duration in hours for tracking crash health threshold.
pub const CRASH_WINDOW_HOURS: i64 = 24;

/// Maximum lead tiles displayed on a platform grid slot.
pub const MAX_LEAD_TILES: usize = 3;

/// Maximum number of slot declarations evaluated per Pod.
pub const MAX_SLOTS_PER_POD: usize = 16;

/// Default URL for the public Pod store index.
pub const DEFAULT_INDEX_URL: &str =
    "https://raw.githubusercontent.com/Giorey01/zpod-index/main/index.json";

/// Name of the subfolder in the application data folder where installed pods reside.
pub const PODS_DIR_NAME: &str = "pods";
