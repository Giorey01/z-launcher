//! Which file extensions count as games on each platform.

use std::collections::{HashMap, HashSet};

/// "On this platform a folder is one game; launch the file matching the glob inside it"
/// (Wii U: `code/*.rpx`). Passed by the caller, never hardcoded in the scanner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderGameRule {
    /// Platform id (any case).
    pub platform: String,
    /// Path inside the game folder, `/`-separated, `*` matches any run of characters within one
    /// segment, case-insensitive: `code/*.rpx`.
    pub launch_glob: String,
}

/// Map platform id to the lower-case extensions (without dot, possibly multi-part such as
/// `p8.png`) that are games there. Built by the caller from the platform database.
#[derive(Debug, Clone, Default)]
pub struct ExtensionRules {
    /// Lower-cased platform id to its allowed extensions.
    map: HashMap<String, HashSet<String>>,
    /// Lower-cased platform id to its folder-game launch globs.
    folder_rules: HashMap<String, Vec<String>>,
}

impl ExtensionRules {
    /// Creates an empty rule set.
    pub fn new() -> ExtensionRules {
        ExtensionRules::default()
    }

    /// Sets the extensions of a platform; a leading dot and upper case are tolerated.
    pub fn insert(&mut self, platform_id: &str, extensions: &[&str]) {
        let set = extensions
            .iter()
            .map(|e| e.trim_start_matches('.').to_ascii_lowercase())
            .filter(|e| !e.is_empty())
            .collect();
        self.map.insert(platform_id.to_ascii_lowercase(), set);
    }

    /// Adds a folder-per-game rule. The platform still needs its extensions set with
    /// [`ExtensionRules::insert`] (possibly empty) to be scanned at all.
    pub fn add_folder_rule(&mut self, rule: FolderGameRule) {
        self.folder_rules
            .entry(rule.platform.to_ascii_lowercase())
            .or_default()
            .push(rule.launch_glob.to_ascii_lowercase());
    }

    /// Folder-game globs of a platform (lower-case id), empty when it has none.
    pub(crate) fn folder_globs(&self, platform_id: &str) -> &[String] {
        self.folder_rules
            .get(platform_id)
            .map_or(&[], Vec::as_slice)
    }

    /// Extensions of a platform given its folder name (any case), if the platform is known.
    pub(crate) fn extensions_for(&self, folder_name: &str) -> Option<&HashSet<String>> {
        self.map.get(&folder_name.to_ascii_lowercase())
    }
}

/// Finds the longest allowed extension a file name ends with and returns `(stem, extension)`.
/// `a.p8.png` with `p8.png` allowed gives `("a", "p8.png")`. The stem is never empty.
pub(crate) fn split_allowed<'a>(
    name: &'a str,
    allowed: &HashSet<String>,
) -> Option<(&'a str, String)> {
    for (index, _) in name.match_indices('.') {
        if index == 0 {
            continue;
        }
        let candidate = name[index + 1..].to_ascii_lowercase();
        if allowed.contains(&candidate) {
            return Some((&name[..index], candidate));
        }
    }
    None
}

/// Matches one path segment against a pattern where `*` is any run of characters.
fn segment_matches(pattern: &str, text: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == text,
        Some((head, tail)) => text.strip_prefix(head).is_some_and(|rest| {
            (0..=rest.len())
                .filter(|&i| rest.is_char_boundary(i))
                .any(|i| segment_matches(tail, &rest[i..]))
        }),
    }
}

/// Whether `relative` (a `/`-separated path inside a game folder) matches the glob.
pub(crate) fn glob_matches(glob: &str, relative: &str) -> bool {
    let relative = relative.to_ascii_lowercase();
    let (g, r): (Vec<&str>, Vec<&str>) = (glob.split('/').collect(), relative.split('/').collect());
    g.len() == r.len() && g.iter().zip(&r).all(|(p, t)| segment_matches(p, t))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Launch globs: wildcard within a segment, same depth, ignoring case.
    #[test]
    fn glob_matching() {
        assert!(glob_matches("code/*.rpx", "Code/Game.RPX"));
        assert!(glob_matches("code/*.rpx", "code/.rpx"));
        assert!(!glob_matches("code/*.rpx", "code/sub/game.rpx"));
        assert!(!glob_matches("code/*.rpx", "meta/game.rpx"));
        assert!(!glob_matches("code/*.rpx", "code/game.rpxx"));
        assert!(glob_matches("a*b*c", "a--b--c"));
        assert!(!glob_matches("a*b*c", "a--b--d"));
        assert!(glob_matches("*/boot.bin", "x/boot.bin"));
    }

    /// Builds a set from literals.
    fn set(items: &[&str]) -> HashSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    /// Multi-part extensions win over their last part and case is ignored.
    #[test]
    fn multipart_extension_matches() {
        let allowed = set(&["p8.png", "png"]);
        assert_eq!(
            split_allowed("Cart.P8.PNG", &allowed),
            Some(("Cart", "p8.png".into()))
        );
        assert_eq!(
            split_allowed("pic.png", &allowed),
            Some(("pic", "png".into()))
        );
        assert_eq!(
            split_allowed("Dr. Mario.nes", &set(&["nes"])),
            Some(("Dr. Mario", "nes".into()))
        );
        assert_eq!(split_allowed("a.txt", &allowed), None);
        assert_eq!(split_allowed(".nes", &set(&["nes"])), None);
    }

    /// Platform lookup ignores case and leading dots in the rules.
    #[test]
    fn lookup_is_case_insensitive() {
        let mut rules = ExtensionRules::new();
        rules.insert("GBA", &[".GBA"]);
        assert!(rules
            .extensions_for("gba")
            .is_some_and(|s| s.contains("gba")));
        assert!(rules.extensions_for("Gba").is_some());
        assert!(rules.extensions_for("nes").is_none());
    }
}
