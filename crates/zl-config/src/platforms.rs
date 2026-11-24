//! Tunables of the platform and emulator database (`zl-platforms`).

/// File name of the platform list, both embedded in the app and in an update folder.
pub const PLATFORMS_FILE_NAME: &str = "platforms.json";

/// File name of the emulator list, both embedded in the app and in an update folder.
pub const EMULATORS_FILE_NAME: &str = "emulators.json";

/// Folder where RetroArch keeps its cores, with `{package}` replaced by the installed RetroArch
/// package. It is the same on every RetroArch build the database knows about.
pub const RETROARCH_CORE_DIR: &str = "/data/data/{package}/cores";
