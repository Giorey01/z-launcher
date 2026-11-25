//! Host policy defining connection limits and default headers.

use std::collections::HashMap;

use zl_config::net as cfg;

/// Connection limits and headers required for a specific host.
#[derive(Debug, Clone)]
pub struct HostPolicy {
    /// Parallel connections allowed to the host.
    pub max_connections: usize,
    /// Headers to include in every request to this host.
    pub headers: HashMap<String, String>,
    /// Optional rate limit in requests per second.
    pub rate_limit_per_sec: Option<u32>,
}

impl Default for HostPolicy {
    fn default() -> Self {
        Self {
            max_connections: cfg::DEFAULT_MAX_CONNECTIONS,
            headers: HashMap::new(),
            rate_limit_per_sec: None,
        }
    }
}
