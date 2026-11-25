//! The launcher's own interface sounds: generated once into the data folder from the recipes
//! of `zl_config::sounds` and used for every action the active theme (or its sound pack, or the
//! user) has no file for. A new recipe version goes to a new folder, so the files are made again.

use std::path::PathBuf;

use zl_config::sounds::{DEFAULT_SOUNDS_FOLDER, RECIPES_VERSION};
use zl_theme::synth::{recipe, render, wav_bytes};
use zl_theme::SoundAction;

use crate::Service;

impl Service {
    /// The folder of the generated sounds of the current recipe version.
    fn default_sounds_dir(&self) -> PathBuf {
        self.data_dir
            .join(DEFAULT_SOUNDS_FOLDER)
            .join(format!("v{RECIPES_VERSION}"))
    }

    /// The launcher's own file for `action`, generated if missing; `None` when it cannot be
    /// written (the action is then silent, as before).
    pub(crate) fn default_sound(&self, action: SoundAction) -> Option<String> {
        let dir = self.default_sounds_dir();
        let path = dir.join(format!("{}.wav", action.as_str()));
        if !path.is_file() {
            let tones = recipe(action.as_str())?;
            std::fs::create_dir_all(&dir).ok()?;
            let tmp = path.with_extension("wav.part");
            std::fs::write(&tmp, wav_bytes(&render(tones))).ok()?;
            std::fs::rename(&tmp, &path).ok()?;
        }
        Some(path.to_string_lossy().into_owned())
    }
}
