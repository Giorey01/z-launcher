//! Pod inspection, installation, management, health, and store indexing.

use std::fs::File;
use std::path::{Path, PathBuf};

use zl_config::pods::PODS_DIR_NAME;
use zl_db::{InstalledPod, PodState};
use zl_pods::{
    inspect_package, parse_index, HealthMonitor, IndexEntry, PodInstaller, PodRegistry, StoreIndex,
};

use crate::error::CoreError;
use crate::Service;

/// Preview information about an uninstalled Pod package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodPreview {
    /// Pod unique identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Semantic version.
    pub version: String,
    /// Target API version.
    pub api: u32,
    /// Author display name.
    pub author_name: String,
    /// Signer key fingerprint.
    pub signer: String,
    /// Entry class name.
    pub entry: String,
    /// Optional description.
    pub description: Option<String>,
    /// Declared UI slots.
    pub slots: Vec<String>,
    /// Declared permissions.
    pub permissions: Vec<String>,
    /// Declared ABI targets.
    pub abis: Option<Vec<String>>,
    /// Optional homepage URL.
    pub homepage: Option<String>,
    /// File count in package.
    pub file_count: u32,
    /// Total uncompressed size in bytes.
    pub uncompressed_size: u64,
}

/// Information about an installed Pod in the system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledPodView {
    /// Pod unique identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Semantic version.
    pub version: String,
    /// Target API version.
    pub api: u32,
    /// Author display name.
    pub author_name: String,
    /// Signer key fingerprint.
    pub signer: String,
    /// Entry class name.
    pub entry: String,
    /// Optional description.
    pub description: Option<String>,
    /// Declared UI slots.
    pub slots: Vec<String>,
    /// Declared permissions.
    pub permissions: Vec<String>,
    /// Declared ABI targets.
    pub abis: Option<Vec<String>>,
    /// Optional homepage URL.
    pub homepage: Option<String>,
    /// Pod state: active, disabled, disabled_by_crashes, incompatible.
    pub state: String,
    /// Timestamp installed (seconds).
    pub installed_at: i64,
    /// Timestamp last updated (seconds).
    pub updated_at: i64,
}

impl From<InstalledPod> for InstalledPodView {
    fn from(p: InstalledPod) -> Self {
        let state_str = match p.state {
            PodState::Active => "active",
            PodState::DisabledByUser => "disabled_by_user",
            PodState::DisabledByCrashes => "disabled_by_crashes",
            PodState::Incompatible => "incompatible",
        };
        Self {
            id: p.id,
            name: p.name,
            version: p.version,
            api: p.api,
            author_name: p.author_name,
            signer: p.signer,
            entry: p.entry,
            description: p.description,
            slots: p.slots,
            permissions: p.permissions,
            abis: p.abis,
            homepage: p.homepage,
            state: state_str.to_string(),
            installed_at: p.installed_at,
            updated_at: p.updated_at,
        }
    }
}

/// An entry in a remote Pod store index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexEntryView {
    /// Pod unique identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Semantic version.
    pub version: String,
    /// Target API version.
    pub api: u32,
    /// Optional description.
    pub description: Option<String>,
    /// Download URL.
    pub url: String,
    /// SHA256 hex string.
    pub sha256: String,
    /// Size in bytes.
    pub size: u64,
    /// Signer key fingerprint.
    pub signer: String,
    /// Compatibility flag code, if any.
    pub flag: Option<String>,
}

impl From<IndexEntry> for IndexEntryView {
    fn from(e: IndexEntry) -> Self {
        Self {
            id: e.id,
            name: e.name,
            version: e.version.to_string(),
            api: e.api,
            description: e.description,
            url: e.url,
            sha256: e.sha256,
            size: e.size,
            signer: e.signer,
            flag: e.flag.map(|f| f.as_code().to_string()),
        }
    }
}

/// Parsed store index view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexStoreView {
    /// Index schema version.
    pub schema: u32,
    /// Updated timestamp string.
    pub updated: String,
    /// Valid kept entries.
    pub entries: Vec<IndexEntryView>,
    /// Issues encountered with ignored entries.
    pub issues: Vec<String>,
}

impl From<StoreIndex> for IndexStoreView {
    fn from(s: StoreIndex) -> Self {
        Self {
            schema: s.schema,
            updated: s.updated,
            entries: s.entries.into_iter().map(Into::into).collect(),
            issues: s
                .issues
                .into_iter()
                .map(|i| {
                    if let Some(entry) = i.entry {
                        format!("{}: {}", entry, i.issue.as_code())
                    } else {
                        i.issue.as_code().to_string()
                    }
                })
                .collect(),
        }
    }
}

impl Service {
    /// Path to the folder where installed Pod packages reside.
    pub fn pods_dir(&self) -> PathBuf {
        self.data_dir.join(PODS_DIR_NAME)
    }

    /// Inspects a `.zlpod` package and returns its preview metadata.
    pub fn inspect_pod(&self, package_path: &Path) -> Result<PodPreview, CoreError> {
        let file = File::open(package_path).map_err(|_| CoreError::Io)?;
        let metadata = file.metadata().map_err(|_| CoreError::Io)?;
        let insp = inspect_package(file, metadata.len()).map_err(|issues| CoreError::Pod {
            issues: issues
                .into_iter()
                .map(|i| i.as_code().to_string())
                .collect(),
        })?;

        Ok(PodPreview {
            id: insp.id,
            name: insp.manifest.name,
            version: insp.version.to_string(),
            api: insp.manifest.api,
            author_name: insp.manifest.author_name,
            signer: insp.signer,
            entry: insp.manifest.entry,
            description: insp.manifest.description,
            slots: insp.manifest.slots,
            permissions: insp.manifest.permissions,
            abis: insp.manifest.abis,
            homepage: insp.manifest.homepage,
            file_count: insp.file_count as u32,
            uncompressed_size: insp.uncompressed_size,
        })
    }

    /// Installs or updates a `.zlpod` package.
    pub fn install_pod(
        &self,
        package_path: &Path,
        expected_hash: Option<&str>,
        expected_signer: Option<&str>,
    ) -> Result<InstalledPodView, CoreError> {
        let db = self.db();
        let installer = PodInstaller::new(&db, self.pods_dir());
        let installed = installer.install(package_path, expected_hash, expected_signer)?;
        Ok(installed.into())
    }

    /// Lists all installed Pods in the system.
    pub fn list_pods(&self) -> Result<Vec<InstalledPodView>, CoreError> {
        let db = self.db();
        let pods_dir = self.pods_dir();
        let registry = PodRegistry::new(&db, &pods_dir);
        let list = registry.list_installed()?;
        Ok(list.into_iter().map(Into::into).collect())
    }

    /// Lists only active Pods (empty when external_pods_enabled is false per §6).
    pub fn list_active_pods(&self) -> Result<Vec<InstalledPodView>, CoreError> {
        if !self.external_pods_enabled()? {
            return Ok(Vec::new());
        }
        let db = self.db();
        let pods_dir = self.pods_dir();
        let registry = PodRegistry::new(&db, &pods_dir);
        let list = registry.list_active()?;
        Ok(list.into_iter().map(Into::into).collect())
    }

    /// Enables or disables an installed Pod.
    pub fn set_pod_enabled(&self, id: &str, enabled: bool) -> Result<bool, CoreError> {
        let db = self.db();
        let pods_dir = self.pods_dir();
        let registry = PodRegistry::new(&db, &pods_dir);
        Ok(registry.set_enabled(id, enabled)?)
    }

    /// Uninstalls a Pod, removing its files and records.
    pub fn uninstall_pod(&self, id: &str) -> Result<bool, CoreError> {
        let db = self.db();
        let pods_dir = self.pods_dir();
        let registry = PodRegistry::new(&db, &pods_dir);
        Ok(registry.uninstall(id)?)
    }

    /// Records a crash for a Pod and auto-disables it if threshold reached.
    pub fn record_pod_crash(&self, id: &str, reason: Option<&str>) -> Result<bool, CoreError> {
        let db = self.db();
        let monitor = HealthMonitor::new(&db);
        Ok(monitor.record_crash(id, reason)?)
    }

    /// Resets crash history and reactivates a disabled Pod.
    pub fn reset_pod_health(&self, id: &str) -> Result<(), CoreError> {
        let db = self.db();
        let monitor = HealthMonitor::new(&db);
        Ok(monitor.reset_health(id)?)
    }

    /// Parses remote store index JSON bytes.
    pub fn parse_pod_index(&self, index_bytes: &[u8]) -> Result<IndexStoreView, CoreError> {
        let parsed = parse_index(index_bytes).map_err(|i| CoreError::Pod {
            issues: vec![i.issue.as_code().to_string()],
        })?;
        Ok(parsed.into())
    }
}
