//! The names a game's media may be filed under.

use std::path::Path;

use unicode_normalization::UnicodeNormalization;

/// The comparable form of a file stem or title: composed (NFC, so `e` + accent and `é` are the
/// same name, as when a card was filled on macOS) and lower case.
pub(crate) fn stem_key(text: &str) -> String {
    text.nfc().collect::<String>().to_lowercase()
}

/// The lower-case names under which media of a game is looked up: the stem of its primary file
/// (name without the last extension) and its title. A multi-disc game is filed under its title
/// (`FF7`) or its first disc (`FF7 (Disc 1)`); a Wii U folder game under its folder name, which is
/// its title. Duplicates are removed, order kept.
pub fn game_keys(primary_path: &str, title: &str) -> Vec<String> {
    let stem = Path::new(primary_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let mut keys: Vec<String> = Vec::new();
    for candidate in [stem, title] {
        let key = stem_key(candidate);
        if !key.is_empty() && !keys.contains(&key) {
            keys.push(key);
        }
    }
    keys
}

#[cfg(test)]
mod tests {
    use super::game_keys;

    /// Stem and title are both keys, lower case, without duplicates.
    #[test]
    fn stem_and_title() {
        assert_eq!(
            game_keys("/r/psx/FF7 (Disc 1).cue", "FF7"),
            vec!["ff7 (disc 1)", "ff7"]
        );
        assert_eq!(game_keys("/r/gba/Zelda.gba", "Zelda"), vec!["zelda"]);
    }

    /// A Wii U game's primary file is `mk8.rpx`; its title (the folder) is the other key.
    #[test]
    fn folder_game_uses_the_title() {
        let keys = game_keys("/r/Wiiu/MK8 [0005]/code/mk8.rpx", "MK8 [0005]");
        assert_eq!(keys, vec!["mk8", "mk8 [0005]"]);
    }
}
