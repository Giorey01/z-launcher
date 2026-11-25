//! The title shown on screen, derived from the one stored (which keeps the file name's tags),
//! and title normalization routines for cross-platform matching and searching.

use regex::Regex;
use std::sync::LazyLock;
use unicode_normalization::UnicodeNormalization;

/// Compiles a static regex pattern known to be valid at development time.
fn compile_regex(pattern: &str) -> Regex {
    match Regex::new(pattern) {
        Ok(re) => re,
        Err(_) => unreachable!("static pattern is valid"),
    }
}

/// Combining marks left after decomposing a character.
static ACCENTS_RE: LazyLock<Regex> = LazyLock::new(|| compile_regex(r"\p{M}+"));

/// Text enclosed in brackets or parentheses.
static BRACKETS_RE: LazyLock<Regex> = LazyLock::new(|| compile_regex(r"\[[^\]]*\]|\([^)]*\)"));

/// A run of Cyrillic letters.
static CYRILLIC_RE: LazyLock<Regex> = LazyLock::new(|| compile_regex(r"[\u{0400}-\u{04ff}]+"));

/// Platform name suffix occasionally found in titles.
static PLATFORM_NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"(?i)\bfor nintendo switch\b|\bnintendo switch\b"));

/// A release number prefix at the start (`1234 - `).
static RELEASE_NUM_RE: LazyLock<Regex> = LazyLock::new(|| compile_regex(r"^\s*\d{3,5}\s+-\s+"));

/// A version number or update/DLC/edition tail suffix.
static VERSION_TAIL_RE: LazyLock<Regex> = LazyLock::new(|| {
    compile_regex(r"(?i)\bv?\d+\.\d+(\.\d+)*\b|\b(update|dlc|deluxe edition|complete edition)\b.*$")
});

/// Anything that is not an ASCII lowercase letter or digit.
static NON_ALNUM_RE: LazyLock<Regex> = LazyLock::new(|| compile_regex(r"[^a-z0-9]+"));

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

/// Strips all combining accent and diacritic marks from `text`.
pub fn strip_accents(text: &str) -> String {
    let nfkd: String = text.nfkd().collect();
    ACCENTS_RE.replace_all(&nfkd, "").to_string()
}

/// Returns a normalized, comparable form of a title: accents, brackets, Cyrillic characters,
/// platform names, release numbers, and version/edition tails removed.
pub fn normalize(title: &str) -> String {
    let stripped = strip_accents(title);
    let t = BRACKETS_RE.replace_all(&stripped, " ");
    let t = CYRILLIC_RE.replace_all(&t, " ");
    let t = PLATFORM_NAME_RE.replace_all(&t, " ");
    let t = RELEASE_NUM_RE.replace_all(&t, " ");
    let t = VERSION_TAIL_RE.replace_all(&t, " ");
    let t = t.replace('&', " and ");
    let t = t.to_lowercase();
    let t = NON_ALNUM_RE.replace_all(&t, " ");
    t.trim().to_string()
}

/// Grouping key across platforms: accents stripped, lowercased, and non-alphanumeric runs replaced with single spaces.
pub fn title_key(title: &str) -> String {
    let lower = strip_accents(title).to_lowercase();
    NON_ALNUM_RE.replace_all(&lower, " ").trim().to_string()
}

/// Returns a URL-safe slug: lowercased ASCII words joined by hyphens, with quotes dropped.
pub fn slugify(title: &str) -> String {
    let stripped = strip_accents(title).to_lowercase();
    let no_quotes = stripped.replace(['\'', '’'], "");
    let hyphenated = NON_ALNUM_RE.replace_all(&no_quotes, "-");
    hyphenated.trim_matches('-').to_string()
}

/// Returns a title key suitable for matching across search sources: apostrophes dropped,
/// then normalized.
pub fn match_key(title: &str) -> String {
    normalize(&title.replace(['\'', '’'], ""))
}

#[cfg(test)]
mod tests {
    use super::{display_title, match_key, normalize, slugify, strip_accents, title_key};

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

    /// Accents and combining marks are stripped.
    #[test]
    fn test_strip_accents() {
        assert_eq!(strip_accents("Pokémon"), "Pokemon");
        assert_eq!(strip_accents("Castlevânia"), "Castlevania");
        assert_eq!(strip_accents("Crème Brûlée"), "Creme Brulee");
    }

    /// Title keys drop accents, lowercase, and collapse non-alphanumerics.
    #[test]
    fn test_title_key() {
        assert_eq!(title_key("Pokémon Gold & Silver"), "pokemon gold silver");
        assert_eq!(title_key("Baldur's Gate 3"), "baldur s gate 3");
        assert_eq!(
            title_key("The Legend of Zelda: Ocarina of Time"),
            "the legend of zelda ocarina of time"
        );
    }

    /// Normalization strips brackets, platform markers, and version tails.
    #[test]
    fn test_normalize() {
        assert_eq!(
            normalize("001 - Super Mario World (USA) [!]"),
            "super mario world"
        );
        assert_eq!(
            normalize("Super Mario Odyssey for Nintendo Switch (Update 1.3.0)"),
            "super mario odyssey"
        );
        assert_eq!(normalize("Mario & Luigi"), "mario and luigi");
    }

    /// Slugs produce clean lowercase hyphenated strings without quotes.
    #[test]
    fn test_slugify() {
        assert_eq!(
            slugify("The Legend of Zelda: Link's Awakening"),
            "the-legend-of-zelda-links-awakening"
        );
        assert_eq!(slugify("Pokémon Red"), "pokemon-red");
    }

    /// Match keys compare equal regardless of apostrophes.
    #[test]
    fn test_match_key() {
        assert_eq!(match_key("Baldur's Gate 3"), match_key("Baldurs Gate 3"));
        assert_eq!(
            match_key("Link's Crossbow Training"),
            "links crossbow training"
        );
    }
}
