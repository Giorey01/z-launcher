//! The downloader against a small HTTP server on the loopback that answers ranges.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::thread;

/// Serves `body` to `requests` connections on a free port, answering a `Range: bytes=N-` with
/// a 206 of the rest when `ranges`; the base URL.
fn serve(body: Vec<u8>, requests: usize, ranges: bool) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for stream in listener.incoming().take(requests) {
            let mut stream = stream.unwrap();
            let mut from = 0usize;
            for line in BufReader::new(stream.try_clone().unwrap()).lines() {
                let line = line.unwrap();
                if line.is_empty() {
                    break;
                }
                if let Some(v) = line.to_ascii_lowercase().strip_prefix("range: bytes=") {
                    from = v.trim_end_matches('-').parse().unwrap();
                }
            }
            let (status, rest) = if ranges && from > 0 {
                ("206 Partial Content", &body[from..])
            } else {
                ("200 OK", &body[..])
            };
            let head = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                rest.len()
            );
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(rest);
        }
    });
    format!("http://127.0.0.1:{port}/file.bin")
}

/// A body of `n` bytes that differ from one another along the file.
fn body(n: usize) -> Vec<u8> {
    (0..n).map(|i| (i % 251) as u8).collect()
}

#[test]
fn a_download_streams_to_the_file_and_tells_its_progress() {
    let data = body(cfg::DOWNLOAD_CHUNK * 3 + 17);
    let url = serve(data.clone(), 1, true);
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("mod.zip");
    let mut seen = Vec::new();
    let size = Downloader::new()
        .download(&url, &dest, &[], &mut |done, total| {
            seen.push((done, total));
            true
        })
        .unwrap();
    assert_eq!(size, data.len() as u64);
    assert_eq!(std::fs::read(&dest).unwrap(), data);
    assert!(!part_of(&dest).exists());
    assert_eq!(seen.first(), Some(&(0, Some(data.len() as u64))));
    assert_eq!(
        seen.last(),
        Some(&(data.len() as u64, Some(data.len() as u64)))
    );
}

#[test]
fn a_stopped_download_resumes_from_its_part() {
    let data = body(cfg::DOWNLOAD_CHUNK * 4);
    let url = serve(data.clone(), 2, true);
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("mod.7z");
    let downloader = Downloader::new();
    let stopped = downloader.download(&url, &dest, &[], &mut |done, _| {
        done < cfg::DOWNLOAD_CHUNK as u64
    });
    assert!(matches!(stopped, Err(HttpError::Cancelled)));
    let kept = std::fs::metadata(part_of(&dest)).unwrap().len();
    // A network read can return less than a chunk, so the stop lands on the first read that
    // reaches one chunk: at least a chunk is kept, never the whole file.
    assert!(kept >= cfg::DOWNLOAD_CHUNK as u64, "kept {kept}");
    assert!(kept < data.len() as u64, "kept {kept}");
    let mut first = None;
    downloader
        .download(&url, &dest, &[], &mut |done, _| {
            first.get_or_insert(done);
            true
        })
        .unwrap();
    assert_eq!(
        first,
        Some(kept),
        "the second request starts where the part ends"
    );
    assert_eq!(std::fs::read(&dest).unwrap(), data);
}

#[test]
fn a_server_without_ranges_starts_over() {
    let data = body(1000);
    let url = serve(data.clone(), 1, false);
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("mod.rar");
    std::fs::write(part_of(&dest), b"stale bytes").unwrap();
    Downloader::new()
        .download(&url, &dest, &[], &mut |_, _| true)
        .unwrap();
    assert_eq!(std::fs::read(&dest).unwrap(), data);
}

#[test]
fn an_error_status_is_told() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        let mut s = listener.incoming().next().unwrap().unwrap();
        let mut buf = [0u8; 1024];
        let _ = s.read(&mut buf);
        let _ = s
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    });
    let dir = tempfile::tempdir().unwrap();
    let r = Downloader::new().download(
        &format!("http://127.0.0.1:{port}/x"),
        &dir.path().join("x"),
        &[],
        &mut |_, _| true,
    );
    assert!(matches!(r, Err(HttpError::Status(404))));
}

#[test]
fn ipv4_addresses_come_first() {
    let addrs = ipv4_first("localhost:80").unwrap_or_default();
    let first_v6 = addrs
        .iter()
        .position(SocketAddr::is_ipv6)
        .unwrap_or(addrs.len());
    assert!(
        addrs[first_v6..].iter().all(SocketAddr::is_ipv6),
        "{addrs:?}"
    );
}
