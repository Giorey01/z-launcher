//! Tests for game details, overrides, checksum hashing and gamelist integration.

use std::path::Path;
use zl_core::{CoreError, Service};
use zl_db::MetaField;

/// Result type for test assertions.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Sets up a service with a test ROM fixture.
fn setup_service_with_rom(dir: &Path) -> Result<Service, CoreError> {
    let service = Service::open(dir)?;
    let roms_dir = dir.join("roms").join("gba");
    std::fs::create_dir_all(&roms_dir).map_err(|_| CoreError::Database)?;
    let rom_path = roms_dir.join("Metroid Fusion (USA).gba");
    std::fs::write(&rom_path, b"TEST GBA ROM").map_err(|_| CoreError::Database)?;
    service.add_root(
        roms_dir
            .parent()
            .unwrap_or(Path::new(""))
            .to_str()
            .unwrap_or(""),
        None,
    )?;
    service.scan(&zl_library::NoProgress)?;
    Ok(service)
}

/// Tests loading game details and setting metadata overrides.
#[test]
fn test_game_details_and_overrides() -> TestResult {
    let dir = tempfile::tempdir()?;
    let service = setup_service_with_rom(dir.path())?;
    let games = service.games(Some("gba"))?;
    assert_eq!(games.len(), 1);
    let game_id = games[0].id;

    let details = service.game_details(game_id)?;
    assert_eq!(details.display_title, "Metroid Fusion");
    assert_eq!(details.platform_name, "Nintendo Game Boy Advance");

    service.set_game_override(game_id, MetaField::Title, "Custom Metroid Title")?;
    service.set_game_override(game_id, MetaField::Description, "Custom synopsis by user")?;

    let updated = service.game_details(game_id)?;
    assert_eq!(updated.display_title, "Custom Metroid Title");
    assert_eq!(
        updated.description.as_deref(),
        Some("Custom synopsis by user")
    );

    service.clear_game_override(game_id, MetaField::Title)?;
    let restored = service.game_details(game_id)?;
    assert_eq!(restored.display_title, "Metroid Fusion");
    Ok(())
}

/// Tests scraper config roundtrip.
#[test]
fn test_scraper_config_roundtrip() -> TestResult {
    let dir = tempfile::tempdir()?;
    let service = Service::open(dir.path())?;
    let config = service.scraper_config()?;
    assert_eq!(config.sources_order.len(), 1);
    assert!(!config.overwrite_protected);

    let mut new_config = config;
    new_config.overwrite_protected = true;
    new_config.steamgriddb_key = Some("key_abc".to_string());
    service.set_scraper_config(&new_config)?;

    let loaded = service.scraper_config()?;
    assert!(loaded.overwrite_protected);
    assert_eq!(loaded.steamgriddb_key.as_deref(), Some("key_abc"));
    Ok(())
}

/// The gamelists of an ES-DE folder are read per platform: a gamelist of `gba` fills only the
/// GBA game even when an NDS game has the same file name, and the ES-DE date becomes ISO.
#[test]
fn imports_esde_gamelists_per_platform() -> TestResult {
    let dir = tempfile::tempdir()?;
    for platform in ["gba", "nds"] {
        let folder = dir.path().join("roms").join(platform);
        std::fs::create_dir_all(&folder)?;
        std::fs::write(folder.join(format!("Same Name.{platform}")), b"rom")?;
    }
    let lists = dir.path().join("ES-DE/gamelists/gba");
    std::fs::create_dir_all(&lists)?;
    std::fs::write(
        lists.join("gamelist.xml"),
        "<gameList><game><path>./Same Name.gba</path><desc>Only GBA</desc>\
         <releasedate>20021117T000000</releasedate><players>2</players></game></gameList>",
    )?;
    let service = Service::open(&dir.path().join("data"))?;
    service.add_root(&dir.path().join("roms").to_string_lossy(), None)?;
    service.scan(&zl_library::NoProgress)?;
    service.set_media_sources(None, Some(&dir.path().join("ES-DE").to_string_lossy()))?;

    assert_eq!(service.import_esde_gamelists()?, 1);
    let gba = service.games(Some("gba"))?[0].id;
    let nds = service.games(Some("nds"))?[0].id;
    let details = service.game_details(gba)?;
    assert_eq!(details.description.as_deref(), Some("Only GBA"));
    assert_eq!(details.release_date.as_deref(), Some("2002-11-17"));
    assert_eq!(details.players, Some(2));
    assert_eq!(service.game_details(nds)?.description, None);
    Ok(())
}
