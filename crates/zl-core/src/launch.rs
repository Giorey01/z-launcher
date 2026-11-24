//! Emulator assignment (platform default, game override) and the launch plan of a stored game.

use std::collections::{BTreeMap, HashMap};

use zl_db::{Game, Root};
use zl_platforms::saf::rom_uris;
use zl_platforms::{LaunchContext, LaunchRequest, Outcome, ProfileRef};

use crate::error::CoreError;
use crate::types::Plan;
use crate::Service;

/// Reads a stored override: absent row, explicit "none" and unreadable keys are all "not set".
fn stored_ref(stored: Option<Option<String>>) -> Option<ProfileRef> {
    stored.flatten().and_then(|key| ProfileRef::from_key(&key))
}

/// The folder of the platform that contains `game_path`: the root plus its first sub-folder.
fn platform_dir(root: &Root, game_path: &str) -> Option<String> {
    let rel = game_path.strip_prefix(&root.path)?.strip_prefix('/')?;
    let (folder, _) = rel.split_once('/')?;
    Some(format!("{}/{folder}", root.path))
}

impl Service {
    /// The default emulator profile of a platform, if the user (or onboarding) set one.
    pub fn platform_default(&self, platform: &str) -> Result<Option<ProfileRef>, CoreError> {
        let id = self.canonical_id(platform);
        Ok(stored_ref(self.db().platform_emulator(&id)?))
    }

    /// Sets (or with `None` clears) the default emulator profile of a platform.
    pub fn set_platform_default(
        &self,
        platform: &str,
        profile: Option<&ProfileRef>,
    ) -> Result<(), CoreError> {
        let id = self.canonical_id(platform);
        let key = profile.map(ProfileRef::to_key);
        self.db().set_platform_emulator(&id, key.as_deref())?;
        Ok(())
    }

    /// The emulator profile set on one game, if any.
    pub fn game_emulator(&self, game_id: i64) -> Result<Option<ProfileRef>, CoreError> {
        Ok(stored_ref(self.db().game_emulator(game_id)?))
    }

    /// Sets (or with `None` clears) the emulator profile of one game.
    pub fn set_game_emulator(
        &self,
        game_id: i64,
        profile: Option<&ProfileRef>,
    ) -> Result<(), CoreError> {
        let db = self.db();
        db.game(game_id)?.ok_or(CoreError::NotFound)?;
        let key = profile.map(ProfileRef::to_key);
        db.set_game_emulator(game_id, key.as_deref())?;
        Ok(())
    }

    /// Everything the launch builder knows about a stored game: its path, the ROM URIs (with
    /// the SAF document when the root has a granted tree), the platform folder and the tags.
    fn context_of(&self, game: &Game, root: &Root) -> Result<LaunchContext, CoreError> {
        let tags: BTreeMap<String, String> = self.db().game_tags(game.id)?.into_iter().collect();
        Ok(LaunchContext {
            rom_path: game.path.clone(),
            rom_uris: rom_uris(&game.path, &root.path, root.saf_tree.as_deref()),
            platform: self.canonical_id(&game.platform_id),
            platform_dir: platform_dir(root, &game.path),
            tags,
            core: None,
            package: None,
        })
    }

    /// Decides how to launch a stored game (RF-08, RF-09): the intent to start, or the emulators
    /// to choose from. `packages` are the launcher activities installed on the device.
    pub fn plan_launch(&self, game_id: i64, packages: &[String]) -> Result<Plan, CoreError> {
        let (game, root) = {
            let db = self.db();
            let game = db.game(game_id)?.ok_or(CoreError::NotFound)?;
            let root = db.root(game.root_id)?.ok_or(CoreError::NotFound)?;
            (game, root)
        };
        let context = self.context_of(&game, &root)?;
        let request = LaunchRequest {
            platform_default: self.platform_default(&context.platform)?,
            game_override: self.game_emulator(game_id)?,
            extension_defaults: HashMap::new(),
            context,
        };
        self.plan_request(packages, &request)
    }

    /// Plans a launch for an explicit request (the same decision as [`Service::plan_launch`]
    /// without a stored game). Used by the shared fixtures.
    pub fn plan_request(
        &self,
        packages: &[String],
        request: &LaunchRequest,
    ) -> Result<Plan, CoreError> {
        let installed = self.platforms.detect_installed(packages).installed;
        Ok(match self.platforms.plan_launch(&installed, request)? {
            Outcome::Launch {
                intent,
                profile,
                by,
            } => Plan::Launch {
                intent,
                profile,
                by,
            },
            Outcome::NeedsChoice { candidates } => Plan::NeedsChoice {
                candidates: candidates.into_iter().map(|c| self.candidate(c)).collect(),
            },
        })
    }
}
