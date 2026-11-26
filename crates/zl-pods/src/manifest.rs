//! Manifest parsing and formal validation according to `docs/spec/pods.md` §3.

use std::collections::HashSet;

use serde_json::{Map, Value};

use crate::error::PodIssue;
use crate::version::PodVersion;

/// The recognized slot names defined in `docs/spec/pods.md` §7.
pub const KNOWN_SLOTS: &[&str] = &[
    "start_menu",
    "platform_grid_lead",
    "game_actions",
    "settings_section",
    "status_widget",
    "home_shortcut",
];

/// The recognized permission names defined in `docs/spec/pods.md` §8.
pub const KNOWN_PERMISSIONS: &[&str] = &[
    "library.read",
    "library.rescan",
    "launch",
    "secrets",
    "http",
    "archive",
    "work",
    "notifications",
    "folders",
    "audio",
];

/// The recognized ABIs supported by the launcher (`docs/spec/pods.md` §3).
pub const KNOWN_ABIS: &[&str] = &["arm64-v8a"];

/// A validated Pod manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodManifest {
    /// Schema version (must be 1).
    pub schema: u32,
    /// Stable identifier (1-64 chars: `a-z`, `0-9`, `.`, `-`, `_`).
    pub id: String,
    /// Human-readable display name (1-40 chars, no control chars).
    pub name: String,
    /// Semantic version.
    pub version: PodVersion,
    /// Required API version.
    pub api: u32,
    /// Author display name.
    pub author_name: String,
    /// Signer public key fingerprint (64 lowercase hex digits).
    pub signer: String,
    /// Fully qualified Java class name of provider entry.
    pub entry: String,
    /// Optional description (up to 200 chars).
    pub description: Option<String>,
    /// Extension slots requested by this Pod.
    pub slots: Vec<String>,
    /// Host permissions requested by this Pod.
    pub permissions: Vec<String>,
    /// Native library ABIs included in package, if any.
    pub abis: Option<Vec<String>>,
    /// Optional homepage URL (must be `https://`).
    pub homepage: Option<String>,
}

/// Raw parsed values before semantic validation.
struct RawManifest<'a> {
    /// Schema integer.
    schema: u64,
    /// ID string.
    id: &'a str,
    /// Name string.
    name: &'a str,
    /// Version string.
    version: &'a str,
    /// API integer.
    api: u32,
    /// Author name string.
    author_name: &'a str,
    /// Signer fingerprint string.
    signer: &'a str,
    /// Entry class string.
    entry: &'a str,
    /// Optional description string.
    description: Option<&'a str>,
    /// Slots JSON array.
    slots: &'a [Value],
    /// Permissions JSON array.
    permissions: &'a [Value],
    /// Optional ABIs JSON array.
    abis: Option<&'a [Value]>,
    /// Optional homepage string.
    homepage: Option<&'a str>,
}

/// Parses and validates a `manifest.json` payload.
pub fn parse_manifest(bytes: &[u8]) -> Result<PodManifest, Vec<PodIssue>> {
    let json: Value =
        serde_json::from_slice(bytes).map_err(|_| vec![PodIssue::ManifestInvalidJson])?;
    let obj = json
        .as_object()
        .ok_or_else(|| vec![PodIssue::ManifestInvalidJson])?;

    check_presence(obj).map_err(|e| vec![e])?;
    let raw = extract_raw(obj).map_err(|e| vec![e])?;
    let version = validate_scalars(&raw).map_err(|e| vec![e])?;

    let slots =
        validate_list(raw.slots, KNOWN_SLOTS, PodIssue::UnknownSlot).map_err(|e| vec![e])?;
    let permissions = validate_list(
        raw.permissions,
        KNOWN_PERMISSIONS,
        PodIssue::UnknownPermission,
    )
    .map_err(|e| vec![e])?;

    let abis = match raw.abis {
        Some(arr) => {
            Some(validate_list(arr, KNOWN_ABIS, PodIssue::UnknownAbi).map_err(|e| vec![e])?)
        }
        None => None,
    };

    Ok(PodManifest {
        schema: 1,
        id: raw.id.to_string(),
        name: raw.name.to_string(),
        version,
        api: raw.api,
        author_name: raw.author_name.to_string(),
        signer: raw.signer.to_string(),
        entry: raw.entry.to_string(),
        description: raw.description.map(str::to_string),
        slots,
        permissions,
        abis,
        homepage: raw.homepage.map(str::to_string),
    })
}

/// Checks required fields presence.
fn check_presence(obj: &Map<String, Value>) -> Result<(), PodIssue> {
    let req = [
        "schema",
        "id",
        "name",
        "version",
        "api",
        "author",
        "entry",
        "slots",
        "permissions",
    ];
    for field in &req {
        if !obj.contains_key(*field) {
            return Err(PodIssue::MissingField);
        }
    }
    if let Some(author) = obj.get("author") {
        if let Some(author_obj) = author.as_object() {
            if !author_obj.contains_key("name") || !author_obj.contains_key("signer") {
                return Err(PodIssue::MissingField);
            }
        } else {
            return Err(PodIssue::WrongType);
        }
    }
    Ok(())
}

/// Extracts author name and signer fingerprint from the author object.
fn extract_author(obj: &Map<String, Value>) -> Result<(&str, &str), PodIssue> {
    let author_obj = obj
        .get("author")
        .and_then(Value::as_object)
        .ok_or(PodIssue::WrongType)?;
    let author_name = author_obj
        .get("name")
        .and_then(Value::as_str)
        .ok_or(PodIssue::WrongType)?;
    let signer = author_obj
        .get("signer")
        .and_then(Value::as_str)
        .ok_or(PodIssue::WrongType)?;
    Ok((author_name, signer))
}

/// Optional manifest fields extracted before validation.
struct OptionalManifestFields<'a> {
    /// Pod description string.
    description: Option<&'a str>,
    /// Target ABI array.
    abis: Option<&'a [Value]>,
    /// Homepage URL string.
    homepage: Option<&'a str>,
}

/// Extracts optional string or array fields from manifest object.
fn extract_optionals(obj: &Map<String, Value>) -> Result<OptionalManifestFields<'_>, PodIssue> {
    let description = match obj.get("description") {
        Some(v) => Some(v.as_str().ok_or(PodIssue::WrongType)?),
        None => None,
    };
    let abis = match obj.get("abis") {
        Some(v) => Some(v.as_array().ok_or(PodIssue::WrongType)?.as_slice()),
        None => None,
    };
    let homepage = match obj.get("homepage") {
        Some(v) => Some(v.as_str().ok_or(PodIssue::WrongType)?),
        None => None,
    };
    Ok(OptionalManifestFields {
        description,
        abis,
        homepage,
    })
}

/// Extracts raw typed fields.
fn extract_raw<'a>(obj: &'a Map<String, Value>) -> Result<RawManifest<'a>, PodIssue> {
    let schema = obj
        .get("schema")
        .and_then(Value::as_u64)
        .ok_or(PodIssue::WrongType)?;
    let id = obj
        .get("id")
        .and_then(Value::as_str)
        .ok_or(PodIssue::WrongType)?;
    let name = obj
        .get("name")
        .and_then(Value::as_str)
        .ok_or(PodIssue::WrongType)?;
    let version = obj
        .get("version")
        .and_then(Value::as_str)
        .ok_or(PodIssue::WrongType)?;
    let api = obj
        .get("api")
        .and_then(Value::as_u64)
        .ok_or(PodIssue::WrongType)? as u32;

    let (author_name, signer) = extract_author(obj)?;
    let entry = obj
        .get("entry")
        .and_then(Value::as_str)
        .ok_or(PodIssue::WrongType)?;
    let slots = obj
        .get("slots")
        .and_then(Value::as_array)
        .ok_or(PodIssue::WrongType)?;
    let permissions = obj
        .get("permissions")
        .and_then(Value::as_array)
        .ok_or(PodIssue::WrongType)?;
    let opt = extract_optionals(obj)?;

    Ok(RawManifest {
        schema,
        id,
        name,
        version,
        api,
        author_name,
        signer,
        entry,
        description: opt.description,
        slots,
        permissions,
        abis: opt.abis,
        homepage: opt.homepage,
    })
}

/// Validates individual scalar field rules.
fn validate_scalars(raw: &RawManifest<'_>) -> Result<PodVersion, PodIssue> {
    if raw.schema != 1 {
        return Err(PodIssue::UnsupportedSchema);
    }
    if raw.id.is_empty()
        || raw.id.len() > 64
        || !raw.id.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-' || c == '_'
        })
    {
        return Err(PodIssue::BadId);
    }
    if raw.name.is_empty() || raw.name.len() > 40 || raw.name.chars().any(|c| c.is_ascii_control())
    {
        return Err(PodIssue::BadName);
    }
    let version = PodVersion::parse(raw.version)?;
    if raw.api > zl_config::pods::API_CURRENT {
        return Err(PodIssue::ApiTooNew);
    }
    if raw.api < zl_config::pods::API_MIN {
        return Err(PodIssue::ApiTooOld);
    }
    if raw.signer.len() != 64
        || !raw
            .signer
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
    {
        return Err(PodIssue::BadSigner);
    }
    if raw.entry.is_empty() || raw.entry.contains(' ') || !is_valid_java_class(raw.entry) {
        return Err(PodIssue::BadEntry);
    }
    if let Some(url) = raw.homepage {
        if !url.starts_with("https://") || url.len() <= 8 {
            return Err(PodIssue::BadUrl);
        }
    }
    Ok(version)
}

/// Validates a list of strings ensuring uniqueness and known membership.
fn validate_list(
    arr: &[Value],
    known: &[&str],
    unknown_err: PodIssue,
) -> Result<Vec<String>, PodIssue> {
    let mut list = Vec::with_capacity(arr.len());
    let mut seen = HashSet::new();
    for item in arr {
        let s = item.as_str().ok_or(PodIssue::WrongType)?;
        if !seen.insert(s) {
            return Err(PodIssue::DuplicateValue);
        }
        if !known.contains(&s) {
            return Err(unknown_err);
        }
        list.push(s.to_string());
    }
    Ok(list)
}

/// Helper to validate qualified Java class name syntax.
fn is_valid_java_class(name: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    if parts.is_empty() {
        return false;
    }
    for part in parts {
        if part.is_empty() {
            return false;
        }
        let mut chars = part.chars();
        let first = match chars.next() {
            Some(c) => c,
            None => return false,
        };
        if !first.is_alphabetic() && first != '_' && first != '$' {
            return false;
        }
        if !chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$') {
            return false;
        }
    }
    true
}
