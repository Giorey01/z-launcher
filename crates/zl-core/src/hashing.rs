//! Hashing the library's files in the background: CRC32, MD5 and SHA1 of every file of every
//! game that has no current digests yet, for matching metadata (and, later, achievements).
//!
//! The core is synchronous: [`Service::start_hashing`] runs the work on its own thread, with its
//! own database connection, and reports with [`HashEvent`]s and a pollable [`HashStatus`]. A file
//! is read again only when its size or modification time changed since the last run (the key of
//! the `rom_hashes` rows), and a file that fails for a lasting reason (too large, encrypted,
//! damaged) is remembered as failed, so it does not cost a read at every scan.

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use zl_config::hashing as cfg;
use zl_db::{Db, GameFile, NewRomHash};
use zl_hash::{hash_file, Container, EntryHash, FileHashes, HashError, Limits};

use crate::error::CoreError;
use crate::Service;

pub use zl_db::RomHash;

/// The numbers of a run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HashReport {
    /// Games covered by the run.
    pub games: u32,
    /// Files hashed.
    pub files: u32,
    /// Entries stored (a zip or a 7z gives one per file inside).
    pub entries: u32,
    /// Files that failed (unreadable now, or for a lasting reason).
    pub failed: u32,
    /// True when the run ended because the user asked to stop.
    pub stopped: bool,
}

/// Something that happened during a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashEvent {
    /// The run begins with this much to do.
    Started {
        /// Games with something to hash.
        games: u32,
        /// Files to hash.
        files: u32,
    },
    /// A game is finished.
    Progress {
        /// Games finished so far.
        done: u32,
        /// Games in the run.
        total: u32,
        /// The game that was finished.
        game_id: i64,
    },
    /// A file could not be hashed.
    FileFailed {
        /// The game it belongs to.
        game_id: i64,
        /// Stable code (`io`, `too_large`, `password`, `damaged`, `unsupported`, `empty`).
        code: String,
    },
    /// The run is over.
    Finished {
        /// Final numbers.
        report: HashReport,
    },
}

/// Receives the events of a run, on the hashing thread.
pub trait HashObserver: Send + Sync {
    /// One event of the run.
    fn on_event(&self, event: HashEvent);
}

/// Whether a run is going.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashRun {
    /// Nothing is running.
    Idle,
    /// A run is going.
    Running,
    /// A run was asked to stop and is finishing the file it is on.
    Stopping,
}

/// The state of the hashing, for the library settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashStatus {
    /// Whether a run is going.
    pub run: HashRun,
    /// Games finished in the current run (0 when idle).
    pub done: u32,
    /// Games in the current run (0 when idle).
    pub total: u32,
    /// Games of the library that have files to hash.
    pub library_games: u32,
    /// Of those, games whose files are all hashed.
    pub library_hashed: u32,
    /// The numbers of the last run that ended in this session.
    pub last: Option<HashReport>,
}

/// What the running thread and the callers share.
#[derive(Default)]
pub(crate) struct HashShared {
    /// A run is going.
    running: AtomicBool,
    /// The user asked to stop.
    stop: AtomicBool,
    /// Games finished in the current run.
    done: AtomicU32,
    /// Games in the current run.
    total: AtomicU32,
    /// The report of the last finished run.
    last: Mutex<Option<HashReport>>,
}

impl HashShared {
    /// Whether a stop was asked.
    fn stop_requested(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }
}

/// Marks a run as over when the thread ends, however it ends.
struct RunGuard(Arc<HashShared>);

impl Drop for RunGuard {
    /// Clears the flags so a new run can start.
    fn drop(&mut self) {
        self.0.running.store(false, Ordering::SeqCst);
        self.0.stop.store(false, Ordering::SeqCst);
        self.0.done.store(0, Ordering::Relaxed);
        self.0.total.store(0, Ordering::Relaxed);
    }
}

/// The rows to store for a file that was hashed.
fn rows_of(hashes: &FileHashes, file: &GameFile) -> Vec<NewRomHash> {
    hashes
        .entries
        .iter()
        .map(|e| entry_row(e, hashes.container, file))
        .collect()
}

/// The row of one entry.
fn entry_row(entry: &EntryHash, container: Container, file: &GameFile) -> NewRomHash {
    let (header_bytes, hl) = match &entry.headerless {
        Some(h) => (i64::from(h.header_bytes), Some(&h.digests)),
        None => (0, None),
    };
    NewRomHash {
        entry: entry.entry.clone(),
        file_size: file.size,
        file_mtime: file.mtime_ms,
        container: container.as_str().to_string(),
        size: i64::try_from(entry.size).unwrap_or(i64::MAX),
        crc32: entry.full.crc32.clone(),
        md5: entry.full.md5.clone(),
        sha1: entry.full.sha1.clone(),
        header_bytes,
        hl_crc32: hl.and_then(|d| d.crc32.clone()),
        hl_md5: hl.and_then(|d| d.md5.clone()),
        hl_sha1: hl.and_then(|d| d.sha1.clone()),
        error: None,
    }
}

/// The row that remembers a lasting failure of a file.
fn error_row(path: &str, file: &GameFile, error: &HashError) -> NewRomHash {
    NewRomHash {
        file_size: file.size,
        file_mtime: file.mtime_ms,
        container: Container::of_path(Path::new(path)).as_str().to_string(),
        error: Some(error.code().to_string()),
        ..NewRomHash::default()
    }
}

/// The pending files grouped by game, in game order.
fn pending_by_game(db: &Db) -> Result<Vec<(i64, Vec<GameFile>)>, CoreError> {
    let mut groups: Vec<(i64, Vec<GameFile>)> = Vec::new();
    for (game_id, file) in db.files_to_hash(&cfg::SKIPPED_EXTENSIONS)? {
        match groups.last_mut() {
            Some((id, files)) if *id == game_id => files.push(file),
            _ => groups.push((game_id, vec![file])),
        }
    }
    Ok(groups)
}

/// What one file did to the run.
enum Step {
    /// Go on with the next file.
    Next,
    /// The user asked to stop.
    Stop,
}

/// The loop of one run: every pending game, file by file, until the end or a stop request.
struct Worker<'a> {
    /// The connection of the thread.
    db: &'a Db,
    /// The state shared with the callers.
    shared: &'a HashShared,
    /// Where the events go.
    emit: &'a dyn Fn(HashEvent),
    /// The numbers so far.
    report: HashReport,
    /// The bounds of the hashing.
    limits: Limits,
}

impl Worker<'_> {
    /// Hashes one file and stores the answer.
    fn file(&mut self, game_id: i64, file: &GameFile) -> Result<Step, CoreError> {
        let stop = || self.shared.stop_requested();
        match hash_file(Path::new(&file.path), &self.limits, &stop) {
            Ok(hashes) => {
                let rows = rows_of(&hashes, file);
                self.db.replace_file_hashes(game_id, &file.path, &rows)?;
                self.report.files += 1;
                self.report.entries += rows.len() as u32;
            }
            Err(HashError::Cancelled) => return Ok(Step::Stop),
            Err(error) => {
                if !error.is_transient() {
                    let row = error_row(&file.path, file, &error);
                    self.db.replace_file_hashes(game_id, &file.path, &[row])?;
                }
                self.report.failed += 1;
                let code = error.code().to_string();
                (self.emit)(HashEvent::FileFailed { game_id, code });
            }
        }
        Ok(Step::Next)
    }

    /// Hashes the files of one game.
    fn game(&mut self, game_id: i64, files: &[GameFile]) -> Result<Step, CoreError> {
        for file in files {
            if matches!(self.file(game_id, file)?, Step::Stop) {
                return Ok(Step::Stop);
            }
        }
        Ok(Step::Next)
    }

    /// Goes through the games.
    fn run(&mut self, groups: &[(i64, Vec<GameFile>)]) -> Result<(), CoreError> {
        let total = groups.len() as u32;
        for (index, (game_id, files)) in groups.iter().enumerate() {
            if matches!(self.game(*game_id, files)?, Step::Stop) || self.shared.stop_requested() {
                self.report.stopped = true;
                return Ok(());
            }
            let done = index as u32 + 1;
            self.shared.done.store(done, Ordering::Relaxed);
            self.report.games = done;
            (self.emit)(HashEvent::Progress {
                done,
                total,
                game_id: *game_id,
            });
            std::thread::sleep(Duration::from_millis(cfg::PAUSE_BETWEEN_GAMES_MS));
        }
        Ok(())
    }
}

/// One whole run on `db`: hashes what is pending and returns the numbers. Events go to `emit`.
fn run_pending(
    db: &Db,
    shared: &HashShared,
    emit: &dyn Fn(HashEvent),
) -> Result<HashReport, CoreError> {
    let groups = pending_by_game(db)?;
    let files = groups.iter().map(|(_, f)| f.len() as u32).sum();
    shared.total.store(groups.len() as u32, Ordering::Relaxed);
    emit(HashEvent::Started {
        games: groups.len() as u32,
        files,
    });
    let mut worker = Worker {
        db,
        shared,
        emit,
        report: HashReport::default(),
        limits: Limits::default(),
    };
    worker.run(&groups)?;
    Ok(worker.report)
}

/// The thread body: opens its own connection, runs, records and announces the end.
fn thread_main(db_path: &Path, shared: &Arc<HashShared>, observer: Option<Arc<dyn HashObserver>>) {
    let _guard = RunGuard(Arc::clone(shared));
    let emit = |event: HashEvent| {
        if let Some(observer) = &observer {
            observer.on_event(event);
        }
    };
    let report = match Db::open(db_path) {
        Ok(db) => run_pending(&db, shared, &emit).unwrap_or_else(|_| HashReport {
            failed: 1,
            ..HashReport::default()
        }),
        Err(_) => HashReport {
            failed: 1,
            ..HashReport::default()
        },
    };
    *shared.last.lock().unwrap_or_else(PoisonError::into_inner) = Some(report.clone());
    emit(HashEvent::Finished { report });
}

impl Service {
    /// Starts hashing the files that have no current digests, on a thread of its own. Returns
    /// `false` when a run is already going (nothing is started then). Safe to call after every
    /// scan: with nothing pending the run ends at once.
    pub fn start_hashing(&self, observer: Option<Arc<dyn HashObserver>>) -> bool {
        let shared = Arc::clone(&self.hashing);
        if shared.running.swap(true, Ordering::SeqCst) {
            return false;
        }
        shared.stop.store(false, Ordering::SeqCst);
        let db_path = self.db_path.clone();
        let spawned = std::thread::Builder::new()
            .name("zl-hash".into())
            .spawn({
                let shared = Arc::clone(&shared);
                move || thread_main(&db_path, &shared, observer)
            })
            .is_ok();
        if !spawned {
            shared.running.store(false, Ordering::SeqCst);
        }
        spawned
    }

    /// Asks the running hashing to stop; it ends within a chunk of reading. Does nothing when
    /// idle.
    pub fn stop_hashing(&self) {
        if self.hashing.running.load(Ordering::SeqCst) {
            self.hashing.stop.store(true, Ordering::SeqCst);
        }
    }

    /// Runs the hashing on the calling thread until it ends (the same work as the background
    /// run; for callers that have a thread of their own, and for tests). `InvalidInput` when a
    /// run is already going.
    pub fn hash_pending(
        &self,
        observer: Option<&dyn HashObserver>,
    ) -> Result<HashReport, CoreError> {
        if self.hashing.running.swap(true, Ordering::SeqCst) {
            return Err(CoreError::InvalidInput);
        }
        let _guard = RunGuard(Arc::clone(&self.hashing));
        let db = Db::open(&self.db_path)?;
        let emit = |event: HashEvent| {
            if let Some(observer) = observer {
                observer.on_event(event);
            }
        };
        run_pending(&db, &self.hashing, &emit)
    }

    /// The state of the hashing and how much of the library is covered.
    pub fn hash_status(&self) -> Result<HashStatus, CoreError> {
        let (library_games, library_hashed) = self.db().hash_coverage(&cfg::SKIPPED_EXTENSIONS)?;
        let shared = &self.hashing;
        let run = match (
            shared.running.load(Ordering::SeqCst),
            shared.stop.load(Ordering::SeqCst),
        ) {
            (false, _) => HashRun::Idle,
            (true, false) => HashRun::Running,
            (true, true) => HashRun::Stopping,
        };
        Ok(HashStatus {
            run,
            done: shared.done.load(Ordering::Relaxed),
            total: shared.total.load(Ordering::Relaxed),
            library_games,
            library_hashed,
            last: shared
                .last
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clone(),
        })
    }

    /// The digests of the files of a game (one row per file, or per entry of an archive).
    pub fn game_hashes(&self, game_id: i64) -> Result<Vec<RomHash>, CoreError> {
        Ok(self.db().game_hashes(game_id)?)
    }
}
