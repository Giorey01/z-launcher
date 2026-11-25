//! The user's Steam library, imported with their own Web API key (RF-67).
//!
//! The import writes one `.steamappid` file per owned game into the Steam folder of the first
//! enabled games folder (Cocoon's format, the same file a user writes by hand) and rescans, so
//! every client (GameNative, GameHub, DroidDeck, ...) launches the games and they sit in the
//! "PC" folder. Games already in the library by app id are skipped, and nothing is ever deleted.
//! The play time Steam reports for each game is stored too (replaced on every import).
//! The key and the account are kept in the settings table; the resolved SteamID64 is kept too,
//! so a custom profile name is resolved once.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use zl_config::steam::STEAM_IMPORT_FOLDER;
use zl_library::ScanProgress;
use zl_net::HttpClient;
use zl_steam::{is_web_api_key, parse_account, AccountInput, OwnedGame, SteamError, SteamWeb};

use crate::app_games::{file_stem, free_path};
use crate::error::CoreError;
use crate::Service;

/// Settings key of the Web API key.
const KEY_STEAM_KEY: &str = "steam.web_api_key";
/// Settings key of the account as the user typed it.
const KEY_STEAM_ACCOUNT: &str = "steam.account";
/// Settings key of the SteamID64 the account resolved to.
const KEY_STEAM_ID: &str = "steam.steam_id";

/// Tag of a Steam tag file. A format fact.
const STEAM_TAG: &str = "steamappid";
/// Platform id of Steam games. A format fact (Cocoon's folder name).
const STEAM_PLATFORM: &str = "steam";

/// What is configured, for the settings screen (the key itself never leaves the core).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamAccount {
    /// Whether a key is stored.
    pub has_key: bool,
    /// The account as typed.
    pub account: Option<String>,
}

/// Why Steam could not be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteamProblem {
    /// No key or no account is set.
    NotSet,
    /// Steam refused the key.
    KeyRefused,
    /// The profile hides its game details.
    PrivateProfile,
    /// No profile has that custom name.
    UnknownProfile,
    /// Steam could not be reached.
    Offline,
}

/// The result of an import.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SteamImport {
    /// Games the account owns.
    pub owned: u32,
    /// Tag files written.
    pub added: u32,
    /// Owned games already in the library.
    pub present: u32,
    /// Why nothing was imported, when that happened.
    pub problem: Option<SteamProblem>,
}

/// The problem a Steam error stands for.
fn problem_of(e: &SteamError) -> SteamProblem {
    match e {
        SteamError::KeyRefused => SteamProblem::KeyRefused,
        SteamError::PrivateProfile => SteamProblem::PrivateProfile,
        SteamError::UnknownProfile => SteamProblem::UnknownProfile,
        SteamError::Network(_) | SteamError::Parse(_) => SteamProblem::Offline,
    }
}

impl Service {
    /// The stored account (without the key).
    pub fn steam_account(&self) -> Result<SteamAccount, CoreError> {
        let db = self.db();
        Ok(SteamAccount {
            has_key: db.setting(KEY_STEAM_KEY)?.is_some_and(|k| !k.is_empty()),
            account: db.setting(KEY_STEAM_ACCOUNT)?.filter(|a| !a.is_empty()),
        })
    }

    /// Stores the Web API key (`None` keeps the stored one, blank removes it) and the account
    /// (a SteamID64, a profile URL or a custom name; `None` keeps it, blank removes it).
    /// `InvalidInput` for a key that is not 32 hexadecimal characters or an unreadable account.
    pub fn set_steam_account(
        &self,
        key: Option<&str>,
        account: Option<&str>,
    ) -> Result<(), CoreError> {
        let key = key.map(str::trim);
        let account = account.map(str::trim);
        if key.is_some_and(|k| !k.is_empty() && !is_web_api_key(k))
            || account.is_some_and(|a| !a.is_empty() && parse_account(a).is_none())
        {
            return Err(CoreError::InvalidInput);
        }
        let db = self.db();
        if let Some(k) = key {
            db.set_setting(KEY_STEAM_KEY, k)?;
        }
        if let Some(a) = account {
            db.set_setting(KEY_STEAM_ACCOUNT, a)?;
            db.set_setting(KEY_STEAM_ID, "")?;
        }
        Ok(())
    }

    /// The SteamID64 of the stored account, resolving a custom name once.
    fn steam_credentials(&self, web: &SteamWeb<'_>) -> Result<u64, SteamProblem> {
        let cached = self.db().setting(KEY_STEAM_ID).ok().flatten();
        if let Some(id) = cached.and_then(|s| s.parse::<u64>().ok()) {
            return Ok(id);
        }
        let account = self.db().setting(KEY_STEAM_ACCOUNT).ok().flatten();
        let account = account.as_deref().and_then(parse_account);
        let account: AccountInput = account.ok_or(SteamProblem::NotSet)?;
        let id = web.steam_id(&account).map_err(|e| problem_of(&e))?;
        // A failed write only means the name is resolved again next time.
        let _ = self.db().set_setting(KEY_STEAM_ID, &id.to_string());
        Ok(id)
    }

    /// The stored key, if any.
    fn steam_key(&self) -> Result<Option<String>, CoreError> {
        Ok(self.db().setting(KEY_STEAM_KEY)?.filter(|k| !k.is_empty()))
    }

    /// Checks the stored key and account against Steam: `None` when the library can be read.
    pub fn check_steam_account(
        &self,
        http: &dyn HttpClient,
    ) -> Result<Option<SteamProblem>, CoreError> {
        let Some(key) = self.steam_key()? else {
            return Ok(Some(SteamProblem::NotSet));
        };
        let web = SteamWeb::new(http, &key);
        Ok(match self.steam_credentials(&web) {
            Err(problem) => Some(problem),
            Ok(id) => web.owned_games(id).err().map(|e| problem_of(&e)),
        })
    }

    /// The Steam tag files already in the library (hidden ones too): app id to game id.
    fn steam_app_ids(&self) -> Result<HashMap<String, i64>, CoreError> {
        let games = self.listed_games(Some(STEAM_PLATFORM))?;
        let db = self.db();
        let mut ids = HashMap::new();
        for game in games {
            if let Some((_, v)) = db
                .game_tags(game.id)?
                .into_iter()
                .find(|(k, _)| k == STEAM_TAG)
            {
                ids.insert(v.trim().to_string(), game.id);
            }
        }
        Ok(ids)
    }

    /// Stores the account's Steam play time of every owned game that is in the library.
    fn save_steam_playtime(&self, owned: &[OwnedGame]) -> Result<(), CoreError> {
        let ids = self.steam_app_ids()?;
        let db = self.db();
        for game in owned {
            if let Some(&id) = ids.get(&game.appid.to_string()) {
                let seconds = i64::try_from(game.playtime_minutes * 60).unwrap_or(i64::MAX);
                db.set_steam_playtime(id, seconds, game.last_played)?;
            }
        }
        Ok(())
    }

    /// Imports the games the stored account owns (see the module docs), then rescans.
    /// `NotFound` when no games folder is enabled, `Io` when the Steam folder cannot be written.
    pub fn import_steam_library(
        &self,
        http: &dyn HttpClient,
        progress: &dyn ScanProgress,
    ) -> Result<SteamImport, CoreError> {
        let mut report = SteamImport::default();
        let Some(key) = self.steam_key()? else {
            report.problem = Some(SteamProblem::NotSet);
            return Ok(report);
        };
        let web = SteamWeb::new(http, &key);
        let owned = match self
            .steam_credentials(&web)
            .and_then(|id| web.owned_games(id).map_err(|e| problem_of(&e)))
        {
            Ok(owned) => owned,
            Err(problem) => {
                report.problem = Some(problem);
                return Ok(report);
            }
        };
        report.owned = u32::try_from(owned.len()).unwrap_or(u32::MAX);
        let root = self.roots()?.into_iter().find(|r| r.enabled);
        let dir = self.console_dir(
            &PathBuf::from(root.ok_or(CoreError::NotFound)?.path),
            STEAM_IMPORT_FOLDER,
        );
        fs::create_dir_all(&dir).map_err(|_| CoreError::Io)?;
        let present = self.steam_app_ids()?;
        for game in &owned {
            if present.contains_key(&game.appid.to_string()) {
                report.present += 1;
                continue;
            }
            let stem = file_stem(&game.name).unwrap_or_else(|| game.appid.to_string());
            let text = format!("[{STEAM_TAG}] {}\n", game.appid);
            let path = free_path(&dir, &stem, STEAM_TAG, &text);
            fs::write(&path, text).map_err(|_| CoreError::Io)?;
            report.added += 1;
        }
        self.scan(progress)?;
        self.save_steam_playtime(&owned)?;
        Ok(report)
    }
}
