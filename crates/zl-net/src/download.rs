//! Downloads of big files to disk: streamed in chunks, told to a [Progress], stoppable, and
//! resumed with a `Range` request from the `.part` a stopped or failed download left. Addresses
//! are tried IPv4 first with a short connect timeout (on a network whose IPv6 route is broken
//! the first address would never answer); there is a read timeout but no limit on the whole
//! download.

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::Duration;

use zl_config::net as cfg;

use crate::error::HttpError;

/// Told the bytes the file holds so far and the total when the server says it; `false` stops
/// the download with [`HttpError::Cancelled`] (the `.part` stays, so it can resume).
pub type Progress<'a> = &'a mut dyn FnMut(u64, Option<u64>) -> bool;

/// HTTP status of an answer that carries only the range asked.
const PARTIAL_CONTENT: u16 = 206;

/// The downloader: one agent shared by every download.
pub struct Downloader {
    /// The ureq agent (connect and read timeouts, IPv4 first).
    agent: ureq::Agent,
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}

impl Downloader {
    /// A downloader with the timeouts of [`zl_config::net`].
    pub fn new() -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(cfg::DOWNLOAD_CONNECT_SECS))
            .timeout_read(Duration::from_secs(cfg::DOWNLOAD_READ_SECS))
            .resolver(ipv4_first)
            .build();
        Self { agent }
    }

    /// Downloads `url` into the file `dest` (written as `dest.part`, renamed at the end) with the
    /// request `headers`; the size of the file. A `dest.part` already there is resumed when the
    /// server answers the range, else started over.
    pub fn download(
        &self,
        url: &str,
        dest: &Path,
        headers: &[(&str, &str)],
        progress: Progress<'_>,
    ) -> Result<u64, HttpError> {
        let part = part_of(dest);
        let have = std::fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
        let mut request = self.agent.get(url);
        for (k, v) in headers {
            request = request.set(k, v);
        }
        if have > 0 {
            request = request.set("Range", &format!("bytes={have}-"));
        }
        let response = request.call().map_err(|e| match e {
            ureq::Error::Status(code, _) => HttpError::Status(code),
            ureq::Error::Transport(t) => HttpError::Network(t.to_string()),
        })?;
        let resumed = response.status() == PARTIAL_CONTENT;
        let start = if resumed { have } else { 0 };
        let total = response
            .header("Content-Length")
            .and_then(|v| v.parse::<u64>().ok())
            .map(|len| len + start);
        let mut file = open_part(&part, resumed)?;
        let size = copy(
            &mut response.into_reader(),
            &mut file,
            start,
            total,
            progress,
        )?;
        drop(file);
        std::fs::rename(&part, dest).map_err(io)?;
        Ok(size)
    }
}

/// The `.part` file next to `dest`.
fn part_of(dest: &Path) -> PathBuf {
    let mut name = dest.as_os_str().to_owned();
    name.push(".part");
    PathBuf::from(name)
}

/// The `.part` file opened to append to (a resumed download) or emptied.
fn open_part(part: &Path, resumed: bool) -> Result<std::fs::File, HttpError> {
    if let Some(dir) = part.parent() {
        std::fs::create_dir_all(dir).map_err(io)?;
    }
    OpenOptions::new()
        .create(true)
        .write(true)
        .append(resumed)
        .truncate(!resumed)
        .open(part)
        .map_err(io)
}

/// A file error as an [`HttpError::Io`].
fn io(e: std::io::Error) -> HttpError {
    HttpError::Io(e.to_string())
}

/// The addresses of `host:port`, IPv4 ones first (each family in the system's order).
fn ipv4_first(netloc: &str) -> std::io::Result<Vec<SocketAddr>> {
    let mut addrs: Vec<SocketAddr> = netloc.to_socket_addrs()?.collect();
    addrs.sort_by_key(|a| !a.is_ipv4());
    Ok(addrs)
}

/// Copies `from` into `to` from byte `start`, telling `progress` before the first chunk and
/// after each one; the bytes the file holds at the end. A read failure is a network error.
fn copy(
    from: &mut dyn Read,
    to: &mut dyn Write,
    start: u64,
    total: Option<u64>,
    progress: Progress<'_>,
) -> Result<u64, HttpError> {
    let mut buf = vec![0u8; cfg::DOWNLOAD_CHUNK];
    let mut done = start;
    if !progress(done, total) {
        return Err(HttpError::Cancelled);
    }
    loop {
        let n = from
            .read(&mut buf)
            .map_err(|e| HttpError::Network(e.to_string()))?;
        if n == 0 {
            return Ok(done);
        }
        to.write_all(&buf[..n]).map_err(io)?;
        done += n as u64;
        if !progress(done, total) {
            return Err(HttpError::Cancelled);
        }
    }
}

#[cfg(test)]
mod tests;
