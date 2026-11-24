//! Per-game media: importing the covers, heroes, logos, screenshots and videos that Cocoon and
//! ES-DE keep on disk, choosing the best file per kind, and the user's own choices.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use zl_config::media::{COCOON_MEDIA_FOLDER, ESDE_DEFAULT_ROOT};
use zl_db::{GameQuery, MediaKind, MediaRow, MediaSource, NewMedia};
use zl_media::{esde_media_dir, game_keys, scan_cocoon, scan_esde, FoundMedia};

use crate::error::CoreError;
use crate::theme_store::describe_cocoon_dir;
use crate::Service;
use zl_config::theme::COCOON_DATA_DIR_CANDIDATES;

/// Setting key of Cocoon's data folder.
const KEY_COCOON_DIR: &str = "media.cocoon_dir";
/// Setting key of ES-DE's folder.
const KEY_ESDE_ROOT: &str = "media.esde_root";

/// Where the importer reads media from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaSources {
    /// Cocoon's data folder chosen by the user; `None` means "look at the detected ones".
    pub cocoon_dir: Option<String>,
    /// ES-DE's folder (the one with `settings/`); the default of the configuration when unset.
    pub esde_root: String,
}

/// How many rows an import stored for one source and kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaCount {
    /// Source folder.
    pub source: MediaSource,
    /// Kind of media.
    pub kind: MediaKind,
    /// Files matched to games and stored.
    pub count: u32,
}

/// What a media import did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaImportReport {
    /// Sources that were read (a source whose folder does not exist is not).
    pub scanned: Vec<MediaSource>,
    /// Stored files per source and kind, in order.
    pub counts: Vec<MediaCount>,
    /// Files that match no game of the library.
    pub unmatched: u32,
}

/// One media file of a game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaItem {
    /// What it shows.
    pub kind: MediaKind,
    /// Absolute path.
    pub path: String,
    /// Where it comes from.
    pub source: MediaSource,
    /// The user's choice.
    pub protected: bool,
}

/// The media of a game: the best file of each kind and the full list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameMedia {
    /// Cover.
    pub icon: Option<String>,
    /// Hero image.
    pub hero: Option<String>,
    /// Logo.
    pub logo: Option<String>,
    /// Screenshots, best first.
    pub screenshots: Vec<String>,
    /// Video.
    pub video: Option<String>,
    /// 3D box.
    pub box3d: Option<String>,
    /// Title screen.
    pub titlescreen: Option<String>,
    /// Every file, best first within each kind (for a picker).
    pub items: Vec<MediaItem>,
}

/// Builds the record of a game from its rows (best first within each kind).
fn game_media_of(rows: Vec<MediaRow>) -> GameMedia {
    let first = |kind: MediaKind| rows.iter().find(|r| r.kind == kind).map(|r| r.path.clone());
    GameMedia {
        icon: first(MediaKind::Icon),
        hero: first(MediaKind::Hero),
        logo: first(MediaKind::Logo),
        screenshots: rows
            .iter()
            .filter(|r| r.kind == MediaKind::Screenshot)
            .map(|r| r.path.clone())
            .collect(),
        video: first(MediaKind::Video),
        box3d: first(MediaKind::Box3d),
        titlescreen: first(MediaKind::TitleScreen),
        items: rows
            .into_iter()
            .map(|r| MediaItem {
                kind: r.kind,
                path: r.path,
                source: r.source,
                protected: r.protected,
            })
            .collect(),
    }
}

impl Service {
    /// Where media is read from.
    pub fn media_sources(&self) -> Result<MediaSources, CoreError> {
        let db = self.db();
        let get = |key: &str| db.setting(key).map(|v| v.filter(|t| !t.is_empty()));
        Ok(MediaSources {
            cocoon_dir: get(KEY_COCOON_DIR)?,
            esde_root: get(KEY_ESDE_ROOT)?.unwrap_or_else(|| ESDE_DEFAULT_ROOT.to_string()),
        })
    }

    /// Sets where media is read from; `None` restores the default (detected Cocoon folders,
    /// ES-DE at its default folder).
    pub fn set_media_sources(
        &self,
        cocoon_dir: Option<&str>,
        esde_root: Option<&str>,
    ) -> Result<(), CoreError> {
        let db = self.db();
        db.set_setting(KEY_COCOON_DIR, cocoon_dir.unwrap_or(""))?;
        db.set_setting(KEY_ESDE_ROOT, esde_root.unwrap_or(""))?;
        Ok(())
    }

    /// The Cocoon folder to read media from: the chosen one, else the first detected folder
    /// that has media.
    fn cocoon_media_root(&self, chosen: Option<&str>) -> Option<PathBuf> {
        if let Some(dir) = chosen {
            return Some(PathBuf::from(dir));
        }
        COCOON_DATA_DIR_CANDIDATES
            .iter()
            .filter_map(|c| describe_cocoon_dir(Path::new(c)))
            .find(|d| d.has_media)
            .map(|d| PathBuf::from(d.path))
    }

    /// Game ids by (canonical platform, lower-case key), for matching media files to games.
    fn game_index(&self) -> Result<HashMap<(String, String), Vec<i64>>, CoreError> {
        let games = self.db().query_games(&GameQuery {
            include_missing: true,
            ..GameQuery::default()
        })?;
        let mut index: HashMap<(String, String), Vec<i64>> = HashMap::new();
        for g in games {
            let platform = self.canonical_id(&g.platform_id);
            for key in game_keys(&g.path, &g.title) {
                index.entry((platform.clone(), key)).or_default().push(g.id);
            }
        }
        Ok(index)
    }

    /// Reads the media of Cocoon and ES-DE (the sources of [`Service::media_sources`] that exist
    /// on disk) and stores what matches a game of the library, by the ROM file's stem (ignoring
    /// case) or the game's title. Per kind, the best file is the user's choice, then a Cocoon
    /// edit (the highest number), then a plain Cocoon file, then ES-DE. The user's choices are
    /// never touched; media of a source that was not found on disk is kept as it was.
    pub fn import_media(&self) -> Result<MediaImportReport, CoreError> {
        let sources = self.media_sources()?;
        let mut found: Vec<FoundMedia> = Vec::new();
        let mut scanned = Vec::new();
        if let Some(dir) = self.cocoon_media_root(sources.cocoon_dir.as_deref()) {
            if dir.join(COCOON_MEDIA_FOLDER).is_dir() {
                found.extend(scan_cocoon(&dir));
                scanned.push(MediaSource::Cocoon);
            }
        }
        let esde = Path::new(&sources.esde_root);
        if esde_media_dir(esde).is_dir() {
            found.extend(scan_esde(esde));
            scanned.push(MediaSource::Esde);
        }
        let (items, unmatched) = self.match_media(&found)?;
        self.db().replace_media(&scanned, &items)?;
        Ok(MediaImportReport {
            scanned,
            counts: count_items(&items),
            unmatched,
        })
    }

    /// Ties each found file to the games it belongs to; returns the rows (without duplicates)
    /// and the number of files that matched nothing.
    fn match_media(&self, found: &[FoundMedia]) -> Result<(Vec<NewMedia>, u32), CoreError> {
        let index = self.game_index()?;
        let mut seen: HashSet<(i64, MediaKind, String)> = HashSet::new();
        let mut items = Vec::new();
        let mut unmatched = 0;
        for f in found {
            let platform = self.canonical_id(&f.platform);
            let Some(ids) = index.get(&(platform, f.stem.clone())) else {
                unmatched += 1;
                continue;
            };
            let path = f.path.to_string_lossy().into_owned();
            for &game_id in ids {
                if seen.insert((game_id, f.kind, path.clone())) {
                    items.push(NewMedia {
                        game_id,
                        kind: f.kind,
                        path: path.clone(),
                        source: f.source,
                        edit: f.edit,
                    });
                }
            }
        }
        Ok((items, unmatched))
    }

    /// The media of a game, best file of each kind and the full list.
    pub fn game_media(&self, game_id: i64) -> Result<GameMedia, CoreError> {
        Ok(game_media_of(self.db().game_media(game_id)?))
    }

    /// Sets the user's file for one kind of a game (a protected choice that imports keep), or
    /// with `None` clears it so the imported media shows again.
    pub fn set_game_media(
        &self,
        game_id: i64,
        kind: MediaKind,
        path: Option<&str>,
    ) -> Result<(), CoreError> {
        let db = self.db();
        db.game(game_id)?.ok_or(CoreError::NotFound)?;
        if path.is_some_and(|p| !p.starts_with('/')) {
            return Err(CoreError::InvalidInput);
        }
        db.set_user_media(game_id, kind, path)?;
        Ok(())
    }

    /// The best cover, hero and logo of every game that has one, by game id (one query for the
    /// whole listing).
    pub(crate) fn best_art(&self) -> Result<HashMap<(i64, MediaKind), String>, CoreError> {
        let rows = self
            .db()
            .best_media(&[MediaKind::Icon, MediaKind::Hero, MediaKind::Logo])?;
        Ok(rows
            .into_iter()
            .map(|r| ((r.game_id, r.kind), r.path))
            .collect())
    }
}

/// Counts the rows per source and kind.
fn count_items(items: &[NewMedia]) -> Vec<MediaCount> {
    let mut counts: BTreeMap<(MediaSource, MediaKind), u32> = BTreeMap::new();
    for m in items {
        *counts.entry((m.source, m.kind)).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|((source, kind), count)| MediaCount {
            source,
            kind,
            count,
        })
        .collect()
}
