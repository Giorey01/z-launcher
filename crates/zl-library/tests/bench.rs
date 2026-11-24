//! Performance benchmark: `cargo test --release -p zl-library -- --ignored --nocapture`.

mod common;

use std::path::Path;
use std::time::Instant;

use common::{quiet_scan, root, write, TestResult};
use zl_db::Db;
use zl_library::ExtensionRules;

/// Platforms of the synthetic library.
const PLATFORMS: [&str; 20] = [
    "nes",
    "snes",
    "gba",
    "gbc",
    "nds",
    "n64",
    "psx",
    "ps2",
    "psp",
    "dreamcast",
    "saturn",
    "genesis",
    "segacd",
    "pce",
    "neogeo",
    "mame",
    "3do",
    "atari2600",
    "switch",
    "wiiu",
];

/// Creates about 5000 files: plain ROMs, m3u sets, cue/bin pairs, disc groups, tag files.
fn generate(root: &Path) -> std::io::Result<()> {
    for (p, platform) in PLATFORMS.iter().enumerate() {
        for i in 0..150 {
            write(root, &format!("{platform}/Game {p}-{i} (Europe).rom"), "x")?;
        }
        for i in 0..20 {
            let sub = format!("{platform}/Hacks/Set {}", i % 4);
            write(root, &format!("{sub}/Hack {i}.rom"), "x")?;
        }
        for i in 0..10 {
            write(root, &format!("{platform}/Multi {i} (Disc 1).rom"), "x")?;
            write(root, &format!("{platform}/Multi {i} (Disc 2).rom"), "x")?;
            write(root, &format!("{platform}/Pl {i} (Disc 1).chd"), "x")?;
            write(root, &format!("{platform}/Pl {i} (Disc 2).chd"), "x")?;
            write(
                root,
                &format!("{platform}/Pl {i}.m3u"),
                &format!("Pl {i} (Disc 1).chd\nPl {i} (Disc 2).chd\n"),
            )?;
            write(
                root,
                &format!("{platform}/Cd {i}.cue"),
                &format!("FILE \"Cd {i}.bin\" BINARY\n"),
            )?;
            write(root, &format!("{platform}/Cd {i}.bin"), "x")?;
            write(
                root,
                &format!("{platform}/Tag {i}.steamappid"),
                &format!("[steamappid] {i}\n"),
            )?;
        }
    }
    Ok(())
}

/// Full scan under 10 s and an unchanged rescan under 2 s (meant for `--release`).
#[test]
#[ignore = "benchmark: run with cargo test --release -- --ignored --nocapture"]
fn scan_5000_files() -> TestResult {
    let dir = tempfile::tempdir()?;
    generate(dir.path())?;
    let mut rules = ExtensionRules::new();
    for platform in PLATFORMS {
        rules.insert(platform, &["rom", "chd", "m3u", "cue", "steamappid"]);
    }
    let db = Db::open(&dir.path().join("lib.db"))?;
    let t = Instant::now();
    let first = quiet_scan(&db, &[root(dir.path())], &rules)?;
    let full = t.elapsed();
    let t = Instant::now();
    let second = quiet_scan(&db, &[root(dir.path())], &rules)?;
    let rescan = t.elapsed();
    println!(
        "files={} games={} full={full:?} rescan={rescan:?}",
        first.files_seen, first.added
    );
    assert!(
        first.files_seen >= 5000,
        "fixture too small: {}",
        first.files_seen
    );
    assert_eq!((second.added, second.updated, second.missing), (0, 0, 0));
    assert_eq!(second.unchanged, first.added);
    assert!(full.as_secs_f64() < 10.0, "full scan too slow: {full:?}");
    assert!(rescan.as_secs_f64() < 2.0, "rescan too slow: {rescan:?}");
    Ok(())
}
