//! A deterministic shared replay covering real journal/Status channel separation.
use observatory_domain::JournalEvent;
use observatory_plugins::{botanist, communicator, explorer};
use serde_json::Value;
use std::collections::BTreeSet;

#[test]
fn bundled_workers_replay_shared_session() {
    let mut settings = explorer::default_settings();
    for key in [
        "HighValueMappable",
        "DiverseLife",
        "GoodFSDBody",
        "GreenSystem",
        "Codex",
        "FastOrbit",
    ] {
        settings[key] = Value::Bool(true);
    }
    let mut explorer = explorer::Explorer::new(settings, None).unwrap();
    let mut botanist = botanist::Botanist::new(botanist::default_settings());
    let mut communicator = communicator::Communicator::new(communicator::default_settings());
    let mut events: Vec<JournalEvent> = include_str!("../../fixtures/parity-journal.jsonl")
        .lines()
        .chain(include_str!("../../fixtures/parity-status.jsonl").lines())
        .map(|line| JournalEvent::parse(line).unwrap())
        .collect();
    events.sort_by(|first, second| first.timestamp.cmp(&second.timestamp));
    let mut titles = BTreeSet::new();
    let mut communicator_alerts = Vec::new();
    let mut checked_sampler = false;
    for event in events {
        if event.event == "Status" {
            botanist.on_status(&event.data);
            if event.timestamp == "2026-01-01T00:00:13Z" {
                let sampler = botanist.sampler().unwrap();
                assert_eq!(sampler.sample, 2);
                assert!(sampler.ready());
                assert!((sampler.first_distance.unwrap() - 1745.329).abs() < 0.01);
                assert!((sampler.second_distance.unwrap() - 872.665).abs() < 0.01);
                checked_sampler = true;
            }
            continue;
        }
        if event.event == "FutureJournalEvent" {
            assert_eq!(event.data["UnrecognizedField"]["nested"], "preserve me");
            assert_eq!(event.id("SystemAddress"), Some(18_446_744_073_709_550_000));
        }
        titles.extend(
            explorer
                .on_event(&event)
                .into_iter()
                .map(|finding| finding.title),
        );
        botanist.on_event(&event);
        communicator_alerts.extend(communicator.on_event(&event));
    }
    for expected in [
        "Landable and Terraformable",
        "Landable with Atmosphere",
        "Landable with High Gravity",
        "Landable Large Planet",
        "Ringed Landable Body",
        "High-Value Body",
        "Wide Ring",
        "Small Object",
        "Shepherd Moon",
        "Diverse Life",
        "5 of 6 Premium Boost Materials",
        "All Premium Boost Materials In System",
        "Fast Orbit",
        "Non-locked Body with Fast Rotation",
        "Landable with Highly Eccentric Orbit",
        "New Organic Structures Codex Entry",
    ] {
        assert!(
            titles.contains(expected),
            "missing expected Explorer finding {expected}; actual {titles:?}"
        );
    }
    assert!(checked_sampler);
    assert!(botanist.sampler().is_none());
    let bio_rows = botanist.rows();
    assert_eq!(bio_rows.len(), 1);
    assert_eq!(bio_rows[0].body, "Observatory Fixture A 1 a");
    assert_eq!(bio_rows[0].bio_total, 8);
    assert!(bio_rows[0].analysed);
    assert_eq!(bio_rows[0].colony_distance, 500);
    assert_eq!(communicator.all_rows().len(), 3);
    assert_eq!(communicator.rows().len(), 2);
    assert_eq!(communicator_alerts.len(), 1);
    assert_eq!(communicator_alerts[0].title, "Survey Pilot");
    assert_eq!(communicator_alerts[0].detail, "Good work");
    assert_eq!(communicator.rows()[0].from, "Parity Commander");
}
