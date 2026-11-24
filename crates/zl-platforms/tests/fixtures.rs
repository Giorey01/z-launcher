//! Runs `fixtures/launch/expected.json` through detection, resolution and intent building.

use std::error::Error;
use std::path::Path;

use serde_json::Value;
use zl_platforms::{LaunchRequest, Outcome, PlatformDb, ProfileRef};

/// Result type of the tests.
type Res = Result<(), Box<dyn Error>>;

/// Loads the fixture document.
fn load() -> Result<Value, Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/launch/expected.json");
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

/// Package names of a case: an array, or the name of a shared list in the document.
fn packages(doc: &Value, case: &Value) -> Result<Vec<String>, Box<dyn Error>> {
    let v = &case["installed_packages"];
    let list = match v.as_str() {
        Some(name) => &doc[name],
        None => v,
    };
    Ok(serde_json::from_value(list.clone())?)
}

#[test]
fn launch_cases_match_expected_outcomes() -> Res {
    let doc = load()?;
    let db = PlatformDb::embedded()?;
    let cases = doc["cases"].as_array().ok_or("no cases")?;
    assert!(cases.len() >= 8, "fixture lost cases");
    for case in cases {
        let installed = db.detect_installed(&packages(&doc, case)?).installed;
        let request: LaunchRequest = serde_json::from_value(case["request"].clone())?;
        let expected: Outcome = serde_json::from_value(case["expected"].clone())?;
        let got = db.plan_launch(&installed, &request)?;
        assert_eq!(got, expected, "case: {}", case["name"]);
    }
    Ok(())
}

#[test]
fn error_cases_report_their_code() -> Res {
    let doc = load()?;
    let db = PlatformDb::embedded()?;
    for case in doc["error_cases"].as_array().ok_or("no error cases")? {
        let installed = db.detect_installed(&packages(&doc, case)?).installed;
        let request: LaunchRequest = serde_json::from_value(case["request"].clone())?;
        let err = db
            .plan_launch(&installed, &request)
            .err()
            .ok_or("expected an error")?;
        assert_eq!(
            err.to_string(),
            case["expected_error"].as_str().ok_or("no code")?
        );
    }
    Ok(())
}

#[test]
fn detection_of_the_device_package_list() -> Res {
    let doc = load()?;
    let db = PlatformDb::embedded()?;
    for case in doc["detect_cases"].as_array().ok_or("no detect cases")? {
        let det = db.detect_installed(&packages(&doc, case)?);
        let expected = case["expected_installed"]
            .as_object()
            .ok_or("no installed")?;
        assert_eq!(det.installed.len(), expected.len(), "installed set differs");
        for (id, pkgs) in expected {
            let found = det.installed.iter().find(|i| &i.emulator == id);
            let found = found.ok_or_else(|| format!("{id} not detected"))?;
            assert_eq!(serde_json::to_value(&found.packages)?, *pkgs, "{id}");
        }
        for (platform, want) in case["expected_defaults"].as_object().ok_or("no defaults")? {
            let want: ProfileRef = serde_json::from_value(want.clone())?;
            assert_eq!(
                det.proposed_defaults.get(platform),
                Some(&want),
                "{platform}"
            );
        }
        for platform in case["expected_no_default"].as_array().ok_or("no list")? {
            let name = platform.as_str().ok_or("not a string")?;
            assert!(
                !det.proposed_defaults.contains_key(name),
                "{name} has a default"
            );
        }
    }
    Ok(())
}
