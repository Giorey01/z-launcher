//! The theme store, the selection and the resolved theme, on the shared pack fixture
//! (`fixtures/themes/cocoon/data_dir_packs`) and on themes written by the tests.

use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use zl_core::{CoreError, FolderSlot, Service, WallpaperKind};
use zl_theme::{AssetCategory, MusicMode, ThemeMode};

/// Result type of the tests.
type Res = Result<(), Box<dyn Error>>;

/// The shared pack fixture, the way a Cocoon data folder looks.
fn cocoon_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/themes/cocoon/data_dir_packs")
}

/// A service over a fresh data folder.
fn fresh() -> Result<(TempDir, Service), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let svc = Service::open(&dir.path().join("data"))?;
    Ok((dir, svc))
}

/// A service with the fixture imported.
fn imported() -> Result<(TempDir, Service), Box<dyn Error>> {
    let (dir, svc) = fresh()?;
    svc.import_from_cocoon(&cocoon_dir().to_string_lossy())?;
    Ok((dir, svc))
}

/// A pack with a theme is copied once; the second import skips everything.
#[test]
fn cocoon_import_copies_then_skips() -> Res {
    let (_dir, svc) = fresh()?;
    let first = svc.import_from_cocoon(&cocoon_dir().to_string_lossy())?;
    assert_eq!(first.themes_added, vec!["demo-theme-123abc"]);
    assert_eq!(first.packs_added.len(), 9);
    assert!(first.themes_skipped.is_empty() && first.packs_skipped.is_empty());
    assert!(
        first.issues.iter().any(|i| i.source == "demo-theme-123abc"),
        "missing sounds pack is reported"
    );
    let second = svc.import_from_cocoon(&cocoon_dir().to_string_lossy())?;
    assert!(second.themes_added.is_empty() && second.packs_added.is_empty());
    assert_eq!(
        (second.themes_skipped.len(), second.packs_skipped.len()),
        (1, 9)
    );
    assert_eq!(svc.installed_themes().len(), 1);
    assert_eq!(svc.installed_packs().len(), 9);
    assert_eq!(
        svc.import_from_cocoon("/nonexistent"),
        Err(CoreError::NotFound)
    );
    Ok(())
}

/// With nothing selected the built-in theme is used, with defaults everywhere.
#[test]
fn builtin_theme_by_default() -> Res {
    let (_dir, svc) = fresh()?;
    let t = svc.active_theme()?;
    assert_eq!(
        (t.theme_id, t.mode, t.fell_back),
        (None, ThemeMode::Dark, false)
    );
    assert_eq!(t.colors, zl_core::ThemeColors::defaults());
    assert_eq!(
        t.colors.selection_gradient_start,
        t.colors.accent_gradient_start
    );
    assert_eq!(t.sounds.len(), 22);
    for sound in &t.sounds {
        let path = sound.path.as_deref().ok_or("no built-in sound")?;
        assert!(path.contains("/sounds/default/"), "{path}");
        assert_eq!(&fs::read(path)?[..4], b"RIFF", "a WAV file");
    }
    assert!(t.wallpaper_main.is_none());
    assert_eq!(t.settings, zl_core::UserSettings::default());
    Ok(())
}

/// Choosing the demo theme: wallpaper, referenced packs, per-slot fallback, explicit packs.
#[test]
fn demo_theme_layers() -> Res {
    let (_dir, svc) = imported()?;
    svc.apply_theme(Some("demo-theme-123abc"), false)?;
    let t = svc.active_theme()?;
    assert_eq!(t.name, "Demo Theme");
    let wall = t.wallpaper_main.ok_or("wallpaper")?;
    assert_eq!(wall.kind, WallpaperKind::Image);
    assert!(wall.path.ends_with("wallpapers/main.png"));
    let gba = svc.icon_overlay("gba");
    assert!(gba.mask.is_some() && gba.overlay.is_some());
    let psx = svc.icon_overlay("psx");
    assert!(
        psx.mask
            .as_deref()
            .is_some_and(|p| p.contains("ovl-a-111111") && p.contains("global")),
        "{psx:?}"
    );
    let sel = svc.theme_selection();
    let overlays = sel
        .layers
        .iter()
        .find(|l| l.category == AssetCategory::IconOverlays)
        .ok_or("layer")?;
    assert_eq!(overlays.effective_pack.as_deref(), Some("ovl-a-111111"));
    assert_eq!(overlays.explicit_pack, None);
    svc.set_layer_pack(AssetCategory::IconOverlays, Some("ovl-b-222222"))?;
    let psx = svc.icon_overlay("psx");
    assert!(
        psx.overlay
            .as_deref()
            .is_some_and(|p| p.contains("ovl-b-222222")),
        "{psx:?}"
    );
    let snes = svc.icon_overlay("sfc");
    assert!(
        snes.mask
            .as_deref()
            .is_some_and(|p| p.contains("ovl-a-111111")),
        "pack A still serves snes: {snes:?}"
    );
    assert_eq!(
        svc.set_layer_pack(AssetCategory::Sounds, Some("nope")),
        Err(CoreError::NotFound)
    );
    svc.set_layer_pack(AssetCategory::IconOverlays, None)?;
    assert!(svc
        .icon_overlay("psx")
        .mask
        .is_some_and(|p| p.contains("ovl-a-111111")));
    Ok(())
}

/// Smart folder art: an explicit pack, independent fallback per image, folder outside
/// `by_platform` first, the user's file over everything, aliases through the user folder.
#[test]
fn smart_folder_art_precedence() -> Res {
    let (dir, svc) = imported()?;
    svc.apply_theme(Some("demo-theme-123abc"), false)?;
    assert_eq!(svc.platform_art("psx"), zl_core::FolderArt::default());
    svc.set_layer_pack(AssetCategory::SmartFolders, Some("fold-a-333333"))?;
    let psx = svc.platform_art("PSX");
    assert!(psx.icon.is_none());
    assert!(psx
        .hero
        .as_deref()
        .is_some_and(|p| p.ends_with("psx/hero.webp")));
    assert!(psx.logo.is_some());
    let fav = svc.smart_folder_art("favorites");
    assert!(
        fav.icon
            .as_deref()
            .is_some_and(|p| p.ends_with("favorites/icon.gif")),
        "{fav:?}"
    );
    assert!(fav
        .hero
        .as_deref()
        .is_some_and(|p| p.ends_with("favorites/hero.webp")));
    let mine = dir.path().join("mine.png");
    fs::write(&mine, b"x")?;
    svc.set_user_folder_art(
        "favorites",
        false,
        FolderSlot::Hero,
        Some(&mine.to_string_lossy()),
    )?;
    let fav = svc.smart_folder_art("favorites");
    assert!(
        fav.hero
            .as_deref()
            .is_some_and(|p| p.contains("/user/smart_folders/favorites/hero.png")),
        "{fav:?}"
    );
    assert!(
        fav.icon.as_deref().is_some_and(|p| p.ends_with("icon.gif")),
        "other slots keep falling back"
    );
    svc.set_user_folder_art(
        "megadrive",
        true,
        FolderSlot::Icon,
        Some(&mine.to_string_lossy()),
    )?;
    for id in ["genesis", "megadrive", "megadrivejp"] {
        assert!(
            svc.platform_art(id)
                .icon
                .is_some_and(|p| p.contains("by_platform/genesis/")),
            "{id}"
        );
    }
    svc.set_user_folder_art("favorites", false, FolderSlot::Hero, None)?;
    assert!(svc
        .smart_folder_art("favorites")
        .hero
        .is_some_and(|p| p.ends_with("hero.webp")));
    let bad = svc.set_user_folder_art("favorites", false, FolderSlot::Hero, Some("/x/file.exe"));
    assert_eq!(bad, Err(CoreError::InvalidInput));
    Ok(())
}

/// Music files are resolved from the referenced pack (case-insensitive names), missing ones
/// are dropped, and the mode is the user's until the theme's settings are applied.
#[test]
fn music_and_sounds_resolve() -> Res {
    let (_dir, svc) = imported()?;
    svc.apply_theme(Some("demo-theme-123abc"), false)?;
    let m = svc.active_theme()?.music;
    assert_eq!(m.mode, MusicMode::Disabled);
    assert_eq!(m.playlist.len(), 1);
    assert!(m.playlist[0].ends_with("music/yotsuba dekakeru.mp3"));
    let times: Vec<(u8, u8)> = m.schedule.iter().map(|s| (s.hour, s.minute)).collect();
    assert_eq!(times, vec![(0, 0), (4, 0)]);
    svc.set_layer_pack(AssetCategory::Sounds, Some("snd-b-555555"))?;
    let t = svc.active_theme()?;
    let path_of = |name: &str| {
        t.sounds
            .iter()
            .find(|s| s.action.as_str() == name)
            .and_then(|s| s.path.clone())
    };
    assert!(path_of("navigate").is_some_and(|p| p.ends_with("navigate.wav")));
    assert!(path_of("select").is_some());
    assert!(
        path_of("pop").is_some_and(|p| p.contains("/sounds/default/")),
        "an action the pack lacks falls back to the launcher's own sound"
    );
    svc.apply_theme(Some("demo-theme-123abc"), true)?;
    assert_eq!(svc.active_theme()?.music.mode, MusicMode::Time);
    let after = svc.active_theme()?;
    assert!(
        after.sounds.iter().all(|s| s
            .path
            .as_deref()
            .is_some_and(|p| p.contains("/sounds/default/"))),
        "choosing a theme clears the layer packs"
    );
    Ok(())
}

/// Writes a theme folder with colours and settings into the data folder.
fn write_theme(data: &Path, id: &str, json: &str) -> Res {
    let dir = data.join("themes").join(id);
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("theme.json"), json)?;
    Ok(())
}

/// A theme's settings are applied only on request, limited to their ranges; colours fall back
/// per key, the selection follows the accent, and the user's accent replaces both.
#[test]
fn settings_colors_and_accent() -> Res {
    let (dir, svc) = fresh()?;
    let json = r##"{"name":"Mine","theme_mode":"OLED","color_scheme":{"accent_gradient_start":"#112233","accent_gradient_end":"rgba(0,0,0,0.5)","text_primary":"#ffffff66"},
        "settings":{"icon_scale":0.7,"cursor_style":"outlined","corner_hints":true,"highlight_thickness":0.9,"dock_on_top":false},
        "sfx_volume":0.25,"music_mode":"PLAYLIST"}"##;
    write_theme(&dir.path().join("data"), "mine-000001", json)?;
    svc.apply_theme(Some("mine-000001"), false)?;
    let t = svc.active_theme()?;
    assert_eq!(t.mode, ThemeMode::Oled);
    assert_eq!(t.colors.accent_gradient_start, 0xFF11_2233);
    assert_eq!(t.colors.accent_gradient_end, 0x8000_0000);
    assert_eq!(t.colors.text_primary, 0x66FF_FFFF);
    assert_eq!(t.colors.selection_gradient_start, 0xFF11_2233);
    assert_eq!(t.colors.divider, zl_config::appearance::DIVIDER);
    assert_eq!(
        t.settings,
        zl_core::UserSettings::default(),
        "not applied yet"
    );
    svc.apply_theme(Some("mine-000001"), true)?;
    let s = svc.active_theme()?.settings;
    assert_eq!(
        (s.icon_scale, s.corner_hints, s.dock_on_top),
        (0.7, true, false)
    );
    assert_eq!(s.cursor_style, zl_theme::CursorStyle::Outlined);
    assert_eq!((s.sfx_volume, s.music_mode), (0.25, MusicMode::Playlist));
    svc.update_user_settings(|s| s.accent_override = Some(0xFFAA_BBCC))?;
    let c = svc.active_theme()?.colors;
    assert_eq!(
        (c.accent_gradient_start, c.selection_gradient_end),
        (0xFFAA_BBCC, 0xFFAA_BBCC)
    );
    assert_eq!(
        c.accent_glow >> 24,
        u32::from(zl_config::appearance::ACCENT_OVERRIDE_GLOW_ALPHA)
    );
    svc.apply_theme(None, true)?;
    assert_eq!(svc.user_settings()?, zl_core::UserSettings::default());
    assert_eq!(
        svc.apply_theme(Some("ghost"), false),
        Err(CoreError::NotFound)
    );
    Ok(())
}

/// A selection that is no longer installed falls back to the built-in theme and says so.
#[test]
fn missing_theme_falls_back() -> Res {
    let (dir, svc) = fresh()?;
    write_theme(&dir.path().join("data"), "tmp-000002", r#"{"name":"Tmp"}"#)?;
    svc.apply_theme(Some("tmp-000002"), false)?;
    fs::remove_dir_all(dir.path().join("data/themes/tmp-000002"))?;
    svc.import_from_cocoon(&dir.path().to_string_lossy())?;
    let t = svc.active_theme()?;
    assert!(t.fell_back && t.theme_id.is_none());
    Ok(())
}

/// The screen and display settings round trip, are limited to their ranges and default as asked.
#[test]
fn screen_settings() -> Res {
    use zl_core::{ExternalDisplayDocking as Dock, ScreenMode, SingleRowStyle};
    let (_dir, svc) = fresh()?;
    let d = svc.user_settings()?;
    assert_eq!(
        (
            d.screen_mode,
            d.reverse_display_roles,
            d.external_display_docking
        ),
        (ScreenMode::Dual, false, Dock::Automatic)
    );
    assert_eq!(
        (d.hero_split, d.single_screen_rows, d.single_row_style),
        (0.4, 3, SingleRowStyle::Standard)
    );
    let s = svc.update_user_settings(|s| {
        s.screen_mode = ScreenMode::Single;
        s.reverse_display_roles = true;
        s.external_display_docking = Dock::NeverDock;
        s.hero_split = 2.0;
        s.single_screen_rows = 0;
        s.single_row_style = SingleRowStyle::CoverFlow;
    })?;
    assert_eq!(
        (
            s.screen_mode,
            s.reverse_display_roles,
            s.external_display_docking
        ),
        (ScreenMode::Single, true, Dock::NeverDock)
    );
    assert_eq!(
        (s.hero_split, s.single_screen_rows, s.single_row_style),
        (1.0, 1, SingleRowStyle::CoverFlow)
    );
    assert_eq!(svc.user_settings()?, s);
    Ok(())
}

/// Builds a ZIP file with the given entries.
fn make_zip(path: &Path, entries: &[(&str, &[u8])]) -> Res {
    let mut w = zip::ZipWriter::new(fs::File::create(path)?);
    let opts = zip::write::SimpleFileOptions::default();
    for (name, bytes) in entries {
        w.start_file(*name, opts)?;
        w.write_all(bytes)?;
    }
    w.finish()?;
    Ok(())
}

/// A ZIP with `themes/<id>/`, one with the theme at the root (id made from the name), and a
/// broken one.
#[test]
fn zip_import() -> Res {
    let (dir, svc) = fresh()?;
    let a = dir.path().join("a.zip");
    make_zip(
        &a,
        &[(
            "themes/zip-theme-abcdef/theme.json",
            br#"{"name":"Zip Theme"}"#,
        )],
    )?;
    assert_eq!(
        svc.import_theme_zip(&a.to_string_lossy())?.themes_added,
        vec!["zip-theme-abcdef"]
    );
    let b = dir.path().join("b.zip");
    make_zip(
        &b,
        &[
            ("theme.json", br#"{"name":"Root Theme"}"#),
            ("wallpapers/main.png", b"x"),
        ],
    )?;
    let report = svc.import_theme_zip(&b.to_string_lossy())?;
    assert_eq!(report.themes_added.len(), 1);
    assert!(report.themes_added[0].starts_with("root-theme-"));
    let wall = dir
        .path()
        .join("data/themes")
        .join(&report.themes_added[0])
        .join("wallpapers/main.png");
    assert!(wall.is_file());
    assert_eq!(svc.installed_themes().len(), 2);
    assert!(!dir.path().join("data/import-staging").exists());
    let bad = dir.path().join("bad.zip");
    fs::write(&bad, b"not a zip")?;
    assert_eq!(
        svc.import_theme_zip(&bad.to_string_lossy()),
        Err(CoreError::Theme { code: "zip".into() })
    );
    let evil = dir.path().join("evil.zip");
    make_zip(&evil, &[("../evil/theme.json", b"{}")])?;
    assert_eq!(
        svc.import_theme_zip(&evil.to_string_lossy()),
        Err(CoreError::Theme {
            code: "unsafe_path".into()
        })
    );
    Ok(())
}

/// The "PC" group folder takes its own art first, then each member platform's, slot by slot.
#[test]
fn group_folder_art_falls_back_to_its_platforms() -> Res {
    let (dir, svc) = imported()?;
    let mine = dir.path().join("mine.png");
    fs::write(&mine, b"x")?;
    let path = mine.to_string_lossy();
    assert_eq!(svc.platform_art("pcgames"), zl_core::FolderArt::default());
    svc.set_user_folder_art("steam", true, FolderSlot::Hero, Some(&path))?;
    svc.set_user_folder_art("steam", true, FolderSlot::Icon, Some(&path))?;
    svc.set_user_folder_art("pcgames", true, FolderSlot::Icon, Some(&path))?;
    let pc = svc.platform_art("PCGames");
    assert!(
        pc.icon
            .as_deref()
            .is_some_and(|p| p.contains("by_platform/pcgames/")),
        "own icon first: {pc:?}"
    );
    assert!(
        pc.hero
            .as_deref()
            .is_some_and(|p| p.contains("by_platform/steam/")),
        "hero from a member: {pc:?}"
    );
    assert_eq!(pc.logo, None);
    Ok(())
}
