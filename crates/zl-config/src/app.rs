//! Identity of the app: names that appear on disk and to the user.

/// Name of the folder that holds the launcher's data (themes, media, backups) on shared storage.
/// Visible to the user in file managers, so it is the product name.
pub const DATA_DIR_NAME: &str = "Z-Launcher";

/// Folder suggested as the ROM root during onboarding when it exists on the device. It is the
/// layout the target users already have (one folder per platform below it).
pub const SUGGESTED_ROM_FOLDER: &str = "/storage/emulated/0/Games";

/// Number of slots of the dock on the Home, the "All apps" button included. Five fit the width of
/// the small screen of a dual-screen handheld with room for the focus cursor.
pub const DOCK_SLOTS: usize = 5;

/// File name of the launcher's database inside the app data folder.
pub const DATABASE_FILE_NAME: &str = "library.db";
