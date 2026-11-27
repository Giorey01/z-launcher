//! The hashing half of the [`Core`] object: start and stop the background hashing of the
//! library's files, read its state, and the digests of a game.

use std::sync::Arc;

use zl_core as core;

use crate::core_object::Core;
use crate::error::CoreError;

/// Result of a core call.
type Res<T> = Result<T, CoreError>;

mirror_enum! {
    /// Whether a hashing run is going.
    HashRun <=> core::HashRun { Idle, Running, Stopping }
}
mirror_record! {
    /// The numbers of a hashing run.
    HashReport <=> core::HashReport {
        /// Games covered.
        plain games: u32,
        /// Files hashed.
        plain files: u32,
        /// Entries stored (a zip or a 7z gives one per file inside).
        plain entries: u32,
        /// Files that failed.
        plain failed: u32,
        /// Whether the run ended because of a stop request.
        plain stopped: bool,
    }
}
mirror_record! {
    /// The state of the hashing, for the library settings.
    HashStatus <=> core::HashStatus {
        /// Whether a run is going.
        plain run: HashRun,
        /// Games finished in the current run.
        plain done: u32,
        /// Games in the current run.
        plain total: u32,
        /// Games of the library that have files to hash.
        plain library_games: u32,
        /// Of those, games whose files are all hashed.
        plain library_hashed: u32,
        /// The last run that ended in this session.
        opt last: HashReport,
    }
}
mirror_record! {
    /// The digests of one file of a game, or of one entry of an archive.
    RomHash <=> core::RomHash {
        /// Game the file belongs to.
        plain game_id: i64,
        /// Path of the file.
        plain file_path: String,
        /// Path inside the archive (empty for a plain file or a CHD).
        plain entry: String,
        /// Size of the file when it was hashed.
        plain file_size: i64,
        /// Modification time of the file when it was hashed, Unix milliseconds.
        plain file_mtime: i64,
        /// `raw`, `zip`, `7z` or `chd`.
        plain container: String,
        /// Size of the content in bytes.
        plain size: i64,
        /// CRC32.
        opt crc32: String,
        /// MD5.
        opt md5: String,
        /// SHA1.
        opt sha1: String,
        /// Bytes of header left out of the headerless digests.
        plain header_bytes: i64,
        /// CRC32 without the header.
        opt hl_crc32: String,
        /// MD5 without the header.
        opt hl_md5: String,
        /// SHA1 without the header.
        opt hl_sha1: String,
        /// Code of the lasting failure (`too_large`, `password`, `damaged`, `unsupported`,
        /// `empty`), when the file could not be hashed.
        opt error: String,
        /// When the row was written, Unix seconds.
        plain hashed_at: i64,
    }
}

/// Something that happened during a hashing run.
#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
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

impl From<core::HashEvent> for HashEvent {
    /// One to one.
    fn from(e: core::HashEvent) -> Self {
        match e {
            core::HashEvent::Started { games, files } => HashEvent::Started { games, files },
            core::HashEvent::Progress {
                done,
                total,
                game_id,
            } => HashEvent::Progress {
                done,
                total,
                game_id,
            },
            core::HashEvent::FileFailed { game_id, code } => {
                HashEvent::FileFailed { game_id, code }
            }
            core::HashEvent::Finished { report } => HashEvent::Finished {
                report: report.into(),
            },
        }
    }
}

/// Receiver of the events of a hashing run, implemented in Kotlin. Called on the hashing thread.
#[uniffi::export(with_foreign)]
pub trait HashListener: Send + Sync {
    /// One event of the run.
    fn on_event(&self, event: HashEvent);
}

/// Adapts the foreign listener to the core's observer trait.
struct Bridge(Arc<dyn HashListener>);

impl core::HashObserver for Bridge {
    /// Forwards the event to Kotlin.
    fn on_event(&self, event: core::HashEvent) {
        self.0.on_event(event.into());
    }
}

#[uniffi::export]
impl Core {
    /// Starts hashing the files that have no current digests, on a thread of the core; events go
    /// to `listener`. Returns `false` when a run is already going. Cheap to call after every
    /// scan: with nothing pending the run ends at once.
    pub fn start_hashing(&self, listener: Option<Arc<dyn HashListener>>) -> bool {
        let observer = listener.map(|l| Arc::new(Bridge(l)) as Arc<dyn core::HashObserver>);
        self.service().start_hashing(observer)
    }

    /// Asks the running hashing to stop; it ends within a moment. Nothing happens when idle.
    pub fn stop_hashing(&self) {
        self.service().stop_hashing();
    }

    /// The state of the hashing and how much of the library is covered.
    pub fn hash_status(&self) -> Res<HashStatus> {
        Ok(self.service().hash_status()?.into())
    }

    /// The digests of the files of a game (one row per file, or per entry of an archive).
    pub fn game_hashes(&self, game_id: i64) -> Res<Vec<RomHash>> {
        Ok(self
            .service()
            .game_hashes(game_id)?
            .into_iter()
            .map(Into::into)
            .collect())
    }
}
