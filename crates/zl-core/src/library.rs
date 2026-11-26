//! Roots, scanning, and the platforms and games the Home shows.

use std::collections::BTreeMap;
use std::path::PathBuf;

use zl_config::platforms::FOLDER_GAME_RULES;
use zl_db::{Db, Game, GameKind, GameQuery, MediaKind, Root};
use zl_library::{scan, ExtensionRules, FolderGameRule, RootSpec, ScanProgress, ScanSummary};
use zl_media::display_title;
use zl_platforms::ANDROID_PACKAGE_TAG;

use crate::error::CoreError;
use crate::types::{FolderKind, GameEntry, PlatformEntry, RootInfo};
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
    /// named `sfc` is read as `snes`), `.localgameid` everywhere, and the folder-per-game rules.
    fn extension_rules(&self) -> ExtensionRules {
        let mut rules = ExtensionRules::new();
        for p in self.platforms.platforms() {
            let mut exts: Vec<&str> = p.extensions.iter().map(String::as_str).collect();
            // Every console folder takes `.localgameid` files: an Android app (a recomp or a
            // port) filed under that console.
            if !exts.contains(&ANDROID_PACKAGE_TAG) {
                exts.push(ANDROID_PACKAGE_TAG);
            }
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

    /// Id of the Home folder a canonical platform shows in: its group folder, else itself.
    pub(crate) fn home_folder(&self, canonical: &str) -> String {
        self.platforms
            .folder_of(canonical)
            .map_or_else(|| canonical.to_string(), |f| f.id.clone())
    }

    /// English name of a Home folder id: the group folder's name, else the platform's.
    fn home_folder_name(&self, id: &str) -> String {
        self.platforms
            .folder(id)
            .map_or_else(|| self.platform_name(id), |f| f.name.clone())
    }

    /// Canonical id of a stored platform id (aliases such as `sfc` map to `snes`).
    pub(crate) fn canonical_id(&self, id: &str) -> String {
        self.platforms
            .platform(id)
            .map_or_else(|| id.to_string(), |p| p.id.clone())
    }

    /// The folders of the Home: the Smart Folders that have games (Favourites, Recent, Never
    /// played, once something was played) first, then one folder per platform with its count, sorted by name, then Hidden
    /// when some game is hidden. Folders that are aliases of the same platform are merged, and
    /// so are the platforms of a group folder (Steam and Windows under "PC"). Hidden games count
    /// nowhere but in Hidden.
    pub fn platforms(&self) -> Result<Vec<PlatformEntry>, CoreError> {
        let visible = self.games(None)?;
        let mut merged: BTreeMap<String, u32> = BTreeMap::new();
        for g in &visible {
            *merged.entry(self.home_folder(&g.platform_id)).or_default() += 1;
        }
        let mut out: Vec<PlatformEntry> = merged
            .into_iter()
            .map(|(id, games)| PlatformEntry {
                name: self.home_folder_name(&id),
                id,
                games,
                kind: FolderKind::Platform,
            })
            .collect();
        out.sort_by_key(|p| p.name.to_lowercase());
        let mut folders = self.smart_entries(&visible)?;
        folders.extend(out);
        folders.extend(self.hidden_entry()?);
        Ok(folders)
    }

    /// Whether a game of canonical platform `canonical` belongs to `wanted`: a group folder id
    /// takes its member platforms, any other id only its own platform.
    fn game_in(&self, wanted: &str, canonical: &str) -> bool {
        match self.platforms.folder(wanted) {
            Some(folder) => folder.platforms.iter().any(|p| p == canonical),
            None => self.canonical_id(wanted) == canonical,
        }
    }

    /// The package of a tag-file game that is an Android app filed under its console.
    fn app_package_of(&self, db: &Db, game: &Game, canonical: &str) -> Option<String> {
        if game.kind != GameKind::TagFile {
            return None;
        }
        let tags = db.game_tags(game.id).ok()?;
        let (_, value) = tags.into_iter().find(|(k, _)| k == ANDROID_PACKAGE_TAG)?;
        self.platforms
            .is_android_app(canonical, &value)
            .then_some(value)
    }

    /// The games of one platform or group folder (any id or alias), or of every platform with
    /// `None`, by title, hidden ones included. Missing games are left out.
    pub(crate) fn listed_games(&self, platform: Option<&str>) -> Result<Vec<GameEntry>, CoreError> {
        let wanted = platform.map(str::to_lowercase);
        let art = self.best_art()?;
        let db = self.db();
        let title_overrides = db
            .all_meta_overrides(zl_db::MetaField::Title)
            .unwrap_or_default();
        let games = db.query_games(&GameQuery::default())?;
        let mut out: Vec<GameEntry> = games
            .into_iter()
            .map(|g| {
                let canonical = self.canonical_id(&g.platform_id);
                (g, canonical)
            })
            .filter(|(_, c)| wanted.as_ref().is_none_or(|w| self.game_in(w, c)))
            .map(|(g, canonical)| {
                let disp_title = title_overrides
                    .get(&g.id)
                    .cloned()
                    .unwrap_or_else(|| display_title(&g.title));
                GameEntry {
                    id: g.id,
                    display_title: disp_title,
                    icon: art.get(&(g.id, MediaKind::Icon)).cloned(),
                    hero: art.get(&(g.id, MediaKind::Hero)).cloned(),
                    logo: art.get(&(g.id, MediaKind::Logo)).cloned(),
                    app_package: self.app_package_of(&db, &g, &canonical),
                    favorite: false,
                    title: g.title,
                    platform_name: self.platform_name(&canonical),
                    platform_id: canonical,
                    kind: g.kind,
                    missing: g.missing,
                }
            })
            .collect();
        out.sort_by_key(|g| g.title.to_lowercase());
        Ok(out)
    }
}
