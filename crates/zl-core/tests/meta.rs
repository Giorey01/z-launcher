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
