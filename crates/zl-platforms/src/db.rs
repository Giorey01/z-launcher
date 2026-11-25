//! The database itself: loading (embedded copy, newer copy from disk), validation and lookups.

use std::collections::HashMap;
use std::path::Path;

use serde::de::DeserializeOwned;
use zl_config::platforms::{EMULATORS_FILE_NAME, PLATFORMS_FILE_NAME};

use crate::error::{DataFile, DbError};
use crate::model::{Emulator, EmulatorsFile, Folder, Platform, PlatformsFile, PrefEntry};
use crate::validate::validate;

/// Format version of the data files this build reads. A format fact.
pub const SUPPORTED_SCHEMA: u32 = 1;

/// Copy of `data/platforms.json` shipped inside the app.
const EMBEDDED_PLATFORMS: &str = include_str!("../../../data/platforms.json");
/// Copy of `data/emulators.json` shipped inside the app.
const EMBEDDED_EMULATORS: &str = include_str!("../../../data/emulators.json");

/// Validated platforms, emulators and preferences.
#[derive(Debug, Clone)]
pub struct PlatformDb {
    /// The platform list file.
    pub(crate) platforms_file: PlatformsFile,
    /// The emulator list file.
    pub(crate) emulators_file: EmulatorsFile,
}

/// Parses one file, checking its schema.
fn parse<T: DeserializeOwned>(
    file: DataFile,
    text: &str,
    schema_of: fn(&T) -> u32,
) -> Result<T, DbError> {
    let value: T = serde_json::from_str(text).map_err(|e| DbError::Parse {
        file,
        line: e.line(),
        column: e.column(),
    })?;
    let found = schema_of(&value);
    if found != SUPPORTED_SCHEMA {
        return Err(DbError::SchemaMismatch {
            file,
            found,
            expected: SUPPORTED_SCHEMA,
        });
    }
    Ok(value)
}

/// Parses `platforms.json` text.
fn parse_platforms(text: &str) -> Result<PlatformsFile, DbError> {
    parse(DataFile::Platforms, text, |f: &PlatformsFile| f.schema)
}

/// Parses `emulators.json` text.
fn parse_emulators(text: &str) -> Result<EmulatorsFile, DbError> {
    parse(DataFile::Emulators, text, |f: &EmulatorsFile| f.schema)
}

/// Reads a file; a missing one is `None`, any other failure is an error.
fn read_optional(path: &Path) -> Result<Option<String>, DbError> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(DbError::Io {
            path: path.display().to_string(),
        }),
    }
}

impl PlatformDb {
    /// Builds a database from the two JSON texts and validates it.
    pub fn from_json(platforms: &str, emulators: &str) -> Result<Self, DbError> {
        let platforms_file = parse_platforms(platforms)?;
        let emulators_file = parse_emulators(emulators)?;
        validate(&platforms_file, &emulators_file)?;
        Ok(Self {
            platforms_file,
            emulators_file,
        })
    }

    /// The copy shipped inside the app.
    pub fn embedded() -> Result<Self, DbError> {
        Self::from_json(EMBEDDED_PLATFORMS, EMBEDDED_EMULATORS)
    }

    /// The embedded copy, with each file replaced by the one in `dir` when that one has a higher
    /// `version`. A missing file in `dir` is fine; a broken, foreign-schema or inconsistent one
    /// is an error, and the caller keeps using [`PlatformDb::embedded`].
    pub fn load_with_updates(dir: &Path) -> Result<Self, DbError> {
        let mut db = Self::embedded()?;
        if let Some(text) = read_optional(&dir.join(PLATFORMS_FILE_NAME))? {
            let newer = parse_platforms(&text)?;
            if newer.version > db.platforms_file.version {
                db.platforms_file = newer;
            }
        }
        if let Some(text) = read_optional(&dir.join(EMULATORS_FILE_NAME))? {
            let newer = parse_emulators(&text)?;
            if newer.version > db.emulators_file.version {
                db.emulators_file = newer;
            }
        }
        validate(&db.platforms_file, &db.emulators_file)?;
        Ok(db)
    }

    /// Versions of the two files in use: (platforms, emulators).
    pub fn versions(&self) -> (u32, u32) {
        (self.platforms_file.version, self.emulators_file.version)
    }

    /// All platforms.
    pub fn platforms(&self) -> &[Platform] {
        &self.platforms_file.platforms
    }

    /// All emulators.
    pub fn emulators(&self) -> &[Emulator] {
        &self.emulators_file.emulators
    }

    /// Looks up an emulator by id.
    pub fn emulator(&self, id: &str) -> Option<&Emulator> {
        self.emulators().iter().find(|e| e.id == id)
    }

    /// Looks up a platform by id or alias, ignoring case; returns the canonical platform.
    pub fn platform(&self, name: &str) -> Option<&Platform> {
        let name = name.to_lowercase();
        self.platforms()
            .iter()
            .find(|p| p.id == name || p.aliases.contains(&name))
    }

    /// All Home folders that group several platforms.
    pub fn folders(&self) -> &[Folder] {
        &self.platforms_file.folders
    }

    /// Looks up a Home folder by id, ignoring case.
    pub fn folder(&self, id: &str) -> Option<&Folder> {
        let id = id.to_lowercase();
        self.folders().iter().find(|f| f.id == id)
    }

    /// The Home folder a platform (id or alias) belongs to, if any.
    pub fn folder_of(&self, platform: &str) -> Option<&Folder> {
        let id = &self.platform(platform)?.id;
        self.folders().iter().find(|f| f.platforms.contains(id))
    }

    /// Platform id (map key) to its extensions, the shape the scanner consumes. Tag-file
    /// extensions are included.
    pub fn extension_rules(&self) -> HashMap<String, Vec<String>> {
        self.platforms()
            .iter()
            .map(|p| (p.id.clone(), p.extensions.clone()))
            .collect()
    }

    /// Preference list of a platform: its own, else the default one.
    pub(crate) fn preference(&self, platform: &str) -> &[PrefEntry] {
        self.emulators_file
            .preferences
            .get(platform)
            .map_or(&self.emulators_file.default_preference[..], Vec::as_slice)
    }
}
