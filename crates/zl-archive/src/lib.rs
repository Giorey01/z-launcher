//! Extraction of downloaded archives (.zip, .7z and .rar) into a destination folder, with
//! protection against path traversal and all-or-nothing rollback. RAR (also split into volumes)
//! is in [`rar`], read in pure Rust on every target.
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

mod options;
pub mod rar;

use options::{zip_entry_path, Meter};
pub use options::{ExtractOptions, Progress};

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Why an archive could not be extracted.
#[derive(Error, Debug)]
pub enum ArchiveError {
    /// The archive file does not exist.
    #[error("not found: {0}")]
    NotFound(PathBuf),

    /// The extension is not `zip`, `7z` or `rar`.
    #[error("unsupported archive format: {0}")]
    UnsupportedFormat(String),

    /// An entry tried to escape the destination folder (zip slip).
    #[error("entry outside the destination: {0}")]
    SecurityError(String),

    /// A filesystem operation failed.
    #[error("io: {0}")]
    IoError(#[from] io::Error),

    /// The zip reader failed.
    #[error("zip: {0}")]
    ZipError(#[from] zip::result::ZipError),

    /// The 7z reader failed (bad password included).
    #[error("7z: {0}")]
    SevenZError(#[from] sevenz_rust2::Error),

    /// The RAR reader failed; `code` is the stable kind `rars` gives (`BAD_PASSWORD`,
    /// `PASSWORD_REQUIRED`, `CHECKSUM_MISMATCH`, `UNSAFE_ENTRY_NAME`, `IO`, …).
    #[error("rar ({code}): {detail}")]
    RarError {
        /// Stable machine-readable kind of the failure.
        code: &'static str,
        /// What the reader said, for the log.
        detail: String,
    },

    /// The caller asked to stop.
    #[error("cancelled")]
    Cancelled,
}

impl From<::rars::Error> for ArchiveError {
    fn from(err: ::rars::Error) -> Self {
        // A full disk stays an I/O error with its kind, so the caller can tell it apart.
        if let ::rars::Error::Io(io) = err.root_cause() {
            if io.kind == io::ErrorKind::StorageFull {
                return ArchiveError::IoError(io::Error::new(io.kind, io.message.clone()));
            }
        }
        ArchiveError::RarError {
            code: err.kind().code(),
            detail: err.to_string(),
        }
    }
}

/// Why an extraction failed, as far as the user can act on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFailure {
    /// The archive is encrypted and no password, or a wrong one, was given.
    Password,
    /// The archive is damaged, incomplete (a volume missing) or not an archive at all.
    Damaged,
    /// The disk filled up while extracting.
    NoSpace,
    /// Anything else (permissions, an unsupported method, a stop request).
    Other,
}

/// The kinds of [`ArchiveError::RarError`] that mean a damaged or incomplete archive.
const RAR_DAMAGED: &[&str] = &[
    "CHECKSUM_MISMATCH",
    "INVALID_ARCHIVE",
    "ENTRY_NOT_FOUND",
    "SOURCE_CHANGED",
];

impl ArchiveError {
    /// What kind of failure this is, for the event the apps show.
    pub fn failure(&self) -> ArchiveFailure {
        match self {
            ArchiveError::RarError { code, .. } => match *code {
                "BAD_PASSWORD" | "PASSWORD_REQUIRED" => ArchiveFailure::Password,
                c if RAR_DAMAGED.contains(&c) => ArchiveFailure::Damaged,
                _ => ArchiveFailure::Other,
            },
            ArchiveError::IoError(e) if e.kind() == io::ErrorKind::StorageFull => {
                ArchiveFailure::NoSpace
            }
            ArchiveError::IoError(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                ArchiveFailure::Damaged
            }
            ArchiveError::ZipError(e) if zip_needs_password(e) => ArchiveFailure::Password,
            ArchiveError::ZipError(_) => ArchiveFailure::Damaged,
            ArchiveError::SevenZError(e) => sevenz_failure(e),
            _ => ArchiveFailure::Other,
        }
    }
}

/// True when the zip reader refused an entry for want of a password, or because it was wrong.
fn zip_needs_password(e: &zip::result::ZipError) -> bool {
    use zip::result::ZipError as Z;
    matches!(e, Z::InvalidPassword)
        || matches!(e, Z::UnsupportedArchive(why) if *why == Z::PASSWORD_REQUIRED)
}

/// What kind of failure a 7z error is.
fn sevenz_failure(e: &sevenz_rust2::Error) -> ArchiveFailure {
    use sevenz_rust2::Error as E;
    match e {
        E::PasswordRequired | E::MaybeBadPassword(_) => ArchiveFailure::Password,
        E::Io(io, _) | E::FileOpen(io, _) if io.kind() == io::ErrorKind::StorageFull => {
            ArchiveFailure::NoSpace
        }
        E::BadSignature(_)
        | E::ChecksumVerificationFailed
        | E::NextHeaderCrcMismatch
        | E::BadTerminatedStreamsInfo(_)
        | E::BadTerminatedUnpackInfo
        | E::BadTerminatedPackInfo(_)
        | E::BadTerminatedSubStreamsInfo
        | E::BadTerminatedHeader(_) => ArchiveFailure::Damaged,
        _ => ArchiveFailure::Other,
    }
}

/// Stateless namespace for the extraction functions.
pub struct ArchiveExtractor;

impl ArchiveExtractor {
    /// Extracts an archive (.zip, .7z or the first volume of a .rar set) into dest_dir with full path-traversal protection
    /// and atomic rollback (if extraction fails, partially written files are deleted).
    pub fn extract(
        archive_path: &Path,
        dest_dir: &Path,
        password: Option<&str>,
    ) -> Result<Vec<PathBuf>, ArchiveError> {
        let opts = ExtractOptions {
            password: password.map(str::to_string),
            ..ExtractOptions::default()
        };
        Self::extract_with(archive_path, dest_dir, &opts)
    }

    /// [`Self::extract`] with a progress and a stop request: the format is told by the
    /// extension, and on any failure (a stop included) the files written are removed.
    pub fn extract_with(
        archive_path: &Path,
        dest_dir: &Path,
        opts: &ExtractOptions,
    ) -> Result<Vec<PathBuf>, ArchiveError> {
        if !archive_path.exists() {
            return Err(ArchiveError::NotFound(archive_path.to_path_buf()));
        }
        fs::create_dir_all(dest_dir)?;
        let dest = dest_dir.canonicalize()?;
        let ext = archive_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let mut created: Vec<PathBuf> = Vec::new();
        let result = match ext.as_str() {
            "zip" => Self::extract_zip_internal(archive_path, &dest, opts, &mut created),
            "7z" => Self::extract_7z_internal(archive_path, &dest, opts, &mut created),
            "rar" => Self::extract_rar_internal(archive_path, &dest, opts, &mut created),
            _ => Err(ArchiveError::UnsupportedFormat(ext)),
        };
        if let Err(err) = result {
            // All-or-nothing rollback
            for file in created {
                let _ = fs::remove_file(file);
            }
            return Err(err);
        }
        Ok(created)
    }

    /// Auto-extracts an archive. If extraction succeeds and delete_archive is true,
    /// deletes the archive file. If extraction fails, preserves the archive and rolls back extracted files.
    pub fn auto_extract(
        archive_path: &Path,
        dest_dir: &Path,
        delete_archive: bool,
    ) -> Result<Vec<PathBuf>, ArchiveError> {
        let extracted = Self::extract(archive_path, dest_dir, None)?;
        if delete_archive {
            let _ = fs::remove_file(archive_path);
        }
        Ok(extracted)
    }

    /// Extracts a zip entry by entry, checking every path stays inside `dest_dir` (an entry that
    /// names no file, like `/`, is skipped). The password opens the encrypted entries (ZipCrypto
    /// or AES); plain entries ignore it.
    fn extract_zip_internal(
        archive_path: &Path,
        dest_dir: &Path,
        opts: &ExtractOptions,
        created: &mut Vec<PathBuf>,
    ) -> Result<(), ArchiveError> {
        let mut archive = zip::ZipArchive::new(File::open(archive_path)?)?;
        let total = (0..archive.len())
            .filter_map(|i| archive.by_index_raw(i).ok().map(|e| e.size()))
            .sum();
        let meter = Meter::new(total, opts);
        for i in 0..archive.len() {
            if meter.stopped() {
                return Err(ArchiveError::Cancelled);
            }
            let mut file = match opts.password.as_deref() {
                Some(pwd) => archive.by_index_decrypt(i, pwd.as_bytes())?,
                None => archive.by_index(i)?,
            };
            let Some(out_path) = zip_entry_path(dest_dir, file.name())? else {
                continue;
            };
            if file.is_dir() {
                fs::create_dir_all(&out_path)?;
                continue;
            }
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&out_path)?;
            created.push(out_path);
            io::copy(&mut meter.reader(&mut file), &mut outfile).map_err(|e| stop_or(e, &meter))?;
        }
        Ok(())
    }

    /// Extracts a 7z (optionally password protected) and lists the files it produced.
    fn extract_7z_internal(
        archive_path: &Path,
        dest_dir: &Path,
        opts: &ExtractOptions,
        created: &mut Vec<PathBuf>,
    ) -> Result<(), ArchiveError> {
        // The files already in the folder are not the archive's (the folder can be a shared
        // one): only what appears is reported, and rolled back on failure.
        let mut before = Vec::new();
        Self::collect_files(dest_dir, &mut before)?;
        let before: std::collections::HashSet<PathBuf> = before.into_iter().collect();
        let password = opts
            .password
            .as_deref()
            .map_or_else(sevenz_rust2::Password::empty, sevenz_rust2::Password::from);
        let total = sevenz_rust2::Archive::open_with_password(archive_path, &password)?
            .files
            .iter()
            .map(|f| f.size)
            .sum();
        let meter = Meter::new(total, opts);
        let result = sevenz_rust2::decompress_with_extract_fn_and_password(
            File::open(archive_path)?,
            dest_dir,
            password,
            |entry, reader, dest| {
                sevenz_rust2::default_entry_extract_fn(entry, &mut meter.reader(reader), dest)
            },
        );
        let mut after = Vec::new();
        Self::collect_files(dest_dir, &mut after)?;
        created.extend(after.into_iter().filter(|f| !before.contains(f)));
        if meter.stopped() {
            return Err(ArchiveError::Cancelled);
        }
        result?;
        Ok(())
    }

    /// Extracts a RAR archive (or the set it starts) and lists the files it produced; the
    /// RAR reader already removes what it wrote when it fails.
    fn extract_rar_internal(
        archive_path: &Path,
        dest_dir: &Path,
        opts: &ExtractOptions,
        created: &mut Vec<PathBuf>,
    ) -> Result<(), ArchiveError> {
        let rar_opts = rar::RarOptions {
            password: opts.password.clone(),
            progress: opts.progress.clone(),
            cancel: opts.cancel.clone(),
            ..rar::RarOptions::default()
        };
        created.extend(rar::extract(archive_path, dest_dir, &rar_opts)?.files);
        Ok(())
    }

    /// Recursively appends every file under `dir` to `files`.
    fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    Self::collect_files(&path, files)?;
                } else {
                    files.push(path);
                }
            }
        }
        Ok(())
    }
}

/// A copy that failed: [`ArchiveError::Cancelled`] when the caller asked to stop, else the I/O error.
fn stop_or(e: io::Error, meter: &Meter) -> ArchiveError {
    if meter.stopped() {
        ArchiveError::Cancelled
    } else {
        ArchiveError::IoError(e)
    }
}
