//! Error types produced by network and HTTP calls.

use thiserror::Error;

/// Reasons an HTTP request can fail.
#[derive(Debug, Error)]
pub enum HttpError {
    /// Non-successful HTTP status code returned by the server.
    #[error("HTTP status {0}")]
    Status(u16),

    /// Every retry attempt failed.
    #[error("Max retries exceeded: {0}")]
    MaxRetriesExceeded(String),

    /// Network connection or transport failed.
    #[error("Network error: {0}")]
    Network(String),

    /// Response could not be read or decoded.
    #[error("I/O error: {0}")]
    Io(String),

    /// The caller asked to stop.
    #[error("cancelled")]
    Cancelled,
}
