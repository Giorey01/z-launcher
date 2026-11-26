#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::process::Command;
use tempfile::tempdir;
use zl_archive::ArchiveExtractor;

#[test]
fn test_7z_extracted_with_folders_nothing_outside() {
    let dir = tempdir().expect("tempdir");
    let src_game = dir.path().join("src/Game");
    fs::create_dir_all(&src_game).expect("create src/Game");

    fs::write(src_game.join("setup.exe"), b"exe").expect("write setup.exe");
    fs::write(src_game.join("data.bin"), b"bin").expect("write data.bin");

    let archive_path = dir.path().join("game.7z");

    let status = Command::new("7z")
        .args(["a", "-bd"])
        .arg(&archive_path)
        .arg("Game")
        .current_dir(dir.path().join("src"))
        .status()
        .expect("run 7z command");

    assert!(status.success(), "7z archive creation failed");

    let out = dir.path().join("out");
    fs::create_dir_all(&out).expect("create out");

    let extracted = ArchiveExtractor::extract(&archive_path, &out, None).expect("extract 7z");

    let mut rel_paths: Vec<String> = extracted
        .iter()
        .map(|p| {
            p.strip_prefix(&out)
                .expect("relative path")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    rel_paths.sort();

    assert_eq!(rel_paths, vec!["Game/data.bin", "Game/setup.exe"]);

    let exe_content = fs::read_to_string(out.join("Game/setup.exe")).expect("read exe");
    assert_eq!(exe_content, "exe");
}
