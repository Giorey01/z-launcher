//! How the library is organised: Smart Folders by time, manual folders and sort orders.

/// A game counts as new in the New folder for this many days after it was added to the library.
/// Two weeks is long enough to cover a weekend of adding ROMs and short enough that the folder
/// empties by itself.
pub const NEW_GAME_DAYS: i64 = 14;

/// Seconds in a day, to turn [`NEW_GAME_DAYS`] into a time span.
pub const SECONDS_PER_DAY: i64 = 24 * 60 * 60;

/// Games in the Most played folder, longest play time first.
pub const MOST_PLAYED_LIMIT: usize = 30;

/// Longest name of a manual folder, in characters: more does not fit under a tile.
pub const MAX_FOLDER_NAME_CHARS: usize = 40;

/// Most manual folders the user can create; stops a runaway loop, far above real use.
pub const MAX_MANUAL_FOLDERS: usize = 200;
