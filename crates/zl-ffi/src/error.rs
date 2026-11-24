//! The error the apps see: a code (and the subject it is about), never a sentence.

use zl_core::CoreError as ServiceError;

/// Failure of a core call. Kotlin receives it as `CoreException`; the app maps each variant to
/// a translated message.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum CoreError {
    /// The library database failed.
    #[error("database")]
    Database,
    /// The database was written by a newer app.
    #[error("database_too_new")]
    DatabaseTooNew,
    /// The platform and emulator data could not be loaded.
    #[error("data {code}")]
    Data {
        /// Stable code of the problem.
        code: String,
    },
    /// A launch could not be planned.
    #[error("launch {code} {detail}")]
    Launch {
        /// Stable code (`unknown_platform`, `missing_variable`, ...).
        code: String,
        /// What the code is about, possibly empty.
        detail: String,
    },
    /// The game, root or platform does not exist.
    #[error("not_found")]
    NotFound,
    /// An argument is not usable.
    #[error("invalid_input")]
    InvalidInput,
    /// The dock has no free slot.
    #[error("dock_full")]
    DockFull,
}

impl From<ServiceError> for CoreError {
    /// One to one.
    fn from(e: ServiceError) -> Self {
        match e {
            ServiceError::Database => CoreError::Database,
            ServiceError::DatabaseTooNew => CoreError::DatabaseTooNew,
            ServiceError::Data { code } => CoreError::Data { code },
            ServiceError::Launch { code, detail } => CoreError::Launch { code, detail },
            ServiceError::NotFound => CoreError::NotFound,
            ServiceError::InvalidInput => CoreError::InvalidInput,
            ServiceError::DockFull => CoreError::DockFull,
        }
    }
}
