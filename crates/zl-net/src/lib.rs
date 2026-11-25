//! Synchronous HTTP client, retry policy and mock implementation.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

pub mod client;
pub mod error;
pub mod host;
pub mod mock;
pub mod ureq_client;

pub use client::{HttpClient, HttpResponse};
pub use error::HttpError;
pub use host::HostPolicy;
pub use mock::MockHttpClient;
pub use ureq_client::UreqClient;
