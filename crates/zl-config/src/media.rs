//! Where per-game media (covers, heroes, logos, screenshots, videos) is read from.

/// Folder of ES-DE's data on shared storage, used when the user has not chosen another one. It
/// is where ES-DE for Android keeps `settings/` and `downloaded_media/` by default.
pub const ESDE_DEFAULT_ROOT: &str = "/storage/emulated/0/ES-DE";

/// File (relative to the ES-DE folder) whose `MediaDirectory` entry, when not empty, moves the
/// media folder somewhere else.
pub const ESDE_SETTINGS_FILE: &str = "settings/es_settings.xml";

/// Name of the setting in the ES-DE settings file that holds the media folder.
pub const ESDE_MEDIA_DIRECTORY_SETTING: &str = "MediaDirectory";

/// Folder inside the ES-DE folder that holds the scraped media, one sub-folder per system.
pub const ESDE_MEDIA_FOLDER: &str = "downloaded_media";

/// Folder inside Cocoon's data folder that holds its media, one sub-folder per platform.
pub const COCOON_MEDIA_FOLDER: &str = "downloaded_media";

/// Marker Cocoon puts in the name of a file the user edited: `<rom stem>__cocoon_edit_<n>_<hex>`.
pub const COCOON_EDIT_MARKER: &str = "__cocoon_edit_";

/// Length of the hexadecimal hash that ends a Cocoon edit file name.
pub const COCOON_EDIT_HASH_LEN: usize = 32;

/// How many folder levels below `downloaded_media/<system>/<kind>/` are searched for files. ES-DE
/// mirrors the sub-folders of the ROM folder there, so a game kept in `Genre/Game.sfc` has its
/// cover in `covers/Genre/Game.png`.
pub const MEDIA_SCAN_DEPTH: usize = 4;

/// Extensions (lower case) of the image files taken as images.
pub const IMAGE_EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "webp", "gif", "bmp"];

/// Extensions (lower case) of the files taken as videos.
pub const VIDEO_EXTENSIONS: [&str; 5] = ["mp4", "webm", "mkv", "avi", "mov"];

/// Extensions (lower case) of the files taken as manuals.
pub const MANUAL_EXTENSIONS: [&str; 1] = ["pdf"];

/// Extensions (lower case) of the files taken as video wallpapers of a theme.
pub const VIDEO_WALLPAPER_EXTENSIONS: [&str; 4] = ["mp4", "webm", "mkv", "mov"];
