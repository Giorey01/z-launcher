//! Tunables of the platform and emulator database (`zl-platforms`).

/// File name of the platform list, both embedded in the app and in an update folder.
pub const PLATFORMS_FILE_NAME: &str = "platforms.json";

/// File name of the emulator list, both embedded in the app and in an update folder.
pub const EMULATORS_FILE_NAME: &str = "emulators.json";

/// Folder where RetroArch keeps its cores, with `{package}` replaced by the installed RetroArch
/// package. It is the same on every RetroArch build the database knows about.
pub const RETROARCH_CORE_DIR: &str = "/data/data/{package}/cores";

/// Platforms whose games are folders, with the file inside the folder that launches the game
/// (path relative to the game folder, `*` matches within one segment). Wii U games are
/// `<title> [ID]/` with the `code/*.rpx` file started by Cemu (`docs/reference/launch-observed.md`).
pub const FOLDER_GAME_RULES: [(&str, &str); 1] = [("wiiu", "code/*.rpx")];
