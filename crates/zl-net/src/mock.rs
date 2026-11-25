//! Mock HTTP client for offline fixture testing.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::client::{HttpClient, HttpResponse};
use crate::error::HttpError;

/// Recorded response mapping for the mock client.
struct RegisteredResponse {
    /// HTTP status code.
    status: u16,
    /// Response body string.
    body: String,
    /// Response headers.
    headers: HashMap<String, String>,
}

/// Offline HTTP client that responds using registered fixtures.
#[derive(Default)]
pub struct MockHttpClient {
    /// Exact and prefix URL mappings to mock responses.
    responses: Mutex<HashMap<String, RegisteredResponse>>,
    /// Recorded URLs requested during tests.
    history: Mutex<Vec<String>>,
}

impl MockHttpClient {
    /// Creates a new empty mock client.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a success response with status 200 for a given URL or pattern.
    pub fn register(&self, url_or_pattern: &str, body: &str) {
        self.register_status(url_or_pattern, 200, body);
    }

    /// Registers a response with a specific status code and body.
    pub fn register_status(&self, url_or_pattern: &str, status: u16, body: &str) {
        let mut map = self.responses.lock().unwrap_or_else(|e| e.into_inner());
        map.insert(
            url_or_pattern.to_string(),
            RegisteredResponse {
                status,
                body: body.to_string(),
                headers: HashMap::new(),
            },
        );
    }

    /// Returns the history of requested URLs.
    pub fn requested_urls(&self) -> Vec<String> {
        self.history
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

impl HttpClient for MockHttpClient {
    fn get(&self, url: &str, _headers: &[(&str, &str)]) -> Result<HttpResponse, HttpError> {
        self.history
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(url.to_string());
        let map = self.responses.lock().unwrap_or_else(|e| e.into_inner());
        // Exact match first, then prefix/contains match
        let found = map.get(url).or_else(|| {
            map.iter()
                .find(|(pat, _)| url.contains(pat.as_str()))
                .map(|(_, r)| r)
        });
        let Some(resp) = found else {
            return Err(HttpError::Status(404));
        };
        Ok(HttpResponse {
            status: resp.status,
            body: resp.body.clone(),
            headers: resp.headers.clone(),
        })
    }

    fn get_bytes(&self, url: &str, headers: &[(&str, &str)]) -> Result<Vec<u8>, HttpError> {
        let resp = self.get(url, headers)?;
        if !resp.is_success() {
            return Err(HttpError::Status(resp.status));
        }
        Ok(resp.body.into_bytes())
    }
}
