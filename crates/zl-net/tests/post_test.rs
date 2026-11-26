//! POST of the shared client against a server on the loopback: the body and its type arrive,
//! a 503 is retried, a 4xx is told.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use zl_net::{HttpError, UreqClient};

/// Answers each connection with the next of `statuses`, sending back what it received
/// (`content-type` line + body); the URL.
fn serve(statuses: Vec<u16>, seen: mpsc::Sender<String>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for (stream, status) in listener.incoming().zip(statuses) {
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let (mut length, mut kind) = (0usize, String::new());
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let line = line.trim_end().to_ascii_lowercase();
                if line.is_empty() {
                    break;
                }
                if let Some(v) = line.strip_prefix("content-length: ") {
                    length = v.parse().unwrap();
                }
                if let Some(v) = line.strip_prefix("content-type: ") {
                    kind = v.to_string();
                }
            }
            let mut body = vec![0u8; length];
            reader.read_exact(&mut body).unwrap();
            let echo = format!("{kind}|{}", String::from_utf8_lossy(&body));
            seen.send(echo.clone()).unwrap();
            let head = format!(
                "HTTP/1.1 {status} X\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                echo.len()
            );
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(echo.as_bytes());
        }
    });
    format!("http://127.0.0.1:{port}/api")
}

#[test]
fn a_post_sends_its_body_and_retries_a_busy_server() {
    let (tx, rx) = mpsc::channel();
    let url = serve(vec![503, 200], tx);
    let answer = UreqClient::new()
        .post_bytes(&url, br#"{"q":1}"#, "application/json", &[])
        .unwrap();
    assert_eq!(answer, br#"application/json|{"q":1}"#);
    assert_eq!(rx.try_iter().count(), 2, "the 503 was retried once");
}

#[test]
fn a_refused_post_is_told_and_not_retried() {
    let (tx, rx) = mpsc::channel();
    let url = serve(vec![403, 200], tx);
    let err = UreqClient::new()
        .post_bytes(&url, b"x", "text/plain", &[])
        .unwrap_err();
    assert!(matches!(err, HttpError::Status(403)), "{err}");
    assert_eq!(rx.try_iter().count(), 1);
}
