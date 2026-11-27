//! Why a file could not be hashed.

/// What went wrong hashing a file. The codes are stable: they are stored and shown by the apps.
#[derive(Debug, thiserror::Error)]
pub enum HashError {
    /// The file could not be opened or read (missing, unreadable, storage gone).
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    /// The file, or all its entries, exceed the size limits.
    #[error("too_large")]
    TooLarge,
    /// The archive is encrypted.
    #[error("password")]
    Password,
    /// The file is damaged, truncated or not what its extension says.
    #[error("damaged")]
    Damaged,
    /// The archive uses a method or a size the reader does not support.
    #[error("unsupported")]
    Unsupported,
    /// The archive holds no file.
    #[error("empty")]
    Empty,
    /// The caller asked to stop.
    #[error("cancelled")]
    Cancelled,
}

impl HashError {
    /// The stable code of the error.
    pub fn code(&self) -> &'static str {
        match self {
            HashError::Io(_) => "io",
            HashError::TooLarge => "too_large",
            HashError::Password => "password",
            HashError::Damaged => "damaged",
            HashError::Unsupported => "unsupported",
            HashError::Empty => "empty",
            HashError::Cancelled => "cancelled",
        }
    }

    /// True when trying again later can give another answer (the storage may come back); false
    /// when the file itself is the problem and the answer will not change until it does.
    pub fn is_transient(&self) -> bool {
        matches!(self, HashError::Io(_) | HashError::Cancelled)
    }

    /// Classifies a read error met while decoding: a short or malformed stream means a damaged
    /// file, anything else is the storage.
    pub(crate) fn from_read(err: std::io::Error) -> HashError {
        use std::io::ErrorKind::{InvalidData, InvalidInput, UnexpectedEof};
        match err.kind() {
            UnexpectedEof | InvalidData | InvalidInput => HashError::Damaged,
            _ => HashError::Io(err),
        }
    }
}
