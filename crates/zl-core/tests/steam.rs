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
