//! The scan itself: walk each platform folder, group files into games, and reconcile the result
//! with the database (insert, update, touch unchanged, flag missing, restore reappeared).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use zl_db::{Db, DbError, ExistingGame, GameFile, NewGame};

use crate::events::{IssueCode, ScanEvent, ScanIssue, ScanProgress, ScanSummary};
use crate::fingerprint::fingerprint;
use crate::group::{group_platform, Unit};
use crate::rules::ExtensionRules;
use crate::walk::{walk_platform, FileEntry};

/// Errors that abort a scan (problems with single files are reported in the summary instead).
#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
    /// The database failed.
    #[error("database error: {0}")]
    Db(#[from] DbError),
}

/// A root folder to scan.
#[derive(Debug, Clone)]
pub struct RootSpec {
    /// Absolute path of the folder that contains the platform folders.
    pub path: PathBuf,
    /// Enter sub-folders of platform folders (up to `zl_config::library::MAX_SCAN_DEPTH`).
    pub scan_subfolders: bool,
}

/// A platform folder to scan.
struct Job {
    /// Index into the list of usable roots.
    root: usize,
    /// Platform id (folder name, lower case).
    platform_id: String,
    /// Path of the platform folder.
    dir: PathBuf,
}

/// A root that exists and is enabled.
struct ActiveRoot {
    /// Database id.
    id: i64,
    /// What the caller asked for.
    spec: RootSpec,
}

/// Mutable state of a scan.
struct Run<'a> {
    /// The database.
    db: &'a Db,
    /// Where events go.
    progress: &'a dyn ScanProgress,
    /// Id of this scan.
    scan_id: i64,
    /// Numbers being accumulated.
    summary: ScanSummary,
    /// Platform folders started so far.
    platform_index: u32,
    /// Platform folders in the whole scan.
    platform_total: u32,
}

impl Run<'_> {
    /// Records a problem (counted always, listed up to the cap).
    fn issue(&mut self, issue: ScanIssue) {
        self.summary.errors += 1;
        if self.summary.issues.len() < zl_config::library::MAX_REPORTED_ISSUES {
            self.summary.issues.push(issue);
        }
    }

    /// Records the problems of a sub-step.
    fn issues(&mut self, list: Vec<ScanIssue>) {
        for item in list {
            self.issue(item);
        }
    }

    /// Counts a file seen and sends a progress event every so often.
    fn file_seen(&mut self) {
        self.summary.files_seen += 1;
        if self
            .summary
            .files_seen
            .is_multiple_of(zl_config::library::PROGRESS_EVERY_FILES)
        {
            self.progress.on_event(ScanEvent::FilesSeen {
                count: self.summary.files_seen,
            });
        }
    }
}

/// Converts a unit to a database row (with its fingerprint).
fn to_new_game(root_id: i64, platform: &str, files: &[FileEntry], unit: &Unit) -> Option<NewGame> {
    let members: Vec<GameFile> = unit
        .members
        .iter()
        .map(|&i| GameFile {
            path: files[i].path.to_string_lossy().into_owned(),
            size: i64::try_from(files[i].size).unwrap_or(i64::MAX),
            mtime_ms: files[i].mtime_ms,
        })
        .collect();
    let primary = members
        .iter()
        .position(|m| m.path == files[unit.primary].path.to_string_lossy())?;
    Some(NewGame {
        root_id,
        platform_id: platform.to_string(),
        title: unit.title.clone(),
        path: members[primary].path.clone(),
        kind: unit.kind,
        size: members.iter().map(|m| m.size).sum(),
        mtime_ms: members.iter().map(|m| m.mtime_ms).max().unwrap_or(0),
        fingerprint: fingerprint(unit.kind, &unit.title, &members),
        files: members,
        tags: unit.tags.clone(),
    })
}

/// What to do with one found game.
enum Action {
    /// Not in the database.
    Insert,
    /// In the database with a different shape or platform.
    Rewrite,
    /// Identical: only mark as seen.
    Touch,
}

/// Compares a found game with the stored one.
fn decide(game: &NewGame, stored: Option<&ExistingGame>) -> Action {
    match stored {
        None => Action::Insert,
        Some(s) if s.fingerprint != game.fingerprint || s.platform_id != game.platform_id => {
            Action::Rewrite
        }
        Some(_) => Action::Touch,
    }
}

/// Results of reconciling one platform.
#[derive(Default)]
struct Reconciled {
    /// Ids of the stored games seen on disk.
    seen: HashSet<i64>,
}

impl Run<'_> {
    /// Writes the games of one platform and updates counters. One transaction.
    fn reconcile(
        &mut self,
        games: &[NewGame],
        existing: &HashMap<String, ExistingGame>,
    ) -> Result<Reconciled, LibraryError> {
        let (db, scan_id) = (self.db, self.scan_id);
        let mut counts = ScanSummary::default();
        let mut out = Reconciled::default();
        db.transaction(|d| {
            let mut touched = Vec::new();
            for game in games {
                let stored = existing.get(&game.path);
                if stored.is_some_and(|s| s.missing) {
                    counts.restored += 1;
                }
                match decide(game, stored) {
                    Action::Insert => {
                        d.upsert_game(game, scan_id)?;
                        counts.added += 1;
                    }
                    Action::Rewrite => {
                        out.seen.insert(d.upsert_game(game, scan_id)?);
                        counts.updated += 1;
                    }
                    Action::Touch => {
                        let id = stored.map_or(0, |s| s.id);
                        touched.push(id);
                        out.seen.insert(id);
                        counts.unchanged += 1;
                    }
                }
                if let Some(s) = stored {
                    out.seen.insert(s.id);
                }
            }
            d.touch_games(&touched, scan_id)?;
            Ok::<(), LibraryError>(())
        })?;
        self.summary.added += counts.added;
        self.summary.updated += counts.updated;
        self.summary.unchanged += counts.unchanged;
        self.summary.restored += counts.restored;
        Ok(out)
    }
}

/// Lists the platform folders of a root whose name has rules, sorted by name.
fn platform_dirs(root: &Path, rules: &ExtensionRules) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut dirs: Vec<(String, PathBuf)> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let name = e.file_name().to_str()?.to_string();
            (!name.starts_with('.') && rules.extensions_for(&name).is_some())
                .then(|| (name.to_ascii_lowercase(), e.path()))
        })
        .collect();
    dirs.sort();
    dirs
}

/// Registers the roots in the database and keeps those that exist and are enabled; the others
/// are reported (a missing root means a removed card: its games are left untouched).
fn activate_roots(run: &mut Run<'_>, roots: &[RootSpec]) -> Result<Vec<ActiveRoot>, LibraryError> {
    let mut active = Vec::new();
    for spec in roots {
        let path = spec.path.to_string_lossy().into_owned();
        let id = run.db.upsert_root(&path, spec.scan_subfolders)?;
        let enabled = run.db.list_roots()?.iter().any(|r| r.id == id && r.enabled);
        if !enabled {
            continue;
        }
        if spec.path.is_dir() {
            active.push(ActiveRoot {
                id,
                spec: spec.clone(),
            });
        } else {
            run.issue(ScanIssue {
                code: IssueCode::RootUnavailable,
                path,
            });
        }
    }
    Ok(active)
}

/// Scans one platform folder and writes its games; returns the seen ids and completeness.
fn scan_platform(
    run: &mut Run<'_>,
    root: &ActiveRoot,
    job: &Job,
    rules: &ExtensionRules,
    visited: &mut HashSet<PathBuf>,
    existing: &HashMap<String, ExistingGame>,
) -> Result<(Reconciled, bool), LibraryError> {
    let Some(exts) = rules.extensions_for(&job.platform_id) else {
        return Ok((Reconciled::default(), true));
    };
    let depth = if root.spec.scan_subfolders {
        zl_config::library::MAX_SCAN_DEPTH
    } else {
        0
    };
    let mut counted = 0u64;
    let walked = walk_platform(&job.dir, depth, visited, &mut || counted += 1);
    for _ in 0..counted {
        run.file_seen();
    }
    run.issues(walked.issues);
    let globs = rules.folder_globs(&job.platform_id);
    let grouped = group_platform(&job.platform_id, &job.dir, exts, globs, &walked.files);
    run.issues(grouped.issues);
    let games: Vec<NewGame> = grouped
        .units
        .iter()
        .filter_map(|u| to_new_game(root.id, &job.platform_id, &walked.files, u))
        .collect();
    let seen = run.reconcile(&games, existing)?;
    Ok((seen, walked.complete))
}

/// Flags as missing the stored games of a root that were not seen, skipping incomplete platforms.
fn flag_missing(
    run: &mut Run<'_>,
    existing: &HashMap<String, ExistingGame>,
    seen: &HashSet<i64>,
    incomplete: &HashSet<String>,
) -> Result<(), LibraryError> {
    let lost: Vec<i64> = existing
        .values()
        .filter(|g| !seen.contains(&g.id) && !g.missing && !incomplete.contains(&g.platform_id))
        .map(|g| g.id)
        .collect();
    run.summary.missing += u32::try_from(lost.len()).unwrap_or(u32::MAX);
    run.db.transaction(|d| d.mark_missing(&lost))?;
    Ok(())
}

/// Scans every platform folder of one root, then flags what disappeared.
fn scan_root(
    run: &mut Run<'_>,
    root: &ActiveRoot,
    jobs: &[&Job],
    rules: &ExtensionRules,
) -> Result<(), LibraryError> {
    let existing: HashMap<String, ExistingGame> =
        run.db.existing_games(root.id)?.into_iter().collect();
    let mut visited = HashSet::new();
    let mut seen = HashSet::new();
    let mut incomplete = HashSet::new();
    for job in jobs {
        run.platform_index += 1;
        run.progress.on_event(ScanEvent::PlatformStarted {
            index: run.platform_index,
            total: run.platform_total,
            platform_id: job.platform_id.clone(),
        });
        let (got, complete) = scan_platform(run, root, job, rules, &mut visited, &existing)?;
        seen.extend(got.seen);
        if !complete {
            incomplete.insert(job.platform_id.clone());
        }
    }
    flag_missing(run, &existing, &seen, &incomplete)
}

/// Scans `roots` for games and updates the database.
///
/// `rules` gives the allowed extensions per platform id. Events go to `progress`. Games that are
/// no longer found are flagged missing, never deleted. A root that does not exist is reported and
/// left untouched.
pub fn scan(
    db: &Db,
    roots: &[RootSpec],
    rules: &ExtensionRules,
    progress: &dyn ScanProgress,
) -> Result<ScanSummary, LibraryError> {
    let scan_id = db.begin_scan()?;
    let mut run = Run {
        db,
        progress,
        scan_id,
        summary: ScanSummary::default(),
        platform_index: 0,
        platform_total: 0,
    };
    let active = activate_roots(&mut run, roots)?;
    let jobs: Vec<Job> = active
        .iter()
        .enumerate()
        .flat_map(|(root, r)| {
            platform_dirs(&r.spec.path, rules)
                .into_iter()
                .map(move |(platform_id, dir)| Job {
                    root,
                    platform_id,
                    dir,
                })
        })
        .collect();
    run.platform_total = u32::try_from(jobs.len()).unwrap_or(u32::MAX);
    progress.on_event(ScanEvent::Started {
        roots: u32::try_from(active.len()).unwrap_or(u32::MAX),
        platforms: run.platform_total,
    });
    for (index, root) in active.iter().enumerate() {
        let mine: Vec<&Job> = jobs.iter().filter(|j| j.root == index).collect();
        scan_root(&mut run, root, &mine, rules)?;
    }
    progress.on_event(ScanEvent::Finished {
        summary: run.summary.clone(),
    });
    Ok(run.summary)
}
