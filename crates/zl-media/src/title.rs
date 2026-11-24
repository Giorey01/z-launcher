//! The title shown on screen, derived from the one stored (which keeps the file name's tags).

/// Strips the trailing `(...)` and `[...]` groups from a title (`Game (Europe) (En,Fr)` gives
/// `Game`, `Mario Kart 8 [0005000010145000][v0]` gives `Mario Kart 8`), and reads a title that
/// has no spaces at all as written with underscores (`Super_Mario_Land` gives `Super Mario
/// Land`). Conservative: only trailing groups go, a title that would end up empty is kept as it
/// is, and nothing else (articles, punctuation, case) is touched.
/// Common suffixes appended by ROM dumping and decrypting utilities.
const DUMP_SUFFIXES: &[&str] = &[
    "-decrypted",
    "-Decrypted",
    "_decrypted",
    "_Decrypted",
    ".decrypted",
];

/// Known geographic regions appended without brackets in some file naming conventions.
const KNOWN_REGIONS: &[&str] = &["Europe", "USA", "Japan", "World", "Asia", "Australia"];

/// Two-letter language codes commonly concatenated in multi-language ROM dumps (e.g. EnFrDeEsIt).
const LANG_CODES: &[&str] = &[
    "En", "Fr", "De", "Es", "It", "Nl", "Pt", "Ru", "Zh", "Ko", "Ja", "Pl", "Sv", "No", "Da", "Fi",
];

/// Strips the trailing `(...)` and `[...]` groups and dump/region/language tags from a title,
/// and reads a title that has no spaces at all as written with underscores (`Super_Mario_Land` gives
/// `Super Mario Land`). Conservative: only trailing groups and suffixes go, a title that would end up
/// empty is kept as it is, and nothing else (articles, punctuation, case) is touched.
pub fn display_title(title: &str) -> String {
    let stripped = strip_all_tags(title.trim());
    let base = if stripped.is_empty() {
        title.trim()
    } else {
        stripped
    };
    if !base.contains(' ') && base.contains('_') {
        let spaced = base.replace('_', " ");
        let spaced = spaced.split_whitespace().collect::<Vec<_>>().join(" ");
        if !spaced.is_empty() {
            return spaced;
        }
    }
    base.to_string()
}

/// Recursively removes dump suffixes, balanced bracket groups, revision tags, language bundles,
/// and region names from the end of `text`.
fn strip_all_tags(mut text: &str) -> &str {
    loop {
        let prev = text;
        text = strip_trailing_dump_suffix(text);
        text = strip_trailing_groups(text);
        text = strip_trailing_revision(text);
        text = strip_trailing_language_bundle(text);
        text = strip_trailing_region(text);
        if text == prev {
            break;
        }
    }
    text
}

/// Removes dump suffixes from the end of `text`.
fn strip_trailing_dump_suffix(text: &str) -> &str {
    let mut cur = text;
    for suffix in DUMP_SUFFIXES {
        if let Some(stripped) = cur.strip_suffix(suffix) {
            cur = stripped.trim_end();
        }
    }
    cur
}

/// Strips trailing "Rev <N>" if present.
fn strip_trailing_revision(text: &str) -> &str {
    if let Some((prefix, last)) = text.rsplit_once(' ') {
        if last.chars().all(|c| c.is_ascii_digit()) {
            if let Some((base, rev_word)) = prefix.rsplit_once(' ') {
                if rev_word.eq_ignore_ascii_case("rev") {
                    return base.trim_end();
                }
            }
        }
    }
    text
}

/// Checks if a string consists entirely of 2-letter language codes (e.g., "EnFrDeEsIt").
fn is_language_bundle(word: &str) -> bool {
    if word.len() < 4 || !word.len().is_multiple_of(2) {
        return false;
    }
    for chunk in word.as_bytes().as_chunks::<2>().0 {
        let Ok(s) = std::str::from_utf8(chunk) else {
            return false;
        };
        if !LANG_CODES.contains(&s) {
            return false;
        }
    }
    true
}

/// Strips trailing language bundle word if present after a space.
fn strip_trailing_language_bundle(text: &str) -> &str {
    if let Some((base, last)) = text.rsplit_once(' ') {
        if is_language_bundle(last) {
            return base.trim_end();
        }
    }
    text
}

/// Strips trailing known region name if present after a space.
fn strip_trailing_region(text: &str) -> &str {
    if let Some((base, last)) = text.rsplit_once(' ') {
        if KNOWN_REGIONS.iter().any(|r| r.eq_ignore_ascii_case(last)) {
            return base.trim_end();
        }
    }
    text
}

/// Removes balanced `(...)` and `[...]` groups from the end of `text`, with the spaces around
/// them. A closing bracket without its opener ends the stripping.
fn strip_trailing_groups(text: &str) -> &str {
    let mut rest = text.trim_end();
    while let Some(start) = trailing_group_start(rest) {
        rest = rest[..start].trim_end();
    }
    rest
}

/// The byte index where the group that ends `text` starts, `None` when `text` does not end in a
/// balanced group.
fn trailing_group_start(text: &str) -> Option<usize> {
    let last = text.chars().next_back()?;
    let (close, open) = match last {
        ')' => (')', '('),
        ']' => (']', '['),
        _ => return None,
    };
    let mut depth = 0usize;
    for (i, c) in text.char_indices().rev() {
        if c == close {
            depth += 1;
        } else if c == open {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::display_title;

    /// Region, language, revision and ID tags at the end go; the rest stays.
    #[test]
    fn strips_trailing_tags() {
        let cases = [
            ("Advance Wars (USA)", "Advance Wars"),
            ("Game (Europe) (En,Fr,De)", "Game"),
            ("Game (Rev 1)", "Game"),
            ("Mario Kart 8 [0005000010145000][v0]", "Mario Kart 8"),
            ("Zelda (USA) [!]", "Zelda"),
            ("Game (Rev (1))", "Game"),
            ("Lonely (Disc 1)", "Lonely"),
            (
                "Super Mario 3D Land (Europe) (En,Fr,De,Es,It,Nl,Pt,Ru) (Rev 2)-decrypted",
                "Super Mario 3D Land",
            ),
            (
                "Angry Birds Trilogy Europe EnFrDeEsIt Rev 1-decrypted",
                "Angry Birds Trilogy",
            ),
            (
                "Cooking Mama - Bon Appetit Europe EnFrDeEsIt-decrypted",
                "Cooking Mama - Bon Appetit",
            ),
            (
                "Pokemon Ultra Moon Europe EnJaFrDeEsItZhKo-decrypted",
                "Pokemon Ultra Moon",
            ),
        ];
        for (input, want) in cases {
            assert_eq!(display_title(input), want, "{input}");
        }
    }

    /// Groups inside the title, leading groups and titles that are only a tag are kept.
    #[test]
    fn keeps_what_is_not_a_trailing_tag() {
        let kept = [
            "Mega Man (Classic) Collection",
            "[BIOS] Console",
            "(Europe)",
            "Game (unclosed",
            "Game ) odd",
            "Ratchet_ Deadlocked",
            "Hack",
        ];
        for input in kept {
            assert_eq!(display_title(input), input, "{input}");
        }
    }

    /// Underscores become spaces only when the name has no space.
    #[test]
    fn underscores_only_without_spaces() {
        assert_eq!(
            display_title("Super_Mario_Land (World)"),
            "Super Mario Land"
        );
        assert_eq!(
            display_title("Ratchet_ Deadlocked (USA)"),
            "Ratchet_ Deadlocked"
        );
        assert_eq!(display_title("_"), "_");
    }

    /// All variations of dump suffixes are stripped.
    #[test]
    fn all_dump_suffixes_stripped() {
        assert_eq!(display_title("Sonic Advance-decrypted"), "Sonic Advance");
        assert_eq!(display_title("Sonic Advance-Decrypted"), "Sonic Advance");
        assert_eq!(display_title("Sonic Advance_decrypted"), "Sonic Advance");
        assert_eq!(display_title("Sonic Advance_Decrypted"), "Sonic Advance");
        assert_eq!(display_title("Sonic Advance.decrypted"), "Sonic Advance");
    }

    /// Known regions without brackets are stripped from the end.
    #[test]
    fn known_regions_stripped() {
        assert_eq!(display_title("Metroid Fusion Europe"), "Metroid Fusion");
        assert_eq!(display_title("Metroid Fusion USA"), "Metroid Fusion");
        assert_eq!(display_title("Metroid Fusion Japan"), "Metroid Fusion");
        assert_eq!(display_title("Metroid Fusion World"), "Metroid Fusion");
        assert_eq!(display_title("Metroid Fusion Asia"), "Metroid Fusion");
        assert_eq!(display_title("Metroid Fusion Australia"), "Metroid Fusion");
    }

    /// Language bundle codes concatenated together are stripped.
    #[test]
    fn language_bundles_stripped() {
        assert_eq!(display_title("WarioWare EnFr"), "WarioWare");
        assert_eq!(display_title("WarioWare DeEsIt"), "WarioWare");
        assert_eq!(display_title("WarioWare EnFrDeEsItNlPtRu"), "WarioWare");
        // Unknown or odd-length words are kept
        assert_eq!(display_title("Title XxYy"), "Title XxYy");
        assert_eq!(display_title("Title EnF"), "Title EnF");
    }

    /// Revision markers after spaces are stripped.
    #[test]
    fn revision_markers_stripped() {
        assert_eq!(display_title("Kirby Rev 1"), "Kirby");
        assert_eq!(display_title("Kirby rev 2"), "Kirby");
        assert_eq!(display_title("Kirby REV 10"), "Kirby");
        assert_eq!(display_title("Kirby Revolution"), "Kirby Revolution");
    }

    /// Combined trailing patterns are stripped recursively.
    #[test]
    fn combined_patterns_stripped() {
        assert_eq!(
            display_title("Castlevania Europe EnFrDe Rev 3-decrypted"),
            "Castlevania"
        );
        assert_eq!(
            display_title("Castlevania (Europe) [v1.1] Rev 1_decrypted"),
            "Castlevania"
        );
        assert_eq!(display_title("   "), "");
    }
}
