//! Store index parsing and tolerant entry validation (`docs/spec/pods.md` §5).

use std::collections::HashSet;

use serde_json::Value;

use crate::error::PodIssue;
use crate::version::PodVersion;

/// Compatibility flag for an index entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiFlag {
    /// Pod requires an API version newer than current launcher support.
    ApiTooNew,
    /// Pod requires an API version older than supported minimum.
    ApiTooOld,
}

impl ApiFlag {
    /// Canonical code string.
    pub const fn as_code(self) -> &'static str {
        match self {
            ApiFlag::ApiTooNew => "API_TOO_NEW",
            ApiFlag::ApiTooOld => "API_TOO_OLD",
        }
    }
}

/// A parsed entry from a store index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexEntry {
    /// Pod ID.
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Semantic version.
    pub version: PodVersion,
    /// Required API level.
    pub api: u32,
    /// Optional short description.
    pub description: Option<String>,
    /// Download HTTPS URL for the `.zlpod` package.
    pub url: String,
    /// Expected SHA-256 hash of the package.
    pub sha256: String,
    /// Package file size in bytes.
    pub size: u64,
    /// Signer public key fingerprint.
    pub signer: String,
    /// Compatibility flag if not installable directly.
    pub flag: Option<ApiFlag>,
}

/// A validation issue encountered when reading an index document or entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexIssue {
    /// 0-based entry index in the `pods` array, or `None` if document-level.
    pub entry: Option<usize>,
    /// The specific issue code.
    pub issue: PodIssue,
}

/// A parsed store index document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreIndex {
    /// Index schema version (must be 1).
    pub schema: u32,
    /// Name of the store repository.
    pub name: String,
    /// Last updated date string.
    pub updated: String,
    /// Valid kept entries.
    pub entries: Vec<IndexEntry>,
    /// Discarded entry issues.
    pub issues: Vec<IndexIssue>,
}

/// Parses a store index JSON document.
///
/// Returns an error only if the document cannot be parsed as JSON or if its top-level schema is unsupported.
/// Invalid individual entries are recorded in [`StoreIndex::issues`] while valid entries are kept.
pub fn parse_index(bytes: &[u8]) -> Result<StoreIndex, IndexIssue> {
    let json: Value = serde_json::from_slice(bytes).map_err(|_| IndexIssue {
        entry: None,
        issue: PodIssue::IndexInvalidJson,
    })?;
    let obj = json.as_object().ok_or(IndexIssue {
        entry: None,
        issue: PodIssue::IndexInvalidJson,
    })?;

    let schema_num = obj.get("schema").and_then(Value::as_u64);
    if schema_num != Some(1) {
        return Err(IndexIssue {
            entry: None,
            issue: PodIssue::UnsupportedSchema,
        });
    }

    let name = obj
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let updated = obj
        .get("updated")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let pods_arr = match obj.get("pods").and_then(Value::as_array) {
        Some(a) => a,
        None => {
            return Err(IndexIssue {
                entry: None,
                issue: PodIssue::IndexInvalidJson,
            })
        }
    };

    let mut entries = Vec::new();
    let mut issues = Vec::new();
    let mut seen_ids = HashSet::new();

    for (idx, item) in pods_arr.iter().enumerate() {
        match parse_index_entry(item, idx, &mut seen_ids) {
            Ok(entry) => entries.push(entry),
            Err(issue) => issues.push(issue),
        }
    }

    Ok(StoreIndex {
        schema: 1,
        name,
        updated,
        entries,
        issues,
    })
}

/// Validates string field formats for an index entry.
fn validate_entry_format(
    id: &str,
    url: &str,
    sha256: &str,
    signer: &str,
    version_str: &str,
) -> Result<PodVersion, PodIssue> {
    if !url.starts_with("https://") || url.len() <= 8 {
        return Err(PodIssue::BadUrl);
    }
    let is_hex64 = |s: &str| s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit());
    if !is_hex64(sha256) || !is_hex64(signer) {
        return Err(PodIssue::BadHash);
    }
    if id.is_empty()
        || id.len() > 64
        || !id.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-' || c == '_'
        })
    {
        return Err(PodIssue::BadId);
    }
    PodVersion::parse(version_str)
}

/// Computes compatibility flag based on API version.
fn api_flag(api: u32) -> Option<ApiFlag> {
    if api > zl_config::pods::API_CURRENT {
        Some(ApiFlag::ApiTooNew)
    } else if api < zl_config::pods::API_MIN {
        Some(ApiFlag::ApiTooOld)
    } else {
        None
    }
}

/// Raw string and numeric fields extracted from an index entry object.
struct RawEntryFields<'a> {
    /// Pod id string.
    id: &'a str,
    /// Pod display name.
    name: &'a str,
    /// Semantic version string.
    version: &'a str,
    /// Target API level.
    api: u32,
    /// Download URL.
    url: &'a str,
    /// SHA256 hex string.
    sha256: &'a str,
    /// Package size in bytes.
    size: u64,
    /// Signer hex key fingerprint.
    signer: &'a str,
}

/// Extracts mandatory string and numeric fields from an entry JSON object.
fn extract_entry_raw_fields(obj: &serde_json::Map<String, Value>) -> Option<RawEntryFields<'_>> {
    Some(RawEntryFields {
        id: obj.get("id")?.as_str()?,
        name: obj.get("name")?.as_str()?,
        version: obj.get("version")?.as_str()?,
        api: obj.get("api")?.as_u64()? as u32,
        url: obj.get("url")?.as_str()?,
        sha256: obj.get("sha256")?.as_str()?,
        size: obj.get("size")?.as_u64()?,
        signer: obj.get("signer")?.as_str()?,
    })
}

/// Helper to parse a single index entry.
fn parse_index_entry(
    val: &Value,
    idx: usize,
    seen_ids: &mut HashSet<String>,
) -> Result<IndexEntry, IndexIssue> {
    let err = |issue| IndexIssue {
        entry: Some(idx),
        issue,
    };

    let obj = val.as_object().ok_or_else(|| err(PodIssue::WrongType))?;
    let raw = extract_entry_raw_fields(obj).ok_or_else(|| err(PodIssue::MissingField))?;

    let version =
        validate_entry_format(raw.id, raw.url, raw.sha256, raw.signer, raw.version).map_err(err)?;

    if !seen_ids.insert(raw.id.to_string()) {
        return Err(err(PodIssue::DuplicateId));
    }

    let description = obj
        .get("description")
        .and_then(Value::as_str)
        .map(str::to_string);

    Ok(IndexEntry {
        id: raw.id.to_string(),
        name: raw.name.to_string(),
        version,
        api: raw.api,
        description,
        url: raw.url.to_string(),
        sha256: raw.sha256.to_string(),
        size: raw.size,
        signer: raw.signer.to_string(),
        flag: api_flag(raw.api),
    })
}
