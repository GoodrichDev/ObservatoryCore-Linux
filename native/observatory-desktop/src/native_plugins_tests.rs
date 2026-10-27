//! Integration checks for the native journal/message/vault pipeline.
use super::{Engine, Snapshot};
use observatory_domain::JournalEvent;
use observatory_io::{ConfigPaths, Settings};
use serde_json::{json, Value};
use std::{
    fs,
    io::{Read, Write},
    net::{Ipv4Addr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

fn settings() -> Settings {
    let mut settings = Settings {
        popup: false,
        speech: false,
        auto_start: false,
        start_read_all: false,
        ..Settings::default()
    };
    settings.extra.insert(
        "disabled_plugins".into(),
        json!(["Explorer", "Botanist", "Communicator", "StellarOverlay"]),
    );
    settings.plugins =
        json!({"Hecate":{"RemoteLookups":false},"BioInsights":{"GalacticCodexRemote":false}});
    settings
}
#[test]
fn large_native_fanout_resumes_without_losing_vault_updates() {
    let (_temporary, paths) = fixture();
    let mut engine = engine(settings(), paths);
    let messages=(0..600).map(|body|("Evaluator".into(),json!({"target":"Hecate","command":"EvaluatorAdd","payload":{"FID":123,"ID64":42,"BodyID":body,"Value":10,"Name":format!("Body {body}"),"DateTime":"2026-10-25T00:00:00Z"}}))).collect();
    engine.native_messages(messages);
    for _ in 0..8 {
        engine.poll_http();
    }
    assert_eq!(engine.hecate.evaluator_total(123).unwrap(), 6000);
    assert!(engine.snapshot.lock().unwrap().errors.is_empty());
}
fn fixture() -> (tempfile::TempDir, ConfigPaths) {
    let directory = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::from_roots(
        directory.path().join("config"),
        directory.path().join("data"),
        directory.path().join("cache"),
    );
    paths.create_dirs().unwrap();
    (directory, paths)
}
fn engine(settings: Settings, paths: ConfigPaths) -> Engine {
    Engine::new(settings, paths, Arc::new(Mutex::new(Snapshot::default()))).unwrap()
}
fn process(engine: &mut Engine, mut value: Value) {
    if value.get("timestamp").is_none() {
        value["timestamp"] = json!("2026-10-25T00:00:00Z");
    }
    engine.process(JournalEvent::parse(&value.to_string()).unwrap());
}
fn journal_events() -> Vec<Value> {
    vec![
        json!({"event":"LoadGame","FID":"F123","Commander":"Native Commander","Odyssey":true}),
        json!({"event":"Location","SystemAddress":42,"StarSystem":"Native System","StarPos":[1,2,3]}),
        json!({"event":"FSSDiscoveryScan","SystemAddress":42,"BodyCount":2}),
        json!({"event":"Scan","SystemAddress":42,"StarSystem":"Native System","BodyID":0,"BodyName":"Native System","StarType":"G","StellarMass":1,"WasDiscovered":false}),
        json!({"event":"Scan","SystemAddress":42,"StarSystem":"Native System","BodyID":1,"BodyName":"Native System 1","PlanetClass":"Earthlike body","MassEM":1,"WasDiscovered":false,"WasMapped":false}),
        json!({"event":"ScanOrganic","SystemAddress":42,"BodyID":1,"ScanType":"Analyse","Genus":"$Codex_Ent_Stratum_Genus_Name;","Species":"$Codex_Ent_Stratum_07_Name;","Variant":"$Codex_Ent_Stratum_07_Green;"}),
    ]
}
fn assert_original_archive(loaded: &Value) {
    // Original Hecate dispatcher captures exclude FSSDiscoveryScan; the two
    // scans and organic event remain in the visit after Location. See all 282
    // independent source-type traces in hecate-archive-dispatch-oracle.json.
    let events: Vec<_> = loaded["Payload"]["JournalEvents"]
        .as_array()
        .unwrap()
        .iter()
        .map(|event| event["event"].as_str().unwrap())
        .collect();
    assert_eq!(events, ["Location", "Scan", "Scan", "ScanOrganic"]);
    assert_eq!(loaded["Payload"]["FIDs"], json!([123, 123, 123, 123]));
}
fn get(port: u16, path: &str) -> String {
    let mut socket = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    write!(
        socket,
        "GET {path} HTTP/1.1\r\nHost: localhost:{port}\r\n\r\n"
    )
    .unwrap();
    let mut text = String::new();
    socket.read_to_string(&mut text).unwrap();
    text
}
fn available_bridge_port() -> u16 {
    loop {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        if (8000..59000).contains(&port) {
            return port;
        }
    }
}

#[test]
fn native_batch_ledgers_survive_restart_and_keep_commander_history() {
    let (_directory, paths) = fixture();
    let exploration;
    {
        let mut engine = engine(settings(), paths.clone());
        engine.state("Reading all journals", true);
        for event in journal_events() {
            process(&mut engine, event);
        }
        engine.state("Idle", false);
        exploration = engine.hecate.evaluator_total(123).unwrap();
        assert!(exploration > 0);
        assert_eq!(engine.hecate.bioinsights_total(123).unwrap(), 95_054_000);
        assert!(engine.snapshot.lock().unwrap().errors.is_empty());
    }
    {
        let mut engine = engine(settings(), paths.clone());
        assert_eq!(engine.hecate.evaluator_total(123).unwrap(), exploration);
        assert_eq!(engine.hecate.bioinsights_total(123).unwrap(), 95_054_000);
        let loaded = engine
            .hecate
            .handle_message(
                "LoadSystem",
                &json!({"ID64":42,"DateTime":"2026-10-26T00:00:00Z"}),
            )
            .unwrap()
            .unwrap();
        assert_original_archive(&loaded);
    }
}

#[test]
fn headless_full_replay_rebuilds_old_ledgers_without_duplicating_visits() {
    let (directory, paths) = fixture();
    let journals = directory.path().join("journals");
    fs::create_dir(&journals).unwrap();
    let mut events = journal_events();
    for event in &mut events {
        event["timestamp"] = json!("2026-10-25T00:00:00Z");
    }
    fs::write(
        journals.join("Journal.2026-10-25T000000.01.log"),
        events
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
    .unwrap();
    let mut settings = settings();
    settings.journal_dir = Some(journals);
    {
        let mut engine = engine(settings.clone(), paths.clone());
        engine.hecate.handle_message("EvaluatorAdd",&json!({"FID":123,"ID64":999,"BodyID":1,"Value":999999999,"Name":"Removed journal","DateTime":"2020-01-01T00:00:00Z"})).unwrap();
    }
    let first = super::replay(settings.clone(), paths.clone(), None).unwrap();
    assert!(
        first["errors"].as_array().unwrap().is_empty(),
        "{}",
        first["errors"]
    );
    let second = super::replay(settings.clone(), paths.clone(), None).unwrap();
    assert_eq!(first["events"], second["events"]);
    let mut engine = engine(settings, paths);
    let value = engine.hecate.evaluator_total(123).unwrap();
    assert!(value > 0 && value < 999999999);
    let loaded = engine
        .hecate
        .handle_message(
            "LoadSystem",
            &json!({"ID64":42,"DateTime":"2026-10-26T00:00:00Z"}),
        )
        .unwrap()
        .unwrap();
    assert_original_archive(&loaded);
}

#[test]
fn configure_releases_bridge_port_flushes_visit_and_restores_registrations() {
    let (_directory, paths) = fixture();
    let port = available_bridge_port();
    let mut settings = settings();
    settings.plugins["Hecate"]["ServerSettings"] =
        json!({"EnableWebServer":true,"ServerPort":port,"ServerLANIPAddress":""});
    let mut engine = engine(settings.clone(), paths.clone());
    engine.state("Preparing current system", true);
    process(
        &mut engine,
        json!({"event":"LoadGame","FID":"F123","Commander":"Native Commander"}),
    );
    process(
        &mut engine,
        json!({"event":"Location","SystemAddress":42,"StarSystem":"Native System"}),
    );
    engine.state("Idle", false);
    engine.native_messages(Vec::new());
    assert!(get(port, "/status").contains("BioInsights"));
    settings.theme = "light".into();
    engine.configure(settings).unwrap();
    assert_eq!(engine.hecate.server_address().unwrap().1, port);
    assert!(get(port, "/status").contains("Evaluator"));
    let loaded = engine
        .hecate
        .handle_message(
            "LoadSystem",
            &json!({"ID64":42,"DateTime":"2026-10-26T00:00:00Z"}),
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        loaded["Payload"]["JournalEvents"].as_array().unwrap().len(),
        1
    );
    engine.native_messages(Vec::new());
    assert!(engine.snapshot.lock().unwrap().errors.is_empty());
    let response = thread::scope(|scope| {
        let request = scope.spawn(|| get(port, "/bioinsights/api/rows"));
        for _ in 0..100 {
            engine.native_messages(Vec::new());
            if request.is_finished() {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        request.join().unwrap()
    });
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
}

#[test]
fn failed_configuration_save_restores_existing_bridge_and_its_settings() {
    let (_directory, mut paths) = fixture();
    let port = available_bridge_port();
    let mut settings = settings();
    settings.plugins["Hecate"]["ServerSettings"] =
        json!({"EnableWebServer":true,"ServerPort":port});
    paths.settings_file = paths.config_dir.clone(); // Existing directory cannot be atomically replaced by a file.
    let mut engine = engine(settings.clone(), paths);
    engine.state("Preparing current system", true);
    engine.state("Idle", false);
    engine.native_messages(Vec::new());
    settings.theme = "light".into();
    assert!(engine.configure(settings).is_err());
    assert_eq!(engine.settings.theme, "dark");
    assert_eq!(engine.hecate.server_address().unwrap().1, port);
    assert!(get(port, "/status").contains("BioInsights"));
}

#[test]
fn disabled_hecate_never_binds_or_rebuilds_durable_history() {
    let (directory, paths) = fixture();
    let port = available_bridge_port();
    let journals = directory.path().join("journals");
    fs::create_dir(&journals).unwrap();
    let mut events = journal_events();
    for event in &mut events {
        event["timestamp"] = json!("2026-10-25T00:00:00Z");
    }
    fs::write(
        journals.join("Journal.2026-10-25T000000.01.log"),
        events
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
    .unwrap();
    {
        let mut engine = engine(settings(), paths.clone());
        engine.hecate.handle_message("EvaluatorAdd",&json!({"FID":123,"ID64":999,"BodyID":1,"Value":123456,"Name":"Existing unsold system","DateTime":"2020-01-01T00:00:00Z"})).unwrap();
    }
    let mut disabled = settings();
    disabled.journal_dir = Some(journals);
    disabled.extra["disabled_plugins"]
        .as_array_mut()
        .unwrap()
        .push(json!("Hecate"));
    disabled.plugins["Hecate"]["ServerSettings"] =
        json!({"EnableWebServer":true,"ServerPort":port});
    {
        let engine = engine(disabled.clone(), paths.clone());
        assert!(engine.hecate.server_address().is_none());
        let _unbound =
            TcpListener::bind((Ipv4Addr::LOCALHOST, port)).expect("disabled Hecate bound a socket");
    }
    super::replay(disabled.clone(), paths.clone(), None).unwrap();
    {
        let controller = super::Controller::spawn(disabled, paths.clone()).unwrap();
        controller.send(super::Command::Replay);
        let started = std::time::Instant::now();
        loop {
            let state = controller.snapshot.lock().unwrap();
            if state.state == "Idle" && state.total == events.len() as u64 {
                break;
            }
            drop(state);
            assert!(started.elapsed() < Duration::from_secs(5));
            thread::sleep(Duration::from_millis(20));
        }
    }
    let engine = engine(settings(), paths);
    assert_eq!(engine.hecate.evaluator_total(123).unwrap(), 123456);
}
