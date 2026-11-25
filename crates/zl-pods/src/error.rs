//! Structured error codes and issues emitted by the Pod system.
//!
//! Defined in `docs/spec/pods.md` (§2, §3, §4, §5, §6).

use std::fmt;

/// Standard issue codes defined in the Pod contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PodIssue {
    /// Relative path contains illegal segments, backslashes, or starts with `/`.
    BadPath,
    /// Unexpected top-level directory or file in package.
    UnexpectedEntry,
    /// Two package entries differ only by case.
    DuplicatePath,
    /// Native library present under an ABI not declared in the manifest.
    LibNotDeclared,
    /// Mandatory file (`manifest.json` or `classes.dex`) is missing.
    MissingFile,
    /// Mandatory `SIGNATURE` file is missing.
    SignatureMissing,
    /// File is not a valid zip archive.
    NotAZip,
    /// Archive contains more entries than [`zl_config::pods::MAX_PACKAGE_ENTRIES`].
    TooManyEntries,
    /// Compressed or uncompressed total package size exceeds configured limit.
    PackageTooLarge,
    /// An entry exceeds the maximum uncompressed size limit.
    EntryTooLarge,
    /// An entry exceeds the maximum compression ratio limit.
    EntryRatio,
    /// `manifest.json` is not valid JSON or not a JSON object.
    ManifestInvalidJson,
    /// Required manifest field is missing.
    MissingField,
    /// Manifest field has incorrect JSON type.
    WrongType,
    /// Manifest `schema` is not supported (currently must be `1`).
    UnsupportedSchema,
    /// Pod ID does not conform to lowercase, digits, `.` `_` `-` (1-64 chars).
    BadId,
    /// Pod display name is empty or exceeds 40 characters.
    BadName,
    /// Version does not follow strict `MAJOR.MINOR.PATCH` unsigned integer format.
    BadVersion,
    /// API version required is newer than [`zl_config::pods::API_CURRENT`].
    ApiTooNew,
    /// API version required is older than [`zl_config::pods::API_MIN`].
    ApiTooOld,
    /// Signer fingerprint is not 64 lowercase hex digits.
    BadSigner,
    /// Entry class name is not a valid qualified Java class name.
    BadEntry,
    /// Declared slot name is not recognized by specification.
    UnknownSlot,
    /// Declared permission name is not recognized by specification.
    UnknownPermission,
    /// Declared ABI name is not recognized (currently only `arm64-v8a`).
    UnknownAbi,
    /// URL is malformed or not an `https` URL.
    BadUrl,
    /// Duplicate value present in a list of unique items (slots, permissions).
    DuplicateValue,
    /// Pod ID is already registered or conflicts with built-in Pod.
    DuplicateId,
    /// `SIGNATURE` file is malformed JSON, wrong algorithm, or invalid key/sig lengths.
    SignatureMalformed,
    /// Cryptographic signature verification failed or archive tampered with.
    SignatureInvalid,
    /// Signer in manifest or index does not match actual package signing key.
    SignerMismatch,
    /// Update attempts to change established signing key fingerprint.
    SignerChanged,
    /// Index file is not valid JSON or schema is unsupported.
    IndexInvalidJson,
    /// Hash or fingerprint string in index is not 64 hex digits.
    BadHash,
    /// Downloaded package SHA-256 does not match index entry.
    HashMismatch,
}

impl PodIssue {
    /// String code defined by `docs/spec/pods.md`.
    pub const fn as_code(self) -> &'static str {
        match self {
            PodIssue::BadPath => "BAD_PATH",
            PodIssue::UnexpectedEntry => "UNEXPECTED_ENTRY",
            PodIssue::DuplicatePath => "DUPLICATE_PATH",
            PodIssue::LibNotDeclared => "LIB_NOT_DECLARED",
            PodIssue::MissingFile => "MISSING_FILE",
            PodIssue::SignatureMissing => "SIGNATURE_MISSING",
            PodIssue::NotAZip => "NOT_A_ZIP",
            PodIssue::TooManyEntries => "TOO_MANY_ENTRIES",
            PodIssue::PackageTooLarge => "PACKAGE_TOO_LARGE",
            PodIssue::EntryTooLarge => "ENTRY_TOO_LARGE",
            PodIssue::EntryRatio => "ENTRY_RATIO",
            PodIssue::ManifestInvalidJson => "MANIFEST_INVALID_JSON",
            PodIssue::MissingField => "MISSING_FIELD",
            PodIssue::WrongType => "WRONG_TYPE",
            PodIssue::UnsupportedSchema => "UNSUPPORTED_SCHEMA",
            PodIssue::BadId => "BAD_ID",
            PodIssue::BadName => "BAD_NAME",
            PodIssue::BadVersion => "BAD_VERSION",
            PodIssue::ApiTooNew => "API_TOO_NEW",
            PodIssue::ApiTooOld => "API_TOO_OLD",
            PodIssue::BadSigner => "BAD_SIGNER",
            PodIssue::BadEntry => "BAD_ENTRY",
            PodIssue::UnknownSlot => "UNKNOWN_SLOT",
            PodIssue::UnknownPermission => "UNKNOWN_PERMISSION",
            PodIssue::UnknownAbi => "UNKNOWN_ABI",
            PodIssue::BadUrl => "BAD_URL",
            PodIssue::DuplicateValue => "DUPLICATE_VALUE",
            PodIssue::DuplicateId => "DUPLICATE_ID",
            PodIssue::SignatureMalformed => "SIGNATURE_MALFORMED",
            PodIssue::SignatureInvalid => "SIGNATURE_INVALID",
            PodIssue::SignerMismatch => "SIGNER_MISMATCH",
            PodIssue::SignerChanged => "SIGNER_CHANGED",
            PodIssue::IndexInvalidJson => "INDEX_INVALID_JSON",
            PodIssue::BadHash => "BAD_HASH",
            PodIssue::HashMismatch => "HASH_MISMATCH",
        }
    }
}

impl fmt::Display for PodIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_code())
    }
}

impl std::error::Error for PodIssue {}

/// Operational error encountered during Pod operations.
#[derive(Debug, thiserror::Error)]
pub enum PodError {
    /// Pod validation issues discovered.
    #[error("validation failed with issues: {0:?}")]
    Validation(Vec<PodIssue>),
    /// Underlying I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Zip archive read error.
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    /// Database error.
    #[error("Database error: {0}")]
    Db(#[from] zl_db::DbError),
    /// HTTP network error.
    #[error("Network error: {0}")]
    Net(#[from] zl_net::HttpError),
}
