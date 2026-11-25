//! SteamGridDB: the shared cases of `fixtures/steamgriddb/sgdb.json` and the client end to end over
//! a mock transport (search, pick, images, download, refused key).

use std::path::Path;

use serde_json::Value;
use zl_db::MediaKind;
use zl_meta::{pick_game, pick_image, MetaError, SgdbGame, SgdbImage, SteamGridDb};
use zl_net::MockHttpClient;

/// Result type of the tests.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// The shared fixture.
fn fixture() -> Result<Value, Box<dyn std::error::Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/steamgriddb/sgdb.json");
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

/// Every `pickGame` case picks the expected id (or none).
#[test]
fn pick_game_cases() -> TestResult {
    for case in fixture()?["pickGame"].as_array().ok_or("pickGame")? {
        let results: Vec<SgdbGame> = case["results"]
            .as_array()
            .ok_or("results")?
            .iter()
            .map(|r| SgdbGame {
                id: r[0].as_u64().unwrap_or_default(),
                name: r[1].as_str().unwrap_or_default().to_string(),
            })
            .collect();
        let title = case["title"].as_str().ok_or("title")?;
        assert_eq!(pick_game(&results, title), case["id"].as_u64(), "{case}");
    }
    Ok(())
}

/// Every `pickImage` case picks the expected URL (or none).
#[test]
fn pick_image_cases() -> TestResult {
    for case in fixture()?["pickImage"].as_array().ok_or("pickImage")? {
        let images: Vec<SgdbImage> = serde_json::from_value(case["grids"].clone())?;
        let expected = case["url"].as_str().map(str::to_string);
        assert_eq!(pick_image(&images), expected, "{}", case["why"]);
    }
    Ok(())
}

/// Search, then the square cover, the hero and the logo, then the download, all through `get`.
#[test]
fn finds_a_game_and_its_images() -> TestResult {
    let http = MockHttpClient::new();
    http.register(
        "/search/autocomplete/Mario%20Kart%207",
        r#"{"success":true,"data":[{"id":7,"name":"Mario Kart 7"},{"id":8,"name":"Mario Kart 8"}]}"#,
    );
    http.register(
        "/grids/game/7?dimensions=1024x1024",
        r#"{"success":true,"data":[{"url":"https://cdn2.steamgriddb.com/grid/mk7.png","upvotes":3}]}"#,
    );
    http.register("/heroes/game/7", r#"{"success":true,"data":[]}"#);
    http.register(
        "/logos/game/7",
        r#"{"success":true,"data":[{"url":"https://cdn2.steamgriddb.com/logo/mk7.png"}]}"#,
    );
    http.register("https://cdn2.steamgriddb.com/grid/mk7.png", "PNGDATA");
    let sgdb = SteamGridDb::new(&http, " key ");

    let found = sgdb.search("Mario Kart 7")?;
    let id = pick_game(&found, "Mario Kart 7").ok_or("no match")?;
    assert_eq!(id, 7);
    let cover = sgdb.best_image(id, MediaKind::Icon)?;
    assert_eq!(
        cover.as_deref(),
        Some("https://cdn2.steamgriddb.com/grid/mk7.png")
    );
    assert_eq!(sgdb.best_image(id, MediaKind::Hero)?, None);
    assert!(sgdb.best_image(id, MediaKind::Logo)?.is_some());
    assert_eq!(sgdb.download(&cover.ok_or("cover")?)?, b"PNGDATA");
    Ok(())
}

/// Without a square cover the portrait query is asked next.
#[test]
fn falls_back_to_portrait_covers() -> TestResult {
    let http = MockHttpClient::new();
    http.register(
        "/grids/game/3?dimensions=1024x1024",
        r#"{"success":true,"data":[]}"#,
    );
    http.register(
        "/grids/game/3?dimensions=600x900",
        r#"{"success":true,"data":[{"url":"https://cdn2.steamgriddb.com/grid/tall.png"}]}"#,
    );
    let sgdb = SteamGridDb::new(&http, "key");
    let cover = sgdb.best_image(3, MediaKind::Icon)?;
    assert_eq!(
        cover.as_deref(),
        Some("https://cdn2.steamgriddb.com/grid/tall.png")
    );
    assert_eq!(http.requested_urls().len(), 2);
    Ok(())
}

/// A refused key is reported as such (the caller stops), a missing game is just "nothing".
#[test]
fn refused_key_and_missing_game() -> TestResult {
    let http = MockHttpClient::new();
    http.register_status("/search/autocomplete/zelda", 401, r#"{"success":false}"#);
    http.register_status("/grids/game/404", 404, "");
    let sgdb = SteamGridDb::new(&http, "bad");
    assert!(matches!(
        sgdb.verify_key(),
        Err(MetaError::AuthRequired { .. })
    ));
    assert_eq!(sgdb.best_image(404, MediaKind::Icon)?, None);
    Ok(())
}

/// Images outside the CDN are never downloaded.
#[test]
fn refuses_downloads_outside_the_cdn() {
    let http = MockHttpClient::new();
    let sgdb = SteamGridDb::new(&http, "key");
    assert!(sgdb
        .download("https://evil.example/cdn2.steamgriddb.com/x.png")
        .is_err());
    assert!(http.requested_urls().is_empty());
}
