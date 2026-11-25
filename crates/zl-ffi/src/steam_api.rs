//! The Steam half of the [`Core`] object: the account (Web API key and profile), its check and
//! the import of the owned games as `.steamappid` files.

use std::sync::Arc;

use zl_core as core;
use zl_net::UreqClient;

use crate::core_object::Core;
use crate::error::CoreError;
use crate::scan::{Bridge, ScanListener};

/// Result of a core call.
type Res<T> = Result<T, CoreError>;

mirror_enum! {
    /// Why Steam could not be used.
    SteamProblem <=> core::SteamProblem { NotSet, KeyRefused, PrivateProfile, UnknownProfile, Offline }
}
mirror_record! {
    /// What is configured (the key itself never goes back to the UI).
    SteamAccount <=> core::SteamAccount {
        /// Whether a key is stored.
        plain has_key: bool,
        /// The account as typed.
        opt account: String,
    }
}
mirror_record! {
    /// The result of an import.
    SteamImport <=> core::SteamImport {
        /// Games the account owns.
        plain owned: u32,
        /// Tag files written.
        plain added: u32,
        /// Owned games already in the library.
        plain present: u32,
        /// Why nothing was imported.
        opt problem: SteamProblem,
    }
}

#[uniffi::export]
impl Core {
    /// The stored Steam account.
    pub fn steam_account(&self) -> Res<SteamAccount> {
        Ok(self.service().steam_account()?.into())
    }

    /// Stores the Web API key and the account; `None` keeps a value, blank removes it. A
    /// malformed value is `InvalidInput`.
    pub fn set_steam_account(&self, key: Option<String>, account: Option<String>) -> Res<()> {
        Ok(self
            .service()
            .set_steam_account(key.as_deref(), account.as_deref())?)
    }

    /// Checks the stored account against Steam (network call); `None` when it works.
    pub fn check_steam_account(&self) -> Res<Option<SteamProblem>> {
        let problem = self.service().check_steam_account(&UreqClient::new())?;
        Ok(problem.map(Into::into))
    }

    /// Imports the owned games and rescans. Blocks: call it off the main thread.
    pub fn import_steam_library(&self, listener: Arc<dyn ScanListener>) -> Res<SteamImport> {
        let report = self
            .service()
            .import_steam_library(&UreqClient::new(), &Bridge(listener))?;
        Ok(report.into())
    }
}
