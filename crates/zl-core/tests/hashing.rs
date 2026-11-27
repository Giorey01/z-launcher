//! The background hashing against a scanned library built from `fixtures/hash`: what is hashed,
//! that an unchanged file is not read again, that a changed one is, and the stop request.

use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use zl_core::{HashEvent, HashObserver, HashRun, Service};
use zl_library::NoProgress;

/// Result type of the tests.
type Res = Result<(), Box<dyn Error>>;

/// Collects the events of a run.
#[derive(Default)]
struct Events(Mutex<Vec<HashEvent>>);

impl HashObserver for Events {
    /// Keeps the event.
    fn on_event(&self, event: HashEvent) {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).push(event);
    }
}

/// The shared fixtures folder.
fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/hash")
}

/// Copies a fixture into the ROM tree.
fn put(roms: &Path, platform: &str, name: &str) -> Res {
    std::fs::create_dir_all(roms.join(platform))?;
    std::fs::copy(fixtures().join(name), roms.join(platform).join(name))?;
    Ok(())
}

/// A service over a scanned library of five fixture files.
fn library() -> Result<(tempfile::TempDir, Service, PathBuf), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let roms = dir.path().join("roms");
    put(&roms, "nes", "mario.nes")?;
    put(&roms, "nes", "set.zip")?;
    put(&roms, "nes", "locked.zip")?;
    put(&roms, "psx", "disc-v5.chd")?;
    put(&roms, "snes", "game.smc")?;
    let svc = Service::open(&dir.path().join("data"))?;
    svc.add_root(&roms.to_string_lossy(), None)?;
    svc.scan(&NoProgress)?;
    Ok((dir, svc, roms))
}

/// The game whose file has this name (its title is the file name without the extension).
fn id_of(svc: &Service, file: &str) -> Result<i64, Box<dyn Error>> {
    svc.games(None)?
        .into_iter()
        .find(|g| {
            file.rsplit_once('.')
                .is_some_and(|(stem, _)| stem == g.title)
        })
        .map(|g| g.id)
        .ok_or_else(|| format!("no game {file}").into())
}

/// Every file is hashed, archives per entry, a CHD from its header, a failure is remembered,
/// and a second run has nothing to do.
#[test]
fn hashes_the_library_once() -> Res {
    let (_dir, svc, _roms) = library()?;
    let events = Events::default();
    let report = svc.hash_pending(Some(&events))?;
    assert_eq!((report.games, report.files, report.failed), (5, 4, 1));
    assert_eq!(
        report.entries, 5,
        "mario, two in the zip, the disc, the smc"
    );
    assert!(!report.stopped);
    let log = events.0.lock().map_err(|_| "poisoned")?;
    assert!(matches!(log[0], HashEvent::Started { games: 5, files: 5 }));
    assert!(log
        .iter()
        .any(|e| matches!(e, HashEvent::FileFailed { code, .. } if code == "password")));
    drop(log);

    let mario = svc.game_hashes(id_of(&svc, "mario.nes")?)?;
    assert_eq!(mario[0].header_bytes, 16);
    assert!(mario[0].crc32.is_some() && mario[0].hl_crc32.is_some());
    let zip = svc.game_hashes(id_of(&svc, "set.zip")?)?;
    assert_eq!(zip.len(), 2);
    let disc = svc.game_hashes(id_of(&svc, "disc-v5.chd")?)?;
    assert_eq!(
        (disc[0].container.as_str(), disc[0].crc32.as_deref()),
        ("chd", None)
    );
    assert!(disc[0].sha1.is_some());
    let locked = svc.game_hashes(id_of(&svc, "locked.zip")?)?;
    assert_eq!(locked[0].error.as_deref(), Some("password"));

    let again = svc.hash_pending(None)?;
    assert_eq!((again.games, again.files, again.failed), (0, 0, 0));
    let status = svc.hash_status()?;
    assert_eq!((status.library_games, status.library_hashed), (5, 5));
    assert_eq!(status.run, HashRun::Idle);
    Ok(())
}

/// A file that changed on disk is hashed again after the next scan, the others are not.
#[test]
fn changed_file_is_rehashed() -> Res {
    let (_dir, svc, roms) = library()?;
    svc.hash_pending(None)?;
    let path = roms.join("snes/game.smc");
    let mut bytes = std::fs::read(&path)?;
    bytes[600] ^= 0xff;
    std::fs::write(&path, &bytes)?;
    // Same size, new content: the modification time is what the scanner sees change.
    let later = std::time::SystemTime::now() + std::time::Duration::from_secs(60);
    std::fs::File::options()
        .write(true)
        .open(&path)?
        .set_modified(later)?;
    svc.scan(&NoProgress)?;
    let before = svc.game_hashes(id_of(&svc, "game.smc")?)?;
    assert!(before.is_empty(), "stale digests are not shown");
    let report = svc.hash_pending(None)?;
    assert_eq!((report.games, report.files), (1, 1));
    let after = svc.game_hashes(id_of(&svc, "game.smc")?)?;
    assert_eq!(after[0].header_bytes, 512);
    Ok(())
}

/// The background run ends by itself, reports through the observer and the status, and refuses
/// a second start while it goes.
#[test]
fn background_run_reports_and_ends() -> Res {
    let (_dir, svc, _roms) = library()?;
    let events = Arc::new(Events::default());
    assert!(svc.start_hashing(Some(events.clone())));
    assert!(!svc.start_hashing(None), "already running");
    for _ in 0..200 {
        if svc.hash_status()?.run == HashRun::Idle {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    let status = svc.hash_status()?;
    assert_eq!(status.run, HashRun::Idle);
    assert_eq!(status.library_hashed, 5);
    let last = status.last.ok_or("no report")?;
    assert_eq!(last.games, 5);
    let log = events.0.lock().map_err(|_| "poisoned")?;
    assert!(matches!(log.last(), Some(HashEvent::Finished { .. })));
    Ok(())
}

/// A stop request ends the run early and the rest stays pending for the next one.
#[test]
fn stop_leaves_the_rest_pending() -> Res {
    let (_dir, svc, _roms) = library()?;
    /// Asks the service to stop as soon as the first game is done.
    struct StopAfterFirst<'a>(&'a Service);
    impl HashObserver for StopAfterFirst<'_> {
        /// Stops on the first progress event.
        fn on_event(&self, event: HashEvent) {
            if matches!(event, HashEvent::Progress { .. }) {
                self.0.stop_hashing();
            }
        }
    }
    let report = svc.hash_pending(Some(&StopAfterFirst(&svc)))?;
    assert!(report.stopped);
    assert!(report.games < 5);
    let status = svc.hash_status()?;
    assert!(status.library_hashed < 5);
    Ok(())
}
