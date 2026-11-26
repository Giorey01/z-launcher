//! How an extraction is asked for ([ExtractOptions]: password, progress, stop request) and the
//! pieces that keep zip and 7z in step with the RAR reader: a byte meter shared by the entries
//! and the check of a zip entry's name.

use crate::ArchiveError;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

pub use crate::rar::Progress;

/// How to extract an archive.
#[derive(Clone, Default)]
pub struct ExtractOptions {
    /// Password of an encrypted archive.
    pub password: Option<String>,
    /// Told the bytes written so far and the total, as extraction goes.
    pub progress: Option<Progress>,
    /// Raised to stop: extraction ends with [`ArchiveError::Cancelled`] and what it wrote goes.
    pub cancel: Option<Arc<AtomicBool>>,
}

/// The bytes written by one extraction, told to its [Progress].
pub(crate) struct Meter {
    /// Bytes written so far.
    done: AtomicU64,
    /// Bytes to write in total.
    total: u64,
    /// Told the bytes written and the total.
    progress: Option<Progress>,
    /// Raised by the caller to stop.
    cancel: Option<Arc<AtomicBool>>,
}

impl Meter {
    /// A meter for `total` bytes with the progress and stop request of `opts`.
    pub(crate) fn new(total: u64, opts: &ExtractOptions) -> Self {
        Self {
            done: AtomicU64::new(0),
            total,
            progress: opts.progress.clone(),
            cancel: opts.cancel.clone(),
        }
    }

    /// True when the caller asked to stop.
    pub(crate) fn stopped(&self) -> bool {
        self.cancel
            .as_ref()
            .is_some_and(|c| c.load(Ordering::Relaxed))
    }

    /// Counts `n` more bytes written and tells the progress.
    fn add(&self, n: u64) {
        let done = self.done.fetch_add(n, Ordering::Relaxed) + n;
        if let Some(p) = &self.progress {
            p(done, self.total);
        }
    }

    /// `inner` read through the meter: every byte counted, a stop request an error.
    pub(crate) fn reader<'a>(&'a self, inner: &'a mut dyn Read) -> Metered<'a> {
        Metered { inner, meter: self }
    }
}

/// A reader whose bytes a [Meter] counts.
pub(crate) struct Metered<'a> {
    /// The entry's data.
    inner: &'a mut dyn Read,
    /// The extraction's meter.
    meter: &'a Meter,
}

impl Read for Metered<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.meter.stopped() {
            // Not `Interrupted`: `io::copy` retries that kind forever.
            return Err(io::Error::other("extraction stopped"));
        }
        let n = self.inner.read(buf)?;
        self.meter.add(n as u64);
        Ok(n)
    }
}

/// Where the zip entry `name` goes under `dest`: None for an entry that names no file (the
/// root, `/`, that some tools write); an error when it would leave `dest`. Backslashes count
/// as separators and a leading `/` is dropped, so `/mod/romfs/a` lands in `dest/mod/romfs/a`.
pub(crate) fn zip_entry_path(dest: &Path, name: &str) -> Result<Option<PathBuf>, ArchiveError> {
    let normalized = name.replace('\\', "/");
    let mut out = dest.to_path_buf();
    let mut any = false;
    for component in Path::new(normalized.trim_start_matches('/')).components() {
        match component {
            Component::Normal(part) => {
                out.push(part);
                any = true;
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(ArchiveError::SecurityError(name.to_string()));
            }
        }
    }
    Ok(any.then_some(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zip_names_are_kept_inside_the_destination() {
        let dest = Path::new("/d");
        let ok = |n: &str| zip_entry_path(dest, n).ok().flatten();
        assert_eq!(ok("/"), None);
        assert_eq!(ok(""), None);
        assert_eq!(ok("./"), None);
        assert_eq!(
            ok("/mod/romfs/a.bin"),
            Some(PathBuf::from("/d/mod/romfs/a.bin"))
        );
        assert_eq!(
            ok("mod\\exefs\\main"),
            Some(PathBuf::from("/d/mod/exefs/main"))
        );
        assert!(zip_entry_path(dest, "../x").is_err());
        assert!(zip_entry_path(dest, "a/../../x").is_err());
        assert!(zip_entry_path(dest, "C:\\x").is_ok_and(|p| p.is_some_and(|p| p.starts_with(dest))));
    }

    #[test]
    fn the_meter_counts_and_stops() {
        let told = Arc::new(AtomicU64::new(0));
        let seen = told.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let opts = ExtractOptions {
            progress: Some(Arc::new(move |done, _| seen.store(done, Ordering::Relaxed))),
            cancel: Some(cancel.clone()),
            ..ExtractOptions::default()
        };
        let meter = Meter::new(10, &opts);
        let mut data: &[u8] = b"0123456789";
        let mut out = Vec::new();
        assert!(io::copy(&mut meter.reader(&mut data), &mut out).is_ok());
        assert_eq!(told.load(Ordering::Relaxed), 10);
        cancel.store(true, Ordering::Relaxed);
        let mut more: &[u8] = b"x";
        assert!(io::copy(&mut meter.reader(&mut more), &mut out).is_err());
    }
}
