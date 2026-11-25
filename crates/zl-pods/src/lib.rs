//! Core Pod management library for Z-Launcher (`docs/spec/pods.md` v2).
//!
//! Provides package inspection, cryptographic Ed25519 signature validation,
//! manifest verification, store index parsing, crash monitoring, and atomic installation.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

pub mod error;
pub mod health;
pub mod index;
pub mod install;
pub mod manifest;
pub mod package;
pub mod policy;
pub mod registry;
pub mod signature;
pub mod trust;
pub mod version;

pub use error::{PodError, PodIssue};
pub use health::HealthMonitor;
pub use index::{parse_index, ApiFlag, IndexEntry, IndexIssue, StoreIndex};
pub use install::{PodInstaller, PodPreview};
pub use manifest::{parse_manifest, PodManifest, KNOWN_ABIS, KNOWN_PERMISSIONS, KNOWN_SLOTS};
pub use package::{inspect_package, inspect_package_bytes, PackageInspection};
pub use policy::{evaluate_policy, PolicyOutcome};
pub use registry::PodRegistry;
pub use signature::{
    compute_key_fingerprint, compute_signed_message, parse_signature_file,
    verify_package_signature, PackageSignature,
};
pub use trust::TrustStore;
pub use version::PodVersion;
