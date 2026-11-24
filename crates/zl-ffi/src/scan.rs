//! Scan progress as UniFFI types: the foreign listener and the events it receives.

use std::sync::Arc;

use zl_library as lib;

/// A problem found during a scan.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ScanIssue {
    /// Stable code (`root_unavailable`, `read_dir_failed`, ...).
    pub code: String,
    /// The file or folder concerned.
    pub path: String,
}

/// The numbers of a finished scan.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ScanSummary {
    /// Games that were not in the library.
    pub added: u32,
    /// Games rewritten because they changed.
    pub updated: u32,
    /// Games found exactly as stored.
    pub unchanged: u32,
    /// Games newly flagged missing.
    pub missing: u32,
    /// Games that reappeared.
    pub restored: u32,
    /// Number of problems.
    pub errors: u32,
    /// Files looked at.
    pub files_seen: u64,
    /// The first problems.
    pub issues: Vec<ScanIssue>,
}

/// Progress of a running scan.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum ScanEvent {
    /// The scan begins.
    Started {
        /// Roots that will be scanned.
        roots: u32,
        /// Platform folders found.
        platforms: u32,
    },
    /// A platform folder starts (`index` is 1-based).
    PlatformStarted {
        /// Position among `total`.
        index: u32,
        /// Platform folders in the scan.
        total: u32,
        /// Platform id (folder name, lower case).
        platform_id: String,
    },
    /// Running count of files looked at.
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

/// Receiver of scan events, implemented in Kotlin. Called on the scanning thread.
#[uniffi::export(with_foreign)]
pub trait ScanListener: Send + Sync {
    /// One event of the scan.
    fn on_event(&self, event: ScanEvent);
}

impl From<lib::ScanSummary> for ScanSummary {
    /// Copies the numbers and turns the issue codes into their stable names.
    fn from(s: lib::ScanSummary) -> Self {
        ScanSummary {
            added: s.added,
            updated: s.updated,
            unchanged: s.unchanged,
            missing: s.missing,
            restored: s.restored,
            errors: s.errors,
            files_seen: s.files_seen,
            issues: s
                .issues
                .into_iter()
                .map(|i| ScanIssue {
                    code: i.code.as_str().to_string(),
                    path: i.path,
                })
                .collect(),
        }
    }
}

impl From<lib::ScanEvent> for ScanEvent {
    /// One to one.
    fn from(e: lib::ScanEvent) -> Self {
        match e {
            lib::ScanEvent::Started { roots, platforms } => ScanEvent::Started { roots, platforms },
            lib::ScanEvent::PlatformStarted {
                index,
                total,
                platform_id,
            } => ScanEvent::PlatformStarted {
                index,
                total,
                platform_id,
            },
            lib::ScanEvent::FilesSeen { count } => ScanEvent::FilesSeen { count },
            lib::ScanEvent::Finished { summary } => ScanEvent::Finished {
                summary: summary.into(),
            },
        }
    }
}

/// Adapts the foreign listener to the scanner's progress trait.
pub struct Bridge(pub Arc<dyn ScanListener>);

impl lib::ScanProgress for Bridge {
    /// Forwards the event to Kotlin.
    fn on_event(&self, event: lib::ScanEvent) {
        self.0.on_event(event.into());
    }
}
