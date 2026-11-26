//! Where the entries of a RAR archive are written: checked paths inside the destination, a
//! writer per file that counts bytes and honours a stop request, and the list of what was made
//! so a failure can take it back.

use rars::filename::checked_output_path;
use rars::{entry_relative_path, ExtractedEntryMeta};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// State the writers of one extraction share.
pub struct Shared {
    /// Bytes written so far.
    done: AtomicU64,
    /// Bytes to write in total.
    total: u64,
    /// Told the bytes written and the total.
    progress: Option<super::Progress>,
    /// Raised by the caller to stop.
    cancel: Option<Arc<AtomicBool>>,
    /// Set once a volume was consumed: from then on a stop request is ignored.
    committed: AtomicBool,
}

impl Shared {
    /// State for an extraction of `total` bytes.
    pub fn new(
        total: u64,
        progress: Option<super::Progress>,
        cancel: Option<Arc<AtomicBool>>,
    ) -> Self {
        Self {
            done: AtomicU64::new(0),
            total,
            progress,
            cancel,
            committed: AtomicBool::new(false),
        }
    }

    /// True when the caller asked to stop and the extraction may still stop.
    pub fn stop_requested(&self) -> bool {
        !self.committed.load(Ordering::Relaxed)
            && self
                .cancel
                .as_ref()
                .is_some_and(|c| c.load(Ordering::Relaxed))
    }

    /// Records that a volume was consumed: the extraction must now run to its end.
    pub fn commit(&self) {
        self.committed.store(true, Ordering::Relaxed);
    }

    /// Adds `n` written bytes and tells the progress callback.
    fn add(&self, n: u64) {
        let done = self.done.fetch_add(n, Ordering::Relaxed) + n;
        if let Some(progress) = &self.progress {
            progress(done, self.total);
        }
    }
}

/// The error a writer returns when the extraction was asked to stop.
fn stopped() -> io::Error {
    io::Error::new(io::ErrorKind::Interrupted, "extraction stopped")
}

/// The writer of one extracted file.
struct EntryWriter {
    /// The file being written.
    file: File,
    /// The extraction's shared state.
    shared: Arc<Shared>,
}

impl Write for EntryWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.shared.stop_requested() {
            return Err(stopped());
        }
        let n = self.file.write(buf)?;
        self.shared.add(n as u64);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

/// What one extraction made, and how to make the next entry.
pub struct Outputs {
    /// The destination folder (canonical).
    root: PathBuf,
    /// The extraction's shared state.
    shared: Arc<Shared>,
    /// The files made, in order (the last one may be half written).
    files: Vec<PathBuf>,
    /// The folders made, in order.
    dirs: Vec<PathBuf>,
}

impl Outputs {
    /// Outputs into `root`.
    pub fn new(root: PathBuf, shared: Arc<Shared>) -> Self {
        Self {
            root,
            shared,
            files: Vec::new(),
            dirs: Vec::new(),
        }
    }

    /// Fails when the caller asked to stop.
    pub fn check_stop(&self) -> rars::Result<()> {
        if self.shared.stop_requested() {
            return Err(stopped().into());
        }
        Ok(())
    }

    /// Opens the writer of the entry `meta`, inside the destination.
    pub fn open(&mut self, meta: &ExtractedEntryMeta) -> rars::Result<Box<dyn Write>> {
        let relative = entry_relative_path(meta.name_bytes())?;
        let path = checked_output_path(&self.root, &relative)?;
        if meta.is_directory {
            self.make_dirs(&path)?;
            return Ok(Box::new(io::sink()));
        }
        if let Some(parent) = path.parent() {
            self.make_dirs(parent)?;
        }
        let file = File::create(&path)?;
        self.files.push(path);
        Ok(Box::new(EntryWriter {
            file,
            shared: self.shared.clone(),
        }))
    }

    /// Creates `dir` and its missing parents, remembering the ones it made.
    fn make_dirs(&mut self, dir: &Path) -> io::Result<()> {
        let mut missing = Vec::new();
        let mut at = Some(dir);
        while let Some(d) = at.filter(|d| !d.exists() && d.starts_with(&self.root)) {
            missing.push(d.to_path_buf());
            at = d.parent();
        }
        fs::create_dir_all(dir)?;
        self.dirs.extend(missing.into_iter().rev());
        Ok(())
    }

    /// The files made, each once (an archive can hold the same name twice: the last one wins).
    pub fn take_files(&mut self) -> Vec<PathBuf> {
        let mut seen = std::collections::HashSet::new();
        let mut files = std::mem::take(&mut self.files);
        files.retain(|f| seen.insert(f.clone()));
        files
    }

    /// Removes everything made (files, then the folders left empty).
    pub fn rollback(&mut self) {
        for f in self.files.drain(..) {
            let _ = fs::remove_file(f);
        }
        for d in self.dirs.drain(..).rev() {
            let _ = fs::remove_dir(d);
        }
    }

    /// Removes only the last file, the one that may be half written.
    pub fn drop_partial(&mut self) {
        if let Some(last) = self.files.pop() {
            let _ = fs::remove_file(last);
        }
    }
}
