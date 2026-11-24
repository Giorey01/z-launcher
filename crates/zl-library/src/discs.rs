//! Recognising disc numbers in file names: `Game (Disc 1)`, `(Disk 2 of 3)`, `(CD1)`, and the
//! bare names (`Disc 1`, `cd2`) used inside a one-game folder.

/// Splits `inner` (text between parentheses) into a disc number if it is a disc marker:
/// `disc|disk|cd`, optional space or underscore, digits, optionally `of N`.
fn marker_number(inner: &str) -> Option<u32> {
    let lower = inner.trim().to_ascii_lowercase();
    let rest = ["disc", "disk", "cd"]
        .iter()
        .find_map(|word| lower.strip_prefix(word))?;
    let rest = rest.trim_start_matches([' ', '_']);
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    let number = digits.parse().ok()?;
    let tail = rest[digits.len()..].trim();
    let ok = tail.is_empty()
        || tail
            .strip_prefix("of")
            .is_some_and(|n| !n.trim().is_empty() && n.trim().chars().all(|c| c.is_ascii_digit()));
    ok.then_some(number)
}

/// Collapses runs of spaces and trims spaces and dashes from the ends.
fn tidy(text: &str) -> String {
    let joined = text.split_whitespace().collect::<Vec<_>>().join(" ");
    joined
        .trim_matches(|c| c == ' ' || c == '-' || c == '_')
        .to_string()
}

/// Finds a `(Disc N)`-style marker in a file stem. Returns the title with the marker removed
/// (other tags such as `(Europe)` stay) and the disc number.
pub(crate) fn disc_key(stem: &str) -> Option<(String, u32)> {
    let mut search_from = 0;
    while let Some(open_rel) = stem[search_from..].find('(') {
        let open = search_from + open_rel;
        let close = open + stem[open..].find(')')?;
        if let Some(number) = marker_number(&stem[open + 1..close]) {
            let title = tidy(&format!("{} {}", &stem[..open], &stem[close + 1..]));
            return (!title.is_empty()).then_some((title, number));
        }
        search_from = open + 1;
    }
    None
}

/// Whether a whole stem is a bare disc name such as `Disc 1`, `disk2`, `CD_3`.
pub(crate) fn bare_disc_number(stem: &str) -> Option<u32> {
    let lower = stem.trim().to_ascii_lowercase();
    let rest = ["disc", "disk", "cd"]
        .iter()
        .find_map(|word| lower.strip_prefix(word))?;
    let rest = rest.trim_start_matches([' ', '_', '-']);
    (!rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()))
        .then(|| rest.parse().ok())
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Markers are removed, other tags stay, numbers are read.
    #[test]
    fn disc_markers() {
        assert_eq!(
            disc_key("Final Fantasy VII (Disc 1)"),
            Some(("Final Fantasy VII".into(), 1))
        );
        assert_eq!(
            disc_key("Game (Europe) (Disk 2 of 3)"),
            Some(("Game (Europe)".into(), 2))
        );
        assert_eq!(
            disc_key("Game (Disc 2) (Europe)"),
            Some(("Game (Europe)".into(), 2))
        );
        assert_eq!(disc_key("Game (CD3)"), Some(("Game".into(), 3)));
        assert_eq!(disc_key("Game (cd 1)"), Some(("Game".into(), 1)));
    }

    /// Things that look similar but are not disc markers.
    #[test]
    fn not_markers() {
        assert_eq!(disc_key("Game (Disc One)"), None);
        assert_eq!(disc_key("Game (Discovery 2)"), None);
        assert_eq!(disc_key("Game (Europe)"), None);
        assert_eq!(disc_key("(Disc 1)"), None);
    }

    /// Bare disc names.
    #[test]
    fn bare_names() {
        assert_eq!(bare_disc_number("Disc 1"), Some(1));
        assert_eq!(bare_disc_number("disk2"), Some(2));
        assert_eq!(bare_disc_number("CD_3"), Some(3));
        assert_eq!(bare_disc_number("Disc"), None);
        assert_eq!(bare_disc_number("Discworld"), None);
    }
}
