//! RAR archives (RAR 1.3 to RAR 7, one file or a volume set), read by the pure-Rust `rars`
//! crate, so the desktop and the Android app extract them with the same code.
//!
//! Extraction writes into a folder with the same zip-slip protection as the other formats,
//! reports bytes written, stops when asked and, when nothing was given up yet, removes what it
//! wrote on failure. For a volume set it can also *consume* the set: every volume is emptied and
//! deleted as soon as extraction is past it, so the disk only has to hold the extracted files plus
//! about one volume instead of the whole set as well (see [`RarOptions::consume`]).

mod consume;
mod output;
mod volumes;

use crate::ArchiveError;
use output::{Outputs, Shared};
use rars::{Archive, ArchiveReadOptions};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub use volumes::{
    is_first_volume, is_first_volume_name, is_rar_name, is_split_volume_name, split_volume_count,
};

/// Told the bytes written so far and the total to write.
pub type Progress = Arc<dyn Fn(u64, u64) + Send + Sync>;

/// How to extract a RAR archive.
#[derive(Clone, Default)]
pub struct RarOptions {
    /// Password of an encrypted archive (files or headers).
    pub password: Option<String>,
    /// Told the bytes written and the total, as extraction goes.
    pub progress: Option<Progress>,
    /// Raised to stop: extraction ends with [`ArchiveError::Cancelled`].
    pub cancel: Option<Arc<AtomicBool>>,
    /// Empties and deletes each volume of a set as soon as extraction no longer needs it.
    ///
    /// Destructive: once a volume is gone a failure cannot be undone, so the files already
    /// extracted stay (only the half-written one is removed) and a stop request is ignored from
    /// that moment on (a paused job could not resume an extraction whose volumes are gone).
    pub consume: bool,
}

/// What a RAR extraction wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RarExtracted {
    /// The extracted files.
    pub files: Vec<PathBuf>,
    /// Every volume of the set (deleted already when the set was consumed).
    pub volumes: Vec<PathBuf>,
    /// True when at least one volume was consumed.
    pub consumed: bool,
}

/// What a RAR archive holds, read from its headers only (nothing is decoded).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RarInfo {
    /// Every volume of the set, in order (one path when the archive is not split).
    pub volumes: Vec<PathBuf>,
    /// Total size of the files once extracted.
    pub unpacked: u64,
    /// Size of the biggest volume on disk.
    pub largest_volume: u64,
    /// True when the archive is solid (files compressed as one stream).
    pub solid: bool,
}

/// Reads what the archive starting at `first` holds, without extracting anything.
pub fn inspect(first: &Path, password: Option<&str>) -> Result<RarInfo, ArchiveError> {
    let set = volumes::open_set(first, password)?;
    Ok(set.info())
}

/// Extracts the RAR archive (or volume set) whose first volume is `first` into `dest`.
pub fn extract(first: &Path, dest: &Path, opts: &RarOptions) -> Result<RarExtracted, ArchiveError> {
    if !first.is_file() {
        return Err(ArchiveError::NotFound(first.to_path_buf()));
    }
    std::fs::create_dir_all(dest)?;
    let root = dest.canonicalize()?;
    let set = volumes::open_set(first, opts.password.as_deref())?;
    let shared = Arc::new(Shared::new(
        set.info().unpacked,
        opts.progress.clone(),
        opts.cancel.clone(),
    ));
    let mut outputs = Outputs::new(root, shared.clone());
    let mut plan = consume::Plan::new(&set, opts.consume, shared.clone());
    let result = run(&set, opts, &mut outputs, &mut plan);
    finish(result, set.paths.clone(), outputs, &plan, &shared)
}

/// Turns the outcome of the extraction into the result, cleaning up after a failure.
fn finish(
    result: Result<(), rars::Error>,
    volumes: Vec<PathBuf>,
    mut outputs: Outputs,
    plan: &consume::Plan,
    shared: &Shared,
) -> Result<RarExtracted, ArchiveError> {
    match result {
        Ok(()) => Ok(RarExtracted {
            files: outputs.take_files(),
            volumes,
            consumed: plan.consumed(),
        }),
        Err(err) => {
            if plan.consumed() {
                outputs.drop_partial();
            } else {
                outputs.rollback();
            }
            Err(error_of(err, shared))
        }
    }
}

/// Runs the extraction of the whole set through the output and the consume plan.
fn run(
    set: &volumes::VolumeSet,
    opts: &RarOptions,
    outputs: &mut Outputs,
    plan: &mut consume::Plan,
) -> Result<(), rars::Error> {
    let password = opts.password.as_deref().map(str::as_bytes);
    let options = ArchiveReadOptions::with_optional_password(password);
    let later: Vec<rars::Result<Archive>> = set.archives.iter().skip(1).cloned().map(Ok).collect();
    set.archives[0].extract_with_volumes(later, options, |meta| {
        outputs.check_stop()?;
        plan.on_member(meta.name_bytes())?;
        outputs.open(meta)
    })
}

/// The error of a failed extraction: a stop request when one was made, else what `rars` said.
fn error_of(err: rars::Error, shared: &Shared) -> ArchiveError {
    if shared.stop_requested() {
        return ArchiveError::Cancelled;
    }
    ArchiveError::from(err)
}
