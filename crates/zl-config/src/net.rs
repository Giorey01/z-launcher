//! Network tunables for the shared HTTP client: timeouts, retries and host limits.

/// Overall timeout (seconds) of a request of the shared client.
pub const REQUEST_TIMEOUT_SECS: u64 = 30;

/// How many times a failed request is retried before giving up.
pub const MAX_RETRIES: u32 = 3;

/// Wait (milliseconds) before the first retry; it doubles after each one.
pub const INITIAL_BACKOFF_MS: u64 = 100;

/// The wait between retries never grows past this many seconds.
pub const MAX_BACKOFF_SECS: u64 = 5;

/// Default number of parallel connections to a host that has no policy of its own.
pub const DEFAULT_MAX_CONNECTIONS: usize = 4;

/// Seconds to connect to one address of a download before trying the next one (a host whose
/// IPv6 route is broken never answers on it).
pub const DOWNLOAD_CONNECT_SECS: u64 = 4;

/// Seconds a download may go without receiving a byte before it fails. There is no limit on
/// the whole download: a big file on a slow link takes as long as it takes.
pub const DOWNLOAD_READ_SECS: u64 = 30;

/// Bytes read from the network at a time by a download.
pub const DOWNLOAD_CHUNK: usize = 64 * 1024;
