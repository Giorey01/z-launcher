//! Reading the file names out of `.m3u` playlists and `.cue` / `.gdi` track sheets.

/// Byte order mark some editors put at the start of a text file.
const BOM: char = '\u{feff}';

/// Entries of an M3U playlist: non-empty, non-comment lines, with `\` turned into `/`.
pub(crate) fn m3u_entries(text: &str) -> Vec<String> {
    text.strip_prefix(BOM)
        .unwrap_or(text)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.replace('\\', "/"))
        .collect()
}

/// Returns the text of a leading `"quoted"` token, if `text` starts with a quote.
fn quoted(text: &str) -> Option<&str> {
    let rest = text.strip_prefix('"')?;
    rest.split_once('"').map(|(inside, _)| inside)
}

/// File names referenced by `FILE "name" BINARY` lines of a CUE sheet (quotes optional).
pub(crate) fn cue_files(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in text.strip_prefix(BOM).unwrap_or(text).lines() {
        let line = line.trim();
        let Some(head) = line.get(..4) else { continue };
        if !head.eq_ignore_ascii_case("file") {
            continue;
        }
        let rest = line[4..].trim();
        let name = match quoted(rest) {
            Some(inside) => inside,
            None => rest
                .rsplit_once(' ')
                .map_or(rest, |(left, _kind)| left.trim()),
        };
        if !name.is_empty() {
            names.push(name.replace('\\', "/"));
        }
    }
    names
}

/// File names of a GDI sheet: first line is the track count, then
/// `track lba type sector file offset` with the file name quoted when it has spaces.
pub(crate) fn gdi_files(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in text.strip_prefix(BOM).unwrap_or(text).lines().skip(1) {
        let line = line.trim();
        let name = match line.find('"') {
            Some(at) => quoted(&line[at..]).map(str::to_string),
            None => line.split_whitespace().nth(4).map(str::to_string),
        };
        if let Some(name) = name.filter(|n| !n.is_empty()) {
            names.push(name.replace('\\', "/"));
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    /// M3U: comments, blanks, BOM, CRLF and Windows separators are handled.
    #[test]
    fn m3u_parsing() {
        let text = "\u{feff}#EXTM3U\r\n\r\nDisc 1.cue\r\nsub\\Disc 2.chd\r\n  # note\r\n";
        assert_eq!(m3u_entries(text), ["Disc 1.cue", "sub/Disc 2.chd"]);
    }

    /// CUE: quoted and unquoted names, lower-case keyword, other lines ignored.
    #[test]
    fn cue_parsing() {
        let text = "REM x\nFILE \"Game (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\nfile Track2.bin BINARY\nFILE \"\" BINARY\n";
        assert_eq!(cue_files(text), ["Game (Track 1).bin", "Track2.bin"]);
    }

    /// GDI: the count line is skipped; quoted and bare names both work.
    #[test]
    fn gdi_parsing() {
        let text = "3\n1 0 4 2352 track01.bin 0\n2 450 0 2352 \"track 02.raw\" 0\n3 45000 4 2352 track03.bin 0\n";
        assert_eq!(
            gdi_files(text),
            ["track01.bin", "track 02.raw", "track03.bin"]
        );
    }
}
