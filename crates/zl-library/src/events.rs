//! Structured progress events and the scan summary. No sentences: the apps translate the codes.

/// Kind of per-file or per-folder problem found during a scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueCode {
    /// A root folder does not exist or is not readable (SD card removed?).
    RootUnavailable,
    /// A folder could not be listed.
    ReadDirFailed,
    /// A file could not be inspected.
    StatFailed,
    /// A name is not valid UTF-8 and was skipped.
    NonUtf8Name,
    /// A tag file is missing its identifying key or has an unknown extension.
    InvalidTagFile,
    /// A tag file could not be read.
    TagFileUnreadable,
    /// A playlist or CUE/GDI sheet could not be read; it is treated as a plain file.
    SheetUnreadable,
}

impl IssueCode {
    /// Stable snake_case name for the apps and the fixtures.
    pub fn as_str(self) -> &'static str {
        match self {
            IssueCode::RootUnavailable => "root_unavailable",
            IssueCode::ReadDirFailed => "read_dir_failed",
            IssueCode::StatFailed => "stat_failed",
            IssueCode::NonUtf8Name => "non_utf8_name",
            IssueCode::InvalidTagFile => "invalid_tag_file",
            IssueCode::TagFileUnreadable => "tag_file_unreadable",
            IssueCode::SheetUnreadable => "sheet_unreadable",
        }
    }
}

/// One problem, with the path it concerns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanIssue {
    /// What went wrong.
    pub code: IssueCode,
    /// The file or folder concerned.
    pub path: String,
}

/// Outcome of a scan.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanSummary {
    /// Games found that were not in the database.
    pub added: u32,
    /// Games whose content changed and were rewritten.
    pub updated: u32,
    /// Games found again exactly as stored.
    pub unchanged: u32,
    /// Games newly flagged missing by this scan.
    pub missing: u32,
    /// Games that were flagged missing and have reappeared.
    pub restored: u32,
    /// Number of problems (can exceed `issues.len()`, which is capped).
    pub errors: u32,
    /// Files looked at.
    pub files_seen: u64,
    /// The first problems found, capped by `zl_config::library::MAX_REPORTED_ISSUES`.
    pub issues: Vec<ScanIssue>,
}

/// Progress of a running scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanEvent {
    /// The scan begins: roots to scan and platform folders found in them.
    Started {
        /// Number of roots that will be scanned.
        roots: u32,
        /// Number of platform folders found.
        platforms: u32,
    },
    /// A platform folder starts (`index` is 1-based).
    PlatformStarted {
        /// Position of this platform among `total`.
        index: u32,
        /// Number of platform folders in the whole scan.
        total: u32,
        /// Platform id (folder name, lower case).
        platform_id: String,
    },
    /// Running count of files looked at, sent every few hundred files.
    FilesSeen {
        /// Files seen so far.
        count: u64,
    },
    /// The scan is over.
    Finished {
        /// Final numbers.
        summary: ScanSummary,
    },
}

/// Receiver of scan events. Must be thread-safe: it will be bridged to Kotlin.
pub trait ScanProgress: Send + Sync {
    /// Called for every event, on the scanning thread.
    fn on_event(&self, event: ScanEvent);
}

/// A progress receiver that ignores everything.
pub struct NoProgress;

impl ScanProgress for NoProgress {
    /// Drops the event.
    fn on_event(&self, _event: ScanEvent) {}
}

impl<F: Fn(ScanEvent) + Send + Sync> ScanProgress for F {
    /// Forwards the event to the closure.
    fn on_event(&self, event: ScanEvent) {
        self(event);
    }
}
