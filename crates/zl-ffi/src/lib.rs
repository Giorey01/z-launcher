//! The foreign interface of the Z-Launcher core: what the Android app (through UniFFI) can call.
//!
//! Only real, implemented capabilities are exported here; each new feature adds its own module.
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

uniffi::setup_scaffolding!();

/// The version of the core library, taken from the workspace package version.
#[uniffi::export]
pub fn core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// The name of the folder the app keeps its data in, from the shared configuration.
#[uniffi::export]
pub fn data_dir_name() -> String {
    zl_config::app::DATA_DIR_NAME.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exported version is the workspace version, not a placeholder.
    #[test]
    fn version_is_the_package_version() {
        assert_eq!(core_version(), env!("CARGO_PKG_VERSION"));
        assert!(!core_version().is_empty());
    }

    /// The exported folder name is the one defined in the configuration crate.
    #[test]
    fn data_dir_name_comes_from_config() {
        assert_eq!(data_dir_name(), zl_config::app::DATA_DIR_NAME);
    }
}
