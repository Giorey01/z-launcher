//! Runs the shared case set `fixtures/library` through the scanner and compares every game.

mod common;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use common::{quiet_scan, root, TestResult};
use serde_json::Value;
use zl_db::{Db, GameQuery};
use zl_library::{ExtensionRules, FolderGameRule};

/// Directory with the shared fixtures.
fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/library")
}

/// Reads `expected.json`.
fn load_expected() -> Result<Value, Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(fixtures_dir().join("expected.json"))?;
    Ok(serde_json::from_str(&text)?)
}

/// Builds the rules described by the expected file.
fn rules_from(doc: &Value) -> Result<ExtensionRules, Box<dyn std::error::Error>> {
    let mut rules = ExtensionRules::new();
    for (platform, exts) in doc["rules"].as_object().ok_or("rules")? {
        let exts: Vec<&str> = exts
            .as_array()
            .ok_or("exts")?
            .iter()
            .filter_map(Value::as_str)
            .collect();
        rules.insert(platform, &exts);
    }
    for rule in doc["folder_rules"].as_array().ok_or("folder_rules")? {
        rules.add_folder_rule(FolderGameRule {
            platform: rule["platform"].as_str().ok_or("platform")?.to_string(),
            launch_glob: rule["launch_glob"].as_str().ok_or("glob")?.to_string(),
        });
    }
    Ok(rules)
}

/// One line describing an expected game.
fn describe_expected(game: &Value) -> String {
    let files: Vec<&str> = game["files"]
        .as_array()
        .map_or(vec![], |a| a.iter().filter_map(Value::as_str).collect());
    format!(
        "{}|{}|{}|{}|{}|{}",
        game["platform"].as_str().unwrap_or(""),
        game["title"].as_str().unwrap_or(""),
        game["kind"].as_str().unwrap_or(""),
        game["primary"].as_str().unwrap_or(""),
        files.join(";"),
        game["tags"]
    )
}

/// Every game of the fixture tree is found as described, nothing else is, and a second scan
/// changes nothing.
#[test]
fn every_expected_game_is_found_and_only_those() -> TestResult {
    let doc = load_expected()?;
    let rules = rules_from(&doc)?;
    let tree = fixtures_dir().join("tree");
    let prefix = format!("{}/", tree.to_string_lossy());
    let db = Db::open_in_memory()?;
    let summary = quiet_scan(&db, &[root(&tree)], &rules)?;
    assert_eq!(
        u64::from(summary.errors),
        doc["expected_errors"].as_u64().ok_or("errors")?
    );
    let mut got = BTreeSet::new();
    let query = GameQuery {
        include_missing: true,
        ..GameQuery::default()
    };
    for game in db.query_games(&query)? {
        let mut files: Vec<String> = db
            .game_files(game.id)?
            .into_iter()
            .map(|f| f.path.replace(&prefix, ""))
            .collect();
        files.sort();
        let tags: Vec<Value> = db
            .game_tags(game.id)?
            .into_iter()
            .map(|(k, v)| serde_json::json!([k, v]))
            .collect();
        let tags = if tags.is_empty() {
            Value::Null
        } else {
            Value::Array(tags)
        };
        got.insert(format!(
            "{}|{}|{}|{}|{}|{}",
            game.platform_id,
            game.title,
            game.kind.as_str(),
            game.path.replace(&prefix, ""),
            files.join(";"),
            tags
        ));
    }
    let expected: BTreeSet<String> = doc["games"]
        .as_array()
        .ok_or("games")?
        .iter()
        .map(describe_expected)
        .collect();
    assert!(expected.len() >= 15, "fixture must list the cases");
    let missing: Vec<_> = expected.difference(&got).collect();
    let extra: Vec<_> = got.difference(&expected).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "missing: {missing:#?}\nunexpected: {extra:#?}"
    );
    let again = quiet_scan(&db, &[root(&tree)], &rules)?;
    assert_eq!((again.added, again.updated, again.missing), (0, 0, 0));
    assert_eq!(usize::try_from(again.unchanged)?, expected.len());
    Ok(())
}
