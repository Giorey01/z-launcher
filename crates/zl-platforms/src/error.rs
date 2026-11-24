//! Structured errors: stable codes the apps translate, never sentences.

use thiserror::Error;

/// Which of the two data files an error refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFile {
    /// `platforms.json`.
    Platforms,
    /// `emulators.json`.
    Emulators,
}

/// Failure while loading or validating the database.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DbError {
    /// A file exists but could not be read.
    #[error("io {path}")]
    Io {
        /// Path that failed.
        path: String,
    },
    /// A file is not valid JSON for its format; position of the first problem.
    #[error("parse {file:?} {line}:{column}")]
    Parse {
        /// Which file.
        file: DataFile,
        /// Line, starting at 1.
        line: usize,
        /// Column, starting at 1.
        column: usize,
    },
    /// A file follows another format version than this build understands.
    #[error("schema_mismatch {file:?} {found} != {expected}")]
    SchemaMismatch {
        /// Which file.
        file: DataFile,
        /// Schema the file declares.
        found: u32,
        /// Schema this build reads.
        expected: u32,
    },
    /// The content breaks a rule; `code` names the rule, `subject` what broke it.
    #[error("{code} {subject}")]
    Invalid {
        /// Stable code (`duplicate_platform_id`, `unknown_variable`, ...).
        code: &'static str,
        /// Id or text the rule is about.
        subject: String,
    },
}

/// Failure while resolving a profile or building an intent.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LaunchError {
    /// The platform id or alias is not in the database.
    #[error("unknown_platform {0}")]
    UnknownPlatform(String),
    /// The emulator id is not in the database.
    #[error("unknown_emulator {0}")]
    UnknownEmulator(String),
    /// The profile id does not exist in that emulator.
    #[error("unknown_profile {emulator}/{profile}")]
    UnknownProfile {
        /// Emulator id.
        emulator: String,
        /// Profile id.
        profile: String,
    },
    /// The requested package is not one of the emulator's packages.
    #[error("unknown_package {0}")]
    UnknownPackage(String),
    /// The profile does not launch this platform.
    #[error("profile_platform_mismatch {profile} {platform}")]
    ProfilePlatformMismatch {
        /// Profile id.
        profile: String,
        /// Platform id.
        platform: String,
    },
    /// A template uses a variable that does not exist.
    #[error("unknown_variable {0}")]
    UnknownVariable(String),
    /// A template has an unclosed or empty `{}`.
    #[error("bad_template {0}")]
    BadTemplate(String),
    /// The variable exists but the caller did not provide its value.
    #[error("missing_variable {0}")]
    MissingVariable(String),
    /// A `{tags.NAME}` variable has no such tag on the game.
    #[error("missing_tag {0}")]
    MissingTag(String),
    /// An integer extra did not render to a number.
    #[error("invalid_integer {0}")]
    InvalidInteger(String),
    /// The profile needs a RetroArch core and the platform has none.
    #[error("no_core {0}")]
    NoCore(String),
    /// The requested core does not exist for this platform.
    #[error("unknown_core {0}")]
    UnknownCore(String),
}
