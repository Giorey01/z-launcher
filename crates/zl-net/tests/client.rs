//! Tests for HTTP client trait, retry logic and mock client.

use zl_net::{HttpClient, MockHttpClient};

/// Test result type.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Tests that the mock client responds with registered payload and records history.
#[test]
fn test_mock_client_records_and_responds() -> TestResult {
    let mock = MockHttpClient::new();
    mock.register("https://example.com/api/test", r#"{"success":true}"#);

    let resp = mock.get("https://example.com/api/test", &[])?;
    assert_eq!(resp.status, 200);
    assert_eq!(resp.body, r#"{"success":true}"#);
    assert!(resp.is_success());

    let urls = mock.requested_urls();
    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0], "https://example.com/api/test");
    Ok(())
}

/// Tests that mock client returns 404 for unmapped requests.
#[test]
fn test_mock_client_unmapped_returns_404() {
    let mock = MockHttpClient::new();
    let res = mock.get("https://unknown.com", &[]);
    assert!(res.is_err());
}

/// Tests that get_bytes returns the raw byte payload.
#[test]
fn test_mock_client_get_bytes() -> TestResult {
    let mock = MockHttpClient::new();
    mock.register("https://example.com/image.png", "PNG_BYTES");

    let bytes = mock.get_bytes("https://example.com/image.png", &[])?;
    assert_eq!(bytes, b"PNG_BYTES");
    Ok(())
}
