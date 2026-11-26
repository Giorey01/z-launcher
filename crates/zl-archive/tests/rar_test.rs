//! RAR extraction against the shared cases of `fixtures/archive/rar.json` (the expected contents come
//! from the official UnRAR): every case, wrong and missing passwords, damaged and missing
//! volumes, stopping, and consuming a volume set while it is extracted.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tempfile::{tempdir, TempDir};
use zl_archive::rar::{self, RarOptions};
use zl_archive::{ArchiveError, ArchiveExtractor};

/// The repository's `fixtures` folder.
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

/// The parsed `fixtures/archive/rar.json`.
fn doc() -> Value {
    serde_json::from_str(&fs::read_to_string(fixtures().join("archive/rar.json")).unwrap()).unwrap()
}

/// The case called `name`.
fn case(name: &str) -> Value {
    doc()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap()
        .clone()
}

/// Copies the volumes of `case` into a fresh folder; returns it and the first volume there.
fn copy_set(case: &Value) -> (TempDir, PathBuf) {
    let tmp = tempdir().unwrap();
    let first = fixtures().join(case["first"].as_str().unwrap());
    for entry in fs::read_dir(first.parent().unwrap()).unwrap() {
        let entry = entry.unwrap();
        fs::copy(entry.path(), tmp.path().join(entry.file_name())).unwrap();
    }
    let copied = tmp.path().join(first.file_name().unwrap());
    (tmp, copied)
}

/// Relative path → sha256 of every file under `dir`.
fn hashes(dir: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p
                    .strip_prefix(dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(rel, hex::encode(Sha256::digest(fs::read(&p).unwrap())));
            }
        }
    }
    out
}

/// The expected files of `case`.
fn expected(case: &Value) -> BTreeMap<String, String> {
    case["files"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
        .collect()
}

/// Options with the case's password.
fn options(case: &Value) -> RarOptions {
    RarOptions {
        password: case["password"].as_str().map(str::to_string),
        ..RarOptions::default()
    }
}

/// The names of every case.
fn case_names() -> Vec<String> {
    doc()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect()
}

/// The error code of a RAR failure.
fn code(err: &ArchiveError) -> &'static str {
    match err {
        ArchiveError::RarError { code, .. } => code,
        ArchiveError::Cancelled => "CANCELLED",
        _ => "OTHER",
    }
}

/// Number of files directly in `dir`.
fn count(dir: &Path) -> usize {
    fs::read_dir(dir).map(|r| r.count()).unwrap_or(0)
}

#[test]
fn every_case_extracts_what_unrar_extracts() {
    for name in case_names() {
        let c = case(&name);
        let (tmp, first) = copy_set(&c);
        let out = tmp.path().join("out");
        let done =
            rar::extract(&first, &out, &options(&c)).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(hashes(&out), expected(&c), "{name}");
        assert_eq!(done.files.len(), expected(&c).len(), "{name}");
        assert_eq!(
            done.volumes.len() as u64,
            c["volumes"].as_u64().unwrap(),
            "{name}"
        );
        assert!(!done.consumed, "{name}");
        assert_eq!(
            count(tmp.path()) as u64,
            c["volumes"].as_u64().unwrap() + 1,
            "{name}: volumes kept"
        );
    }
}

#[test]
fn inspect_reads_size_volumes_and_solidity() {
    for name in case_names() {
        let c = case(&name);
        let (tmp, first) = copy_set(&c);
        let info = rar::inspect(&first, c["password"].as_str()).unwrap();
        assert_eq!(
            info.volumes.len() as u64,
            c["volumes"].as_u64().unwrap(),
            "{name}"
        );
        assert_eq!(info.solid, c["solid"].as_bool().unwrap(), "{name}");
        let out = tmp.path().join("out");
        rar::extract(&first, &out, &options(&c)).unwrap();
        let size: u64 = hashes(&out)
            .keys()
            .map(|k| out.join(k).metadata().unwrap().len())
            .sum();
        assert_eq!(info.unpacked, size, "{name}");
        assert!(info.largest_volume > 0, "{name}");
    }
}

#[test]
fn file_names_that_start_a_set() {
    let d = doc();
    for n in d["names"]["first"].as_array().unwrap() {
        assert!(rar::is_first_volume_name(n.as_str().unwrap()), "{n}");
    }
    for n in d["names"]["later"].as_array().unwrap() {
        assert!(!rar::is_first_volume_name(n.as_str().unwrap()), "{n}");
    }
    let (_tmp, first) = copy_set(&case("rar5-volumes"));
    assert!(rar::is_first_volume(&first));
    assert!(!rar::is_first_volume(
        &first.with_file_name("set.part2.rar")
    ));
}

#[test]
fn wrong_or_missing_password_writes_nothing() {
    for name in ["rar5-password", "rar5-headers"] {
        let c = case(name);
        let (tmp, first) = copy_set(&c);
        let out = tmp.path().join("out");
        for (password, codes) in [
            (Some("wrong"), ["BAD_PASSWORD", "BAD_PASSWORD"]),
            (None, ["PASSWORD_REQUIRED", "BAD_PASSWORD"]),
        ] {
            let opts = RarOptions {
                password: password.map(str::to_string),
                ..RarOptions::default()
            };
            let err = rar::extract(&first, &out, &opts).unwrap_err();
            assert!(codes.contains(&code(&err)), "{name} {password:?}: {err}");
            assert!(hashes(&out).is_empty(), "{name}: nothing left behind");
            assert_eq!(count(tmp.path()) as u64, c["volumes"].as_u64().unwrap() + 1);
        }
    }
}

/// Flips one byte in the middle of the data of `volume`.
fn damage(volume: &Path) {
    let mut bytes = fs::read(volume).unwrap();
    let at = bytes.len() / 2;
    bytes[at] ^= 0xFF;
    fs::write(volume, bytes).unwrap();
}

#[test]
fn a_damaged_volume_rolls_everything_back() {
    let (tmp, first) = copy_set(&case("rar5-volumes"));
    damage(&first.with_file_name("set.part3.rar"));
    let out = tmp.path().join("out");
    let err = rar::extract(&first, &out, &RarOptions::default()).unwrap_err();
    assert_eq!(code(&err), "CHECKSUM_MISMATCH", "{err}");
    assert!(hashes(&out).is_empty());
    assert_eq!(count(tmp.path()), 5, "all four volumes kept");
}

#[test]
fn a_missing_volume_is_an_error_and_writes_nothing() {
    let (tmp, first) = copy_set(&case("rar5-volumes"));
    fs::remove_file(first.with_file_name("set.part3.rar")).unwrap();
    let out = tmp.path().join("out");
    assert!(rar::extract(&first, &out, &RarOptions::default()).is_err());
    assert!(hashes(&out).is_empty());
}

#[test]
fn a_stop_request_cancels_and_cleans_up() {
    let (tmp, first) = copy_set(&case("rar5-volumes"));
    let out = tmp.path().join("out");
    let opts = RarOptions {
        cancel: Some(Arc::new(AtomicBool::new(true))),
        ..RarOptions::default()
    };
    let err = rar::extract(&first, &out, &opts).unwrap_err();
    assert!(matches!(err, ArchiveError::Cancelled), "{err}");
    assert!(hashes(&out).is_empty());
}

/// Options that consume the set and record, at every progress report, how many volumes are left.
fn consuming(c: &Value, dir: &Path, seen: Arc<Mutex<Vec<usize>>>) -> RarOptions {
    let dir = dir.to_path_buf();
    RarOptions {
        consume: true,
        progress: Some(Arc::new(move |_, _| {
            let left = fs::read_dir(&dir)
                .unwrap()
                .filter(|e| e.as_ref().unwrap().path().is_file())
                .count();
            seen.lock().unwrap().push(left);
        })),
        ..options(c)
    }
}

#[test]
fn consuming_a_set_frees_volumes_while_it_extracts() {
    for name in [
        "rar5-volumes",
        "rar5-solid-volumes",
        "rar5-password",
        "rar4-volumes",
    ] {
        let c = case(name);
        let (tmp, first) = copy_set(&c);
        let out = tmp.path().join("out");
        let seen = Arc::new(Mutex::new(Vec::new()));
        let done = rar::extract(&first, &out, &consuming(&c, tmp.path(), seen.clone())).unwrap();
        assert_eq!(hashes(&out), expected(&c), "{name}");
        assert!(done.consumed, "{name}");
        let volumes = c["volumes"].as_u64().unwrap() as usize;
        let seen = seen.lock().unwrap();
        assert!(
            seen.iter().any(|&left| left < volumes),
            "{name}: no volume freed early"
        );
        assert!(
            seen.windows(2).all(|w| w[1] <= w[0]),
            "{name}: volumes only go"
        );
    }
}

#[test]
fn consuming_keeps_the_extracted_files_when_a_later_volume_is_damaged() {
    let c = case("rar5-volumes");
    let (tmp, first) = copy_set(&c);
    damage(&first.with_file_name("set.part4.rar"));
    let out = tmp.path().join("out");
    let opts = RarOptions {
        consume: true,
        ..RarOptions::default()
    };
    let err = rar::extract(&first, &out, &opts).unwrap_err();
    assert_eq!(code(&err), "CHECKSUM_MISMATCH", "{err}");
    assert!(!first.exists(), "the first volume was consumed");
    let left = hashes(&out);
    assert!(!left.is_empty(), "the files already out stay");
    let want = expected(&c);
    for (path, hash) in &left {
        assert_eq!(want.get(path), Some(hash), "{path} is whole");
    }
}

#[test]
fn a_stop_request_is_ignored_once_a_volume_is_gone() {
    let c = case("rar5-volumes");
    let (tmp, first) = copy_set(&c);
    let out = tmp.path().join("out");
    let cancel = Arc::new(AtomicBool::new(false));
    let (flag, first_seen) = (cancel.clone(), first.clone());
    let calls = Arc::new(AtomicUsize::new(0));
    let opts = RarOptions {
        consume: true,
        cancel: Some(cancel),
        progress: Some(Arc::new(move |_, _| {
            calls.fetch_add(1, Ordering::Relaxed);
            if !first_seen.exists() {
                flag.store(true, Ordering::Relaxed);
            }
        })),
        ..RarOptions::default()
    };
    let done = rar::extract(&first, &out, &opts).unwrap();
    assert!(done.consumed);
    assert_eq!(hashes(&out), expected(&c));
}

#[test]
fn consuming_a_single_volume_archive_keeps_it() {
    let c = case("rar5-single");
    let (tmp, first) = copy_set(&c);
    let out = tmp.path().join("out");
    let opts = RarOptions {
        consume: true,
        ..RarOptions::default()
    };
    let done = rar::extract(&first, &out, &opts).unwrap();
    assert!(!done.consumed);
    assert!(first.exists());
}

#[test]
fn the_generic_extractor_reads_rar_too() {
    let c = case("rar4-volumes");
    let (tmp, first) = copy_set(&c);
    let out = tmp.path().join("out");
    let files = ArchiveExtractor::extract(&first, &out, None).unwrap();
    assert_eq!(files.len(), expected(&c).len());
    assert_eq!(hashes(&out), expected(&c));
}

#[test]
fn how_many_parts_a_list_of_names_makes() {
    for case in doc()["names"]["splitCount"].as_array().unwrap() {
        let names: Vec<&str> = case["names"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_str().unwrap())
            .collect();
        let want = case["parts"].as_u64().unwrap() as usize;
        assert_eq!(rar::split_volume_count(names.clone()), want, "{names:?}");
    }
}
