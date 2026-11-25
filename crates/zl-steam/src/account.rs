//! What the user typed as their Steam account, and the shape of a Web API key.

/// Lowest SteamID64 of an individual account (universe 1, type 1, instance 1). A format fact.
const STEAM_ID64_BASE: u64 = 76_561_197_960_265_728;

/// Length of a Web API key: 32 hexadecimal characters. A format fact.
const WEB_API_KEY_LEN: usize = 32;

/// An account as the user gave it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountInput {
    /// A SteamID64, ready to use.
    SteamId(u64),
    /// A custom profile name, to resolve through the Web API.
    Vanity(String),
}

/// Whether `id` is the SteamID64 of an individual account.
fn is_individual(id: u64) -> bool {
    id > STEAM_ID64_BASE && id < STEAM_ID64_BASE + (1 << 32)
}

/// The part of a profile URL after `/profiles/` or `/id/`, with the kind it is.
fn from_url(text: &str) -> Option<(bool, &str)> {
    let rest = text
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.");
    let rest = rest.strip_prefix("steamcommunity.com/")?;
    let (kind, tail) = rest.split_once('/')?;
    let name = tail.split(['/', '?', '#']).next().unwrap_or_default();
    match kind {
        "profiles" => Some((true, name)),
        "id" => Some((false, name)),
        _ => None,
    }
}

/// Whether `name` can be a custom profile name: 2 to 32 letters, digits, `_` or `-`.
fn is_vanity(name: &str) -> bool {
    (2..=32).contains(&name.len())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Reads an account: a SteamID64, a `steamcommunity.com/profiles/<id>` or `/id/<name>` URL, or a
/// bare custom name. `None` when it is none of these.
pub fn parse_account(text: &str) -> Option<AccountInput> {
    let text = text.trim();
    let (numeric, value) = from_url(text).unwrap_or((true, text));
    if let Ok(id) = value.parse::<u64>() {
        return is_individual(id).then_some(AccountInput::SteamId(id));
    }
    if numeric && from_url(text).is_some() {
        return None;
    }
    is_vanity(value).then(|| AccountInput::Vanity(value.to_string()))
}

/// Whether `key` has the shape of a Web API key (32 hexadecimal characters).
pub fn is_web_api_key(key: &str) -> bool {
    let key = key.trim();
    key.len() == WEB_API_KEY_LEN && key.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ids, both URL forms and bare names are read; anything else is refused.
    #[test]
    fn accounts_are_read_from_every_form() {
        let id = AccountInput::SteamId(76_561_197_960_287_930);
        assert_eq!(parse_account(" 76561197960287930 "), Some(id.clone()));
        assert_eq!(
            parse_account("https://steamcommunity.com/profiles/76561197960287930/"),
            Some(id)
        );
        assert_eq!(
            parse_account("steamcommunity.com/id/gabelogannewell?l=it"),
            Some(AccountInput::Vanity("gabelogannewell".into()))
        );
        assert_eq!(
            parse_account("my_name"),
            Some(AccountInput::Vanity("my_name".into()))
        );
        assert_eq!(parse_account("12345"), None, "not an individual id");
        assert_eq!(
            parse_account("https://steamcommunity.com/profiles/abc"),
            None
        );
        assert_eq!(parse_account("https://example.com/id/x"), None);
        assert_eq!(parse_account("two words"), None);
        assert_eq!(parse_account(""), None);
    }

    /// A key is exactly 32 hexadecimal characters.
    #[test]
    fn keys_are_32_hex_characters() {
        assert!(is_web_api_key("0123456789ABCDEF0123456789abcdef"));
        assert!(!is_web_api_key("0123456789ABCDEF0123456789abcde"));
        assert!(!is_web_api_key("0123456789ABCDEF0123456789abcdeg"));
    }
}
