//! The Web API and store clients over the mock transport, with the answers of `fixtures/steam/`.

use std::path::Path;

use serde_json::Value;
use zl_net::MockHttpClient;
use zl_steam::store::{iso_date, plain_text};
use zl_steam::{AccountInput, SteamError, SteamStore, SteamWeb};

/// Result type of the tests.
type Res = Result<(), Box<dyn std::error::Error>>;

/// Key used by the tests (its shape only matters to `is_web_api_key`).
const KEY: &str = "0123456789abcdef0123456789abcdef";

/// One shared fixture file of `fixtures/steam/`.
fn fixture(name: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../fixtures/steam/{name}"));
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

/// A mock that answers `pattern` with the fixture entry `key` of `file`.
fn mock_with(
    file: &str,
    pattern: &str,
    key: &str,
) -> Result<MockHttpClient, Box<dyn std::error::Error>> {
    let http = MockHttpClient::new();
    http.register(pattern, &fixture(file)?[key].to_string());
    Ok(http)
}

#[test]
fn owned_games_are_read_with_names_and_playtime() -> Res {
    let http = mock_with("web_api.json", "GetOwnedGames", "owned_games")?;
    let games = SteamWeb::new(&http, KEY).owned_games(76_561_197_960_287_930)?;
    let summary: Vec<(u32, &str, u64)> = games
        .iter()
        .map(|g| (g.appid, g.name.as_str(), g.playtime_minutes))
        .collect();
    assert_eq!(
        summary,
        [
            (400, "Portal", 312),
            (1_145_360, "Hades", 0),
            (620, "Portal 2: <Coop>", 45)
        ]
    );
    assert_eq!(games[0].last_played, 1_700_000_000);
    let url = &http.requested_urls()[0];
    assert!(
        url.contains("steamid=76561197960287930") && url.contains("include_appinfo=1"),
        "{url}"
    );
    assert!(url.contains(&format!("key={KEY}")), "{url}");
    Ok(())
}

#[test]
fn a_private_profile_is_not_an_empty_library() -> Res {
    let http = mock_with("web_api.json", "GetOwnedGames", "owned_games_private")?;
    assert_eq!(
        SteamWeb::new(&http, KEY).owned_games(1),
        Err(SteamError::PrivateProfile)
    );
    let http = mock_with("web_api.json", "GetOwnedGames", "owned_games_empty")?;
    assert_eq!(SteamWeb::new(&http, KEY).owned_games(1)?, []);
    Ok(())
}

#[test]
fn refused_keys_and_custom_names() -> Res {
    let http = MockHttpClient::new();
    http.register_status("GetOwnedGames", 403, "<html>Forbidden</html>");
    assert_eq!(
        SteamWeb::new(&http, KEY).owned_games(1),
        Err(SteamError::KeyRefused)
    );
    let http = mock_with("web_api.json", "ResolveVanityURL", "vanity_found")?;
    let web = SteamWeb::new(&http, KEY);
    assert_eq!(
        web.steam_id(&AccountInput::Vanity("gaben".into()))?,
        76_561_197_960_287_930
    );
    assert_eq!(
        web.steam_id(&AccountInput::SteamId(5))?,
        5,
        "an id needs no call"
    );
    assert_eq!(http.requested_urls().len(), 1);
    let http = mock_with("web_api.json", "ResolveVanityURL", "vanity_missing")?;
    assert_eq!(
        SteamWeb::new(&http, KEY).resolve_vanity("nobody"),
        Err(SteamError::UnknownProfile)
    );
    Ok(())
}

#[test]
fn store_pages_become_library_details() -> Res {
    let http = mock_with("store.json", "appids=400", "app_400")?;
    let app = SteamStore::new(&http).app(400)?.ok_or("no page")?;
    assert_eq!(app.name, "Portal");
    assert!(app
        .description
        .as_deref()
        .is_some_and(|d| d.starts_with("Portal™ is a new")));
    assert_eq!(app.release_date.as_deref(), Some("2007-10-10"));
    assert_eq!(app.developer.as_deref(), Some("Valve"));
    assert_eq!(app.genres.as_deref(), Some("Action"));
    assert_eq!(app.rating, Some(4.5));
    assert_eq!(
        app.screenshots,
        [
            "https://shared.akamai.steamstatic.com/a.jpg",
            "https://shared.akamai.steamstatic.com/b.jpg"
        ]
    );
    assert_eq!(
        app.trailers,
        [
            "https://video.akamai.steamstatic.com/t_max.mp4",
            "https://video.akamai.steamstatic.com/t2/hls_264_master.m3u8"
        ]
    );
    let http = mock_with("store.json", "appids=1145360", "app_quote")?;
    let hades = SteamStore::new(&http).app(1_145_360)?.ok_or("no page")?;
    assert_eq!(hades.release_date.as_deref(), Some("2020-09-17"));
    assert_eq!(
        hades.genres.as_deref(),
        Some("Action, Adventure, Indie, RPG")
    );
    assert!(hades
        .description
        .as_deref()
        .is_some_and(|d| d.ends_with("\"Bastion\" & Transistor.")));
    assert_eq!(hades.rating, None);
    let http = mock_with("store.json", "appids=999999999", "app_missing")?;
    assert_eq!(SteamStore::new(&http).app(999_999_999)?, None);
    Ok(())
}

#[test]
fn dates_and_text_of_the_store() {
    assert_eq!(iso_date("10 Oct, 2007").as_deref(), Some("2007-10-10"));
    assert_eq!(iso_date("Sep 7, 2020").as_deref(), Some("2020-09-07"));
    assert_eq!(iso_date("Coming soon"), None);
    assert_eq!(iso_date("Q1 2026"), None);
    assert_eq!(plain_text("A&amp;B <i>x</i>  y"), "A&B x y");
    assert_eq!(
        plain_text("<b>Bold</b>, then <i>end</i>."),
        "Bold, then end."
    );
}
