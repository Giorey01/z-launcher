//! Integration tests validating `zl-pods` against `fixtures/pods/`.

use serde_json::Value;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use zl_db::Db;
use zl_pods::{
    evaluate_policy, inspect_package_bytes, parse_index, parse_manifest, PodInstaller, PodVersion,
};

type Res = Result<(), Box<dyn Error>>;

fn fixtures_dir() -> Result<PathBuf, Box<dyn Error>> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("no parent 1")?
        .parent()
        .ok_or("no parent 2")?
        .join("fixtures")
        .join("pods");
    Ok(p)
}

#[test]
fn test_manifest_fixtures() -> Res {
    let dir = fixtures_dir()?;
    let content = fs::read_to_string(dir.join("expected.json"))?;
    let expected_json: Value = serde_json::from_str(&content)?;
    let manifests_expected = expected_json["manifests"]
        .as_object()
        .ok_or("expected manifests object")?;

    for (file_name, expected_val) in manifests_expected {
        let manifest_bytes = fs::read(dir.join("manifests").join(file_name))?;

        if expected_val.get("ok").and_then(Value::as_bool) == Some(true) {
            let parsed = parse_manifest(&manifest_bytes)
                .map_err(|e| format!("manifest {file_name} should be valid, got issues: {e:?}"))?;
            let exp_id = expected_val["id"].as_str().ok_or("expected id")?;
            let exp_ver = expected_val["version"].as_str().ok_or("expected version")?;
            assert_eq!(parsed.id, exp_id, "{file_name} id mismatch");
            assert_eq!(
                parsed.version.to_string(),
                exp_ver,
                "{file_name} version mismatch"
            );
        } else {
            let issues_arr = expected_val["issues"].as_array().ok_or("expected issues")?;
            let mut expected_issues = Vec::new();
            for v in issues_arr {
                expected_issues.push(v.as_str().ok_or("expected issue str")?);
            }

            match parse_manifest(&manifest_bytes) {
                Ok(m) => {
                    return Err(format!(
                        "manifest {file_name} should fail with {expected_issues:?}, but parsed as {m:?}"
                    )
                    .into());
                }
                Err(actual_issues) => {
                    let actual_codes: Vec<&str> =
                        actual_issues.iter().map(|i| i.as_code()).collect();
                    for expected_issue in expected_issues {
                        assert!(
                            actual_codes.contains(&expected_issue),
                            "{file_name}: expected issue {expected_issue} in {actual_codes:?}"
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
fn test_package_fixtures() -> Res {
    let dir = fixtures_dir()?;
    let content = fs::read_to_string(dir.join("expected.json"))?;
    let expected_json: Value = serde_json::from_str(&content)?;
    let packages_expected = expected_json["packages"]
        .as_object()
        .ok_or("expected packages object")?;

    for (file_name, expected_val) in packages_expected {
        let package_bytes = fs::read(dir.join("packages").join(file_name))?;

        if expected_val.get("ok").and_then(Value::as_bool) == Some(true) {
            let parsed = inspect_package_bytes(&package_bytes)
                .map_err(|e| format!("package {file_name} should be valid, got issues: {e:?}"))?;
            let exp_id = expected_val["id"].as_str().ok_or("expected id")?;
            let exp_ver = expected_val["version"].as_str().ok_or("expected ver")?;
            let exp_sig = expected_val["signer"].as_str().ok_or("expected signer")?;
            assert_eq!(parsed.id, exp_id, "{file_name} id mismatch");
            assert_eq!(
                parsed.version.to_string(),
                exp_ver,
                "{file_name} version mismatch"
            );
            assert_eq!(parsed.signer, exp_sig, "{file_name} signer mismatch");
        } else {
            let issues_arr = expected_val["issues"].as_array().ok_or("expected issues")?;
            let mut expected_issues = Vec::new();
            for v in issues_arr {
                expected_issues.push(v.as_str().ok_or("expected str")?);
            }

            match inspect_package_bytes(&package_bytes) {
                Ok(p) => {
                    return Err(format!(
                        "package {file_name} should fail with {expected_issues:?}, but parsed as {p:?}"
                    )
                    .into());
                }
                Err(actual_issues) => {
                    let actual_codes: Vec<&str> =
                        actual_issues.iter().map(|i| i.as_code()).collect();
                    for expected_issue in expected_issues {
                        assert!(
                            actual_codes.contains(&expected_issue),
                            "{file_name}: expected issue {expected_issue} in {actual_codes:?}"
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
fn test_policy_fixtures() -> Res {
    let dir = fixtures_dir()?;
    let content = fs::read_to_string(dir.join("policy.json"))?;
    let policy_cases: Value = serde_json::from_str(&content)?;

    for case in policy_cases.as_array().ok_or("expected array")? {
        let cand_ver_str = case["candidate"]["version"].as_str().ok_or("cand ver")?;
        let candidate_version = PodVersion::parse(cand_ver_str)?;
        let candidate_signer = case["candidate"]["signer"].as_str().ok_or("cand signer")?;

        let installed_info = if case["installed"].is_null() {
            None
        } else {
            let inst_ver_str = case["installed"]["version"].as_str().ok_or("inst ver")?;
            let v = PodVersion::parse(inst_ver_str)?;
            let s = case["installed"]["signer"].as_str().ok_or("inst signer")?;
            Some((v, s))
        };

        let outcome = evaluate_policy(
            installed_info.as_ref().map(|(v, s)| (v, *s)),
            &candidate_version,
            candidate_signer,
        );

        let expected_str = case["expected"].as_str().ok_or("expected str")?;
        assert_eq!(
            outcome.as_str(),
            expected_str,
            "Policy mismatch for case {case:?}"
        );
    }
    Ok(())
}

#[test]
fn test_index_fixtures() -> Res {
    let dir = fixtures_dir()?;
    let content = fs::read_to_string(dir.join("expected.json"))?;
    let expected_json: Value = serde_json::from_str(&content)?;
    let indexes_expected = expected_json["indexes"]
        .as_object()
        .ok_or("expected indexes object")?;

    for (file_name, expected_val) in indexes_expected {
        let index_bytes = fs::read(dir.join("indexes").join(file_name))?;
        let issues_arr = expected_val
            .get("issues")
            .and_then(Value::as_array)
            .ok_or("expected issues array")?;
        let is_doc_error = issues_arr.iter().any(|i| i.get("entry").is_none());

        if is_doc_error {
            let expected_code = issues_arr[0]["code"].as_str().ok_or("code str")?;
            match parse_index(&index_bytes) {
                Ok(_) => {
                    return Err(
                        format!("index {file_name} should fail with {expected_code}").into(),
                    );
                }
                Err(issue) => {
                    assert_eq!(
                        issue.issue.as_code(),
                        expected_code,
                        "{file_name} document issue mismatch"
                    );
                }
            }
        } else {
            let parsed = parse_index(&index_bytes)
                .map_err(|e| format!("index {file_name} should parse, got {e:?}"))?;

            if let Some(kept_arr) = expected_val.get("kept").and_then(Value::as_array) {
                assert_eq!(
                    parsed.entries.len(),
                    kept_arr.len(),
                    "{file_name} kept count mismatch"
                );
                for (idx, kept_expected) in kept_arr.iter().enumerate() {
                    let entry = &parsed.entries[idx];
                    let exp_id = kept_expected["id"].as_str().ok_or("id str")?;
                    assert_eq!(entry.id, exp_id);
                    let exp_flag = kept_expected["flag"].as_str();
                    assert_eq!(entry.flag.map(|f| f.as_code()), exp_flag);
                }
            }

            assert_eq!(
                parsed.issues.len(),
                issues_arr.len(),
                "{file_name} issue count mismatch"
            );
            for (idx, issue_expected) in issues_arr.iter().enumerate() {
                let actual = &parsed.issues[idx];
                assert_eq!(
                    actual.entry,
                    issue_expected
                        .get("entry")
                        .and_then(Value::as_u64)
                        .map(|n| n as usize)
                );
                let exp_code = issue_expected["code"].as_str().ok_or("code str")?;
                assert_eq!(actual.issue.as_code(), exp_code);
            }
        }
    }
    Ok(())
}

#[test]
fn test_install_fixtures() -> Res {
    let dir = fixtures_dir()?;
    let content = fs::read_to_string(dir.join("expected.json"))?;
    let expected_json: Value = serde_json::from_str(&content)?;
    let installs = expected_json["installs"].as_array().ok_or("installs arr")?;

    for install_case in installs {
        let index_file = install_case["index"].as_str().ok_or("index str")?;
        let package_file = install_case["package"].as_str().ok_or("pkg str")?;
        let expected_val = &install_case["expected"];

        let index_bytes = fs::read(dir.join("indexes").join(index_file))?;
        let parsed_index = parse_index(&index_bytes)
            .map_err(|e| format!("index {index_file} parse error: {e:?}"))?;
        let target_id = install_case["id"].as_str().ok_or("id str")?;
        let index_entry = parsed_index
            .entries
            .iter()
            .find(|e| e.id == target_id)
            .ok_or("pod id must be in index")?;

        let package_path = dir.join("packages").join(package_file);

        let temp_deploy = tempfile::tempdir()?;
        let db = Db::open_in_memory()?;
        let installer = PodInstaller::new(&db, temp_deploy.path());

        let result = installer.install(
            &package_path,
            Some(&index_entry.sha256),
            Some(&index_entry.signer),
        );

        if expected_val.get("ok").and_then(Value::as_bool) == Some(true) {
            assert!(
                result.is_ok(),
                "{install_case:?} should succeed, got {result:?}"
            );
        } else {
            let issues_arr = expected_val["issues"].as_array().ok_or("issues arr")?;
            let mut expected_issues = Vec::new();
            for v in issues_arr {
                expected_issues.push(v.as_str().ok_or("issue str")?);
            }

            match result {
                Ok(p) => {
                    return Err(format!(
                        "{install_case:?} should fail with {expected_issues:?}, got {p:?}"
                    )
                    .into());
                }
                Err(zl_pods::PodError::Validation(actual_issues)) => {
                    let actual_codes: Vec<&str> =
                        actual_issues.iter().map(|i| i.as_code()).collect();
                    for exp in expected_issues {
                        assert!(
                            actual_codes.contains(&exp),
                            "{install_case:?}: expected {exp} in {actual_codes:?}"
                        );
                    }
                }
                Err(other) => return Err(format!("unexpected error type: {other:?}").into()),
            }
        }
    }
    Ok(())
}
