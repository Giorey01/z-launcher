//! Roots, scanning, and the platforms and games the Home shows.

use std::collections::BTreeMap;
use std::path::PathBuf;

use zl_config::platforms::FOLDER_GAME_RULES;
use zl_db::{Db, GameQuery, Root};
use zl_library::{scan, ExtensionRules, FolderGameRule, RootSpec, ScanProgress, ScanSummary};

use crate::error::CoreError;
use crate::types::{GameEntry, PlatformEntry, RootInfo};
use crate::Service;

/// Converts a stored root.
fn root_info(root: Root) -> RootInfo {
    RootInfo {
        id: root.id,
        path: root.path,
        saf_tree: root.saf_tree,
        enabled: root.enabled,
    }
}

impl Service {
    /// Adds a ROM folder, with the SAF tree the user granted for it (if any), or updates the
    /// tree of a folder already added. Returns the stored root.
    pub fn add_root(&self, path: &str, saf_tree: Option<&str>) -> Result<RootInfo, CoreError> {
        let path = path.trim().trim_end_matches('/');
        if path.is_empty() || !path.starts_with('/') {
            return Err(CoreError::InvalidInput);
        }
        let db = self.db();
        let id = db.upsert_root_with_saf(path, true, saf_tree)?;
        db.root(id)?.map(root_info).ok_or(CoreError::NotFound)
    }

    /// The folders added so far, in insertion order.
    pub fn roots(&self) -> Result<Vec<RootInfo>, CoreError> {
        Ok(self.db().list_roots()?.into_iter().map(root_info).collect())
    }

    /// Includes or excludes a folder from scans without losing its games.
    pub fn set_root_enabled(&self, root_id: i64, enabled: bool) -> Result<(), CoreError> {
        self.db().set_root_enabled(root_id, enabled)?;
        Ok(())
    }

    /// The scanner rules: the extensions of every platform (also under each alias, so a folder
    /// named `sfc` is read as `snes`) and the folder-per-game rules.
    fn extension_rules(&self) -> ExtensionRules {
        let mut rules = ExtensionRules::new();
        for p in self.platforms.platforms() {
            let exts: Vec<&str> = p.extensions.iter().map(String::as_str).collect();
            for name in std::iter::once(&p.id).chain(&p.aliases) {
                rules.insert(name, &exts);
            }
        }
        for (platform, glob) in FOLDER_GAME_RULES {
            rules.add_folder_rule(FolderGameRule {
                platform: platform.to_string(),
                launch_glob: glob.to_string(),
            });
        }
        rules
    }

    /// Scans the enabled folders and updates the library. Runs on its own connection so reads
    /// keep working; events go to `progress`. Games no longer found are flagged, not deleted.
    pub fn scan(&self, progress: &dyn ScanProgress) -> Result<ScanSummary, CoreError> {
        let roots: Vec<RootSpec> = self
            .roots()?
            .into_iter()
            .filter(|r| r.enabled)
            .map(|r| RootSpec {
                path: PathBuf::from(r.path),
                scan_subfolders: true,
            })
            .collect();
        let db = Db::open(&self.db_path)?;
        Ok(scan(&db, &roots, &self.extension_rules(), progress)?)
    }

    /// English name of a platform id or alias; the id itself when unknown.
    pub(crate) fn platform_name(&self, id: &str) -> String {
        self.platforms
            .platform(id)
            .map_or_else(|| id.to_string(), |p| p.name.clone())
    }

    /// Canonical id of a stored platform id (aliases such as `sfc` map to `snes`).
    pub(crate) fn canonical_id(&self, id: &str) -> String {
        self.platforms
            .platform(id)
            .map_or_else(|| id.to_string(), |p| p.id.clone())
    }

    /// Platforms that have games, with their counts, sorted by name. Folders that are aliases of
    /// the same platform are merged.
    pub fn platforms(&self) -> Result<Vec<PlatformEntry>, CoreError> {
        let mut merged: BTreeMap<String, u32> = BTreeMap::new();
        for c in self.db().platform_counts()? {
            let games = u32::try_from(c.games).unwrap_or(u32::MAX);
            *merged.entry(self.canonical_id(&c.platform_id)).or_default() += games;
        }
        let mut out: Vec<PlatformEntry> = merged
            .into_iter()
            .map(|(id, games)| PlatformEntry {
                name: self.platform_name(&id),
                id,
                games,
            })
            .collect();
        out.sort_by_key(|p| p.name.to_lowercase());
        Ok(out)
    }

    /// The games of one platform (any id or alias), or of every platform with `None`, by title.
    /// Missing games are left out.
    pub fn games(&self, platform: Option<&str>) -> Result<Vec<GameEntry>, CoreError> {
        let wanted = platform.map(|p| self.canonical_id(p));
        let db = self.db();
        let games = db.query_games(&GameQuery::default())?;
        let mut out: Vec<GameEntry> = games
            .into_iter()
            .map(|g| {
                let canonical = self.canonical_id(&g.platform_id);
                (g, canonical)
            })
            .filter(|(_, c)| wanted.as_ref().is_none_or(|w| w == c))
            .map(|(g, canonical)| GameEntry {
                id: g.id,
                title: g.title,
                platform_name: self.platform_name(&canonical),
                platform_id: canonical,
                kind: g.kind,
                missing: g.missing,
            })
            .collect();
        out.sort_by_key(|g| g.title.to_lowercase());
        Ok(out)
    }
}
