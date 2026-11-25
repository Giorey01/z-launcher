//! Synchronous HTTP client interface and response models.

use std::collections::HashMap;

use crate::error::HttpError;

/// An HTTP response payload received from a server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    /// HTTP status code (e.g. 200, 404, 429).
    pub status: u16,
    /// Response body as a UTF-8 text string.
    pub body: String,
    /// HTTP response headers with lower-cased keys.
    pub headers: HashMap<String, String>,
}

impl HttpResponse {
    /// Returns the value of a header, matching case-insensitively.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(&name.to_ascii_lowercase())
            .map(String::as_str)
    }

    /// Returns `true` if the HTTP status code is 2xx.
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// Synchronous HTTP client trait for executing requests.
pub trait HttpClient: Send + Sync {
    /// Performs a GET request and returns the decoded text response.
    fn get(&self, url: &str, headers: &[(&str, &str)]) -> Result<HttpResponse, HttpError>;

    /// Performs a GET request and returns the raw response bytes.
    fn get_bytes(&self, url: &str, headers: &[(&str, &str)]) -> Result<Vec<u8>, HttpError>;
}
