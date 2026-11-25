//! Regression tests of edge cases in title keys, media matching and `gamelist.xml`.

use std::error::Error;
use std::fs;

use zl_media::{
    esde_media_dir, export_gamelist, game_keys, parse_cocoon_name, parse_gamelist, scan_esde,
    strip_accents, title_key, GamelistEntry,
};

/// Result type of the tests.
type Res = Result<(), Box<dyn Error>>;

/// A `gamelist.xml` with one game whose body is `inner`.
fn one_game(inner: &str) -> String {
    format!("<?xml version=\"1.0\"?>\n<gameList><game>{inner}</game></gameList>")
}

#[test]
fn titles_in_other_scripts_keep_a_key() {
    for title in [
        "ドラゴンクエスト",
        "鬼滅の刃",
        "리듬 게임",
        "Ελληνικά",
        "لعبة",
    ] {
        assert!(!title_key(title).is_empty(), "{title}");
    }
    assert_ne!(
        title_key("ドラゴンクエスト"),
        title_key("ファイナルファンタジー")
    );
    assert_eq!(
        title_key("Dragon Quest ドラクエ (Japan)"),
        "dragon quest ドラクエ japan"
    );
}

#[test]
fn kana_voicing_marks_are_not_accents() {
    // が and か are different letters; only Latin-style accents are dropped.
    assert_ne!(title_key("ガ"), title_key("カ"));
    assert_eq!(strip_accents("ガ"), "ガ");
    assert_eq!(strip_accents("한국어"), "한국어");
}

#[test]
fn letters_without_a_decomposition_are_spelled_out() {
    assert_eq!(strip_accents("Straße"), "Strasse");
    assert_eq!(strip_accents("Bjørn Ærø"), "Bjorn AEro");
    assert_eq!(title_key("Łódź Œuvre"), "lodz oeuvre");
    assert_eq!(title_key("Straße"), title_key("Strasse"));
}

#[test]
fn composed_and_decomposed_names_are_the_same_key() {
    let composed = "Pok\u{e9}mon Rouge.gb";
    let decomposed = "Poke\u{301}mon Rouge.gb";
    assert_ne!(composed, decomposed);
    let a = game_keys(&format!("/r/gb/{composed}"), "Pok\u{e9}mon Rouge");
    let b = game_keys(&format!("/r/gb/{decomposed}"), "Poke\u{301}mon Rouge");
    assert_eq!(a, b);
    assert_eq!(a, vec!["pok\u{e9}mon rouge"]);
}

#[test]
fn a_decomposed_file_name_on_disk_matches_the_composed_rom_name() -> Res {
    let dir = tempfile::tempdir()?;
    let covers = dir.path().join("downloaded_media/gb/covers");
    fs::create_dir_all(&covers)?;
    fs::write(covers.join("Poke\u{301}mon Rouge.png"), "x")?;
    let found = scan_esde(dir.path());
    assert_eq!(found.len(), 1);
    let keys = game_keys("/roms/gb/Pok\u{e9}mon Rouge.gb", "Pok\u{e9}mon Rouge");
    assert!(
        keys.contains(&found[0].stem),
        "{keys:?} vs {}",
        found[0].stem
    );
    Ok(())
}

#[test]
fn hidden_files_and_folders_are_not_media() -> Res {
    let dir = tempfile::tempdir()?;
    let covers = dir.path().join("downloaded_media/nes/covers");
    fs::create_dir_all(covers.join(".thumbs"))?;
    fs::create_dir_all(dir.path().join("downloaded_media/.Trashes/covers"))?;
    fs::write(covers.join("Kirby.png"), "x")?;
    fs::write(covers.join("._Kirby.png"), "x")?;
    fs::write(covers.join(".DS_Store"), "x")?;
    fs::write(covers.join(".thumbs/Other.png"), "x")?;
    fs::write(
        dir.path().join("downloaded_media/.Trashes/covers/Bin.png"),
        "x",
    )?;
    let found = scan_esde(dir.path());
    let stems: Vec<&str> = found.iter().map(|f| f.stem.as_str()).collect();
    assert_eq!(stems, vec!["kirby"]);
    Ok(())
}

#[test]
fn the_media_directory_setting_is_unescaped() -> Res {
    let dir = tempfile::tempdir()?;
    fs::create_dir_all(dir.path().join("settings"))?;
    let write = |value: &str| {
        let xml = format!("<string name=\"MediaDirectory\" value=\"{value}\" />");
        fs::write(dir.path().join("settings/es_settings.xml"), xml)
    };
    write("/sdcard/Roms &amp; Media/it&apos;s &#233;")?;
    assert_eq!(
        esde_media_dir(dir.path()).to_str(),
        Some("/sdcard/Roms & Media/it's \u{e9}")
    );
    write("/sdcard/a &unknown; b & c")?;
    assert_eq!(
        esde_media_dir(dir.path()).to_str(),
        Some("/sdcard/a &unknown; b & c")
    );
    Ok(())
}

#[test]
fn an_edit_number_is_plain_digits() {
    let hash = "0123456789abcdef0123456789abcdef";
    let stem = format!("Game__cocoon_edit_+3_{hash}");
    assert_eq!(parse_cocoon_name(&stem), (stem.clone(), None));
    let ok = format!("Game__cocoon_edit_3_{hash}");
    assert_eq!(parse_cocoon_name(&ok), ("Game".to_string(), Some(3)));
}

#[test]
fn gamelist_reads_cdata_and_entities() -> Res {
    let xml = one_game(
        "<path>./A &amp; B.nes</path><name><![CDATA[Fish & <Chips>]]></name>\
         <desc>First line&#10;second &lt;line&gt;</desc>",
    );
    let entries = parse_gamelist(&xml)?;
    assert_eq!(entries[0].path, "./A & B.nes");
    assert_eq!(entries[0].name.as_deref(), Some("Fish & <Chips>"));
    assert_eq!(
        entries[0].desc.as_deref(),
        Some("First line\nsecond <line>")
    );
    Ok(())
}

#[test]
fn gamelist_with_a_bare_ampersand_still_parses() -> Res {
    let entries = parse_gamelist(&one_game(
        "<path>./Tom & Jerry.nes</path><name>Tom & Jerry</name>",
    ))?;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name.as_deref(), Some("Tom & Jerry"));
    Ok(())
}

#[test]
fn gamelist_empty_elements_leave_the_field_unset() -> Res {
    let xml = one_game("<path>./a.nes</path><image/><video></video><desc>   </desc><name>A</name>");
    let e = &parse_gamelist(&xml)?[0];
    assert_eq!(
        (e.image.as_deref(), e.video.as_deref(), e.desc.as_deref()),
        (None, None, None)
    );
    assert_eq!(e.name.as_deref(), Some("A"));
    Ok(())
}

#[test]
fn gamelist_rating_is_a_finite_number() -> Res {
    let rating = |text: &str| -> Result<Option<f32>, String> {
        Ok(parse_gamelist(&one_game(&format!("<path>a</path><rating>{text}</rating>")))?[0].rating)
    };
    assert_eq!(rating("0,85")?, Some(0.85));
    assert_eq!(rating("nan")?, None);
    assert_eq!(rating("inf")?, None);
    assert_eq!(rating("good")?, None);
    Ok(())
}

#[test]
fn gamelist_export_keeps_every_field() -> Res {
    let entry = GamelistEntry {
        path: "./Caf\u{e9} & Co.gba".into(),
        name: Some("Caf\u{e9} <Co>".into()),
        desc: Some("Line one\nline two".into()),
        image: Some("./covers/a.png".into()),
        marquee: Some("./marquees/a.png".into()),
        video: Some("./videos/a.mp4".into()),
        release_date: Some("20040204T000000".into()),
        developer: Some("Dev".into()),
        publisher: Some("Pub".into()),
        genre: Some("Action".into()),
        players: Some("1-4".into()),
        rating: Some(0.75),
    };
    let xml = export_gamelist(std::slice::from_ref(&entry))?;
    assert_eq!(parse_gamelist(&xml)?, vec![entry]);
    Ok(())
}
