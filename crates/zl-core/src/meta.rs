//! Game metadata, overrides and gamelist exchange.

use std::path::Path;

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

    /// Imports metadata from an ES-DE gamelist.xml file, matching games by ROM filename.
    pub fn import_esde_gamelist(&self, xml_path: &str) -> Result<u32, CoreError> {
        let content = std::fs::read_to_string(xml_path).map_err(|_| CoreError::Io)?;
        let entries = parse_gamelist(&content).map_err(|_| CoreError::InvalidInput)?;
        let games = self.db().query_games(&zl_db::GameQuery::default())?;
        let mut matched = 0;

        for entry in entries {
            let stem = Path::new(&entry.path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            if let Some(game) = games.iter().find(|g| {
                Path::new(&g.path)
                    .file_stem()
                    .is_some_and(|s| s.to_string_lossy().eq_ignore_ascii_case(stem))
            }) {
                let meta = GameMeta {
                    game_id: game.id,
                    description: entry.desc,
                    release_date: entry.release_date,
                    developer: entry.developer,
                    publisher: entry.publisher,
                    genres: entry.genre,
                    players: entry.players.and_then(|p| p.parse().ok()),
                    rating: entry.rating.map(|r| r * 5.0),
                    source: "esde".to_string(),
                    scraped_at: 0,
                };
                self.db().save_game_meta(&meta, false)?;
                matched += 1;
            }
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
