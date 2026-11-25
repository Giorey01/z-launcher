//! Regression tests of tag file edge cases: line endings, encodings and odd lines.

use zl_tags::{decode_tag_bytes, parse_tag_file, read_tag_file, IssueCode, TagKind};

/// UTF-16 little-endian bytes of `text`, with a byte order mark.
fn utf16_le(text: &str) -> Vec<u8> {
    let mut out = vec![0xff, 0xfe];
    for unit in text.encode_utf16() {
        out.extend(unit.to_le_bytes());
    }
    out
}

/// UTF-16 big-endian bytes of `text`, with a byte order mark.
fn utf16_be(text: &str) -> Vec<u8> {
    let mut out = vec![0xfe, 0xff];
    for unit in text.encode_utf16() {
        out.extend(unit.to_be_bytes());
    }
    out
}

#[test]
fn a_lone_carriage_return_ends_a_line() {
    let parsed = parse_tag_file("G.steamappid", "[steamappid] 10\r[name] Caf\u{e9}\r");
    assert_eq!(
        parsed.file.tags,
        vec![
            ("steamappid".to_string(), "10".to_string()),
            ("name".to_string(), "Caf\u{e9}".to_string())
        ]
    );
    assert!(parsed.issues.is_empty());
}

#[test]
fn crlf_counts_as_one_line_break_for_issue_numbers() {
    let parsed = parse_tag_file("G.dpt", "[a] 1\r\n\r\nbad\r\n[] x\r\n");
    let lines: Vec<(usize, &str)> = parsed
        .issues
        .iter()
        .map(|i| (i.line, i.code.as_str()))
        .collect();
    assert_eq!(lines, vec![(3, "malformed_line"), (4, "empty_key")]);
    assert_eq!(parsed.issues[1].code, IssueCode::EmptyKey);
}

#[test]
fn utf16_files_with_a_byte_order_mark_are_decoded() {
    let text = "[localgameid] com.exemple.jeu\r\n[name] \u{c9}t\u{e9}\r\n";
    assert_eq!(decode_tag_bytes(&utf16_le(text)), text);
    assert_eq!(decode_tag_bytes(&utf16_be(text)), text);
    assert_eq!(decode_tag_bytes(text.as_bytes()), text);
    let parsed = parse_tag_file("Jeu.localgameid", &decode_tag_bytes(&utf16_le(text)));
    assert_eq!(parsed.file.get("localgameid"), Some("com.exemple.jeu"));
    assert_eq!(parsed.file.get("NAME"), Some("\u{c9}t\u{e9}"));
    assert_eq!(parsed.file.kind(None), TagKind::LocalGameId);
}

#[test]
fn a_cut_or_broken_utf16_file_does_not_panic() {
    let mut bytes = utf16_le("[a] b");
    bytes.push(0x41);
    assert_eq!(decode_tag_bytes(&bytes), "[a] b");
    let lone_surrogate = [0xff, 0xfe, 0x00, 0xd8];
    assert_eq!(decode_tag_bytes(&lone_surrogate), "\u{fffd}");
}

#[test]
fn read_tag_file_decodes_utf16_from_disk() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join("Notepad Game.steamappid");
    std::fs::write(&path, utf16_le("[steamappid] 220\r\n"))?;
    let parsed = read_tag_file(&path)?;
    std::fs::remove_file(&path)?;
    assert_eq!(parsed.file.name, "Notepad Game");
    assert_eq!(parsed.file.get("steamappid"), Some("220"));
    assert!(parsed.issues.is_empty());
    Ok(())
}
