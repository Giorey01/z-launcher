//! Parser and generator for EmulationStation / ES-DE `gamelist.xml` metadata files.

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::io::Cursor;

/// One game entry read from or written to a `gamelist.xml`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GamelistEntry {
    /// Relative or absolute path of the ROM file.
    pub path: String,
    /// Display title of the game.
    pub name: Option<String>,
    /// Synopsis or description.
    pub desc: Option<String>,
    /// Path to cover art image.
    pub image: Option<String>,
    /// Path to marquee / logo image.
    pub marquee: Option<String>,
    /// Path to video preview.
    pub video: Option<String>,
    /// Release date (YYYYMMDDTHHMMSS or YYYY-MM-DD).
    pub release_date: Option<String>,
    /// Developer studio.
    pub developer: Option<String>,
    /// Publisher company.
    pub publisher: Option<String>,
    /// Comma-separated genres.
    pub genre: Option<String>,
    /// Player count string (e.g. "1" or "1-4").
    pub players: Option<String>,
    /// Rating normalized to 0.0 - 1.0.
    pub rating: Option<f32>,
}

/// Parses a `gamelist.xml` document into a list of [`GamelistEntry`].
pub fn parse_gamelist(xml_content: &str) -> Result<Vec<GamelistEntry>, String> {
    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);
    let mut entries = Vec::new();
    let mut current: Option<GamelistEntry> = None;
    let mut current_tag = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "game" {
                    current = Some(GamelistEntry::default());
                } else if current.is_some() {
                    current_tag = name;
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().map_err(|err| err.to_string())?.to_string();
                if let Some(ref mut entry) = current {
                    apply_gamelist_field(entry, &current_tag, &text);
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "game" {
                    if let Some(entry) = current.take() {
                        entries.push(entry);
                    }
                }
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(e.to_string()),
            _ => {}
        }
        buf.clear();
    }

    Ok(entries)
}

/// Helper mapping an XML tag to a [`GamelistEntry`] field.
fn apply_gamelist_field(entry: &mut GamelistEntry, tag: &str, text: &str) {
    match tag {
        "path" => entry.path = text.to_string(),
        "name" => entry.name = Some(text.to_string()),
        "desc" => entry.desc = Some(text.to_string()),
        "image" => entry.image = Some(text.to_string()),
        "marquee" => entry.marquee = Some(text.to_string()),
        "video" => entry.video = Some(text.to_string()),
        "releasedate" => entry.release_date = Some(text.to_string()),
        "developer" => entry.developer = Some(text.to_string()),
        "publisher" => entry.publisher = Some(text.to_string()),
        "genre" => entry.genre = Some(text.to_string()),
        "players" => entry.players = Some(text.to_string()),
        "rating" => entry.rating = text.parse().ok(),
        _ => {}
    }
}

/// Generates a `gamelist.xml` document string from a slice of [`GamelistEntry`].
pub fn export_gamelist(entries: &[GamelistEntry]) -> Result<String, String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Start(BytesStart::new("gameList")))
        .map_err(|e| e.to_string())?;

    for entry in entries {
        write_entry(&mut writer, entry).map_err(|e| e.to_string())?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("gameList")))
        .map_err(|e| e.to_string())?;

    let bytes = writer.into_inner().into_inner();
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

/// Writes a single `<game>` tag with its children.
fn write_entry<W: std::io::Write>(
    writer: &mut Writer<W>,
    entry: &GamelistEntry,
) -> Result<(), quick_xml::Error> {
    writer.write_event(Event::Start(BytesStart::new("game")))?;
    write_tag(writer, "path", &entry.path)?;
    if let Some(ref name) = entry.name {
        write_tag(writer, "name", name)?;
    }
    if let Some(ref desc) = entry.desc {
        write_tag(writer, "desc", desc)?;
    }
    if let Some(ref image) = entry.image {
        write_tag(writer, "image", image)?;
    }
    if let Some(ref dev) = entry.developer {
        write_tag(writer, "developer", dev)?;
    }
    if let Some(ref publ) = entry.publisher {
        write_tag(writer, "publisher", publ)?;
    }
    writer.write_event(Event::End(BytesEnd::new("game")))?;
    Ok(())
}

/// Helper writing an XML tag `<tag>value</tag>`.
fn write_tag<W: std::io::Write>(
    writer: &mut Writer<W>,
    tag: &str,
    value: &str,
) -> Result<(), quick_xml::Error> {
    writer.write_event(Event::Start(BytesStart::new(tag)))?;
    writer.write_event(Event::Text(BytesText::new(value)))?;
    writer.write_event(Event::End(BytesEnd::new(tag)))?;
    Ok(())
}
