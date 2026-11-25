//! The Steam account and library import end to end: settings, checks, tag files written into the
//! Steam folder, games already present skipped, the games in the "PC" folder.

use std::error::Error;
use std::fs;
use std::path::Path;

use serde_json::Value;
use zl_core::{CoreError, Service, SteamProblem};
use zl_library::NoProgress;
use zl_net::MockHttpClient;

/// Result type of the tests.
type Res = Result<(), Box<dyn Error>>;

/// A key of the right shape.
const KEY: &str = "0123456789abcdef0123456789abcdef";

/// The Web API answers of `fixtures/steam/web_api.json`.
fn answers() -> Result<Value, Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/steam/web_api.json");
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

/// A mock that answers the owned games with entry `owned` and the vanity lookup with a match.
fn steam(owned: &str) -> Result<MockHttpClient, Box<dyn Error>> {
    let all = answers()?;
    let http = MockHttpClient::new();
    http.register("GetOwnedGames", &all[owned].to_string());
    http.register("ResolveVanityURL", &all["vanity_found"].to_string());
    Ok(http)
}

#[test]
fn account_settings_are_checked_and_kept() -> Res {
    let dir = tempfile::tempdir()?;
    let svc = Service::open(dir.path())?;
    assert_eq!(
        svc.check_steam_account(&MockHttpClient::new())?,
        Some(SteamProblem::NotSet)
    );
    assert_eq!(
        svc.set_steam_account(Some("short"), None),
        Err(CoreError::InvalidInput)
    );
    assert_eq!(
        svc.set_steam_account(None, Some("two words")),
        Err(CoreError::InvalidInput)
    );
    svc.set_steam_account(Some(KEY), Some("https://steamcommunity.com/id/gaben"))?;
    let account = svc.steam_account()?;
    assert!(account.has_key);
    assert_eq!(
        account.account.as_deref(),
        Some("https://steamcommunity.com/id/gaben")
    );
    let http = steam("owned_games")?;
    assert_eq!(svc.check_steam_account(&http)?, None);
    assert_eq!(svc.check_steam_account(&http)?, None);
    let vanity_calls = http
        .requested_urls()
        .iter()
        .filter(|u| u.contains("ResolveVanityURL"))
        .count();
    assert_eq!(vanity_calls, 1, "the custom name is resolved once");
    assert_eq!(
        svc.check_steam_account(&steam("owned_games_private")?)?,
        Some(SteamProblem::PrivateProfile)
    );
    let refused = MockHttpClient::new();
    refused.register_status("GetOwnedGames", 401, "");
    assert_eq!(
        svc.check_steam_account(&refused)?,
        Some(SteamProblem::KeyRefused)
    );
    svc.set_steam_account(Some(""), None)?;
    assert!(!svc.steam_account()?.has_key, "a blank key removes it");
    Ok(())
}

#[test]
fn import_writes_tag_files_and_skips_games_already_there() -> Res {
    let dir = tempfile::tempdir()?;
    let root = dir.path().join("Games");
    fs::create_dir_all(root.join("Steam"))?;
    fs::write(
        root.join("Steam/My Hades.steamappid"),
        "[steamappid] 1145360\n",
    )?;
    let svc = Service::open(&dir.path().join("data"))?;
    svc.add_root(&root.to_string_lossy(), None)?;
    svc.scan(&NoProgress)?;
    svc.set_steam_account(Some(KEY), Some("76561197960287930"))?;
    let report = svc.import_steam_library(&steam("owned_games")?, &NoProgress)?;
    assert_eq!(
        (report.owned, report.added, report.present, report.problem),
        (3, 2, 1, None)
    );
    assert_eq!(
        fs::read_to_string(root.join("Steam/Portal.steamappid"))?,
        "[steamappid] 400\n"
    );
    assert!(
        root.join("Steam/Portal 2 Coop.steamappid").exists(),
        "unsafe characters dropped"
    );
    let pc: Vec<String> = svc
        .games(Some("pcgames"))?
        .into_iter()
        .map(|g| g.title)
        .collect();
    assert_eq!(pc, ["My Hades", "Portal", "Portal 2 Coop"]);
    let again = svc.import_steam_library(&steam("owned_games")?, &NoProgress)?;
    assert_eq!(
        (again.added, again.present),
        (0, 3),
        "a second import adds nothing"
    );
    let private = svc.import_steam_library(&steam("owned_games_private")?, &NoProgress)?;
    assert_eq!(private.problem, Some(SteamProblem::PrivateProfile));
    Ok(())
}

#[test]
fn import_needs_a_games_folder() -> Res {
    let dir = tempfile::tempdir()?;
    let svc = Service::open(dir.path())?;
    svc.set_steam_account(Some(KEY), Some("76561197960287930"))?;
    assert_eq!(
        svc.import_steam_library(&steam("owned_games")?, &NoProgress),
        Err(CoreError::NotFound)
    );
    Ok(())
}

/// Records progress and never cancels.
struct Quiet;

impl zl_core::ScrapeProgress for Quiet {
    /// Ignored.
    fn on_game(&self, _done: u32, _total: u32) {}
    /// Never.
    fn is_cancelled(&self) -> bool {
        false
    }
}

#[test]
fn store_details_fill_steam_games_without_a_description() -> Res {
    let dir = tempfile::tempdir()?;
    let root = dir.path().join("Games");
    fs::create_dir_all(root.join("steam"))?;
    fs::write(root.join("steam/Portal.steamappid"), "[steamappid] 400\n")?;
    fs::create_dir_all(root.join("psp"))?;
    fs::write(root.join("psp/Patapon.iso"), b"x")?;
    let svc = Service::open(&dir.path().join("data"))?;
    svc.add_root(&root.to_string_lossy(), None)?;
    svc.scan(&NoProgress)?;
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/steam/store.json");
    let store: Value = serde_json::from_str(&fs::read_to_string(path)?)?;
    let http = MockHttpClient::new();
    http.register("appids=400", &store["app_400"].to_string());
    http.register("steamstatic.com/a.jpg", "jpeg-a");
    http.register("steamstatic.com/b.jpg", "jpeg-b");
    let all = zl_core::ScrapeTarget::All;
    let pause = std::time::Duration::ZERO;
    let report = svc.fetch_steam_details(&all, &http, &Quiet, pause)?;
    assert_eq!(
        (report.games, report.updated, report.images, report.complete),
        (2, 1, 2, 1)
    );
    assert_eq!(report.stopped, None);
    let portal = svc.games(Some("steam"))?[0].id;
    let details = svc.game_details(portal)?;
    assert_eq!(details.release_date.as_deref(), Some("2007-10-10"));
    assert_eq!(details.developer.as_deref(), Some("Valve"));
    assert!(details
        .description
        .is_some_and(|d| d.starts_with("Portal™")));
    assert_eq!(details.screenshots.len(), 2);
    assert_eq!(fs::read_to_string(&details.screenshots[0])?, "jpeg-a");
    let store_calls = |h: &MockHttpClient| {
        h.requested_urls()
            .iter()
            .filter(|u| u.contains("appdetails"))
            .count()
    };
    assert_eq!(store_calls(&http), 1, "the PSP game is not asked");
    let again = svc.fetch_steam_details(&all, &http, &Quiet, pause)?;
    assert_eq!(
        (again.updated, again.complete),
        (0, 2),
        "a described game is not asked again"
    );
    assert_eq!(store_calls(&http), 1);
    Ok(())
}

#[test]
fn store_details_stop_when_offline() -> Res {
    let dir = tempfile::tempdir()?;
    let root = dir.path().join("Games");
    fs::create_dir_all(root.join("steam"))?;
    for (name, id) in [("A", 1), ("B", 2), ("C", 3), ("D", 4)] {
        fs::write(
            root.join(format!("steam/{name}.steamappid")),
            format!("[steamappid] {id}\n"),
        )?;
    }
    let svc = Service::open(&dir.path().join("data"))?;
    svc.add_root(&root.to_string_lossy(), None)?;
    svc.scan(&NoProgress)?;
    let http = MockHttpClient::new();
    http.register_status("appdetails", 503, "");
    let report = svc.fetch_steam_details(
        &zl_core::ScrapeTarget::All,
        &http,
        &Quiet,
        std::time::Duration::ZERO,
    )?;
    assert_eq!(
        (report.failed, report.stopped),
        (3, Some(zl_core::ScrapeStop::Offline))
    );
    Ok(())
}
