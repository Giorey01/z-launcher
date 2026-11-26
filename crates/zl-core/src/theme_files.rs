//! Loading the files of the selected theme and layer packs into one precedence-ordered set.
//!
//! Done once per selection (the result is cached in the service) so asking for the art of a
//! thousand tiles does not touch the disk.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use zl_config::theme::{DEFAULT_MUSIC_DIR_NAME, USER_ART_DIR_NAME};
use zl_theme::{
    list_installed, read_package, read_theme_layers, resolve_music_file, resolve_slot,
    slots_of_package, AssetCategory, AssetPack, ColorScheme, InstalledContent, LayerSet,
    LayerSource, MusicConfig, MusicMode, PlaybackMode, SoundAction, SourceLevel, ThemeMode,
    ThemePackage,
};

use crate::appearance_types::{MusicSetup, ScheduleTrack};
use crate::Service;

/// Setting key of the selected theme (absent or empty: the built-in one).
pub(crate) const KEY_ACTIVE_THEME: &str = "theme.active";

/// Setting key of the explicit pack of a layer.
pub(crate) fn pack_key(category: AssetCategory) -> String {
    format!("theme.pack.{}", category.as_str())
}

/// The four layers, in the order they are listed to the user.
pub(crate) const LAYERS: [AssetCategory; 4] = [
    AssetCategory::IconOverlays,
    AssetCategory::SmartFolders,
    AssetCategory::Sounds,
    AssetCategory::Music,
];

/// A set of files by slot with the level it has in the precedence.
pub(crate) struct Layer {
    /// Precedence level.
    pub level: SourceLevel,
    /// Files by slot key (platform ids already canonical).
    pub files: BTreeMap<String, PathBuf>,
}

/// Everything read from disk for the current selection.
pub(crate) struct ThemeFiles {
    /// Folder name of the theme, `None` for the built-in one.
    pub id: Option<String>,
    /// Display name.
    pub name: String,
    /// A selected theme could not be used.
    pub fell_back: bool,
    /// Declared mode (dark for the built-in theme).
    pub mode: ThemeMode,
    /// Colours of the theme.
    pub scheme: Option<ColorScheme>,
    /// Main wallpaper file.
    pub wallpaper_main: Option<PathBuf>,
    /// External wallpaper file.
    pub wallpaper_external: Option<PathBuf>,
    /// Sources in precedence order (earlier wins a tie of the same level).
    pub layers: Vec<Layer>,
    /// Music content (the mode is the user's setting, filled in later).
    pub music: MusicSetup,
    /// The mode the theme or the music pack declares, used when applying settings.
    pub declared_music_mode: Option<MusicMode>,
}

impl ThemeFiles {
    /// The file that serves `slot`, through the layer precedence.
    pub fn resolve(&self, slot: &str) -> Option<PathBuf> {
        let sources: Vec<LayerSource<'_>> = self
            .layers
            .iter()
            .map(|l| LayerSource {
                level: l.level,
                files: &l.files,
            })
            .collect();
        resolve_slot(&sources, slot).path
    }

    /// The sound file of an action.
    pub fn sound(&self, action: SoundAction) -> Option<PathBuf> {
        self.resolve(&format!("sound/{}", action.as_str()))
    }
}

/// Wraps a layer set into a package so `slots_of_package` can flatten it.
fn package_of(layers: LayerSet) -> ThemePackage {
    ThemePackage {
        icon_overlays: layers.icon_overlays,
        icon_overlay_global: layers.icon_overlay_global,
        smart_folders: layers.smart_folders,
        smart_folders_by_platform: layers.smart_folders_by_platform,
        sounds: layers.sounds,
        ..ThemePackage::default()
    }
}

/// Index of the platform id inside a slot key, if it has one.
fn platform_segment(key: &str) -> Option<(&str, &str, &str)> {
    let mut parts = key.splitn(3, '/');
    let (prefix, id, rest) = (parts.next()?, parts.next()?, parts.next()?);
    matches!(prefix, "icon_overlay" | "smart_folder_platform").then_some((prefix, id, rest))
}

/// Rewrites the platform ids of a slot map to canonical ids: a folder named `megadrive` serves
/// `genesis`. A folder that already has the canonical name wins over an alias.
fn canonical_slots(
    slots: BTreeMap<String, PathBuf>,
    canonical: &dyn Fn(&str) -> String,
) -> BTreeMap<String, PathBuf> {
    let mut out = BTreeMap::new();
    let mut aliased = Vec::new();
    for (key, path) in slots {
        match platform_segment(&key) {
            Some((prefix, id, rest)) if canonical(id) != id => {
                aliased.push((format!("{prefix}/{}/{rest}", canonical(id)), path));
            }
            _ => {
                out.insert(key, path);
            }
        }
    }
    for (key, path) in aliased {
        out.entry(key).or_insert(path);
    }
    out
}

/// The pack of `category` with this id.
fn find_pack<'a>(
    content: &'a InstalledContent,
    category: AssetCategory,
    id: &str,
) -> Option<&'a AssetPack> {
    content
        .packs
        .iter()
        .find(|p| p.category == category && p.id == id)
}

/// The music of a candidate source: its config and a function finding a file for an entry.
fn music_from(config: &MusicConfig, find: &dyn Fn(&str) -> Option<PathBuf>) -> Option<MusicSetup> {
    let has = config.playlist.as_ref().is_some_and(|p| !p.is_empty())
        || config.schedule.as_ref().is_some_and(|s| !s.is_empty());
    if !has {
        return None;
    }
    let path_text = |p: PathBuf| p.to_string_lossy().into_owned();
    Some(MusicSetup {
        mode: MusicMode::Disabled,
        playback_mode: config.playback_mode.unwrap_or(PlaybackMode::InOrder),
        playlist: config
            .playlist
            .iter()
            .flatten()
            .filter_map(|e| find(e))
            .map(path_text)
            .collect(),
        schedule: config
            .schedule
            .iter()
            .flatten()
            .filter_map(|e| {
                find(&e.file).map(|p| ScheduleTrack {
                    hour: e.hour,
                    minute: e.minute,
                    path: path_text(p),
                })
            })
            .collect(),
    })
}

/// Finds a track among the music files of a theme folder, by file name and ignoring case.
fn find_in_theme(pkg: &ThemePackage, entry: &str) -> Option<PathBuf> {
    let name = entry.trim().replace('\\', "/");
    let name = name.rsplit('/').next().unwrap_or_default().to_lowercase();
    pkg.music
        .iter()
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.to_lowercase() == name)
        })
        .cloned()
}

/// Chooses the music: the explicit pack, else the theme (its own folder, then the pack it
/// refers to), else the referenced pack's own settings.
fn pick_music(
    pkg: Option<&ThemePackage>,
    explicit: Option<&AssetPack>,
    referenced: Option<&AssetPack>,
) -> MusicSetup {
    let by_pack = |pack: &AssetPack| {
        let cfg = pack.meta.as_ref().and_then(|m| m.settings.clone())?;
        music_from(&cfg, &|e| resolve_music_file(pack, e))
    };
    let by_theme = || {
        let cfg = pkg?.manifest.as_ref()?.music.clone();
        music_from(&cfg, &|e| {
            referenced
                .and_then(|p| resolve_music_file(p, e))
                .or_else(|| pkg.and_then(|t| find_in_theme(t, e)))
        })
    };
    explicit
        .and_then(by_pack)
        .or_else(by_theme)
        .or_else(|| referenced.and_then(by_pack))
        .unwrap_or(MusicSetup {
            mode: MusicMode::Disabled,
            playback_mode: PlaybackMode::InOrder,
            playlist: Vec::new(),
            schedule: Vec::new(),
        })
}

/// The launcher's own tracks in `dir`, by name; none when the folder is missing.
fn default_music(dir: &Path) -> Vec<String> {
    let mut tracks: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    tracks.sort();
    tracks
}

/// The mode a source declares for its music.
fn declared_mode(pkg: Option<&ThemePackage>, referenced: Option<&AssetPack>) -> Option<MusicMode> {
    pkg.and_then(|p| p.manifest.as_ref())
        .and_then(|m| m.music.mode)
        .or_else(|| referenced?.meta.as_ref()?.settings.as_ref()?.mode)
}

impl Service {
    /// Forgets the cached theme files; the next read loads them again.
    pub(crate) fn invalidate_theme(&self) {
        *self
            .theme_cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }

    /// The files of the current selection, from the cache or the disk.
    pub(crate) fn theme_files(&self) -> std::sync::Arc<ThemeFiles> {
        let mut cache = self
            .theme_cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(files) = cache.as_ref() {
            return files.clone();
        }
        let loaded = std::sync::Arc::new(self.load_theme_files());
        *cache = Some(loaded.clone());
        loaded
    }

    /// The stored text of a theme setting, empty text counting as absent.
    pub(crate) fn theme_setting(&self, key: &str) -> Option<String> {
        self.db()
            .setting(key)
            .ok()
            .flatten()
            .filter(|v| !v.is_empty())
    }

    /// Reads the selection from the database and the files from the disk.
    fn load_theme_files(&self) -> ThemeFiles {
        let content = list_installed(&self.data_dir);
        let wanted = self.theme_setting(KEY_ACTIVE_THEME);
        let pkg = wanted
            .as_deref()
            .and_then(|id| {
                content
                    .themes
                    .iter()
                    .find(|d| d.file_name().and_then(|n| n.to_str()) == Some(id))
            })
            .and_then(|dir| read_package(dir).ok())
            .filter(|p| p.manifest.is_some());
        let referenced = |category: AssetCategory| {
            let refs = pkg.as_ref()?.manifest.as_ref()?.asset_refs.as_ref()?;
            let r = refs.iter().find(|r| r.category == category)?;
            find_pack(&content, category, &r.id)
        };
        let explicit = |category: AssetCategory| {
            let id = self.theme_setting(&pack_key(category))?;
            find_pack(&content, category, &id)
        };
        let layers = self.layer_list(&content, pkg.as_ref());
        let mut music = pick_music(
            pkg.as_ref(),
            explicit(AssetCategory::Music),
            referenced(AssetCategory::Music),
        );
        if music.playlist.is_empty() && music.schedule.is_empty() {
            music.playlist = default_music(&self.data_dir.join(DEFAULT_MUSIC_DIR_NAME));
        }
        let declared_music_mode = declared_mode(pkg.as_ref(), referenced(AssetCategory::Music));
        self.assemble(wanted, pkg, layers, music, declared_music_mode)
    }

    /// The sources in precedence order: the user's art, the explicit packs, the theme, the packs
    /// the theme refers to.
    fn layer_list(&self, content: &InstalledContent, pkg: Option<&ThemePackage>) -> Vec<Layer> {
        let referenced = |category: AssetCategory| {
            let refs = pkg?.manifest.as_ref()?.asset_refs.as_ref()?;
            let r = refs.iter().find(|r| r.category == category)?;
            find_pack(content, category, &r.id)
        };
        let explicit = |category: AssetCategory| {
            let id = self.theme_setting(&pack_key(category))?;
            find_pack(content, category, &id)
        };
        let canonical = |id: &str| self.canonical_id(id);
        let mut layers = Vec::new();
        let push = |layers: &mut Vec<Layer>, level: SourceLevel, set: LayerSet| {
            let slots = slots_of_package(&package_of(set));
            layers.push(Layer {
                level,
                files: canonical_slots(slots, &canonical),
            });
        };
        let user = read_theme_layers(&self.data_dir.join(USER_ART_DIR_NAME));
        push(&mut layers, SourceLevel::User, user);
        for category in LAYERS {
            if let Some(pack) = explicit(category) {
                push(
                    &mut layers,
                    SourceLevel::Explicit,
                    zl_theme::read_pack_content(pack),
                );
            }
        }
        if let Some(p) = pkg {
            let slots = canonical_slots(slots_of_package(p), &canonical);
            layers.push(Layer {
                level: SourceLevel::Base,
                files: slots,
            });
        }
        for category in LAYERS {
            if let Some(pack) = referenced(category) {
                push(
                    &mut layers,
                    SourceLevel::Base,
                    zl_theme::read_pack_content(pack),
                );
            }
        }
        layers
    }

    /// Puts the loaded parts together, falling back to the built-in theme when none loaded.
    fn assemble(
        &self,
        wanted: Option<String>,
        pkg: Option<ThemePackage>,
        layers: Vec<Layer>,
        music: MusicSetup,
        declared_music_mode: Option<MusicMode>,
    ) -> ThemeFiles {
        let manifest = pkg.as_ref().and_then(|p| p.manifest.as_ref());
        ThemeFiles {
            id: pkg.as_ref().map(|p| p.id.clone()),
            name: manifest.map_or_else(
                || zl_config::appearance::BUILTIN_THEME_NAME.to_string(),
                |m| m.name.clone(),
            ),
            fell_back: wanted.is_some() && pkg.is_none(),
            mode: manifest
                .and_then(|m| m.theme_mode)
                .unwrap_or(ThemeMode::Dark),
            scheme: manifest.and_then(|m| m.color_scheme.clone()),
            wallpaper_main: pkg.as_ref().and_then(|p| p.wallpaper_main.clone()),
            wallpaper_external: pkg.as_ref().and_then(|p| p.wallpaper_external.clone()),
            layers,
            music,
            declared_music_mode,
        }
    }
}

/// Absolute-path text of an optional path.
pub(crate) fn path_text(path: Option<PathBuf>) -> Option<String> {
    path.map(|p| p.to_string_lossy().into_owned())
}
