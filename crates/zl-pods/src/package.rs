//! Package inspection and validation for `.zlpod` archives.
//!
//! Defined in `docs/spec/pods.md` §2, §3, §4, §6.

use ring::digest::{digest, SHA256};
use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read, Seek};
use zip::ZipArchive;

use crate::error::PodIssue;
use crate::manifest::{parse_manifest, PodManifest};
use crate::signature::verify_package_signature;
use crate::version::PodVersion;

/// The outcome of a successfully inspected Pod package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageInspection {
    /// Pod ID from manifest.
    pub id: String,
    /// Semantic version from manifest.
    pub version: PodVersion,
    /// Signer public key fingerprint (64 lowercase hex digits).
    pub signer: String,
    /// Validated manifest details.
    pub manifest: PodManifest,
    /// Total number of files in package.
    pub file_count: usize,
    /// Total uncompressed size in bytes.
    pub uncompressed_size: u64,
}

/// Inspects and validates a `.zlpod` package from a seekable reader.
///
/// Follows the strict phased verification order defined in `docs/spec/pods.md` §2.
pub fn inspect_package<R: Read + Seek>(
    reader: R,
    package_file_size: u64,
) -> Result<PackageInspection, Vec<PodIssue>> {
    if package_file_size > zl_config::pods::MAX_PACKAGE_SIZE_BYTES {
        return Err(vec![PodIssue::PackageTooLarge]);
    }

    let mut archive = ZipArchive::new(reader).map_err(|_| vec![PodIssue::NotAZip])?;
    let (valid_file_names, total_uncompressed) = validate_entries_and_limits(&mut archive)?;
    check_mandatory_files(&valid_file_names)?;

    let (signer_fingerprint, manifest_bytes) =
        read_and_verify_signature(&mut archive, &valid_file_names)?;
    let manifest = parse_manifest(&manifest_bytes)?;
    validate_manifest_coherence(&manifest, &signer_fingerprint, &valid_file_names)?;

    Ok(PackageInspection {
        id: manifest.id.clone(),
        version: manifest.version,
        signer: signer_fingerprint,
        file_count: valid_file_names.len(),
        uncompressed_size: total_uncompressed,
        manifest,
    })
}

/// Phase 2: validates entries, paths, and size limits.
fn validate_entries_and_limits<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<(Vec<String>, u64), Vec<PodIssue>> {
    let total_entries = archive.len();
    if total_entries > zl_config::pods::MAX_PACKAGE_ENTRIES {
        return Err(vec![PodIssue::TooManyEntries]);
    }

    let mut issues = Vec::new();
    let mut lower_paths = HashSet::new();
    let mut valid_file_names = Vec::new();
    let mut total_uncompressed: u64 = 0;

    for i in 0..total_entries {
        let entry = match archive.by_index_raw(i) {
            Ok(e) => e,
            Err(_) => {
                issues.push(PodIssue::BadPath);
                continue;
            }
        };

        let raw_name = entry.name().to_string();
        if raw_name.ends_with('/') && entry.size() == 0 {
            continue;
        }

        check_single_entry(
            &raw_name,
            entry.size(),
            entry.compressed_size(),
            &mut issues,
        );
        if !lower_paths.insert(raw_name.to_ascii_lowercase()) {
            issues.push(PodIssue::DuplicatePath);
        }

        total_uncompressed = total_uncompressed.saturating_add(entry.size());
        if total_uncompressed > zl_config::pods::MAX_TOTAL_UNCOMPRESSED_SIZE_BYTES {
            issues.push(PodIssue::PackageTooLarge);
        }

        valid_file_names.push(raw_name);
    }

    if !issues.is_empty() {
        issues.dedup();
        return Err(issues);
    }
    Ok((valid_file_names, total_uncompressed))
}

/// Checks path legality, size, and compression ratio for an individual archive entry.
fn check_single_entry(name: &str, uncompressed: u64, compressed: u64, issues: &mut Vec<PodIssue>) {
    if !is_valid_entry_path(name) {
        issues.push(PodIssue::BadPath);
    }
    if !is_allowed_package_path(name) {
        issues.push(PodIssue::UnexpectedEntry);
    }
    if uncompressed > zl_config::pods::MAX_ENTRY_UNCOMPRESSED_SIZE_BYTES {
        issues.push(PodIssue::EntryTooLarge);
    }
    if name == "manifest.json" && uncompressed > zl_config::pods::MAX_MANIFEST_SIZE_BYTES {
        issues.push(PodIssue::EntryTooLarge);
    }
    if uncompressed >= zl_config::pods::ENTRY_RATIO_THRESHOLD_BYTES {
        let ratio = uncompressed.checked_div(compressed).unwrap_or(u64::MAX);
        if ratio > zl_config::pods::MAX_ENTRY_RATIO {
            issues.push(PodIssue::EntryRatio);
        }
    }
}

/// Phase 3: checks for required package files.
fn check_mandatory_files(valid_file_names: &[String]) -> Result<(), Vec<PodIssue>> {
    let has_manifest = valid_file_names.iter().any(|n| n == "manifest.json");
    let has_dex = valid_file_names.iter().any(|n| n == "classes.dex");
    let has_signature = valid_file_names.iter().any(|n| n == "SIGNATURE");

    let mut issues = Vec::new();
    if !has_manifest || !has_dex {
        issues.push(PodIssue::MissingFile);
    }
    if !has_signature {
        issues.push(PodIssue::SignatureMissing);
    }

    if !issues.is_empty() {
        return Err(issues);
    }
    Ok(())
}

/// Phase 4: reads file bytes, calculates SHA256 hashes, and verifies the Ed25519 signature.
fn read_and_verify_signature<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    valid_file_names: &[String],
) -> Result<(String, Vec<u8>), Vec<PodIssue>> {
    let mut file_hashes: HashMap<String, [u8; 32]> = HashMap::new();
    let mut signature_bytes = Vec::new();
    let mut manifest_bytes = Vec::new();

    for name in valid_file_names {
        let mut entry = archive
            .by_name(name)
            .map_err(|_| vec![PodIssue::MissingFile])?;
        let mut data = Vec::with_capacity(entry.size() as usize);
        if entry.read_to_end(&mut data).is_err() {
            return Err(vec![PodIssue::BadPath]);
        }

        if name == "SIGNATURE" {
            signature_bytes = data;
        } else {
            let hash = digest(&SHA256, &data);
            let mut h = [0u8; 32];
            h.copy_from_slice(hash.as_ref());
            file_hashes.insert(name.clone(), h);

            if name == "manifest.json" {
                manifest_bytes = data;
            }
        }
    }

    let files_for_sig: Vec<(&str, &[u8; 32])> =
        file_hashes.iter().map(|(n, h)| (n.as_str(), h)).collect();

    let signer = verify_package_signature(&signature_bytes, files_for_sig).map_err(|i| vec![i])?;
    Ok((signer, manifest_bytes))
}

/// Phase 6: validates coherence between manifest and package structure.
fn validate_manifest_coherence(
    manifest: &PodManifest,
    signer_fingerprint: &str,
    valid_file_names: &[String],
) -> Result<(), Vec<PodIssue>> {
    let mut issues = Vec::new();
    if manifest.signer != signer_fingerprint {
        issues.push(PodIssue::SignerMismatch);
    }

    for name in valid_file_names {
        if let Some(rest) = name.strip_prefix("lib/") {
            if let Some(abi) = rest.split('/').next() {
                let declared = manifest
                    .abis
                    .as_ref()
                    .is_some_and(|abis| abis.iter().any(|a| a == abi));
                if !declared {
                    issues.push(PodIssue::LibNotDeclared);
                    break;
                }
            }
        }
    }

    if !issues.is_empty() {
        return Err(issues);
    }
    Ok(())
}

/// Convenience helper to inspect package bytes.
pub fn inspect_package_bytes(bytes: &[u8]) -> Result<PackageInspection, Vec<PodIssue>> {
    let len = bytes.len() as u64;
    inspect_package(Cursor::new(bytes), len)
}

/// Checks whether a path string contains invalid segments, backslashes, or absolute roots.
fn is_valid_entry_path(path: &str) -> bool {
    if path.starts_with('/') || path.contains('\\') {
        return false;
    }
    for seg in path.split('/') {
        if seg.is_empty() || seg == ".." || seg == "." {
            return false;
        }
    }
    true
}

/// Checks whether a path corresponds to an allowed top-level directory or file.
fn is_allowed_package_path(path: &str) -> bool {
    if path == "manifest.json" || path == "classes.dex" || path == "SIGNATURE" {
        return true;
    }
    if let Some(rest) = path.strip_prefix("lib/") {
        let parts: Vec<&str> = rest.split('/').collect();
        return parts.len() == 2 && parts[1].ends_with(".so");
    }
    if path.starts_with("assets/") && path.len() > 7 {
        return true;
    }
    if let Some(rest) = path.strip_prefix("i18n/") {
        let parts: Vec<&str> = rest.split('/').collect();
        return parts.len() == 1 && parts[0].ends_with(".json");
    }
    false
}
