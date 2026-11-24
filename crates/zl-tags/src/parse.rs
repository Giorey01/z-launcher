//! Parsing of the text of a tag file into a [`TagFile`] plus the problems found.

use std::path::Path;

/// Byte order mark some editors put at the start of a UTF-8 file.
const BOM: char = '\u{feff}';

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
    for (index, raw) in content.lines().enumerate() {
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

/// Reads and parses a tag file from disk. Invalid UTF-8 is replaced, not rejected.
pub fn read_tag_file(path: &Path) -> Result<ParsedTag, TagError> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or(TagError::NoFileName)?;
    let bytes = std::fs::read(path)?;
    Ok(parse_tag_file(file_name, &String::from_utf8_lossy(&bytes)))
}

/// Name of the folder that contains `path`, to pass as the platform id to [`TagFile::kind`].
pub fn parent_folder_name(path: &Path) -> Option<String> {
    path.parent()?.file_name()?.to_str().map(str::to_string)
}
