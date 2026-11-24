//! One call from "game + device state" to "intent or question": what the FFI exposes to Kotlin.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::db::PlatformDb;
use crate::detect::InstalledEmulator;
use crate::error::LaunchError;
use crate::launch::{build_launch, LaunchContext, LaunchIntent};
use crate::resolve::{ProfileRef, Resolution, ResolveInput, ResolvedBy};

/// A request to launch one game.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchRequest {
    /// What is known about the game; `platform` may be an alias or have any case.
    pub context: LaunchContext,
    /// Profile set on the game.
    #[serde(default)]
    pub game_override: Option<ProfileRef>,
    /// Profile set as the platform's default.
    #[serde(default)]
    pub platform_default: Option<ProfileRef>,
    /// Profiles by extension.
    #[serde(default)]
    pub extension_defaults: HashMap<String, ProfileRef>,
}

/// What to do for a [`LaunchRequest`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Start this intent.
    Launch {
        /// The intent.
        intent: Box<LaunchIntent>,
        /// The profile it comes from.
        profile: ProfileRef,
        /// The step that chose the profile.
        by: ResolvedBy,
    },
    /// Ask the user to pick one of the candidates.
    NeedsChoice {
        /// Installed profiles that can launch the game.
        candidates: Vec<ProfileRef>,
    },
}

impl PlatformDb {
    /// Resolves the profile (RF-08) and builds the intent (RF-09).
    pub fn plan_launch(
        &self,
        installed: &[InstalledEmulator],
        request: &LaunchRequest,
    ) -> Result<Outcome, LaunchError> {
        let platform = self
            .platform(&request.context.platform)
            .ok_or_else(|| LaunchError::UnknownPlatform(request.context.platform.clone()))?;
        let file_name = request
            .context
            .rom_path
            .rsplit('/')
            .next()
            .unwrap_or_default();
        let input = ResolveInput {
            platform: &platform.id,
            file_name,
            game_override: request.game_override.as_ref(),
            platform_default: request.platform_default.as_ref(),
            extension_defaults: &request.extension_defaults,
            installed,
        };
        match self.resolve_profile(&input)? {
            Resolution::NeedsChoice { candidates } => Ok(Outcome::NeedsChoice { candidates }),
            Resolution::Resolved { profile, by } => {
                let intent = self.intent_for(&profile, installed, request, &platform.id)?;
                let mut profile = profile;
                if profile.core.is_some() && request.context.core.is_some() {
                    profile.core.clone_from(&request.context.core);
                }
                Ok(Outcome::Launch {
                    intent: Box::new(intent),
                    profile,
                    by,
                })
            }
        }
    }

    /// Builds the intent of a resolved profile with the installed package.
    fn intent_for(
        &self,
        profile_ref: &ProfileRef,
        installed: &[InstalledEmulator],
        request: &LaunchRequest,
        platform_id: &str,
    ) -> Result<LaunchIntent, LaunchError> {
        let emulator = self
            .emulator(&profile_ref.emulator)
            .ok_or_else(|| LaunchError::UnknownEmulator(profile_ref.emulator.clone()))?;
        let profile = emulator
            .profiles
            .iter()
            .find(|p| p.id == profile_ref.profile)
            .ok_or_else(|| LaunchError::UnknownProfile {
                emulator: emulator.id.clone(),
                profile: profile_ref.profile.clone(),
            })?;
        let mut ctx = request.context.clone();
        ctx.platform = platform_id.to_string();
        ctx.core = ctx.core.or_else(|| profile_ref.core.clone());
        if ctx.package.is_none() {
            ctx.package = installed
                .iter()
                .find(|i| i.emulator == emulator.id)
                .and_then(|i| i.packages.first().cloned());
        }
        build_launch(emulator, profile, &ctx)
    }
}
