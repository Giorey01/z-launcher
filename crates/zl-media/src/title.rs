//! The title shown on screen, derived from the one stored (which keeps the file name's tags).

/// Strips the trailing `(...)` and `[...]` groups from a title (`Game (Europe) (En,Fr)` gives
/// `Game`, `Mario Kart 8 [0005000010145000][v0]` gives `Mario Kart 8`), and reads a title that
/// has no spaces at all as written with underscores (`Super_Mario_Land` gives `Super Mario
/// Land`). Conservative: only trailing groups go, a title that would end up empty is kept as it
/// is, and nothing else (articles, punctuation, case) is touched.
pub fn display_title(title: &str) -> String {
    let stripped = strip_trailing_groups(title.trim());
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
}
