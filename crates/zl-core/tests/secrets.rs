//! API keys live in the app's secret store, never in the database; keys an older version kept
//! in the settings table move into the store.

use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

use zl_config::app::DATABASE_FILE_NAME;
use zl_core::{CoreError, SecretStore, Service};
use zl_db::Db;

/// Result type of the tests.
type Res = Result<(), Box<dyn Error>>;

/// A store that remembers what it was given and can refuse writes.
#[derive(Default)]
struct FakeStore {
    /// The values.
    values: Mutex<HashMap<String, String>>,
    /// Refuse every write.
    refuse: bool,
}

impl SecretStore for FakeStore {
    /// From the map.
    fn get(&self, name: &str) -> Option<String> {
        self.values.lock().ok()?.get(name).cloned()
    }

    /// Into the map, unless refusing.
    fn set(&self, name: &str, value: Option<&str>) -> bool {
        if self.refuse {
            return false;
        }
        let Ok(mut map) = self.values.lock() else {
            return false;
        };
        match value {
            Some(v) => map.insert(name.into(), v.into()),
            None => map.remove(name),
        };
        true
    }
}

/// The SteamGridDB key a valid-looking test uses.
const SGDB: &str = "abcdefabcdefabcdefabcdefabcdef12";
/// The Steam key a test uses.
const STEAM: &str = "0123456789abcdef0123456789abcdef";

#[test]
fn legacy_keys_move_out_of_the_database() -> Res {
    let dir = tempfile::tempdir()?;
    {
        let db = Db::open(&dir.path().join(DATABASE_FILE_NAME))?;
        db.set_setting("scraper.steamgriddb_key", SGDB)?;
        db.set_setting("steam.web_api_key", STEAM)?;
    }
    let svc = Service::open(dir.path())?;
    let store = Arc::new(FakeStore::default());
    assert_eq!(svc.set_secret_store(store.clone())?, 2);
    assert_eq!(store.get("steamgriddb_key").as_deref(), Some(SGDB));
    assert_eq!(store.get("steam_web_api_key").as_deref(), Some(STEAM));
    assert_eq!(svc.scraper_config()?.steamgriddb_key.as_deref(), Some(SGDB));
    assert!(svc.steam_account()?.has_key);
    drop(svc);
    let db = Db::open(&dir.path().join(DATABASE_FILE_NAME))?;
    assert_eq!(
        db.setting("scraper.steamgriddb_key")?.as_deref(),
        Some(""),
        "blanked"
    );
    assert_eq!(db.setting("steam.web_api_key")?.as_deref(), Some(""));
    Ok(())
}

#[test]
fn new_keys_go_to_the_store_only() -> Res {
    let dir = tempfile::tempdir()?;
    let svc = Service::open(dir.path())?;
    let store = Arc::new(FakeStore::default());
    svc.set_secret_store(store.clone())?;
    svc.set_steamgriddb_key(Some(SGDB))?;
    svc.set_steam_account(Some(STEAM), Some("76561197960287930"))?;
    assert_eq!(store.get("steamgriddb_key").as_deref(), Some(SGDB));
    assert_eq!(store.get("steam_web_api_key").as_deref(), Some(STEAM));
    let mut text = std::fs::read(dir.path().join(DATABASE_FILE_NAME))?;
    if let Ok(wal) = std::fs::read(dir.path().join(format!("{DATABASE_FILE_NAME}-wal"))) {
        text.extend(wal);
    }
    let in_db = |key: &str| text.windows(key.len()).any(|w| w == key.as_bytes());
    assert!(
        !in_db(SGDB) && !in_db(STEAM),
        "no key written to the database or its log"
    );
    svc.set_steamgriddb_key(None)?;
    assert_eq!(store.get("steamgriddb_key"), None);
    Ok(())
}

#[test]
fn a_refusing_store_keeps_the_old_rows_and_reports_writes() -> Res {
    let dir = tempfile::tempdir()?;
    {
        let db = Db::open(&dir.path().join(DATABASE_FILE_NAME))?;
        db.set_setting("scraper.steamgriddb_key", SGDB)?;
    }
    let svc = Service::open(dir.path())?;
    let store = Arc::new(FakeStore {
        refuse: true,
        ..FakeStore::default()
    });
    assert_eq!(svc.set_secret_store(store)?, 0, "nothing moved");
    assert_eq!(svc.set_steamgriddb_key(Some(SGDB)), Err(CoreError::Io));
    drop(svc);
    let db = Db::open(&dir.path().join(DATABASE_FILE_NAME))?;
    assert_eq!(
        db.setting("scraper.steamgriddb_key")?.as_deref(),
        Some(SGDB),
        "kept for next time"
    );
    Ok(())
}
