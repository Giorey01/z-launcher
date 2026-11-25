//! Consistency rules of the database: the checks that make a bad update fail at load time
//! instead of at launch time.

use std::collections::HashSet;

use crate::error::DbError;
use crate::model::{
    EmulatorKind, EmulatorsFile, ExtraTemplateValue, IntTemplate, IntentTemplate, PlatformsFile,
    Profile, RomAccess,
};
use crate::template;

/// Flags a profile may name; the app maps each to the Android constant. A format fact.
pub const KNOWN_FLAGS: &[&str] = &[
    "clear_task",
    "clear_top",
    "new_task",
    "no_animation",
    "grant_read_uri_permission",
    "grant_write_uri_permission",
];

/// Builds an [`DbError::Invalid`].
fn invalid(code: &'static str, subject: impl Into<String>) -> DbError {
    DbError::Invalid {
        code,
        subject: subject.into(),
    }
}

/// Whether `id` can be part of a stored profile reference (`emulator/profile/core`): not empty
/// and without the `/` separator or whitespace.
fn plain_id(id: &str) -> bool {
    !id.is_empty() && !id.contains('/') && !id.chars().any(char::is_whitespace)
}

/// Checks both files together; returns the first broken rule.
pub fn validate(p: &PlatformsFile, e: &EmulatorsFile) -> Result<(), DbError> {
    let ids = platform_ids(p)?;
    let mut emu_ids = HashSet::new();
    for emu in &e.emulators {
        if !plain_id(&emu.id) {
            return Err(invalid("bad_id", &emu.id));
        }
        if !emu_ids.insert(emu.id.as_str()) {
            return Err(invalid("duplicate_emulator_id", &emu.id));
        }
        validate_emulator(emu, &ids)?;
    }
    validate_preferences(e, &emu_ids, &ids)
}

/// Collects platform ids and aliases (lowercase, unique across both) and checks each platform.
fn platform_ids(p: &PlatformsFile) -> Result<HashSet<&str>, DbError> {
    let mut seen = HashSet::new();
    let mut ids = HashSet::new();
    for plat in &p.platforms {
        for name in std::iter::once(&plat.id).chain(&plat.aliases) {
            if name.is_empty() || *name != name.to_lowercase() || !seen.insert(name.as_str()) {
                return Err(invalid("bad_or_duplicate_platform_name", name));
            }
        }
        if plat
            .extensions
            .iter()
            .any(|x| x.is_empty() || *x != x.to_lowercase() || x.starts_with('.'))
        {
            return Err(invalid("bad_extension", &plat.id));
        }
        ids.insert(plat.id.as_str());
    }
    Ok(ids)
}

/// Checks packages, profiles and cores of one emulator.
fn validate_emulator(emu: &crate::model::Emulator, ids: &HashSet<&str>) -> Result<(), DbError> {
    if emu.packages.is_empty() || emu.packages.iter().any(|k| k.package.is_empty()) {
        return Err(invalid("bad_package", &emu.id));
    }
    let known = |p: &String| ids.contains(p.as_str());
    if !emu.platforms.iter().all(known) || !emu.cores.keys().all(known) {
        return Err(invalid("unknown_platform", &emu.id));
    }
    for core in emu.cores.values().flatten() {
        if !plain_id(&core.id) {
            return Err(invalid("bad_id", format!("{}/{}", emu.id, core.id)));
        }
    }
    let mut profile_ids = HashSet::new();
    for profile in &emu.profiles {
        if !plain_id(&profile.id) {
            return Err(invalid("bad_id", format!("{}/{}", emu.id, profile.id)));
        }
        if !profile_ids.insert(profile.id.as_str()) {
            return Err(invalid(
                "duplicate_profile_id",
                format!("{}/{}", emu.id, profile.id),
            ));
        }
        if !profile.platforms.iter().all(known) {
            return Err(invalid(
                "unknown_platform",
                format!("{}/{}", emu.id, profile.id),
            ));
        }
        validate_profile(emu.kind, &emu.id, profile)?;
        let no_core =
            |p: &String| profile.uses_core && !emu.cores.get(p).is_some_and(|c| !c.is_empty());
        if let Some(platform) = profile.platforms.iter().find(|p| no_core(p)) {
            let subject = format!("{}/{}/{platform}", emu.id, profile.id);
            return Err(invalid("no_core_for_platform", subject));
        }
    }
    Ok(())
}

/// Every string template of an intent, for variable checks.
fn templates(it: &IntentTemplate) -> Vec<&str> {
    let mut out: Vec<&str> = it.data.iter().map(String::as_str).collect();
    out.extend(it.action.iter().map(String::as_str));
    out.extend(it.clip_data.iter().map(|c| c.uri.as_str()));
    out.extend(it.uri_grants.iter().map(|g| g.uri.as_str()));
    for x in &it.extras {
        match &x.value {
            ExtraTemplateValue::String { value } => out.push(value),
            ExtraTemplateValue::Int {
                value: IntTemplate::Template(t),
            } => out.push(t),
            ExtraTemplateValue::StringArray { value } => {
                out.extend(value.iter().map(String::as_str))
            }
            _ => {}
        }
    }
    out
}

/// Checks a profile: known flags, valid variables, ROM access and core use consistent.
fn validate_profile(kind: EmulatorKind, emu: &str, profile: &Profile) -> Result<(), DbError> {
    let subject = format!("{emu}/{}", profile.id);
    if profile
        .intent
        .flags
        .iter()
        .any(|f| !KNOWN_FLAGS.contains(&f.as_str()))
    {
        return Err(invalid("unknown_flag", subject));
    }
    let mut uses_rom = HashSet::new();
    let mut uses_core = false;
    for t in templates(&profile.intent) {
        for var in template::variables(t).map_err(|_| invalid("bad_template", &subject))? {
            match var {
                "rom" => uses_rom.insert(RomAccess::Path),
                "rom_uri" => uses_rom.insert(RomAccess::FileUri),
                "rom_provider" => uses_rom.insert(RomAccess::ContentUri),
                "rom_saf" => uses_rom.insert(RomAccess::Saf),
                "core" | "core_path" => {
                    uses_core = true;
                    false
                }
                _ => false,
            };
        }
    }
    let access_ok = match profile.rom_access {
        RomAccess::None => uses_rom.is_empty(),
        a => uses_rom.len() == 1 && uses_rom.contains(&a),
    };
    if !access_ok {
        return Err(invalid("rom_access_mismatch", subject));
    }
    if uses_core != profile.uses_core || (uses_core && kind != EmulatorKind::Retroarch) {
        return Err(invalid("core_use_mismatch", subject));
    }
    Ok(())
}

/// Checks that preference entries name existing emulators and cores.
fn validate_preferences(
    e: &EmulatorsFile,
    emu_ids: &HashSet<&str>,
    platform_ids: &HashSet<&str>,
) -> Result<(), DbError> {
    if let Some(name) = e
        .preferences
        .keys()
        .find(|k| !platform_ids.contains(k.as_str()))
    {
        return Err(invalid("unknown_preference_platform", name));
    }
    let all = e
        .default_preference
        .iter()
        .chain(e.preferences.values().flatten());
    for entry in all {
        if !emu_ids.contains(entry.emulator.as_str()) {
            return Err(invalid("unknown_preference_emulator", &entry.emulator));
        }
        let known_core = |c: &String| {
            e.emulators
                .iter()
                .find(|x| x.id == entry.emulator)
                .is_some_and(|x| x.cores.values().flatten().any(|k| &k.id == c))
        };
        if entry.core.as_ref().is_some_and(|c| !known_core(c)) {
            return Err(invalid("unknown_preference_core", &entry.emulator));
        }
    }
    Ok(())
}
