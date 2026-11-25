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

/// Tag that names an Android package (Cocoon's `.localgameid`). A format fact.
pub const ANDROID_PACKAGE_TAG: &str = "localgameid";

/// Emulator id of the profile reported for an Android app game; no real emulator may use it.
pub const ANDROID_APP_EMULATOR: &str = "android";

/// Profile id reported for an Android app game.
pub const ANDROID_APP_PROFILE: &str = "app";

/// Whether `value` reads as an Android package name: two or more dot-separated parts, each
/// starting with a letter and made of letters, digits and underscores.
fn is_package_name(value: &str) -> bool {
    let parts: Vec<&str> = value.split('.').collect();
    parts.len() >= 2
        && parts.iter().all(|part| {
            part.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
                && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

/// The launch of an Android app game: its launcher activity, in a new task.
fn android_app_outcome(package: &str) -> Outcome {
    Outcome::Launch {
        intent: Box::new(LaunchIntent {
            action: Some("android.intent.action.MAIN".into()),
            package: package.to_string(),
            categories: vec!["android.intent.category.LAUNCHER".into()],
            flags: vec!["new_task".into()],
            ..LaunchIntent::default()
        }),
        profile: ProfileRef {
            emulator: ANDROID_APP_EMULATOR.into(),
            profile: ANDROID_APP_PROFILE.into(),
            core: None,
        },
        by: ResolvedBy::AndroidApp,
    }
}

impl PlatformDb {
    /// Resolves the profile (RF-08) and builds the intent (RF-09). A game of a console whose
    /// `localgameid` tag names an Android package (a recomp or port filed under that console)
    /// starts that app, whatever emulator is configured. Platforms whose own tag files carry
    /// `localgameid` (Windows) keep resolving their emulators, which read the tag themselves.
    pub fn plan_launch(
        &self,
        installed: &[InstalledEmulator],
        request: &LaunchRequest,
    ) -> Result<Outcome, LaunchError> {
        let platform = self
            .platform(&request.context.platform)
            .ok_or_else(|| LaunchError::UnknownPlatform(request.context.platform.clone()))?;
        let package = request.context.tags.get(ANDROID_PACKAGE_TAG);
        if let Some(package) = package.filter(|p| self.is_android_app(&platform.id, p)) {
            return Ok(android_app_outcome(package));
        }
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

    /// Whether a game of `platform` (id or alias) whose `localgameid` tag is `value` is an
    /// Android app filed under that console: the value reads as a package name and the
    /// platform's own tag files do not use `localgameid` for something else (Windows).
    pub fn is_android_app(&self, platform: &str, value: &str) -> bool {
        let own_tag = self
            .platform(platform)
            .is_some_and(|p| p.tag_files.iter().any(|t| t.key == ANDROID_PACKAGE_TAG));
        !own_tag && is_package_name(value)
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
