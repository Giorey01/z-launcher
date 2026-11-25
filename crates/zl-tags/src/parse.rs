//! Parsing of the text of a tag file into a [`TagFile`] plus the problems found.

use std::path::Path;

/// Byte order mark some editors put at the start of a UTF-8 file.
const BOM: char = '\u{feff}';

/// First bytes of a UTF-16 little-endian file with a byte order mark.
const UTF16_LE_BOM: [u8; 2] = [0xff, 0xfe];

/// First bytes of a UTF-16 big-endian file with a byte order mark.
const UTF16_BE_BOM: [u8; 2] = [0xfe, 0xff];

/// A parsed tag file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagFile {
    /// File name without its extension: the game's name.
    pub name: String,
    /// Extension without the dot, lower-cased (empty if the file has none).
    pub extension: String,
    /// The `(key, value)` pairs in file order, keys spelled as written.
    pub tags: Vec<(String, String)>,
}

/// What is wrong with a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueCode {
    /// The line is not of the form `[key] value`.
    MalformedLine,
    /// The brackets are empty: `[] value`.
    EmptyKey,
    /// The key has no value after it.
    EmptyValue,
}

impl IssueCode {
    /// Stable snake_case name, shared with the fixtures and the apps.
    pub fn as_str(self) -> &'static str {
        match self {
            IssueCode::MalformedLine => "malformed_line",
            IssueCode::EmptyKey => "empty_key",
            IssueCode::EmptyValue => "empty_value",
        }
    }
}

/// A problem found on one line; the line is skipped and parsing goes on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagIssue {
    /// 1-based line number.
    pub line: usize,
    /// What is wrong.
    pub code: IssueCode,
}

/// Result of parsing: the file and the problems found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTag {
    /// The tags that were read.
    pub file: TagFile,
    /// Lines that were skipped.
    pub issues: Vec<TagIssue>,
}

/// Failure to read a tag file from disk.
#[derive(Debug, thiserror::Error)]
pub enum TagError {
    /// The file could not be read.
    #[error("cannot read tag file: {0}")]
    Io(#[from] std::io::Error),
    /// The path has no file name.
    #[error("path has no file name")]
    NoFileName,
}

impl TagFile {
    /// Returns the value of the first tag whose key equals `key`, ignoring case.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.tags
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }
}

/// Splits a file name into its stem and lower-cased extension.
fn split_name(file_name: &str) -> (String, String) {
    match file_name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => (stem.to_string(), ext.to_ascii_lowercase()),
        _ => (file_name.to_string(), String::new()),
    }
}

/// Parses one non-blank, trimmed line into `(key, value)` or the reason it is bad.
fn parse_line(line: &str) -> Result<(String, String), IssueCode> {
    let rest = line.strip_prefix('[').ok_or(IssueCode::MalformedLine)?;
    let (key, value) = rest.split_once(']').ok_or(IssueCode::MalformedLine)?;
    let key = key.trim();
    let value = value.trim();
    if key.is_empty() {
        return Err(IssueCode::EmptyKey);
    }
    if value.is_empty() {
        return Err(IssueCode::EmptyValue);
    }
    Ok((key.to_string(), value.to_string()))
}

/// Parses the text of a tag file whose file name (not path) is `file_name`.
///
/// Tolerates a BOM, CRLF line ends, blank lines and spaces around lines.
pub fn parse_tag_file(file_name: &str, content: &str) -> ParsedTag {
    let (name, extension) = split_name(file_name);
    let content = content.strip_prefix(BOM).unwrap_or(content);
    let mut tags = Vec::new();
    let mut issues = Vec::new();
    for (index, raw) in split_lines(content).enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        match parse_line(line) {
            Ok(pair) => tags.push(pair),
            Err(code) => issues.push(TagIssue {
                line: index + 1,
                code,
            }),
        }
    }
    ParsedTag {
        file: TagFile {
            name,
            extension,
            tags,
        },
        issues,
    }
}

/// Turns two bytes of a UTF-16 file into a code unit, in the file's byte order.
type UnitReader = fn([u8; 2]) -> u16;

/// Splits text into lines at `\n`, `\r\n` and a lone `\r` (old Mac editors), so the line numbers
/// of the issues match what an editor shows.
fn split_lines(content: &str) -> impl Iterator<Item = &str> {
    let mut rest = Some(content);
    std::iter::from_fn(move || {
        let text = rest?;
        match text.find(['\r', '\n']) {
            Some(at) => {
                let skip = if text[at..].starts_with("\r\n") { 2 } else { 1 };
                rest = Some(&text[at + skip..]);
                Some(&text[..at])
            }
            None => {
                rest = None;
                (!text.is_empty()).then_some(text)
            }
        }
    })
}

/// Text of the bytes of a tag file: UTF-16 when it starts with a UTF-16 byte order mark (what
/// Windows Notepad writes as "Unicode"), else UTF-8 with invalid bytes replaced.
pub fn decode_tag_bytes(bytes: &[u8]) -> String {
    let (rest, from_pair): (&[u8], UnitReader) = if bytes.starts_with(&UTF16_LE_BOM) {
        (&bytes[2..], u16::from_le_bytes)
    } else if bytes.starts_with(&UTF16_BE_BOM) {
        (&bytes[2..], u16::from_be_bytes)
    } else {
        return String::from_utf8_lossy(bytes).into_owned();
    };
    let units = rest.chunks_exact(2).map(|c| from_pair([c[0], c[1]]));
    char::decode_utf16(units)
        .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect()
}

/// Reads and parses a tag file from disk. Invalid UTF-8 is replaced, not rejected; UTF-16 files
/// with a byte order mark are decoded.
pub fn read_tag_file(path: &Path) -> Result<ParsedTag, TagError> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or(TagError::NoFileName)?;
    let bytes = std::fs::read(path)?;
    Ok(parse_tag_file(file_name, &decode_tag_bytes(&bytes)))
}

/// Name of the folder that contains `path`, to pass as the platform id to [`TagFile::kind`].
pub fn parent_folder_name(path: &Path) -> Option<String> {
    path.parent()?.file_name()?.to_str().map(str::to_string)
}
