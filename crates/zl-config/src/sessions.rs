//! Play sessions and the Smart Folders built on them.

/// A session shorter than this (seconds) is not recorded: the game was opened and closed, or
/// failed to start.
pub const MIN_SESSION_SECONDS: i64 = 30;

/// A launch older than this (seconds) with no return to the launcher is dropped instead of
/// being counted: the device was probably restarted while the game ran.
pub const MAX_PENDING_SECONDS: i64 = 24 * 60 * 60;

/// Longest session (seconds) counted when the usage events cannot be read and the time is the
/// launch-to-return time: someone who left a game open overnight did not play all of it.
pub const MAX_ESTIMATED_SECONDS: i64 = 4 * 60 * 60;

/// Games in the Recent folder, newest first.
pub const RECENT_LIMIT: usize = 30;
