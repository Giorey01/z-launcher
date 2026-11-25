//! The Pods half of the [`Core`] object: inspection, installation, management, health and store indexes.

use std::path::Path;

use zl_core as core;

use crate::core_object::Core;
use crate::error::CoreError;

/// Result of a core call.
type Res<T> = Result<T, CoreError>;

/// Preview information about an uninstalled Pod package.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
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

impl From<core::PodPreview> for PodPreview {
    fn from(p: core::PodPreview) -> Self {
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
            file_count: p.file_count,
            uncompressed_size: p.uncompressed_size,
        }
    }
}

/// Information about an installed Pod in the system.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
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
    /// Pod state: active, disabled_by_user, disabled_by_crashes, incompatible.
    pub state: String,
    /// Timestamp installed (seconds).
    pub installed_at: i64,
    /// Timestamp last updated (seconds).
    pub updated_at: i64,
}

impl From<core::InstalledPodView> for InstalledPodView {
    fn from(p: core::InstalledPodView) -> Self {
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
            state: p.state,
            installed_at: p.installed_at,
            updated_at: p.updated_at,
        }
    }
}

/// An entry in a remote Pod store index.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
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

impl From<core::IndexEntryView> for IndexEntryView {
    fn from(e: core::IndexEntryView) -> Self {
        Self {
            id: e.id,
            name: e.name,
            version: e.version,
            api: e.api,
            description: e.description,
            url: e.url,
            sha256: e.sha256,
            size: e.size,
            signer: e.signer,
            flag: e.flag,
        }
    }
}

/// Parsed store index view.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
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

impl From<core::IndexStoreView> for IndexStoreView {
    fn from(s: core::IndexStoreView) -> Self {
        Self {
            schema: s.schema,
            updated: s.updated,
            entries: s.entries.into_iter().map(Into::into).collect(),
            issues: s.issues,
        }
    }
}

#[uniffi::export]
impl Core {
    /// Inspects a `.zlpod` package and returns its preview metadata.
    pub fn inspect_pod(&self, package_path: String) -> Res<PodPreview> {
        Ok(self.service().inspect_pod(Path::new(&package_path))?.into())
    }

    /// Installs or updates a `.zlpod` package.
    pub fn install_pod(
        &self,
        package_path: String,
        expected_hash: Option<String>,
        expected_signer: Option<String>,
    ) -> Res<InstalledPodView> {
        Ok(self
            .service()
            .install_pod(
                Path::new(&package_path),
                expected_hash.as_deref(),
                expected_signer.as_deref(),
            )?
            .into())
    }

    /// Lists all installed Pods in the system.
    pub fn list_pods(&self) -> Res<Vec<InstalledPodView>> {
        Ok(self
            .service()
            .list_pods()?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    /// Lists active Pods.
    pub fn list_active_pods(&self) -> Res<Vec<InstalledPodView>> {
        Ok(self
            .service()
            .list_active_pods()?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    /// Enables or disables an installed Pod.
    pub fn set_pod_enabled(&self, id: String, enabled: bool) -> Res<bool> {
        Ok(self.service().set_pod_enabled(&id, enabled)?)
    }

    /// Uninstalls a Pod, removing its files and records.
    pub fn uninstall_pod(&self, id: String) -> Res<bool> {
        Ok(self.service().uninstall_pod(&id)?)
    }

    /// Records a crash for a Pod and auto-disables it if threshold reached.
    pub fn record_pod_crash(&self, id: String, reason: Option<String>) -> Res<bool> {
        Ok(self.service().record_pod_crash(&id, reason.as_deref())?)
    }

    /// Resets crash history and reactivates a disabled Pod.
    pub fn reset_pod_health(&self, id: String) -> Res<()> {
        Ok(self.service().reset_pod_health(&id)?)
    }

    /// Parses remote store index JSON bytes.
    pub fn parse_pod_index(&self, index_bytes: Vec<u8>) -> Res<IndexStoreView> {
        Ok(self.service().parse_pod_index(&index_bytes)?.into())
    }

    /// Checks whether external Pods are enabled.
    pub fn external_pods_enabled(&self) -> Res<bool> {
        Ok(self.service().external_pods_enabled()?)
    }

    /// Sets whether external Pods are enabled.
    pub fn set_external_pods_enabled(&self, enabled: bool) -> Res<()> {
        Ok(self.service().set_external_pods_enabled(enabled)?)
    }
}
