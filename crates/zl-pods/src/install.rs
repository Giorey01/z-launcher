//! Atomic package installation and update workflows (`docs/spec/pods.md` §6).

use ring::digest::{digest, SHA256};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use zip::ZipArchive;
use zl_db::{Db, InstalledPod, PodState};

use crate::error::{PodError, PodIssue};
use crate::package::{inspect_package, PackageInspection};
use crate::policy::{evaluate_policy, PolicyOutcome};
use crate::version::PodVersion;

/// Preview information returned when inspecting a candidate package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodPreview {
    /// Pod ID.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Semantic version.
    pub version: PodVersion,
    /// Author name.
    pub author_name: String,
    /// Signer public key fingerprint.
    pub signer: String,
    /// Description.
    pub description: Option<String>,
    /// Slots requested.
    pub slots: Vec<String>,
    /// Permissions requested.
    pub permissions: Vec<String>,
    /// Uncompressed size in bytes.
    pub uncompressed_size: u64,
    /// Installation policy outcome.
    pub outcome: PolicyOutcome,
}

/// Orchestrates atomic installation of `.zlpod` packages.
pub struct PodInstaller<'a> {
    /// Database handle.
    db: &'a Db,
    /// Base directory on filesystem where pods are deployed (`pods/`).
    pods_dir: PathBuf,
}

impl<'a> PodInstaller<'a> {
    /// Creates a new installer instance.
    pub fn new(db: &'a Db, pods_dir: impl Into<PathBuf>) -> Self {
        Self {
            db,
            pods_dir: pods_dir.into(),
        }
    }

    /// Inspects a package without modifying state.
    pub fn inspect(&self, package_path: &Path) -> Result<PodPreview, PodError> {
        let inspection = self.read_and_inspect(package_path)?;
        let existing = self.db.get_pod(&inspection.id)?;

        let installed_info = existing.as_ref().and_then(|p| {
            PodVersion::parse(&p.version)
                .ok()
                .map(|v| (v, p.signer.as_str()))
        });

        let outcome = evaluate_policy(
            installed_info.as_ref().map(|(v, s)| (v, *s)),
            &inspection.version,
            &inspection.signer,
        );

        Ok(PodPreview {
            id: inspection.id,
            name: inspection.manifest.name,
            version: inspection.version,
            author_name: inspection.manifest.author_name,
            signer: inspection.signer,
            description: inspection.manifest.description,
            slots: inspection.manifest.slots,
            permissions: inspection.manifest.permissions,
            uncompressed_size: inspection.uncompressed_size,
            outcome,
        })
    }

    /// Performs atomic installation or update of a package.
    pub fn install(
        &self,
        package_path: &Path,
        expected_hash: Option<&str>,
        expected_signer: Option<&str>,
    ) -> Result<InstalledPod, PodError> {
        if let Some(expected) = expected_hash {
            let actual = compute_file_sha256(package_path)?;
            if !actual.eq_ignore_ascii_case(expected) {
                return Err(PodError::Validation(vec![PodIssue::HashMismatch]));
            }
        }

        let inspection = self.read_and_inspect(package_path)?;
        if let Some(expected) = expected_signer {
            if !inspection.signer.eq_ignore_ascii_case(expected) {
                return Err(PodError::Validation(vec![PodIssue::SignerMismatch]));
            }
        }

        let existing = self.db.get_pod(&inspection.id)?;
        if let Some(same_version) = check_install_policy(existing.as_ref(), &inspection)? {
            return Ok(same_version);
        }

        let target_dir = self
            .pods_dir
            .join(&inspection.id)
            .join(inspection.version.to_string());
        unpack_archive_atomic(package_path, &target_dir, &self.pods_dir)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let pod_record = build_installed_record(inspection, existing.as_ref(), now);
        self.db.save_pod(&pod_record)?;
        self.db
            .save_pod_trust(&pod_record.id, &pod_record.signer, now)?;

        Ok(pod_record)
    }

    /// Internal helper to inspect package from path.
    fn read_and_inspect(&self, path: &Path) -> Result<PackageInspection, PodError> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        inspect_package(file, metadata.len()).map_err(PodError::Validation)
    }
}

/// Evaluates install policy against the currently installed pod if present.
fn check_install_policy(
    existing: Option<&InstalledPod>,
    inspection: &PackageInspection,
) -> Result<Option<InstalledPod>, PodError> {
    let installed_info = existing.and_then(|p| {
        PodVersion::parse(&p.version)
            .ok()
            .map(|v| (v, p.signer.as_str()))
    });

    let outcome = evaluate_policy(
        installed_info.as_ref().map(|(v, s)| (v, *s)),
        &inspection.version,
        &inspection.signer,
    );

    match outcome {
        PolicyOutcome::SignerChanged => Err(PodError::Validation(vec![PodIssue::SignerChanged])),
        PolicyOutcome::Downgrade => Err(PodError::Validation(vec![PodIssue::BadVersion])),
        PolicyOutcome::SameVersion => Ok(existing.cloned()),
        PolicyOutcome::New | PolicyOutcome::Update => Ok(None),
    }
}

/// Builds an `InstalledPod` record from an inspection and timestamp.
fn build_installed_record(
    inspection: PackageInspection,
    existing: Option<&InstalledPod>,
    now: i64,
) -> InstalledPod {
    InstalledPod {
        id: inspection.id,
        name: inspection.manifest.name,
        version: inspection.version.to_string(),
        api: inspection.manifest.api,
        author_name: inspection.manifest.author_name,
        signer: inspection.signer,
        entry: inspection.manifest.entry,
        description: inspection.manifest.description,
        slots: inspection.manifest.slots,
        permissions: inspection.manifest.permissions,
        abis: inspection.manifest.abis,
        homepage: inspection.manifest.homepage,
        state: PodState::Active,
        installed_at: existing.map_or(now, |p| p.installed_at),
        updated_at: now,
    }
}

/// Computes SHA-256 hex digest of a file.
fn compute_file_sha256(path: &Path) -> Result<String, PodError> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let hash = digest(&SHA256, &buffer);
    let mut hex = String::with_capacity(64);
    for b in hash.as_ref() {
        use std::fmt::Write;
        let _ = write!(hex, "{:02x}", b);
    }
    Ok(hex)
}

/// Atomically unpacks zip contents into `target_dir` using a staging temp directory.
fn unpack_archive_atomic(
    zip_path: &Path,
    target_dir: &Path,
    base_dir: &Path,
) -> Result<(), PodError> {
    let temp_parent = base_dir.join(".staging");
    fs::create_dir_all(&temp_parent)?;
    let temp_dir = tempfile::Builder::new()
        .prefix("pod_extract_")
        .tempdir_in(&temp_parent)?;

    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        if name.ends_with('/') {
            continue;
        }

        let out_path = temp_dir.path().join(&name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut out_file = File::create(&out_path)?;
        std::io::copy(&mut entry, &mut out_file)?;
    }

    // Set classes.dex read-only (security requirement Android 14+)
    let dex_path = temp_dir.path().join("classes.dex");
    if dex_path.exists() {
        let mut perms = fs::metadata(&dex_path)?.permissions();
        perms.set_readonly(true);
        fs::set_permissions(&dex_path, perms)?;
    }

    // Atomic move into final location
    if let Some(parent) = target_dir.parent() {
        fs::create_dir_all(parent)?;
    }
    if target_dir.exists() {
        fs::remove_dir_all(target_dir)?;
    }
    fs::rename(temp_dir.path(), target_dir)?;

    Ok(())
}
