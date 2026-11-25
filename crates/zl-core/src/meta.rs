//! Game metadata, overrides and gamelist exchange.

use std::path::Path;

use zl_config::media::{ESDE_GAMELISTS_FOLDER, ESDE_GAMELIST_FILE};
use zl_db::{GameMeta, MetaField, ScrapeMode, ScraperSource};
use zl_media::display_title;
use zl_media::gamelist::{export_gamelist, parse_gamelist, GamelistEntry};

use crate::error::CoreError;
use crate::types::{GameDetails, ScraperConfig};
use crate::Service;

/// Setting key for scraper sources priority list.
const KEY_SCRAPER_SOURCES: &str = "scraper.sources_order";
/// Setting key for scraper mode.
const KEY_SCRAPER_MODE: &str = "scraper.mode";
/// Setting key for overwrite protected flag.
const KEY_SCRAPER_OVERWRITE: &str = "scraper.overwrite_protected";
/// Setting key for SteamGridDB key.
const KEY_SGDB_KEY: &str = "scraper.steamgriddb_key";

impl Service {
    /// Loads complete game details including metadata and best media.
    pub fn game_details(&self, game_id: i64) -> Result<GameDetails, CoreError> {
        let db = self.db();
        let game = db.game(game_id)?.ok_or(CoreError::NotFound)?;
        let mut meta = db.game_meta(game_id)?.unwrap_or_default();
        let overrides = db.meta_overrides(game_id)?;
        drop(db);

        apply_overrides(&mut meta, &overrides);
        let media = self.game_media(game_id)?;
        let plat_id = self.canonical_id(&game.platform_id);
        let plat_name = self.platform_name(&plat_id);

        let title_override = overrides.iter().find(|o| o.field == MetaField::Title);
        let disp_title = match title_override {
            Some(ov) => ov.value.clone(),
            None => display_title(&game.title),
        };

        Ok(build_game_details(
            game, plat_id, plat_name, disp_title, meta, media,
        ))
    }

    /// Sets a user override for a metadata field.
    pub fn set_game_override(
        &self,
        game_id: i64,
        field: MetaField,
        value: &str,
    ) -> Result<(), CoreError> {
        self.db().set_meta_override(game_id, field, value)?;
        Ok(())
    }

    /// Clears a user override for a metadata field.
    pub fn clear_game_override(&self, game_id: i64, field: MetaField) -> Result<(), CoreError> {
        self.db().clear_meta_override(game_id, field)?;
        Ok(())
    }

    /// Reads scraper configuration settings.
    pub fn scraper_config(&self) -> Result<ScraperConfig, CoreError> {
        let db = self.db();
        let get = |k: &str| db.setting(k).map(|v| v.filter(|s| !s.is_empty()));
        let sources = get(KEY_SCRAPER_SOURCES)?
            .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
            .map(|list| {
                list.iter()
                    .filter_map(|s| ScraperSource::parse(s).ok())
                    .collect()
            })
            .unwrap_or_else(zl_meta::default_source_priority);
        let mode = get(KEY_SCRAPER_MODE)?
            .and_then(|s| ScrapeMode::parse(&s).ok())
            .unwrap_or(ScrapeMode::MissingOnly);
        let overwrite = get(KEY_SCRAPER_OVERWRITE)?.is_some_and(|s| s == "true" || s == "1");

        Ok(ScraperConfig {
            sources_order: sources,
            mode,
            overwrite_protected: overwrite,
            steamgriddb_key: get(KEY_SGDB_KEY)?,
        })
    }

    /// Saves scraper configuration settings.
    pub fn set_scraper_config(&self, config: &ScraperConfig) -> Result<(), CoreError> {
        let db = self.db();
        let sources_json = serde_json::to_string(
            &config
                .sources_order
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
        )
        .map_err(|_| CoreError::Database)?;
        db.set_setting(KEY_SCRAPER_SOURCES, &sources_json)?;
        db.set_setting(KEY_SCRAPER_MODE, config.mode.as_str())?;
        db.set_setting(
            KEY_SCRAPER_OVERWRITE,
            if config.overwrite_protected {
                "true"
            } else {
                "false"
            },
        )?;
        db.set_setting(
            KEY_SGDB_KEY,
            config.steamgriddb_key.as_deref().unwrap_or(""),
        )?;
        Ok(())
    }

    /// Imports metadata from one ES-DE `gamelist.xml`, matching games of any platform by ROM
    /// file name. Returns how many games got metadata.
    pub fn import_esde_gamelist(&self, xml_path: &str) -> Result<u32, CoreError> {
        self.import_gamelist_file(Path::new(xml_path), None)
    }

    /// Imports every `gamelists/<system>/gamelist.xml` of the ES-DE folder, each one matched
    /// only against the games of the platform folder of the same name. Returns how many games
    /// got metadata; a missing folder is zero.
    pub fn import_esde_gamelists(&self) -> Result<u32, CoreError> {
        let root = Path::new(&self.media_sources()?.esde_root).join(ESDE_GAMELISTS_FOLDER);
        let Ok(dirs) = std::fs::read_dir(&root) else {
            return Ok(0);
        };
        let mut matched = 0;
        for dir in dirs.flatten() {
            let file = dir.path().join(ESDE_GAMELIST_FILE);
            let system = dir.file_name().to_string_lossy().to_lowercase();
            if file.is_file() {
                matched += self.import_gamelist_file(&file, Some(&system))?;
            }
        }
        Ok(matched)
    }

    /// Reads one gamelist and stores the metadata of the games it names (only those of
    /// `platform`, the folder name as stored, when given).
    fn import_gamelist_file(&self, file: &Path, platform: Option<&str>) -> Result<u32, CoreError> {
        let content = std::fs::read_to_string(file).map_err(|_| CoreError::Io)?;
        let entries = parse_gamelist(&content).map_err(|_| CoreError::InvalidInput)?;
        let games: Vec<zl_db::Game> = self
            .db()
            .query_games(&zl_db::GameQuery::default())?
            .into_iter()
            .filter(|g| platform.is_none_or(|p| g.platform_id.eq_ignore_ascii_case(p)))
            .collect();
        let mut matched = 0;
        for entry in entries {
            let Some(game) = games.iter().find(|g| same_stem(&g.path, &entry.path)) else {
                continue;
            };
            self.db().save_game_meta(&meta_of(game.id, entry), false)?;
            matched += 1;
        }
        Ok(matched)
    }

    /// Exports library games of a platform to an ES-DE gamelist.xml file.
    pub fn export_esde_gamelist(&self, platform_id: &str, out_path: &str) -> Result<(), CoreError> {
        let games = self.db().list_games_by_platform(platform_id)?;
        let mut entries = Vec::new();
        for g in games {
            let meta = self.db().game_meta(g.id)?.unwrap_or_default();
            entries.push(GamelistEntry {
                path: g.path,
                name: Some(g.title),
                desc: meta.description,
                image: None,
                marquee: None,
                video: None,
                release_date: meta.release_date,
                developer: meta.developer,
                publisher: meta.publisher,
                genre: meta.genres,
                players: meta.players.map(|p| p.to_string()),
                rating: meta.rating.map(|r| r / 5.0),
            });
        }
        let xml = export_gamelist(&entries).map_err(|_| CoreError::Io)?;
        std::fs::write(out_path, xml).map_err(|_| CoreError::Io)?;
        Ok(())
    }
}

/// Helper building [`GameDetails`] from records.
fn build_game_details(
    game: zl_db::Game,
    plat_id: String,
    plat_name: String,
    display_title: String,
    meta: GameMeta,
    media: crate::media::GameMedia,
) -> GameDetails {
    GameDetails {
        id: game.id,
        platform_id: plat_id,
        platform_name: plat_name,
        title: game.title,
        display_title,
        description: meta.description,
        release_date: meta.release_date,
        developer: meta.developer,
        publisher: meta.publisher,
        genres: meta.genres,
        players: meta.players,
        rating: meta.rating,
        icon: media.icon,
        hero: media.hero,
        logo: media.logo,
        screenshots: media.screenshots,
        video: media.video,
    }
}

/// Applies user overrides to stored metadata.
fn apply_overrides(meta: &mut GameMeta, overrides: &[zl_db::MetaOverride]) {
    for ov in overrides {
        match ov.field {
            MetaField::Title => {}
            MetaField::Description => meta.description = Some(ov.value.clone()),
            MetaField::ReleaseDate => meta.release_date = Some(ov.value.clone()),
            MetaField::Developer => meta.developer = Some(ov.value.clone()),
            MetaField::Publisher => meta.publisher = Some(ov.value.clone()),
            MetaField::Genres => meta.genres = Some(ov.value.clone()),
            MetaField::Players => meta.players = ov.value.parse().ok(),
            MetaField::Rating => meta.rating = ov.value.parse().ok(),
        }
    }
}

/// Whether two paths name the same file, ignoring folders, extension and case.
fn same_stem(a: &str, b: &str) -> bool {
    let stem = |p: &str| {
        Path::new(p)
            .file_stem()
            .map(|s| s.to_string_lossy().to_lowercase())
    };
    stem(a).is_some_and(|s| Some(s) == stem(b))
}

/// The stored metadata of a gamelist entry. ES-DE dates (`20110919T000000`) become `2011-09-19`
/// and its 0-1 rating becomes 0-5.
fn meta_of(game_id: i64, entry: GamelistEntry) -> GameMeta {
    GameMeta {
        game_id,
        description: entry.desc,
        release_date: entry.release_date.map(|d| esde_date(&d)),
        developer: entry.developer,
        publisher: entry.publisher,
        genres: entry.genre,
        players: entry.players.and_then(|p| p.trim().parse().ok()),
        rating: entry.rating.map(|r| r * 5.0),
        source: "esde".to_string(),
        scraped_at: 0,
    }
}

/// `YYYYMMDDT...` as `YYYY-MM-DD`; anything else is kept as written.
fn esde_date(text: &str) -> String {
    let digits: String = text.chars().take(8).collect();
    if digits.len() == 8 && digits.chars().all(|c| c.is_ascii_digit()) {
        format!("{}-{}-{}", &digits[..4], &digits[4..6], &digits[6..8])
    } else {
        text.to_string()
    }
}
