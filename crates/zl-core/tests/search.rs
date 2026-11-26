//! Tests for typo-tolerant search against fixtures/search/search.json.

use std::error::Error;
use std::path::Path;

use serde::Deserialize;
use zl_core::FuzzySearch;

/// Root fixture structure.
#[derive(Debug, Deserialize)]
struct SearchFixture {
    /// Available titles.
    titles: Vec<String>,
    /// Test cases to run.
    cases: Vec<TestCase>,
}

/// A single test case.
#[derive(Debug, Deserialize)]
struct TestCase {
    /// Query string.
    q: String,
    /// Title expected to be ranked first.
    #[serde(default)]
    first: Option<String>,
    /// Titles that must be present in the hits.
    #[serde(default)]
    has: Vec<String>,
    /// Titles that must NOT be present in the hits.
    #[serde(default)]
    not: Vec<String>,
    /// True when no matches are expected.
    #[serde(default)]
    none: bool,
}

#[test]
fn test_search_fixture_cases() -> Result<(), Box<dyn Error>> {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("no parent")?
        .parent()
        .ok_or("no parent")?
        .join("fixtures/search/search.json");

    let content = std::fs::read_to_string(fixture_path)?;
    let fixture: SearchFixture = serde_json::from_str(&content)?;

    let search = FuzzySearch::new(&fixture.titles);

    for case in &fixture.cases {
        let indices = search.search(&case.q, 10);
        let results: Vec<&str> = indices
            .iter()
            .map(|&i| fixture.titles[i].as_str())
            .collect();

        if case.none {
            assert!(
                results.is_empty(),
                "query {:?} expected no results, got {:?}",
                case.q,
                results
            );
            continue;
        }

        if let Some(ref expected_first) = case.first {
            assert!(
                !results.is_empty(),
                "query {:?} expected first {:?}, but got 0 results",
                case.q,
                expected_first
            );
            assert_eq!(
                results[0],
                expected_first.as_str(),
                "query {:?} first result mismatch",
                case.q
            );
        }

        for expected_has in &case.has {
            assert!(
                results.contains(&expected_has.as_str()),
                "query {:?} expected to contain {:?}, got {:?}",
                case.q,
                expected_has,
                results
            );
        }

        for expected_not in &case.not {
            assert!(
                !results.contains(&expected_not.as_str()),
                "query {:?} expected NOT to contain {:?}, got {:?}",
                case.q,
                expected_not,
                results
            );
        }
    }

    Ok(())
}
