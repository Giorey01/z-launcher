//! Parser of `{variable}` templates, shared by the validator and the intent builder.

use crate::error::LaunchError;

/// Variables a template may use, besides `{tags.NAME}`. A format fact: the builder implements
/// exactly these.
pub const KNOWN_VARIABLES: &[&str] = &[
    "rom",
    "rom_uri",
    "rom_saf",
    "rom_saf_tree",
    "rom_provider",
    "rom_name",
    "rom_file",
    "rom_dir",
    "core",
    "core_path",
    "platform",
    "platform_dir",
    "package",
];

/// Prefix of the tag variables.
pub const TAG_PREFIX: &str = "tags.";

/// A piece of a template: text to copy or a variable to substitute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part<'a> {
    /// Literal text.
    Text(&'a str),
    /// A variable name, without the braces.
    Var(&'a str),
}

/// Whether `name` is a variable the builder understands.
pub fn is_known(name: &str) -> bool {
    KNOWN_VARIABLES.contains(&name)
        || name
            .strip_prefix(TAG_PREFIX)
            .is_some_and(|tag| !tag.is_empty())
}

/// Splits a template into text and variables. An unclosed, empty or nested brace is
/// [`LaunchError::BadTemplate`]; an unknown name is [`LaunchError::UnknownVariable`].
pub fn parse(template: &str) -> Result<Vec<Part<'_>>, LaunchError> {
    let mut parts = Vec::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        if open > 0 {
            parts.push(Part::Text(&rest[..open]));
        }
        let after = &rest[open + 1..];
        let close = after
            .find('}')
            .ok_or_else(|| LaunchError::BadTemplate(template.to_string()))?;
        let name = &after[..close];
        if name.is_empty() || name.contains('{') {
            return Err(LaunchError::BadTemplate(template.to_string()));
        }
        if !is_known(name) {
            return Err(LaunchError::UnknownVariable(name.to_string()));
        }
        parts.push(Part::Var(name));
        rest = &after[close + 1..];
    }
    if !rest.is_empty() {
        parts.push(Part::Text(rest));
    }
    Ok(parts)
}

/// Names of the variables a template uses, in order of appearance.
pub fn variables(template: &str) -> Result<Vec<&str>, LaunchError> {
    Ok(parse(template)?
        .into_iter()
        .filter_map(|p| match p {
            Part::Var(name) => Some(name),
            Part::Text(_) => None,
        })
        .collect())
}
