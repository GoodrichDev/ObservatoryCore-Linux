//! Loaded as an Explorer unit-test module so production history maps can be
//! inspected without global environment changes or test-only public APIs.
#![cfg(target_os = "linux")]
use super::*;

fn runtime() -> PathBuf {
    if let Some(path) = std::env::var_os("OBSERVATORY_CLR_CRITERIA_HOST") {
        return PathBuf::from(path);
    }
    PathBuf::from(std::env::var_os("HOME").unwrap())
        .join(".cache/observatory-clr-criteria-net8/observatory-clr-criteria")
}

fn native_explorer(source: &str, label: &str) -> (Explorer, PathBuf) {
    let path =
        std::env::temp_dir().join(format!("clr-explorer-{label}-{}.lua", std::process::id()));
    std::fs::write(&path, source).unwrap();
    let mut settings = default_settings();
    for value in settings.as_object_mut().unwrap().values_mut() {
        if value.is_boolean() {
            *value = json!(false);
        }
    }
    let mut explorer = Explorer::prepare_preferences(settings).unwrap();
    explorer.settings["EnableCustomCriteria"] = json!(true);
    explorer.criteria = Some(Criteria::with_managed_runtime(path.clone(), &runtime()).unwrap());
    explorer.current_system = "CLR Oracle".into();
    explorer.current_address = 42;
    (explorer, path)
}

fn scan(id: u64, parents: Value) -> JournalEvent {
    JournalEvent::parse(&json!({"timestamp":"2026-10-26T01:02:03Z","event":"Scan","SystemAddress":42,"StarSystem":"CLR Oracle","BodyID":id,"BodyName":format!("CLR Oracle {id}"),"Parents":parents,"Radius":1234}).to_string()).unwrap()
}

#[test]
fn clr_mutations_round_trip_into_explorer_history_under_original_keys() {
    let source = r#"---@Complex mutation
if scan.BodyID == 2 then
  savedScan = scan
  scan.BodyName = "mutated current body"
  scan.BodyID = 22
  scan.Radius = 4321
  scan.Parent = parents[0].Scan.Parent
  parents[0].Scan.BodyID = 17
  parents[0].Scan.BodyName = "mutated parent body"
  parents[0].Scan.Radius = 99
  return true, "mutation", "first"
elseif scan.BodyID == 3 then
  return true, "retained", savedScan.BodyID .. ";" .. parents[0].Scan.BodyID .. ";" .. parents[0].Scan.Radius .. ";" .. savedScan.Parent[0].Item1:ToString() .. ";" .. savedScan.Parents[0].Planet
end
return false, "", ""
---@End
"#;
    let (mut explorer, path) = native_explorer(source, "history");
    assert!(explorer.on_event(&scan(0, json!([]))).is_empty());
    assert!(explorer.on_event(&scan(1, json!([{"Star":0}]))).is_empty());
    explorer.settings["LandableLarge"] = json!(true);
    let mut incoming = scan(2, json!([{"Planet":1},{"Star":0}]));
    incoming.data["Landable"] = json!(true);
    incoming.data["Radius"] = json!(19_000_000);
    incoming.raw_json = None;
    let findings = explorer.on_event(&incoming);
    assert!(findings
        .iter()
        .any(|finding| finding.title == "Landable Large Planet"));
    assert!(
        findings
            .iter()
            .all(|finding| finding.body == "mutated current body"),
        "source formats built-in and custom results after custom scan property mutation"
    );
    let history = &explorer.systems[&42];
    assert_eq!(history[&2]["BodyID"], 22);
    assert_eq!(history[&2]["Radius"].as_f64(), Some(4321.0));
    assert_eq!(history[&1]["BodyID"], 17);
    assert_eq!(history[&1]["Radius"].as_f64(), Some(99.0));
    assert!(!history.contains_key(&22) && !history.contains_key(&17));
    assert_eq!(history[&2]["__clr_parent"], json!([{"Star":0}]));
    assert_eq!(history[&2]["Parents"][0]["Planet"], 1);
    let next = explorer.on_event(&scan(3, json!([{"Planet":1},{"Star":0}])));
    assert_eq!(
        next.iter()
            .find(|finding| finding.title == "retained")
            .unwrap()
            .detail,
        "22;17;99.0;Star;1"
    );
    assert!(
        explorer.on_event(&incoming).is_empty(),
        "duplicate suppression uses original dictionary key 2"
    );
    assert!(explorer.take_diagnostics().is_empty());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn clr_signal_mutations_feed_next_native_default_criteria_check() {
    let inputs: Value =
        serde_json::from_str(include_str!("../../../fixtures/clr-criteria-inputs.json")).unwrap();
    let case = inputs
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "mutable-signal-identity-original-key")
        .unwrap();
    let (mut explorer, path) = native_explorer(case["source"].as_str().unwrap(), "signals");
    explorer.settings["DiverseLife"] = json!(true);
    let signal = JournalEvent::parse(case["steps"][0]["raw_json"].as_str().unwrap()).unwrap();
    assert!(explorer.on_event(&signal).is_empty());
    assert_eq!(explorer.signals[&42][&2]["BodyID"], 999);
    assert_eq!(explorer.signals[&42][&2]["Signals"][0]["Count"], 9);
    assert!(!explorer.signals[&42].contains_key(&999));
    let event = JournalEvent::parse(case["steps"][1]["raw_json"].as_str().unwrap()).unwrap();
    let findings = explorer.on_event(&event);
    assert_eq!(
        findings
            .iter()
            .find(|finding| finding.title == "Diverse Life")
            .unwrap()
            .detail,
        "Biological Signals: 9"
    );
    assert_eq!(
        findings
            .iter()
            .find(|finding| finding.title == "signal-mutation")
            .unwrap()
            .detail,
        "9;999"
    );
    assert!(explorer.take_diagnostics().is_empty());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn idle_clr_callback_mutations_update_native_explorer_history_before_next_scan() {
    let inputs: Value =
        serde_json::from_str(include_str!("../../../fixtures/clr-criteria-inputs.json")).unwrap();
    let case = inputs
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "idle-timer-mutates-retained-scan")
        .unwrap();
    let (mut explorer, path) = native_explorer(case["source"].as_str().unwrap(), "idle");
    let first = JournalEvent::parse(case["steps"][0]["raw_json"].as_str().unwrap()).unwrap();
    let first_rows = explorer.on_event(&first);
    assert_eq!(first_rows.len(), 1);
    std::thread::sleep(std::time::Duration::from_millis(1000));
    let idle = explorer.poll_custom();
    assert_eq!(idle.len(), 1);
    assert_eq!(idle[0].body, "idle-mutation");
    assert_eq!(explorer.systems[&42][&2]["BodyName"], "timer-mutated body");
    assert_eq!(explorer.systems[&42][&2]["Radius"].as_f64(), Some(888.0));
    let payload = explorer.take_notification_payload(&idle[0]).unwrap();
    assert_eq!(payload["title"], "idle-mutation");
    assert_eq!(payload["extended_details"], "888.0");
    let next = JournalEvent::parse(case["steps"][2]["raw_json"].as_str().unwrap()).unwrap();
    let next_rows = explorer.on_event(&next);
    assert_eq!(
        next_rows
            .iter()
            .find(|finding| finding.title == "after-timer")
            .unwrap()
            .detail,
        "timer-mutated body;888.0"
    );
    assert!(explorer.take_diagnostics().is_empty());
    std::fs::remove_file(path).unwrap();
}
