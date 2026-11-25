//! Errors of the application service: stable codes the apps translate, never sentences.

use zl_db::DbError;
use zl_library::LibraryError;
use zl_platforms::{DbError as DataError, LaunchError};
use zl_theme::ThemeError;

/// Everything the service can fail with.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CoreError {
    /// The library database failed (open, read or write).
    #[error("database")]
    Database,
    /// The database file comes from a newer version of the app.
    #[error("database_too_new")]
    DatabaseTooNew,
    /// The platform and emulator data could not be loaded; `code` is the data error code.
    #[error("data {code}")]
    Data {
        /// Stable code of the problem (`parse`, `schema_mismatch`, ...).
        code: String,
    },
    /// A launch could not be planned; `code` is the launch error code and `detail` its subject.
    #[error("launch {code} {detail}")]
    Launch {
        /// Stable code (`unknown_platform`, `missing_variable`, ...).
        code: String,
        /// Id or name the code is about, possibly empty.
        detail: String,
    },
    /// The game, root or platform asked for does not exist.
    #[error("not_found")]
    NotFound,
    /// An argument is not usable (empty path, malformed profile reference).
    #[error("invalid_input")]
    InvalidInput,
    /// A theme file or archive could not be imported; `code` says why (`zip`, `io`, `no_theme`,
    /// `unsafe_path`, `symlink`, `too_many_entries`, `too_large`, `compression_ratio`,
    /// `missing_manifest`).
    #[error("theme {code}")]
    Theme {
        /// Stable code of the problem.
        code: String,
    },
    /// The dock already holds the maximum number of shortcuts.
    #[error("dock_full")]
    DockFull,
    /// File I/O failed.
    #[error("io")]
    Io,
}

impl From<DbError> for CoreError {
    /// Maps a storage failure to its code.
    fn from(e: DbError) -> Self {
        match e {
            DbError::NewerSchema { .. } => CoreError::DatabaseTooNew,
            _ => CoreError::Database,
        }
    }
}

impl From<LibraryError> for CoreError {
    /// A scan only aborts on database errors.
    fn from(e: LibraryError) -> Self {
        match e {
            LibraryError::Db(inner) => inner.into(),
        }
    }
}

impl From<DataError> for CoreError {
    /// Keeps the first word of the data error (its code).
    fn from(e: DataError) -> Self {
        let text = e.to_string();
        let code = match &e {
            DataError::Invalid { code, .. } => (*code).to_string(),
            _ => text.split(' ').next().unwrap_or_default().to_string(),
        };
        CoreError::Data { code }
    }
}

impl From<LaunchError> for CoreError {
    /// Splits the launch error text into its code and subject.
    fn from(e: LaunchError) -> Self {
        let text = e.to_string();
        let (code, detail) = text.split_once(' ').unwrap_or((&text, ""));
        CoreError::Launch {
            code: code.to_string(),
            detail: detail.to_string(),
        }
    }
}

impl From<ThemeError> for CoreError {
    /// Maps the failure of an import to its code.
    fn from(e: ThemeError) -> Self {
        let code = match e {
            ThemeError::Io(_) => "io",
            ThemeError::Zip(_) => "zip",
            ThemeError::MissingManifest => "missing_manifest",
            ThemeError::NoThemeInArchive => "no_theme",
            ThemeError::UnsafePath(_) => "unsafe_path",
            ThemeError::SymlinkEntry(_) => "symlink",
            ThemeError::TooManyEntries => "too_many_entries",
            ThemeError::TooLarge => "too_large",
            ThemeError::CompressionRatio(_) => "compression_ratio",
        };
        CoreError::Theme {
            code: code.to_string(),
        }
    }
}
