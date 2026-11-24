//! Helpers that read typed values out of a JSON object and record problems as [`Issue`]s.

use serde_json::{Map, Value};

use crate::colour::Rgba;
use crate::enums::StringEnum;
use crate::issue::{Issue, IssueCode, Severity};

/// Collects issues while a JSON object is read field by field.
#[derive(Debug, Default)]
pub(crate) struct Reader {
    /// Problems found so far, in reading order.
    pub issues: Vec<Issue>,
}

/// Builds the JSON path of `key` inside the object at `path`.
pub(crate) fn join(path: &str, key: &str) -> String {
    format!("{path}.{key}")
}

impl Reader {
    /// Records a warning.
    pub fn warn(&mut self, code: IssueCode, location: String) {
        self.issues
            .push(Issue::new(Severity::Warning, code, location));
    }

    /// Records an error.
    pub fn error(&mut self, code: IssueCode, location: String) {
        self.issues
            .push(Issue::new(Severity::Error, code, location));
    }

    /// Reads a string field; any other JSON type is a `WrongType` warning and yields `None`.
    pub fn string(&mut self, obj: &Map<String, Value>, path: &str, key: &str) -> Option<String> {
        match obj.get(key)? {
            Value::String(s) => Some(s.clone()),
            _ => {
                self.warn(IssueCode::WrongType, join(path, key));
                None
            }
        }
    }

    /// Reads a boolean field.
    pub fn boolean(&mut self, obj: &Map<String, Value>, path: &str, key: &str) -> Option<bool> {
        match obj.get(key)? {
            Value::Bool(b) => Some(*b),
            _ => {
                self.warn(IssueCode::WrongType, join(path, key));
                None
            }
        }
    }

    /// Reads a number, limiting it to `range` (inclusive) with a `ValueClamped` warning.
    pub fn number(
        &mut self,
        obj: &Map<String, Value>,
        path: &str,
        key: &str,
        range: Option<(f64, f64)>,
    ) -> Option<f64> {
        let Some(value) = obj.get(key)?.as_f64() else {
            self.warn(IssueCode::WrongType, join(path, key));
            return None;
        };
        match range {
            Some((lo, hi)) if value < lo || value > hi => {
                self.warn(IssueCode::ValueClamped, join(path, key));
                Some(value.clamp(lo, hi))
            }
            _ => Some(value),
        }
    }

    /// Reads an enum field, case-insensitively; an unknown value is a warning and yields `None`.
    pub fn enum_value<E: StringEnum>(
        &mut self,
        obj: &Map<String, Value>,
        path: &str,
        key: &str,
    ) -> Option<E> {
        let text = self.string(obj, path, key)?;
        let parsed = E::parse_text(&text);
        if parsed.is_none() {
            self.warn(IssueCode::UnknownEnumValue, join(path, key));
        }
        parsed
    }

    /// Parses a colour value found at `location`; a bad one is a `BadColor` warning.
    pub fn colour(&mut self, value: &Value, location: String) -> Option<Rgba> {
        let parsed = value.as_str().and_then(|s| s.parse::<Rgba>().ok());
        if parsed.is_none() {
            self.warn(IssueCode::BadColor, location);
        }
        parsed
    }

    /// Warns `UnknownKey` for each key of `obj` that is not in `known`.
    pub fn unknown_keys(&mut self, obj: &Map<String, Value>, path: &str, known: &[&str]) {
        for key in obj.keys().filter(|k| !known.contains(&k.as_str())) {
            self.warn(IssueCode::UnknownKey, join(path, key));
        }
    }
}
