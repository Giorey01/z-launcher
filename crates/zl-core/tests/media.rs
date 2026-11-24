//! Runs `fixtures/media/expected.json`: scan the fixture ROM folder, import the ES-DE and Cocoon
//! media, and check the best file of every kind, the counts and that the user's choice survives.

use std::collections::BTreeMap;
use std::error::Error;
use std::path::PathBuf;

use serde_json::Value;
use zl_core::{GameEntry, Service};
use zl_db::{MediaKind, MediaSource};
use zl_library::NoProgress;

/// Result type of the tests.
type Res = Result<(), Box<dyn Error>>;

/// The shared fixtures folder, resolved.
fn fixtures() -> Result<PathBuf, Box<dyn Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/media")
        .canonicalize()?)
}

/// A service with the fixture library scanned and the media sources pointing at the fixtures.
fn service(base: &std::path::Path) -> Result<(tempfile::TempDir, Service), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let svc = Service::open(&dir.path().join("data"))?;
    svc.add_root(&base.join("roms").to_string_lossy(), None)?;
    svc.scan(&NoProgress)?;
    svc.set_media_sources(
        Some(&base.join("cocoon").to_string_lossy()),
        Some(&base.join("esde").to_string_lossy()),
    )?;
    Ok((dir, svc))
}

/// The expected document.
fn expected(base: &std::path::Path) -> Result<Value, Box<dyn Error>> {
    Ok(serde_json::from_str(&std::fs::read_to_string(
        base.join("expected.json"),
    )?)?)
}

/// A path relative to the fixtures folder, as text.
fn rel(base: &std::path::Path, path: &str) -> String {
    path.strip_prefix(&format!("{}/", base.display()))
        .unwrap_or(path)
        .to_string()
}

/// The game of the listing with this stored title.
fn game<'a>(games: &'a [GameEntry], title: &str) -> Result<&'a GameEntry, Box<dyn Error>> {
    games
        .iter()
        .find(|g| g.title == title)
        .ok_or_else(|| format!("no game {title}").into())
}

/// Import results and best media per game match the fixture, display titles included.
#[test]
fn import_matches_expected() -> Res {
    let base = fixtures()?;
    let (_dir, svc) = service(&base)?;
    let doc = expected(&base)?;
    let report = svc.import_media()?;
    assert_eq!(report.scanned, vec![MediaSource::Cocoon, MediaSource::Esde]);
    assert_eq!(
        u64::from(report.unmatched),
        doc["unmatched"].as_u64().unwrap_or(99)
    );
    let mut got: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();
    for c in &report.counts {
        got.entry(c.source.as_str().into())
            .or_default()
            .insert(c.kind.as_str().into(), u64::from(c.count));
    }
    let want: BTreeMap<String, BTreeMap<String, u64>> =
        serde_json::from_value(doc["counts"].clone())?;
    assert_eq!(got, want);
    let games = svc.games(None)?;
    for (title, case) in doc["games"].as_object().ok_or("games")? {
        let entry = game(&games, title)?;
        assert_eq!(
            entry.display_title,
            case["display_title"].as_str().unwrap_or_default(),
            "{title}"
        );
        let media = svc.game_media(entry.id)?;
        let mut best: BTreeMap<String, String> = BTreeMap::new();
        for item in media
            .items
            .iter()
            .filter(|i| i.kind != MediaKind::Screenshot)
        {
            best.entry(item.kind.as_str().to_string())
                .or_insert_with(|| rel(&base, &item.path));
        }
        let want_best: BTreeMap<String, String> = serde_json::from_value(case["best"].clone())?;
        assert_eq!(best, want_best, "{title}");
        let shots: Vec<String> = media.screenshots.iter().map(|p| rel(&base, p)).collect();
        let want_shots: Vec<String> = serde_json::from_value(case["screenshots"].clone())?;
        assert_eq!(shots, want_shots, "{title}");
        assert_eq!(
            entry.icon.as_deref().map(|p| rel(&base, p)),
            want_best.get("icon").cloned(),
            "{title}"
        );
        assert_eq!(
            entry.logo.as_deref().map(|p| rel(&base, p)),
            want_best.get("logo").cloned(),
            "{title}"
        );
        assert_eq!(
            entry.hero.as_deref().map(|p| rel(&base, p)),
            want_best.get("hero").cloned(),
            "{title}"
        );
    }
    Ok(())
}

/// A user's choice wins over every import and survives a second import; clearing it brings the
/// imported file back.
#[test]
fn user_choice_is_protected() -> Res {
    let base = fixtures()?;
    let (_dir, svc) = service(&base)?;
    let doc = expected(&base)?;
    let over = &doc["user_override"];
    let games = svc.games(None)?;
    let id = game(&games, over["game"].as_str().unwrap_or_default())?.id;
    let mine = base
        .join(over["path"].as_str().unwrap_or_default())
        .to_string_lossy()
        .into_owned();
    svc.set_game_media(id, MediaKind::Icon, Some(&mine))?;
    svc.import_media()?;
    svc.import_media()?;
    let media = svc.game_media(id)?;
    assert_eq!(media.icon.as_deref(), Some(mine.as_str()));
    let user_rows: Vec<_> = media.items.iter().filter(|i| i.protected).collect();
    assert_eq!(user_rows.len(), 1);
    assert_eq!(user_rows[0].source, MediaSource::User);
    assert!(media
        .items
        .iter()
        .any(|i| i.kind == MediaKind::Icon && i.source == MediaSource::Cocoon));
    assert_eq!(
        svc.games(None)?
            .iter()
            .find(|g| g.id == id)
            .and_then(|g| g.icon.clone()),
        Some(mine)
    );
    svc.set_game_media(id, MediaKind::Icon, None)?;
    let icon = svc.game_media(id)?.icon.unwrap_or_default();
    assert!(
        icon.ends_with("cocoon/downloaded_media/snes/icon/zelda (usa).png"),
        "{icon}"
    );
    Ok(())
}

/// A source whose folder is missing is not scanned and its stored media is kept.
#[test]
fn missing_source_keeps_its_rows() -> Res {
    let base = fixtures()?;
    let (dir, svc) = service(&base)?;
    svc.import_media()?;
    svc.set_media_sources(
        Some(&base.join("cocoon").to_string_lossy()),
        Some(&dir.path().join("nowhere").to_string_lossy()),
    )?;
    let report = svc.import_media()?;
    assert_eq!(report.scanned, vec![MediaSource::Cocoon]);
    let games = svc.games(None)?;
    let video = svc
        .game_media(game(&games, "Advance Wars (USA)")?.id)?
        .video;
    assert!(video.is_some(), "ES-DE rows must stay");
    Ok(())
}

/// Removing a root takes its games and their media with it.
#[test]
fn removing_a_root_removes_games() -> Res {
    let base = fixtures()?;
    let (_dir, svc) = service(&base)?;
    svc.import_media()?;
    let root = svc.roots()?.remove(0);
    svc.remove_root(root.id)?;
    assert!(svc.games(None)?.is_empty());
    assert!(svc.roots()?.is_empty());
    Ok(())
}

/// The settings screen lists every platform with its default and the installed candidates.
#[test]
fn platform_emulator_settings() -> Res {
    let base = fixtures()?;
    let (_dir, svc) = service(&base)?;
    let none = svc.platform_emulators(&[])?;
    assert!(none.iter().any(|p| p.platform_id == "gba"));
    assert!(none
        .iter()
        .all(|p| p.current.is_none() && p.candidates.is_empty()));
    let detection = svc.detect(&["org.ppsspp.ppsspp".to_string(), "com.retroarch".to_string()])?;
    let first = detection.proposed.first().ok_or("a proposal")?;
    svc.set_platform_default(&first.platform_id, Some(&first.candidate.profile))?;
    let listed = svc.platform_emulators(&["com.retroarch".to_string()])?;
    let row = listed
        .iter()
        .find(|p| p.platform_id == first.platform_id)
        .ok_or("row")?;
    assert_eq!(
        row.current.as_ref().map(|c| &c.profile),
        Some(&first.candidate.profile)
    );
    Ok(())
}
