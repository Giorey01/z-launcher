//! API keys (SteamGridDB, Steam Web API) kept outside the database.
//!
//! The app gives the core a [`SecretStore`]: on Android, values encrypted with a key that never
//! leaves the Keystore. The core asks it for a key only when a call needs it and never writes a
//! key to the database. Keys that an older version kept in the settings table are moved into
//! the store when it is set, and their rows are blanked. Until a store is set (tests, a fresh
//! process) the keys live in memory only.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use crate::error::CoreError;
use crate::Service;

/// Name of the SteamGridDB key in the store.
pub const SECRET_STEAMGRIDDB: &str = "steamgriddb_key";
/// Name of the Steam Web API key in the store.
pub const SECRET_STEAM_WEB: &str = "steam_web_api_key";

/// Settings rows where older versions kept the keys, with the secret each one becomes.
const LEGACY_ROWS: [(&str, &str); 2] = [
    ("scraper.steamgriddb_key", SECRET_STEAMGRIDDB),
    ("steam.web_api_key", SECRET_STEAM_WEB),
];

/// Where the keys are kept.
pub trait SecretStore: Send + Sync {
    /// The value of `name`, if one is stored.
    fn get(&self, name: &str) -> Option<String>;
    /// Stores `value` under `name` (`None` removes it); `false` when it could not be stored.
    fn set(&self, name: &str, value: Option<&str>) -> bool;
}

/// Keys in memory only: the store in use until the app sets its own.
#[derive(Debug, Default)]
pub struct MemorySecrets(Mutex<HashMap<String, String>>);

impl SecretStore for MemorySecrets {
    /// From the map.
    fn get(&self, name: &str) -> Option<String> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(name)
            .cloned()
    }

    /// Into the map.
    fn set(&self, name: &str, value: Option<&str>) -> bool {
        let mut map = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        match value {
            Some(v) => map.insert(name.to_string(), v.to_string()),
            None => map.remove(name),
        };
        true
    }
}

impl Service {
    /// Uses `store` for the keys from now on, and moves the keys an older version kept in the
    /// database into it. Returns how many were moved; a key the store refuses stays where it was.
    pub fn set_secret_store(&self, store: Arc<dyn SecretStore>) -> Result<u32, CoreError> {
        let mut moved = 0;
        for (row, name) in LEGACY_ROWS {
            let Some(value) = self.db().setting(row)?.filter(|v| !v.is_empty()) else {
                continue;
            };
            if store.set(name, Some(&value)) {
                self.db().set_setting(row, "")?;
                moved += 1;
            }
        }
        *self.secrets.lock().unwrap_or_else(PoisonError::into_inner) = store;
        Ok(moved)
    }

    /// The store in use.
    fn secret_store(&self) -> Arc<dyn SecretStore> {
        Arc::clone(&self.secrets.lock().unwrap_or_else(PoisonError::into_inner))
    }

    /// The key `name`, when one is stored.
    pub(crate) fn secret(&self, name: &str) -> Option<String> {
        self.secret_store().get(name).filter(|v| !v.is_empty())
    }

    /// Stores (or with `None` removes) the key `name`; `Io` when the store refuses.
    pub(crate) fn set_secret(&self, name: &str, value: Option<&str>) -> Result<(), CoreError> {
        if self.secret_store().set(name, value) {
            Ok(())
        } else {
            Err(CoreError::Io)
        }
    }
}
