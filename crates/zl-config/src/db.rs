//! Settings of the SQLite database that holds the library.

/// How long a statement waits for a lock held by another connection before failing, in
/// milliseconds. The scanner writes while the UI reads; WAL keeps readers unblocked, so this only
/// matters when two writers meet, and a few seconds is longer than any scan transaction.
pub const BUSY_TIMEOUT_MS: u64 = 5000;
