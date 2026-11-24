//! Errors of the database layer.

/// Everything that can go wrong talking to the database.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// SQLite reported an error.
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// The database was written by a newer version of the app than this one.
    #[error("database schema {found} is newer than the supported {supported}")]
    NewerSchema {
        /// `user_version` found in the file.
        found: u32,
        /// Highest version this build knows.
        supported: u32,
    },
    /// A row holds a value that is not a known enum spelling.
    #[error("invalid stored value: {0}")]
    InvalidValue(String),
}
