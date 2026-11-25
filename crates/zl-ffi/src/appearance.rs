//! UniFFI mirrors of the theme, settings and media types of the core.

use zl_core as core;
use zl_db::{MediaKind as DbMediaKind, MediaSource as DbMediaSource};
use zl_theme as theme;

mirror_enum! {
    /// Light, dark, OLED or follow the system.
    ThemeMode <=> theme::ThemeMode { Light, Dark, Oled, System }
}
mirror_enum! {
    /// How the hero image of a game is shown.
    HeroDisplayStyle <=> theme::HeroDisplayStyle { Vignette, Fullscreen, None }
}
mirror_enum! {
    /// Gradient over the hero image.
    HeroGradientStyle <=> theme::HeroGradientStyle { None, BlackGradient }
}
mirror_enum! {
    /// How icons are coloured.
    IconStyle <=> theme::IconStyle { Colored, Accent, Flat }
}
mirror_enum! {
    /// Material of the surfaces.
    SurfaceMaterial <=> theme::SurfaceMaterial { Solid, Glass }
}
mirror_enum! {
    /// Look of the selection cursor.
    CursorStyle <=> theme::CursorStyle { Animated, Outlined, AlternativeAnimated, Alternative }
}
mirror_enum! {
    /// Shape of the dock icons.
    DockIconShape <=> theme::DockIconShape { Circle, Squircle, RoundedSquare, Square }
}
mirror_enum! {
    /// How background music is chosen.
    MusicMode <=> theme::MusicMode { Time, Playlist, Disabled }
}
mirror_enum! {
    /// Order a playlist plays in.
    PlaybackMode <=> theme::PlaybackMode { InOrder, Shuffle }
}
mirror_enum! {
    /// A layer of a theme that comes from its own pack.
    AssetCategory <=> theme::AssetCategory { IconOverlays, SmartFolders, Sounds, Music }
}
mirror_enum! {
    /// How serious a problem in a theme is.
    Severity <=> theme::Severity { Error, Warning }
}
mirror_enum! {
    /// What is wrong in a theme, as a stable code to translate.
    IssueCode <=> theme::IssueCode {
        InvalidJson, MissingName, MissingWallpaperFile, UnknownKey, BadColor, ValueClamped,
        UnknownEnumValue, WrongType, BadSchedule, MissingAssetPack, UnknownSoundName,
        MissingMusicFile
    }
}
mirror_enum! {
    /// An interface action that has a sound.
    SoundAction <=> theme::SoundAction {
        Navigate, Select, Back, FolderOpen, FolderClose, Launch, Error, Notification,
        DiscordOpen, DiscordClose, ScreenSwap, GridZoomIn, GridZoomOut, ReorderPickup,
        ReorderPlace, OpenEdit, Pop, SliderIncrease, SliderDecrease, Saving, SavingEnd,
        Retroachievements
    }
}
mirror_enum! {
    /// What kind of file a wallpaper is.
    WallpaperKind <=> core::WallpaperKind { Image, Gif, Video }
}
mirror_enum! {
    /// Which image of a folder is replaced.
    FolderSlot <=> core::FolderSlot { Icon, Hero, Logo }
}
mirror_enum! {
    /// One display or two.
    ScreenMode <=> core::ScreenMode { Dual, Single }
}
mirror_enum! {
    /// Whether the Home moves to an external display.
    ExternalDisplayDocking <=> core::ExternalDisplayDocking { Automatic, AlwaysDock, NeverDock }
}
mirror_enum! {
    /// Look of the rows of the single-screen grid.
    SingleRowStyle <=> core::SingleRowStyle { Standard, Carousel, CoverFlow }
}
mirror_enum! {
    /// Which display a game starts on in the dual-screen mode.
    LaunchScreen <=> core::LaunchScreen { Hero, Grid }
}
mirror_enum! {
    /// Animation speed of the launcher UI.
    AnimationSpeed <=> core::AnimationSpeed { Normal, Fast, Off }
}
mirror_enum! {
    /// What a media file shows.
    MediaKind <=> DbMediaKind {
        Icon, Hero, Logo, Screenshot, Video, Box3d, TitleScreen, BackCover, MixImage,
        PhysicalMedia, Manual
    }
}
mirror_enum! {
    /// Where a media file comes from.
    MediaSource <=> DbMediaSource { Cocoon, Esde, User, SteamGridDb }
}

mirror_record! {
    /// Every colour of a theme as `0xAARRGGBB` (Android `Color` int). Missing colours have the
    /// built-in defaults; the selection gradient follows the accent; a user accent replaces both.
    ThemeColors <=> core::ThemeColors {
        /// Start of the background gradient.
        plain background_gradient_start: u32,
        /// End of the background gradient.
        plain background_gradient_end: u32,
        /// Start of the card gradient.
        plain card_gradient_start: u32,
        /// End of the card gradient.
        plain card_gradient_end: u32,
        /// Main text.
        plain text_primary: u32,
        /// Secondary text.
        plain text_secondary: u32,
        /// Tint of monochrome icons.
        plain icon_tint: u32,
        /// Background of a tile.
        plain tile_background: u32,
        /// Border of a tile.
        plain tile_border: u32,
        /// Start of the gradient of a switched-off toggle.
        plain toggle_off_gradient_start: u32,
        /// End of the gradient of a switched-off toggle.
        plain toggle_off_gradient_end: u32,
        /// Start of the gradient of a toggle thumb.
        plain toggle_thumb_gradient_start: u32,
        /// End of the gradient of a toggle thumb.
        plain toggle_thumb_gradient_end: u32,
        /// Outer shadow.
        plain drop_shadow: u32,
        /// Light inner shadow.
        plain inner_shadow_light: u32,
        /// Dark inner shadow.
        plain inner_shadow_dark: u32,
        /// Success colour.
        plain success: u32,
        /// Warning colour.
        plain warning: u32,
        /// Divider lines.
        plain divider: u32,
        /// Start of the accent gradient.
        plain accent_gradient_start: u32,
        /// End of the accent gradient.
        plain accent_gradient_end: u32,
        /// Glow around accent elements.
        plain accent_glow: u32,
        /// Start of the selection (cursor) gradient.
        plain selection_gradient_start: u32,
        /// End of the selection (cursor) gradient.
        plain selection_gradient_end: u32,
    }
}

mirror_record! {
    /// The user's look-and-sound settings, defaults applied. One record for the settings screen
    /// and for drawing; write it back whole with `set_user_settings` or change one value with
    /// the single setters.
    UserSettings <=> core::UserSettings {
        /// Corner roundness of icons, 0.0-0.5.
        plain icon_roundness: f32,
        /// Scale of icons in their cell, 0.5-1.0.
        plain icon_scale: f32,
        /// Scale of the item under the cursor.
        plain hover_scale: f32,
        /// How the hero image is shown.
        plain hero_display_style: HeroDisplayStyle,
        /// Scale of the game logo over the hero.
        plain hero_logo_scale: f32,
        /// Scale of the folder logo over the hero.
        plain hero_folder_logo_scale: f32,
        /// Gradient over the hero image.
        plain hero_gradient_style: HeroGradientStyle,
        /// Size of the hero vignette.
        plain hero_vignette_size: f32,
        /// How icons are coloured.
        plain icon_style: IconStyle,
        /// Material of the surfaces.
        plain surface_material: SurfaceMaterial,
        /// Blur of glass surfaces, 0-10.
        plain glass_blur: f32,
        /// Glass refraction, 0-1.
        plain glass_refraction: f32,
        /// Glass highlight, 0-1.
        plain glass_highlight: f32,
        /// Glass rim strength, 0-1.
        plain glass_rim_strength: f32,
        /// Glass tint, 0-1.
        plain glass_tint: f32,
        /// Whether tiles are glass too.
        plain glass_on_tiles: bool,
        /// Look of the selection cursor.
        plain cursor_style: CursorStyle,
        /// Thickness of the selection highlight, 0.5-1.0.
        plain highlight_thickness: f32,
        /// Shape of the dock icons.
        plain dock_icon_shape: DockIconShape,
        /// Hide the dock background.
        plain hide_dock_background: bool,
        /// Dock at the top of the screen.
        plain dock_on_top: bool,
        /// Always show the corner hints.
        plain corner_hints: bool,
        /// Open folders full screen.
        plain fullscreen_folders: bool,
        /// Accent colour picked by the user (`0xAARRGGBB`); replaces the theme's accent.
        opt accent_override: u32,
        /// One display or two (the app picks the first value from the display count).
        plain screen_mode: ScreenMode,
        /// Swap which display shows the grid and which the hero.
        plain reverse_display_roles: bool,
        /// Whether the Home moves to an external display.
        plain external_display_docking: ExternalDisplayDocking,
        /// Single screen: fraction (0-1) of the screen given to the hero area.
        plain hero_split: f32,
        /// Single screen: number of grid rows, 1-5.
        plain single_screen_rows: u8,
        /// Single screen: style of the rows.
        plain single_row_style: SingleRowStyle,
        /// Interface sound effects on.
        plain sfx_enabled: bool,
        /// Volume of the sound effects, 0-1.
        plain sfx_volume: f32,
        /// Background music mode.
        plain music_mode: MusicMode,
        /// Volume of the music, 0-1.
        plain music_volume: f32,
        /// Volume of the jingles, 0-1.
        plain jingle_volume: f32,
        /// Whether to show the battery percentage in the status pill.
        plain show_battery_percentage: bool,
        /// Show the home grid as discrete swipeable pages.
        plain discrete_paging: bool,
        /// Animation speed of the UI.
        plain animation_speed: AnimationSpeed,
        /// Dimming factor of the now playing screen, 0-1.
        plain now_playing_dimming: f32,
        /// Saturation factor of the now playing screen, 0-1.
        plain now_playing_saturation: f32,
        /// Seconds without a touch before the now playing screen dims.
        plain now_playing_idle_seconds: u16,
        /// Which display a game starts on in the dual-screen mode.
        plain launch_screen: LaunchScreen,
        /// Selected app icon pack id (`None` for Default).
        opt app_icon_pack: String,
        /// RetroAchievements account integration enabled.
        plain retroachievements_enabled: bool,
        /// Discord Rich Presence enabled.
        plain discord_presence_enabled: bool,
    }
}

mirror_record! {
    /// A wallpaper file.
    Wallpaper <=> core::Wallpaper {
        /// Absolute path.
        plain path: String,
        /// Image, GIF or video.
        plain kind: WallpaperKind,
    }
}
mirror_record! {
    /// The file of one interface sound.
    SoundEntry <=> core::SoundEntry {
        /// The action.
        plain action: SoundAction,
        /// Absolute path; `None` means play the app's default sound.
        opt path: String,
    }
}
mirror_record! {
    /// A track of a time-of-day schedule.
    ScheduleTrack <=> core::ScheduleTrack {
        /// Hour it starts, 0-23.
        plain hour: u8,
        /// Minute it starts, 0-59.
        plain minute: u8,
        /// Absolute path of the file.
        plain path: String,
    }
}
mirror_record! {
    /// Background music of the theme, files resolved.
    MusicSetup <=> core::MusicSetup {
        /// How music is chosen (the user's setting).
        plain mode: MusicMode,
        /// Order of a playlist.
        plain playback_mode: PlaybackMode,
        /// Playlist files (missing ones left out).
        list playlist: String,
        /// Time schedule (missing files left out).
        list schedule: ScheduleTrack,
    }
}
mirror_record! {
    /// Everything the UI needs to draw the launcher in the active theme.
    ResolvedTheme <=> core::ResolvedTheme {
        /// Folder name of the theme; `None` for the built-in one.
        opt theme_id: String,
        /// Display name.
        plain name: String,
        /// The selected theme is gone or unreadable and the built-in one is shown.
        plain fell_back: bool,
        /// Light, dark, OLED or system.
        plain mode: ThemeMode,
        /// All colours.
        plain colors: ThemeColors,
        /// The user's settings.
        plain settings: UserSettings,
        /// Wallpaper of the main (small) screen; `None` means draw the background gradient.
        opt wallpaper_main: Wallpaper,
        /// Wallpaper of the external (large) screen.
        opt wallpaper_external: Wallpaper,
        /// One entry per sound action (22), in a fixed order.
        list sounds: SoundEntry,
        /// Music.
        plain music: MusicSetup,
    }
}
mirror_record! {
    /// Images of a smart folder or a platform folder (absolute paths).
    FolderArt <=> core::FolderArt {
        /// Icon, possibly an animated GIF.
        opt icon: String,
        /// Hero image.
        opt hero: String,
        /// Logo.
        opt logo: String,
    }
}
mirror_record! {
    /// Mask and overlay of a platform's cover frame (absolute paths).
    OverlayArt <=> core::OverlayArt {
        /// The cover shows where the mask's alpha is opaque.
        opt mask: String,
        /// Frame drawn over the masked cover.
        opt overlay: String,
    }
}
mirror_record! {
    /// A pack of one layer, as referenced or imported.
    PackRef <=> core::PackRef {
        /// Layer.
        plain category: AssetCategory,
        /// Pack id.
        plain id: String,
        /// Display name.
        plain name: String,
    }
}
mirror_record! {
    /// An installed theme.
    InstalledTheme <=> core::InstalledTheme {
        /// Folder name; pass it to `apply_theme`.
        plain id: String,
        /// Display name.
        plain name: String,
        /// Author.
        opt author: String,
        /// Version text.
        opt version: String,
        /// Description.
        opt description: String,
        /// Declared mode.
        opt mode: ThemeMode,
        /// `preview.png` path.
        opt preview: String,
        /// Main wallpaper path, for a thumbnail when there is no preview.
        opt wallpaper_main: String,
        /// The theme carries settings the user can apply.
        plain has_settings: bool,
        /// Packs it refers to that are not installed.
        list missing_packs: PackRef,
        /// Errors in the theme.
        plain error_count: u32,
        /// Warnings in the theme.
        plain warning_count: u32,
        /// This is the selected theme.
        plain active: bool,
    }
}
mirror_record! {
    /// An installed asset pack.
    InstalledPack <=> core::InstalledPack {
        /// Folder name; pass it to `set_layer_pack`.
        plain id: String,
        /// Layer.
        plain category: AssetCategory,
        /// Display name.
        plain name: String,
        /// Author.
        opt author: String,
        /// Version text.
        opt version: String,
        /// `preview.png` path.
        opt preview: String,
        /// The pack in use for its layer.
        plain in_use: bool,
        /// The user chose it explicitly.
        plain explicit: bool,
    }
}
mirror_record! {
    /// The choice for one layer.
    LayerChoice <=> core::LayerChoice {
        /// Layer.
        plain category: AssetCategory,
        /// The pack the user chose; `None` follows the theme.
        opt explicit_pack: String,
        /// The pack in use (explicit, else the theme's, if installed).
        opt effective_pack: String,
    }
}
mirror_record! {
    /// The selected theme and layer packs.
    ThemeSelection <=> core::ThemeSelection {
        /// Selected theme; `None` is the built-in one.
        opt theme_id: String,
        /// The four layers.
        list layers: LayerChoice,
    }
}
mirror_record! {
    /// A problem in a theme or pack: a code and where, never a sentence.
    ThemeIssue <=> core::ThemeIssue {
        /// Theme or pack id.
        plain source: String,
        /// Error or warning.
        plain severity: Severity,
        /// What is wrong.
        plain code: IssueCode,
        /// JSON path or file path.
        plain location: String,
    }
}
mirror_record! {
    /// What an import did.
    ImportReport <=> core::ImportReport {
        /// Themes copied.
        list themes_added: String,
        /// Themes skipped (same id and version already installed).
        list themes_skipped: String,
        /// Packs copied.
        list packs_added: PackRef,
        /// Packs skipped.
        list packs_skipped: PackRef,
        /// Problems of what was read; a copy failure has location `copy_failed`.
        list issues: ThemeIssue,
    }
}
mirror_record! {
    /// A Cocoon data folder found on the device.
    CocoonDataDir <=> core::CocoonDataDir {
        /// Absolute path.
        plain path: String,
        /// Themes in it.
        plain themes: u32,
        /// Packs in it.
        plain packs: u32,
        /// It has `downloaded_media/` too.
        plain has_media: bool,
    }
}
mirror_record! {
    /// Where media is read from.
    MediaSources <=> core::MediaSources {
        /// Cocoon data folder chosen by the user; `None` looks at the detected ones.
        opt cocoon_dir: String,
        /// ES-DE folder (the one with `settings/`).
        plain esde_root: String,
    }
}
mirror_record! {
    /// Files stored by an import for one source and kind.
    MediaCount <=> core::MediaCount {
        /// Source.
        plain source: MediaSource,
        /// Kind.
        plain kind: MediaKind,
        /// Files matched to games and stored.
        plain count: u32,
    }
}
mirror_record! {
    /// What a media import did.
    MediaImportReport <=> core::MediaImportReport {
        /// Sources that were read.
        list scanned: MediaSource,
        /// Stored files per source and kind.
        list counts: MediaCount,
        /// Files that matched no game.
        plain unmatched: u32,
    }
}
mirror_record! {
    /// One media file of a game.
    MediaItem <=> core::MediaItem {
        /// What it shows.
        plain kind: MediaKind,
        /// Absolute path.
        plain path: String,
        /// Where it comes from.
        plain source: MediaSource,
        /// The user's choice.
        plain protected: bool,
    }
}
mirror_record! {
    /// The media of a game: the best file of each kind and the full list.
    GameMedia <=> core::GameMedia {
        /// Cover.
        opt icon: String,
        /// Hero image.
        opt hero: String,
        /// Logo.
        opt logo: String,
        /// Screenshots, best first.
        list screenshots: String,
        /// Video.
        opt video: String,
        /// 3D box.
        opt box3d: String,
        /// Title screen.
        opt titlescreen: String,
        /// Every file, best first within each kind.
        list items: MediaItem,
    }
}
mirror_record! {
    /// A platform of the library with its default emulator.
    PlatformEmulatorSetting <=> core::PlatformEmulatorSetting {
        /// Canonical platform id.
        plain platform_id: String,
        /// English platform name.
        plain platform_name: String,
        /// The default set for it, if any.
        opt current: crate::types::Candidate,
        /// Installed emulators that can launch it, preferred first.
        list candidates: crate::types::Candidate,
    }
}
