//! Checked-in results come from the unchanged original C# criteria and types,
//! rather than a second Rust implementation of the rules under test.
use observatory_domain::JournalEvent;
use observatory_plugins::explorer::{default_settings, Explorer};
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize, PartialEq)]
struct ExpectedFinding {
    body: String,
    title: String,
    detail: String,
}

fn all_scan_checks() -> Explorer {
    let mut settings = default_settings();
    for (name, value) in settings.as_object_mut().unwrap() {
        if value.is_boolean()
            && !matches!(
                name.as_str(),
                "OnlyShowCurrentSystem" | "EnableCustomCriteria"
            )
        {
            *value = true.into();
        }
    }
    Explorer::new(settings, None).unwrap()
}

#[test]
fn original_source_criteria_match_exactly_across_59_contexts() {
    let mut explorer = all_scan_checks();
    let mut contexts = HashSet::new();
    let mut actual = Vec::new();
    for (line_number, line) in include_str!("../../fixtures/oracle-cases.jsonl")
        .lines()
        .enumerate()
    {
        let event = JournalEvent::parse(line)
            .unwrap_or_else(|error| panic!("oracle fixture line {}: {error}", line_number + 1));
        contexts.insert(event.id("SystemAddress").unwrap());
        actual.extend(
            explorer
                .on_event(&event)
                .into_iter()
                .map(|finding| ExpectedFinding {
                    body: finding.body,
                    title: finding.title,
                    detail: finding.detail,
                }),
        );
    }
    let expected: Vec<ExpectedFinding> =
        serde_json::from_str(include_str!("../../fixtures/oracle-expected.json")).unwrap();
    assert_eq!(
        contexts.len(),
        59,
        "the independent source contexts must not merge"
    );
    assert_eq!(
        expected.len(),
        40,
        "the captured source oracle contains 40 findings"
    );
    assert_eq!(
        actual, expected,
        "body, description, detail, and ordering must match the original C# source oracle"
    );
}

#[test]
fn existing_session_matches_source_and_handles_empty_root_parents() {
    let mut explorer = all_scan_checks();
    let mut actual = Vec::new();
    for line in include_str!("../../fixtures/parity-journal.jsonl")
        .lines()
        .filter(|line| !line.trim().is_empty())
    {
        let event = JournalEvent::parse(line).unwrap();
        let findings = explorer.on_event(&event);
        // Codex is a separate handler outside DefaultCriteria.CheckInterest.
        if event.event != "CodexEntry" {
            actual.extend(findings.into_iter().map(|finding| ExpectedFinding {
                body: finding.body,
                title: finding.title,
                detail: finding.detail,
            }));
        }
    }
    let expected: Vec<ExpectedFinding> =
        serde_json::from_str(include_str!("../../fixtures/oracle-session-expected.json")).unwrap();
    assert_eq!(expected.len(), 17);
    assert_eq!(actual, expected);
}
