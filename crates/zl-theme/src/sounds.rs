//! The sound actions a theme can replace (spec section 5).

string_enum! {
    /// An interface action that has a sound; a theme supplies `sounds/<action>.<ext>`.
    SoundAction {
        /// Moving the cursor.
        Navigate => "navigate",
        /// Confirming.
        Select => "select",
        /// Going back.
        Back => "back",
        /// Opening a folder.
        FolderOpen => "folder_open",
        /// Closing a folder.
        FolderClose => "folder_close",
        /// Launching a game.
        Launch => "launch",
        /// An error.
        Error => "error",
        /// A notification.
        Notification => "notification",
        /// Opening the Discord panel.
        DiscordOpen => "discord_open",
        /// Closing the Discord panel.
        DiscordClose => "discord_close",
        /// Swapping screens.
        ScreenSwap => "screen_swap",
        /// Zooming the grid in.
        GridZoomIn => "grid_zoom_in",
        /// Zooming the grid out.
        GridZoomOut => "grid_zoom_out",
        /// Picking an item up to reorder it.
        ReorderPickup => "reorder_pickup",
        /// Placing a reordered item.
        ReorderPlace => "reorder_place",
        /// Opening the editor.
        OpenEdit => "open_edit",
        /// A pop.
        Pop => "pop",
        /// Increasing a slider.
        SliderIncrease => "slider_increase",
        /// Decreasing a slider.
        SliderDecrease => "slider_decrease",
        /// Saving started.
        Saving => "saving",
        /// Saving finished.
        SavingEnd => "saving_end",
        /// A RetroAchievements unlock.
        Retroachievements => "retroachievements",
    }
}

/// Extensions a sound file may have.
pub const SOUND_EXTENSIONS: &[&str] = &["wav", "mp3", "ogg", "m4a", "flac"];

/// Splits a file name in `sounds/` into a known action when both name and extension are valid.
pub(crate) fn action_of_file(file_name: &str) -> Option<crate::sounds::SoundAction> {
    use crate::enums::StringEnum;
    let (stem, ext) = file_name.rsplit_once('.')?;
    let ext = ext.to_ascii_lowercase();
    if !SOUND_EXTENSIONS.contains(&ext.as_str()) {
        return None;
    }
    SoundAction::parse_text(stem)
}

/// Whether a file name has an extension a sound may have (the name itself is not looked at).
pub(crate) fn is_sound_extension(file_name: &str) -> bool {
    file_name
        .rsplit_once('.')
        .is_some_and(|(_, ext)| SOUND_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
}
