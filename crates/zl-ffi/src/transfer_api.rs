//! HTTP requests, downloads and archive extraction for the app and the Pods (`PodHost`): GET and
//! POST with retries, a file streamed to
//! disk with progress, stop and resume ([`zl_net::Downloader`]), and zip, 7z and RAR extracted
//! with checked paths ([`zl_archive`]). Failures are codes the app translates.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

use zl_archive::{ArchiveError, ArchiveExtractor, ArchiveFailure, ExtractOptions};
use zl_net::{Downloader, HttpClient, HttpError, UreqClient};

/// Bytes between two calls of a [TransferListener].
const PROGRESS_STEP: u64 = 256 * 1024;

/// Told how a download or an extraction goes, implemented in Kotlin; it can stop the work.
#[uniffi::export(with_foreign)]
pub trait TransferListener: Send + Sync {
    /// `done` bytes of `total` (None when not known), at most every 256 KB and at the end.
    fn on_progress(&self, done: u64, total: Option<u64>);
    /// Whether the user asked to stop.
    fn is_cancelled(&self) -> bool;
}

/// Why a download or an extraction failed.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum TransferError {
    /// The server could not be reached or the connection dropped.
    #[error("network: {detail}")]
    Network {
        /// What the transport said, for the log.
        detail: String,
    },
    /// The server answered with this status.
    #[error("status {code}")]
    Status {
        /// The HTTP status.
        code: u16,
    },
    /// Stopped because the caller asked.
    #[error("cancelled")]
    Cancelled,
    /// Not an archive this build reads (by its extension).
    #[error("unsupported {format}")]
    Unsupported {
        /// The extension.
        format: String,
    },
    /// An entry would land outside the destination.
    #[error("unsafe entry {entry}")]
    Unsafe {
        /// The entry's name.
        entry: String,
    },
    /// The archive is encrypted and the password is missing or wrong.
    #[error("password")]
    Password,
    /// The archive is damaged or incomplete.
    #[error("damaged: {detail}")]
    Damaged {
        /// What the reader said.
        detail: String,
    },
    /// The disk is full.
    #[error("no space")]
    NoSpace,
    /// Reading or writing a file failed.
    #[error("io: {detail}")]
    Io {
        /// What the system said.
        detail: String,
    },
}

/// The downloader shared by every download.
fn downloader() -> &'static Downloader {
    static DOWNLOADER: OnceLock<Downloader> = OnceLock::new();
    DOWNLOADER.get_or_init(Downloader::new)
}

/// Downloads `url` into the file `dest` with the request `headers`, telling `listener` as it
/// goes; the size of the file. A stopped or failed download leaves `dest.part`, and the next
/// call for the same `dest` resumes it when the server can.
#[uniffi::export]
pub fn download_file(
    url: String,
    dest: String,
    headers: HashMap<String, String>,
    listener: Arc<dyn TransferListener>,
) -> Result<u64, TransferError> {
    let headers: Vec<(&str, &str)> = headers
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let mut told = None::<u64>;
    let mut progress = |done: u64, total: Option<u64>| {
        if listener.is_cancelled() {
            return false;
        }
        let end = total.is_some_and(|t| done >= t);
        if told.is_none_or(|t| end || done >= t + PROGRESS_STEP) {
            told = Some(done);
            listener.on_progress(done, total);
        }
        true
    };
    Ok(downloader().download(&url, Path::new(&dest), &headers, &mut progress)?)
}

/// Extracts the archive `archive` (.zip, .7z or the first volume of a .rar set) into the folder
/// `dest`, telling `listener` as it goes; the files written. On a failure, a stop included, the
/// files written are removed.
#[uniffi::export]
pub fn extract_archive(
    archive: String,
    dest: String,
    password: Option<String>,
    listener: Arc<dyn TransferListener>,
) -> Result<Vec<String>, TransferError> {
    let cancel = Arc::new(AtomicBool::new(false));
    let told = Arc::new(AtomicU64::new(u64::MAX));
    let (flag, last, watcher) = (cancel.clone(), told.clone(), listener.clone());
    let progress = Arc::new(move |done: u64, total: u64| {
        if watcher.is_cancelled() {
            flag.store(true, Ordering::Relaxed);
        }
        let prev = last.load(Ordering::Relaxed);
        let due = prev == u64::MAX || done >= total || done >= prev.saturating_add(PROGRESS_STEP);
        if done != prev && due {
            last.store(done, Ordering::Relaxed);
            watcher.on_progress(done, Some(total));
        }
    });
    let opts = ExtractOptions {
        password,
        progress: Some(progress),
        cancel: Some(cancel),
    };
    let files = ArchiveExtractor::extract_with(Path::new(&archive), Path::new(&dest), &opts)?;
    Ok(files
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect())
}

/// The shared client of the HTTP requests (retries and backoff of [`zl_net::UreqClient`]).
fn client() -> &'static UreqClient {
    static CLIENT: OnceLock<UreqClient> = OnceLock::new();
    CLIENT.get_or_init(UreqClient::new)
}

/// GETs `url` with `headers`; the body. A non-success status is [TransferError::Status].
#[uniffi::export]
pub fn http_get(url: String, headers: HashMap<String, String>) -> Result<Vec<u8>, TransferError> {
    let headers: Vec<(&str, &str)> = headers
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    Ok(client().get_bytes(&url, &headers)?)
}

/// POSTs `body` as `content_type` to `url` with `headers`; the body of the answer.
#[uniffi::export]
pub fn http_post(
    url: String,
    body: Vec<u8>,
    content_type: String,
    headers: HashMap<String, String>,
) -> Result<Vec<u8>, TransferError> {
    let headers: Vec<(&str, &str)> = headers
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    Ok(client().post_bytes(&url, &body, &content_type, &headers)?)
}

impl From<HttpError> for TransferError {
    fn from(e: HttpError) -> Self {
        match e {
            HttpError::Status(code) => TransferError::Status { code },
            HttpError::Cancelled => TransferError::Cancelled,
            HttpError::Io(detail) => TransferError::Io { detail },
            HttpError::Network(detail) | HttpError::MaxRetriesExceeded(detail) => {
                TransferError::Network { detail }
            }
        }
    }
}

impl From<ArchiveError> for TransferError {
    fn from(e: ArchiveError) -> Self {
        match (&e, e.failure()) {
            (ArchiveError::Cancelled, _) => TransferError::Cancelled,
            (ArchiveError::UnsupportedFormat(f), _) => {
                TransferError::Unsupported { format: f.clone() }
            }
            (ArchiveError::SecurityError(entry), _) => TransferError::Unsafe {
                entry: entry.clone(),
            },
            (_, ArchiveFailure::Password) => TransferError::Password,
            (_, ArchiveFailure::NoSpace) => TransferError::NoSpace,
            (_, ArchiveFailure::Damaged) => TransferError::Damaged {
                detail: e.to_string(),
            },
            _ => TransferError::Io {
                detail: e.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// A listener that records what it is told and stops after `stop_after` calls.
    struct Recorder {
        /// The progress told.
        seen: Mutex<Vec<u64>>,
        /// Calls of `on_progress` before it asks to stop.
        stop_after: usize,
    }

    impl TransferListener for Recorder {
        fn on_progress(&self, done: u64, _: Option<u64>) {
            self.seen
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(done);
        }
        fn is_cancelled(&self) -> bool {
            self.seen
                .lock()
                .map_or(true, |s| s.len() >= self.stop_after)
        }
    }

    #[test]
    fn an_extraction_reports_its_files_and_codes() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let archive = dir.path().join("mod.zip");
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&archive)?);
        zip.start_file("Mod/romfs/a.bin", zip::write::SimpleFileOptions::default())?;
        std::io::Write::write_all(&mut zip, &[5u8; 600_000])?;
        zip.finish()?;
        let out = dir.path().join("out").to_string_lossy().into_owned();
        let rec = Arc::new(Recorder {
            seen: Mutex::new(Vec::new()),
            stop_after: usize::MAX,
        });
        let files = extract_archive(
            archive.to_string_lossy().into_owned(),
            out.clone(),
            None,
            rec.clone(),
        )?;
        assert_eq!(files.len(), 1);
        let seen = rec.seen.lock().map(|s| s.clone()).unwrap_or_default();
        assert_eq!(seen.last(), Some(&600_000));
        assert!(seen.len() <= 4, "throttled: {seen:?}");
        let other = dir.path().join("mod.lzh").to_string_lossy().into_owned();
        std::fs::write(&other, b"x")?;
        let err = extract_archive(other, out, None, rec);
        assert!(
            matches!(err, Err(TransferError::Unsupported { .. })),
            "{err:?}"
        );
        Ok(())
    }

    #[test]
    fn a_stop_request_ends_the_extraction() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let archive = dir.path().join("mod.zip");
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&archive)?);
        for name in ["a.bin", "b.bin"] {
            zip.start_file(name, zip::write::SimpleFileOptions::default())?;
            std::io::Write::write_all(&mut zip, &[1u8; 400_000])?;
        }
        zip.finish()?;
        let out = dir.path().join("out");
        let rec = Arc::new(Recorder {
            seen: Mutex::new(Vec::new()),
            stop_after: 1,
        });
        let err = extract_archive(
            archive.to_string_lossy().into_owned(),
            out.to_string_lossy().into_owned(),
            None,
            rec,
        );
        assert!(matches!(err, Err(TransferError::Cancelled)), "{err:?}");
        assert!(!out.join("b.bin").exists());
        Ok(())
    }
}
