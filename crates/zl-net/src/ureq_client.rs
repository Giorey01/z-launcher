//! Synchronous HTTP client implementation using `ureq` and `rustls`.

use std::collections::HashMap;
use std::time::Duration;

use zl_config::net as cfg;

use crate::client::{HttpClient, HttpResponse};
use crate::error::HttpError;
use crate::host::HostPolicy;

/// The next backoff: doubled, up to the configured maximum.
fn grow(backoff: Duration) -> Duration {
    (backoff * 2).min(Duration::from_secs(cfg::MAX_BACKOFF_SECS))
}

/// How long to wait before retrying a response, or `None` if final.
fn wait_before_retry(
    status: u16,
    retry_after: Option<&str>,
    attempt: u32,
    backoff: Duration,
) -> Option<Duration> {
    if (200..400).contains(&status) || attempt > cfg::MAX_RETRIES {
        return None;
    }
    if status == 429 {
        let parsed = retry_after.and_then(|s| s.parse::<u64>().ok());
        return Some(parsed.map_or(backoff, Duration::from_secs));
    }
    if status >= 500 {
        return Some(backoff);
    }
    None
}

/// Real HTTP client wrapping `ureq::Agent` with retry and backoff logic.
pub struct UreqClient {
    /// Underlying ureq agent.
    agent: ureq::Agent,
    /// Optional host policy for headers and connection limits.
    policy: Option<HostPolicy>,
}

impl UreqClient {
    /// Creates a new `UreqClient` with default timeout and policy.
    pub fn new() -> Self {
        Self::with_options(None, None)
    }

    /// Creates a new client with custom timeout and host policy.
    pub fn with_options(timeout_secs: Option<u64>, policy: Option<HostPolicy>) -> Self {
        let secs = timeout_secs.unwrap_or(cfg::REQUEST_TIMEOUT_SECS);
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(secs))
            .build();
        Self { agent, policy }
    }

    /// Prepares a GET request with policy headers and custom headers.
    fn prepare_request(&self, url: &str, headers: &[(&str, &str)]) -> ureq::Request {
        let mut req = self.agent.get(url);
        if let Some(p) = &self.policy {
            for (k, v) in &p.headers {
                req = req.set(k, v);
            }
        }
        for (k, v) in headers {
            req = req.set(k, v);
        }
        req
    }

    /// Reads an HTTP response into an [`HttpResponse`].
    fn read_response(&self, resp: ureq::Response) -> Result<HttpResponse, HttpError> {
        let status = resp.status();
        let mut headers = HashMap::new();
        for name in resp.headers_names() {
            if let Some(val) = resp.header(&name) {
                headers.insert(name.to_ascii_lowercase(), val.to_string());
            }
        }
        let body = resp
            .into_string()
            .map_err(|e| HttpError::Io(e.to_string()))?;
        Ok(HttpResponse {
            status,
            body,
            headers,
        })
    }
}

impl Default for UreqClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient for UreqClient {
    fn get(&self, url: &str, headers: &[(&str, &str)]) -> Result<HttpResponse, HttpError> {
        let mut backoff = Duration::from_millis(cfg::INITIAL_BACKOFF_MS);
        for attempt in 1.. {
            let req = self.prepare_request(url, headers);
            match req.call() {
                Ok(resp) => return self.read_response(resp),
                Err(ureq::Error::Status(status, resp)) => {
                    let ra = resp.header("Retry-After").map(str::to_string);
                    match wait_before_retry(status, ra.as_deref(), attempt, backoff) {
                        Some(wait) => std::thread::sleep(wait),
                        None => return self.read_response(resp),
                    }
                }
                Err(ureq::Error::Transport(err)) => {
                    if attempt > cfg::MAX_RETRIES {
                        return Err(HttpError::Network(err.to_string()));
                    }
                    std::thread::sleep(backoff);
                }
            }
            backoff = grow(backoff);
        }
        Err(HttpError::MaxRetriesExceeded(url.to_string()))
    }

    fn get_bytes(&self, url: &str, headers: &[(&str, &str)]) -> Result<Vec<u8>, HttpError> {
        let mut backoff = Duration::from_millis(cfg::INITIAL_BACKOFF_MS);
        for attempt in 1.. {
            let req = self.prepare_request(url, headers);
            match req.call() {
                Ok(resp) => {
                    let mut bytes = Vec::new();
                    use std::io::Read;
                    resp.into_reader()
                        .read_to_end(&mut bytes)
                        .map_err(|e| HttpError::Io(e.to_string()))?;
                    return Ok(bytes);
                }
                Err(ureq::Error::Status(status, resp)) => {
                    let ra = resp.header("Retry-After").map(str::to_string);
                    match wait_before_retry(status, ra.as_deref(), attempt, backoff) {
                        Some(wait) => std::thread::sleep(wait),
                        None => return Err(HttpError::Status(status)),
                    }
                }
                Err(ureq::Error::Transport(err)) => {
                    if attempt > cfg::MAX_RETRIES {
                        return Err(HttpError::Network(err.to_string()));
                    }
                    std::thread::sleep(backoff);
                }
            }
            backoff = grow(backoff);
        }
        Err(HttpError::MaxRetriesExceeded(url.to_string()))
    }
}
