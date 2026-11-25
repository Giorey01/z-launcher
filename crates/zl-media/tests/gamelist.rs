//! Tests for ES-DE gamelist.xml parser and exporter.

use zl_media::{export_gamelist, parse_gamelist, GamelistEntry};

/// Result type for test assertions.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Sample XML content representing an EmulationStation gamelist.
const SAMPLE_XML: &str = r#"<?xml version="1.0"?>
<gameList>
    <game>
        <path>./Kirby.nds</path>
        <name>Kirby Mass Attack</name>
        <desc>A Nintendo DS platformer starring Kirby.</desc>
        <image>./covers/Kirby.png</image>
        <developer>HAL Laboratory</developer>
        <publisher>Nintendo</publisher>
        <genre>Platformer</genre>
        <players>1</players>
        <rating>0.85</rating>
    </game>
</gameList>"#;

/// Tests parsing a gamelist.xml.
#[test]
fn test_parse_gamelist() -> TestResult {
    let entries = parse_gamelist(SAMPLE_XML)?;
    assert_eq!(entries.len(), 1);
    let e = &entries[0];
    assert_eq!(e.path, "./Kirby.nds");
    assert_eq!(e.name.as_deref(), Some("Kirby Mass Attack"));
    assert_eq!(e.developer.as_deref(), Some("HAL Laboratory"));
    assert_eq!(e.publisher.as_deref(), Some("Nintendo"));
    assert_eq!(e.genre.as_deref(), Some("Platformer"));
    assert_eq!(e.rating, Some(0.85));
    Ok(())
}

/// Tests exporting and re-parsing a gamelist.
#[test]
fn test_export_and_reparse() -> TestResult {
    let entry = GamelistEntry {
        path: "./Zelda.gba".to_string(),
        name: Some("The Legend of Zelda".to_string()),
        desc: Some("Classic adventure".to_string()),
        image: Some("./covers/Zelda.png".to_string()),
        marquee: None,
        video: None,
        release_date: Some("2004-11-04".to_string()),
        developer: Some("Capcom".to_string()),
        publisher: Some("Nintendo".to_string()),
        genre: Some("Action-Adventure".to_string()),
        players: Some("1".to_string()),
        rating: Some(0.95),
    };
    let xml = export_gamelist(std::slice::from_ref(&entry))?;
    let reparsed = parse_gamelist(&xml)?;
    assert_eq!(reparsed.len(), 1);
    assert_eq!(reparsed[0].path, entry.path);
    assert_eq!(reparsed[0].name, entry.name);
    assert_eq!(reparsed[0].developer, entry.developer);
    Ok(())
}
