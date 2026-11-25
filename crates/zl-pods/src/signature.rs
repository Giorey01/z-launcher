//! Cryptographic signature verification and key fingerprinting for Pod packages.
//!
//! Defined in `docs/spec/pods.md` §4.

use base64::prelude::*;
use ring::digest::{digest, SHA256};
use ring::signature::{UnparsedPublicKey, ED25519};
use serde_json::Value;

use crate::error::PodIssue;

/// Structure representing parsed `SIGNATURE` file contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageSignature {
    /// Public key bytes (32 bytes for Ed25519).
    pub public_key: [u8; 32],
    /// Signature bytes (64 bytes for Ed25519).
    pub signature: [u8; 64],
}

/// Parses the `SIGNATURE` file payload.
pub fn parse_signature_file(bytes: &[u8]) -> Result<PackageSignature, PodIssue> {
    let json: Value = serde_json::from_slice(bytes).map_err(|_| PodIssue::SignatureMalformed)?;
    let obj = json.as_object().ok_or(PodIssue::SignatureMalformed)?;

    let algo = obj
        .get("algorithm")
        .and_then(Value::as_str)
        .ok_or(PodIssue::SignatureMalformed)?;
    if algo != "ed25519" {
        return Err(PodIssue::SignatureMalformed);
    }

    let pk_str = obj
        .get("publicKey")
        .and_then(Value::as_str)
        .ok_or(PodIssue::SignatureMalformed)?;
    let pk_bytes = BASE64_STANDARD
        .decode(pk_str)
        .map_err(|_| PodIssue::SignatureMalformed)?;
    if pk_bytes.len() != 32 {
        return Err(PodIssue::SignatureMalformed);
    }

    let sig_str = obj
        .get("signature")
        .and_then(Value::as_str)
        .ok_or(PodIssue::SignatureMalformed)?;
    let sig_bytes = BASE64_STANDARD
        .decode(sig_str)
        .map_err(|_| PodIssue::SignatureMalformed)?;
    if sig_bytes.len() != 64 {
        return Err(PodIssue::SignatureMalformed);
    }

    let mut public_key = [0u8; 32];
    public_key.copy_from_slice(&pk_bytes);

    let mut signature = [0u8; 64];
    signature.copy_from_slice(&sig_bytes);

    Ok(PackageSignature {
        public_key,
        signature,
    })
}

/// Computes the canonical signed message string bytes for a package.
///
/// Format:
/// ```text
/// zlpod-v1\n
/// <sha256>  <file1>\n
/// <sha256>  <file2>\n
/// ```
/// Files are sorted by byte order of their relative path name.
pub fn compute_signed_message(mut files: Vec<(&str, &[u8; 32])>) -> Vec<u8> {
    files.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

    let mut message = Vec::new();
    message.extend_from_slice(b"zlpod-v1\n");

    for (name, hash) in files {
        let hex_hash = hex_encode(hash);
        message.extend_from_slice(hex_hash.as_bytes());
        message.extend_from_slice(b"  ");
        message.extend_from_slice(name.as_bytes());
        message.push(b'\n');
    }

    message
}

/// Computes the 64-character lowercase hex SHA-256 fingerprint of a public key.
pub fn compute_key_fingerprint(public_key: &[u8]) -> String {
    let hash = digest(&SHA256, public_key);
    hex_encode(hash.as_ref())
}

/// Verifies that the package signature is valid for the provided file entries and returns the signer fingerprint.
pub fn verify_package_signature(
    signature_bytes: &[u8],
    files: Vec<(&str, &[u8; 32])>,
) -> Result<String, PodIssue> {
    let parsed = parse_signature_file(signature_bytes)?;
    let message = compute_signed_message(files);

    let peer_public_key = UnparsedPublicKey::new(&ED25519, &parsed.public_key);
    peer_public_key
        .verify(&message, &parsed.signature)
        .map_err(|_| PodIssue::SignatureInvalid)?;

    Ok(compute_key_fingerprint(&parsed.public_key))
}

/// Encodes bytes to lowercase hexadecimal.
fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}
