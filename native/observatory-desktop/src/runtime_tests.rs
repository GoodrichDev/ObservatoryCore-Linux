//! Pipeline tests exercise the same background controller used by the desktop.
//! Real game-style files are changed on disk; workers and storage are not mocked.
use super::{Command, Controller, Snapshot};
use crate::store::Store;
use observatory_io::{ConfigPaths, Settings};
use serde_json::{json, Value};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

#[test]
fn plugins_can_reply_to_messages_without_losing_source_identity() {
    let temporary = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::from_roots(
        temporary.path().join("config"),
        temporary.path().join("data"),
        temporary.path().join("cache"),
    );
    paths.create_dirs().unwrap();
    fs::write(paths.plugin_dir.join("a.lua"),r#"return {
        id='example.requester',name='Requester',
        on_journal=function(e) core.send_message({kind='request'}) end,
        on_message=function(m) if m.data.kind=='response' then core.add_row({sender=m.source.id,value=m.data.value}) end end
    }"#).unwrap();
    fs::write(paths.plugin_dir.join("b.lua"),r#"return {
        id='example.responder',name='Responder',
        on_message=function(m) if m.data.kind=='request' then core.send_message({kind='response',value=42}) end end
    }"#).unwrap();
    let snapshot = std::sync::Arc::new(std::sync::Mutex::new(Snapshot::default()));
    let mut engine = super::Engine::new(
        Settings {
            popup: false,
            speech: false,
            plugins: json!({"BioInsights":{"GalacticCodexRemote":false},"Hecate":{"RemoteLookups":false}}),
            ..Settings::default()
        },
        paths,
        snapshot.clone(),
    )
    .unwrap();
    engine.process(observatory_domain::JournalEvent::parse(r#"{"event":"Probe"}"#).unwrap());
    engine.publish();
    let state = snapshot.lock().unwrap();
    assert_eq!(
        state.plugin_rows["Requester"][0],
        json!({"sender":"example.responder","value":42})
    );
    assert!(state.errors.is_empty(), "{:?}", state.errors);
}

#[test]
fn read_all_while_idle_does_not_start_live_monitoring() {
    let session = Session::new(&fixture());
    session.controller.send(Command::Replay);
    session.wait(|state| state.total == fixture().len() as u64 && state.state == "Idle");
    session.append(&[json!({"timestamp":"2026-10-25T15:00:00Z","event":"IdleProbe"})]);
    thread::sleep(Duration::from_millis(150));
    assert_eq!(
        session.controller.snapshot.lock().unwrap().total,
        fixture().len() as u64
    );
    session.controller.send(Command::Start);
    session
        .wait(|state| state.total == fixture().len() as u64 + 1 && state.last_event == "IdleProbe");
}

#[test]
fn asynchronous_http_response_reaches_only_the_originating_plugin() {
    use std::io::Read;
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/test", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut request = [0u8; 4096];
        let _ = stream.read(&mut request).unwrap();
        stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 13\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nprivate reply").unwrap();
    });
    let temporary = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::from_roots(
        temporary.path().join("config"),
        temporary.path().join("data"),
        temporary.path().join("cache"),
    );
    paths.create_dirs().unwrap();
    let requester = format!(
        r#"return {{id='tests.http.requester',name='Requester',
        on_journal=function(e) core.http_request({{url='{url}'}}) end,
        on_http_response=function(r) core.add_row({{status=r.status,body=r.body}}) end}}"#
    );
    fs::write(paths.plugin_dir.join("a.lua"), requester).unwrap();
    fs::write(paths.plugin_dir.join("b.lua"),r#"return {id='tests.http.observer',name='Observer',on_http_response=function(r)core.add_row({leak=r.body})end}"#).unwrap();
    let snapshot = std::sync::Arc::new(std::sync::Mutex::new(Snapshot::default()));
    let mut engine = super::Engine::new(
        Settings {
            popup: false,
            speech: false,
            plugins: json!({"BioInsights":{"GalacticCodexRemote":false},"Hecate":{"RemoteLookups":false}}),
            ..Settings::default()
        },
        paths,
        snapshot.clone(),
    )
    .unwrap();
    engine.process(observatory_domain::JournalEvent::parse(r#"{"event":"Probe"}"#).unwrap());
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        engine.poll_http();
        engine.publish();
        let state = snapshot.lock().unwrap();
        if state.plugin_rows.contains_key("Requester") {
            assert_eq!(
                state.plugin_rows["Requester"][0],
                json!({"status":200,"body":"private reply"})
            );
            assert!(!state.plugin_rows.contains_key("Observer"));
            assert!(state.errors.is_empty(), "{:?}", state.errors);
            break;
        }
        drop(state);
        assert!(Instant::now() < deadline, "HTTP callback timed out");
        thread::sleep(Duration::from_millis(10));
    }
    server.join().unwrap();
}

struct Session {
    // Stop and join the worker before deleting its files.
    controller: Controller,
    temporary: tempfile::TempDir,
    paths: ConfigPaths,
    journal: PathBuf,
    directory: PathBuf,
}

impl Session {
    fn new(events: &[Value]) -> Self {
        Self::with_plugin(events, None)
    }

    fn with_plugin(events: &[Value], source: Option<&str>) -> Self {
        let temporary = tempfile::tempdir().unwrap();
        let directory = temporary.path().join("Elite Dangerous");
        fs::create_dir(&directory).unwrap();
        let journal = directory.join("Journal.2026-10-25T120000.01.log");
        fs::write(&journal, records(events)).unwrap();
        let paths = ConfigPaths::from_roots(
            temporary.path().join("config"),
            temporary.path().join("data"),
            temporary.path().join("cache"),
        );
        paths.create_dirs().unwrap();
        if let Some(source) = source {
            fs::write(paths.plugin_dir.join("shutdown-saver.lua"), source).unwrap();
        }
        let settings = Settings {
            journal_dir: Some(directory.clone()),
            poll_interval_ms: 50,
            popup: false,
            speech: false,
            plugins: json!({"BioInsights":{"GalacticCodexRemote":false},"Hecate":{"RemoteLookups":false}}),
            auto_start: false,
            ..Settings::default()
        };
        let controller = Controller::spawn(settings, paths.clone()).unwrap();
        Self {
            controller,
            temporary,
            paths,
            journal,
            directory,
        }
    }

    fn wait(&self, condition: impl Fn(&Snapshot) -> bool) -> Snapshot {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let snapshot = self.controller.snapshot.lock().unwrap().clone();
            assert_ne!(
                snapshot.state, "Error",
                "runtime failed: {:?}",
                snapshot.errors
            );
            if condition(&snapshot) {
                return snapshot;
            }
            assert!(
                Instant::now() < deadline,
                "timed out: state={}, total={}, last={}, findings={}, sampler={:?}, errors={:?}",
                snapshot.state,
                snapshot.total,
                snapshot.last_event,
                snapshot.findings.len(),
                snapshot.sampler,
                snapshot.errors
            );
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn start(&self) -> Snapshot {
        self.controller.send(Command::Start);
        self.wait(|snapshot| snapshot.state == "Monitoring" && !snapshot.botanist.is_empty())
    }

    fn append(&self, events: &[Value]) {
        append_bytes(&self.journal, records(events).as_bytes());
    }

    fn export(&self, name: &str, delimiter: u8, plugin: Option<String>) -> PathBuf {
        let path = self.temporary.path().join(name);
        self.controller
            .send(Command::Export(path.clone(), delimiter, plugin));
        let suffix = path.to_string_lossy();
        self.wait(|snapshot| {
            snapshot.state.starts_with("Exported ") && snapshot.state.ends_with(suffix.as_ref())
        });
        path
    }
}

fn append_bytes(path: &Path, bytes: &[u8]) {
    let mut file = OpenOptions::new().append(true).open(path).unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}

fn records(events: &[Value]) -> String {
    events.iter().map(|event| format!("{event}\n")).collect()
}

fn event(second: u8, mut value: Value) -> Value {
    value["timestamp"] = format!("2026-10-25T12:00:{second:02}Z").into();
    value
}

fn body_scan(second: u8, body: u64) -> Value {
    event(
        second,
        json!({"event":"Scan","BodyID":body,"SystemAddress":42,
        "StarSystem":"Test System","BodyName":format!("Test System {body}"),
        "PlanetClass":"Rocky body","Landable":true,"Radius":1_000_000,"SurfaceGravity":40.0}),
    )
}

fn organic(second: u8, scan_type: &str) -> Value {
    event(
        second,
        json!({"event":"ScanOrganic","SystemAddress":42,"Body":2,
        "Species":"$Codex_Ent_Tussocks_01_Name;","Species_Localised":"Tussock Capillum",
        "Genus":"$Codex_Ent_Tussocks_Genus_Name;","ScanType":scan_type}),
    )
}

fn fixture() -> Vec<Value> {
    vec![
        event(
            0,
            json!({"event":"Fileheader","gameversion":"4.0.0.0","Odyssey":true}),
        ),
        event(
            1,
            json!({"event":"LoadGame","Commander":"Test Commander","Odyssey":true}),
        ),
        event(
            2,
            json!({"event":"Location","SystemAddress":42,"StarSystem":"Test System"}),
        ),
        event(
            3,
            json!({"event":"Touchdown","SystemAddress":42,"BodyID":2,"Body":"Test System 2"}),
        ),
        body_scan(4, 2),
        event(
            5,
            json!({"event":"SAASignalsFound","SystemAddress":42,"BodyID":2,"BodyName":"Test System 2",
            "Signals":[{"Type":"$SAA_SignalType_Biological;","Count":2}]}),
        ),
        organic(6, "Log"),
    ]
}

#[test]
fn preread_initializes_workers_and_suppresses_historical_sampler() {
    let session = Session::new(&fixture());
    let snapshot = session.start();
    assert_eq!(snapshot.total, 7);
    assert_eq!(snapshot.findings.len(), 1);
    assert_eq!(snapshot.findings[0].plugin, "Explorer");
    assert_eq!(snapshot.findings[0].title, "Landable with High Gravity");
    assert_eq!(snapshot.botanist.len(), 1);
    assert_eq!(snapshot.botanist[0].body, "Test System 2");
    assert_eq!(snapshot.botanist[0].bio_total, 2);
    assert_eq!(snapshot.botanist[0].species, "Tussock Capillum");
    assert!(!snapshot.botanist[0].analysed);
    assert!(snapshot.sampler.is_none());
    assert!(snapshot.errors.is_empty(), "{:?}", snapshot.errors);
}

#[test]
fn split_append_rotation_and_stop_resume_deliver_each_event_once() {
    let session = Session::new(&fixture());
    assert_eq!(session.start().total, 7);
    let line = format!("{}\n", body_scan(8, 3));
    let split = line.len() / 2;
    append_bytes(&session.journal, &line.as_bytes()[..split]);
    thread::sleep(Duration::from_millis(150));
    assert_eq!(session.controller.snapshot.lock().unwrap().total, 7);
    append_bytes(&session.journal, &line.as_bytes()[split..]);
    let snapshot = session.wait(|snapshot| snapshot.total == 8);
    assert_eq!(snapshot.findings.len(), 2);

    let rotated = session.directory.join("Journal.2026-10-25T120010.01.log");
    fs::write(
        &rotated,
        records(&[
            event(9, json!({"event":"Fileheader","gameversion":"4.0.0.0"})),
            body_scan(10, 4),
        ]),
    )
    .unwrap();
    let snapshot = session.wait(|snapshot| snapshot.total == 10);
    assert_eq!(snapshot.findings.len(), 3);
    // A late write to the previous file must still be read after rotation.
    session.append(&[body_scan(11, 5)]);
    assert_eq!(
        session.wait(|snapshot| snapshot.total == 11).findings.len(),
        4
    );

    session.controller.send(Command::Stop);
    session.wait(|snapshot| snapshot.state == "Idle");
    session.append(&[body_scan(12, 6)]);
    thread::sleep(Duration::from_millis(150));
    assert_eq!(session.controller.snapshot.lock().unwrap().total, 11);
    session.start();
    let snapshot = session.wait(|snapshot| snapshot.total == 12);
    assert_eq!(snapshot.findings.len(), 5);
    session.controller.send(Command::Stop);
    session.wait(|snapshot| snapshot.state == "Idle");
    session.start();
    thread::sleep(Duration::from_millis(150));
    let snapshot = session.controller.snapshot.lock().unwrap();
    assert_eq!(snapshot.total, 12);
    assert_eq!(
        snapshot
            .findings
            .iter()
            .filter(|finding| finding.body == "Test System 3")
            .count(),
        1
    );
    assert!(snapshot.errors.is_empty(), "{:?}", snapshot.errors);
}

#[test]
fn live_status_moves_sampler_then_analysis_closes_it() {
    let session = Session::new(&fixture());
    let status_path = session.directory.join("Status.json");
    fs::write(
        &status_path,
        json!({"event":"Status","Latitude":0.0,"Longitude":0.0,"PlanetRadius":1_000_000})
            .to_string(),
    )
    .unwrap();
    session.start();
    session.wait(|snapshot| snapshot.status["PlanetRadius"] == 1_000_000);
    session.append(&[organic(8, "Log")]);
    let snapshot = session.wait(|snapshot| {
        snapshot
            .sampler
            .as_ref()
            .is_some_and(|sampler| sampler.first_distance == Some(0.0))
    });
    assert_eq!(snapshot.total, 8);
    assert_eq!(snapshot.sampler.unwrap().colony_distance, 200);

    fs::write(
        &status_path,
        json!({"event":"Status","Latitude":0.0,"Longitude":0.02,"PlanetRadius":1_000_000})
            .to_string(),
    )
    .unwrap();
    session.wait(|snapshot| {
        snapshot.sampler.as_ref().is_some_and(|sampler| {
            sampler.first_distance.is_some_and(|d| d > 340.0) && sampler.ready()
        })
    });
    session.append(&[organic(9, "Sample")]);
    session.wait(|snapshot| {
        snapshot
            .sampler
            .as_ref()
            .is_some_and(|sampler| sampler.sample == 2 && sampler.second_distance == Some(0.0))
    });
    fs::write(
        &status_path,
        json!({"event":"Status","Latitude":0.0,"Longitude":0.04,"PlanetRadius":1_000_000})
            .to_string(),
    )
    .unwrap();
    session.wait(|snapshot| {
        snapshot
            .sampler
            .as_ref()
            .is_some_and(|sampler| sampler.sample == 2 && sampler.ready())
    });
    session.append(&[organic(10, "Analyse")]);
    let snapshot = session.wait(|snapshot| snapshot.total == 10 && snapshot.sampler.is_none());
    assert_eq!(snapshot.botanist.len(), 1);
    assert!(snapshot.botanist[0].analysed);
    assert!(snapshot.errors.is_empty(), "{:?}", snapshot.errors);
}

#[test]
fn headless_replay_completes_source_lifecycle_and_publishes_final_rows_and_callbacks() {
    let temporary = tempfile::tempdir().unwrap();
    let directory = temporary.path().join("journals");
    fs::create_dir(&directory).unwrap();
    fs::write(
        directory.join("Journal.2026-01-01T000000.01.log"),
        include_str!("../../fixtures/parity-journal.jsonl"),
    )
    .unwrap();
    let paths = ConfigPaths::from_roots(
        temporary.path().join("config"),
        temporary.path().join("data"),
        temporary.path().join("cache"),
    );
    paths.create_dirs().unwrap();
    let criteria = paths.data_dir.join("headless-startup.lua");
    fs::write(
        &criteria,
        "---@Global\nnotify('Headless startup','Headless startup','Initialised')\n---@End",
    )
    .unwrap();
    fs::write(
        paths.plugin_dir.join("headless-lifecycle.lua"),
        r#"return {id='tests.headless-lifecycle',name='Headless lifecycle',
        settings={finished=false},accept_batch_notifications=true,
        on_state_changed=function(s)
            core.add_row{kind='state',state=s.state,batch=s.batch,flags=core.get_log_monitor_state()}
            if s.state=='Idle' then
                core.save_settings{finished=true}
                core.add_finding{title='Headless completion'}
            end
        end,
        on_message=function(m)
            if m.source.name=='Core' and m.data.command=='SourceReadAllState' then
                core.add_row{kind='read',state=m.data.payload.State,
                    batch=core.is_log_monitor_batch_reading(),flags=core.get_log_monitor_state()}
            end
        end,
        on_journal=function(e)
            if e.event=='Shutdown' then
                core.add_row{kind='journal',event=e.event,
                    batch=core.is_log_monitor_batch_reading(),flags=core.get_log_monitor_state()}
            end
        end,
        on_notification=function(n)
            if n.title=='Headless completion' then
                core.add_row{kind='notification',title=n.title,
                    batch=core.is_log_monitor_batch_reading(),flags=core.get_log_monitor_state()}
            end
        end}"#,
    )
    .unwrap();
    let settings = Settings {
        journal_dir: Some(directory),
        popup: true,
        speech: true,
        plugins: json!({"Explorer":{"CustomCriteriaFile":criteria,"EnableCustomCriteria":false},"BioInsights":{"GalacticCodexRemote":false},"Hecate":{"RemoteLookups":false}}),
        ..Settings::default()
    };
    settings.save(&paths.settings_file).unwrap();
    let export = temporary.path().join("findings.csv");
    let replay = super::replay(settings, paths.clone(), Some(export.clone())).unwrap();
    assert_eq!(replay["events"], 19);
    assert_eq!(replay["botanist"].as_array().unwrap().len(), 1);
    assert_eq!(replay["botanist"][0]["species"], "Stratum Tectonicas");
    assert_eq!(replay["botanist"][0]["analysed"], true);
    assert_eq!(replay["messages"].as_array().unwrap().len(), 2);
    assert_eq!(
        replay["plugins"]["Headless lifecycle"],
        json!([
            {"kind":"state","state":"Reading all journals","batch":true,"flags":2},
            {"kind":"read","state":"Started","batch":true,"flags":2},
            {"kind":"journal","event":"Shutdown","batch":true,"flags":2},
            {"kind":"read","state":"Completed","batch":true,"flags":2},
            {"kind":"state","state":"Idle","batch":false,"flags":0},
            {"kind":"notification","title":"Headless completion","batch":false,"flags":0}
        ])
    );
    for summary in ["bio_summary", "eval_summary"] {
        assert!(replay[summary]["UIAlerts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|alert| alert["Type"] == "ReadAllCompleted"));
    }
    assert!(replay["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["title"] == "Headless completion"));
    assert!(replay["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["title"] == "Headless startup"));
    assert!(fs::read_to_string(export)
        .unwrap()
        .contains("Headless completion"));
    let saved = Settings::load(&paths.settings_file).unwrap();
    assert!(saved.popup && saved.speech);
    assert_eq!(saved.plugins["Headless lifecycle"]["finished"], true);
    assert_eq!(replay["errors"], json!([]));
}

#[test]
fn headless_notification_mode_suppresses_startup_completion_and_direct_audio_delivery() {
    use super::Engine;
    use std::sync::{Arc, Mutex};

    let temporary = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::from_roots(
        temporary.path().join("config"),
        temporary.path().join("data"),
        temporary.path().join("cache"),
    );
    paths.create_dirs().unwrap();
    let criteria = paths.data_dir.join("headless-startup.lua");
    fs::write(
        &criteria,
        "---@Global\nnotify('Headless startup','Headless startup','Initialised')\n---@End",
    )
    .unwrap();
    fs::write(
        paths.plugin_dir.join("headless-completion.lua"),
        r#"return {id='tests.headless-completion',name='Headless completion',
        on_state_changed=function(s)
            if s.state=='Idle' then core.add_finding{title='Headless completion'} end
        end}"#,
    )
    .unwrap();
    let settings = Settings {
        popup: true,
        speech: true,
        plugins: json!({"Explorer":{"CustomCriteriaFile":criteria,"EnableCustomCriteria":false},"BioInsights":{"GalacticCodexRemote":false},"Hecate":{"RemoteLookups":false}}),
        ..Settings::default()
    };
    let shared = Arc::new(Mutex::new(Snapshot::default()));
    let mut engine =
        Engine::with_native_notifications(settings, paths, shared.clone(), false).unwrap();
    assert!(engine
        .findings
        .iter()
        .any(|finding| finding.title == "Headless startup"));
    assert!(engine.native_popups.snapshot().is_empty());
    engine.state("Reading all journals", true);
    engine.finish_read(false);
    let snapshot = shared.lock().unwrap().clone();
    assert!(snapshot
        .findings
        .iter()
        .any(|finding| finding.title == "Headless completion"));
    assert!(snapshot.native_popups.is_empty());
    // A missing audio file normally produces a completion callback. Headless
    // delivery bypasses the player entirely, including direct plugin requests.
    engine.native_messages(vec![("Core".into(),json!({"target":"Core","command":"PlayAudioFile","payload":{"Filename":temporary.path().join("missing.wav")}}))]);
    assert!(engine.notifications.take_audio_completions().is_empty());
    assert!(snapshot.errors.is_empty(), "{:?}", snapshot.errors);
}

#[test]
fn replay_is_idempotent_in_database_and_exports_quoted_csv() {
    let mut events = fixture();
    events[4]["Atmosphere"] = "thin argon, \"trace\" atmosphere".into();
    let session = Session::new(&events);
    let initial = session.start();
    assert_eq!(initial.findings.len(), 2);
    session.controller.send(Command::Replay);
    let first = session.export("first.csv", b';', Some("Explorer".into()));
    session.controller.send(Command::Replay);
    let second = session.export("second.csv", b';', Some("Explorer".into()));
    assert_eq!(fs::read(&first).unwrap(), fs::read(&second).unwrap());
    let rows = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_path(second)
        .unwrap()
        .records()
        .collect::<csv::Result<Vec<_>>>()
        .unwrap();
    assert_eq!(rows.len(), 2);
    let atmosphere = rows
        .iter()
        .find(|row| &row[4] == "Landable with Atmosphere")
        .unwrap();
    assert_eq!(&atmosphere[5], "Thin Argon, \"Trace\" Atmosphere");
    let database = Store::open(&session.paths.data_dir.join("findings.sqlite3")).unwrap();
    let findings = database.recent(100).unwrap();
    assert_eq!(findings.len(), 2);
    assert_eq!(
        session.controller.snapshot.lock().unwrap().findings.len(),
        2
    );
    assert!(session
        .controller
        .snapshot
        .lock()
        .unwrap()
        .errors
        .is_empty());
}

#[test]
fn configure_shutdown_settings_cannot_overwrite_new_preferences() {
    let source = r#"return {
        name='Shutdown Saver', id='tests.shutdown-saver', version='1',
        settings={marker='initial'},
        on_shutdown=function() core.save_settings({marker='old engine shutdown'}) end
    }"#;
    let session = Session::with_plugin(&fixture(), Some(source));
    session.start();
    let new_directory = session.temporary.path().join("New journals");
    fs::create_dir(&new_directory).unwrap();
    fs::copy(
        &session.journal,
        new_directory.join("Journal.2026-10-25T120000.01.log"),
    )
    .unwrap();
    let mut settings = Settings {
        journal_dir: Some(new_directory.clone()),
        theme: "light".into(),
        poll_interval_ms: 150,
        auto_start: false,
        popup: false,
        speech: false,
        plugins: json!({"BioInsights":{"GalacticCodexRemote":false},"Hecate":{"RemoteLookups":false}}),
        ..Settings::default()
    };
    settings
        .extra
        .insert("user_choice".into(), json!({"keep":true}));
    session.controller.send(Command::Configure(settings));
    // Export is queued after Configure and the worker's pending initial read,
    // so observing it confirms that replacement and old shutdown both finished.
    session.export("after-configure.csv", b',', None);
    let saved = Settings::load(&session.paths.settings_file).unwrap();
    assert_eq!(saved.journal_dir, Some(new_directory));
    assert_eq!(saved.theme, "light");
    assert_eq!(saved.poll_interval_ms, 150);
    assert_eq!(saved.extra["user_choice"], json!({"keep":true}));
    assert!(!saved.popup);
    assert!(!saved.speech);
    assert!(session
        .controller
        .snapshot
        .lock()
        .unwrap()
        .errors
        .is_empty());
}

#[test]
fn organic_and_status_in_same_poll_capture_latest_sample_origin() {
    use super::Engine;
    use observatory_domain::JournalEvent;
    use observatory_io::{AuxiliaryUpdate, ScanBatch};
    use std::sync::{Arc, Mutex};
    let temporary = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::from_roots(
        temporary.path().join("config"),
        temporary.path().join("data"),
        temporary.path().join("cache"),
    );
    paths.create_dirs().unwrap();
    let shared = Arc::new(Mutex::new(Snapshot::default()));
    let settings = Settings {
        popup: false,
        speech: false,
        plugins: json!({"BioInsights":{"GalacticCodexRemote":false},"Hecate":{"RemoteLookups":false}}),
        auto_start: false,
        ..Settings::default()
    };
    let mut engine = Engine::new(settings, paths, shared.clone()).unwrap();
    // Original Botanist only starts a sampler once the body exists in its
    // signal ledger; first-ever organic on an unknown body only inserts a row.
    for value in fixture().into_iter().take(6) {
        engine.process(JournalEvent::parse(&value.to_string()).unwrap());
    }
    let status = |longitude| AuxiliaryUpdate {
        name: "Status.json".into(),
        data: json!({"Latitude":0.0,"Longitude":longitude,"PlanetRadius":1_000_000}),
    };
    engine.scan(ScanBatch {
        updates: vec![status(0.0)],
        ..Default::default()
    });
    // Pollers can deliver a journal append and its surface snapshot together.
    // The sample origin must use the new coordinates from that same batch.
    engine.scan(ScanBatch {
        events: vec![JournalEvent::parse(&organic(8, "Log").to_string()).unwrap()],
        updates: vec![status(0.02)],
        ..Default::default()
    });
    engine.publish();
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .sampler
            .as_ref()
            .unwrap()
            .first_distance,
        Some(0.0)
    );
    engine.scan(ScanBatch {
        updates: vec![status(0.04)],
        ..Default::default()
    });
    engine.publish();
    let shared = shared.lock().unwrap();
    let distance = shared.sampler.as_ref().unwrap().first_distance.unwrap();
    assert!(
        (distance - 349.066).abs() < 0.000001,
        "sample used stale origin: {distance}"
    );
    assert!(shared.errors.is_empty());
}

#[test]
fn notification_message_cycle_is_bounded_and_worker_remains_usable() {
    use super::Engine;
    use observatory_domain::JournalEvent;
    use std::sync::{Arc, Mutex};
    let temporary = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::from_roots(
        temporary.path().join("config"),
        temporary.path().join("data"),
        temporary.path().join("cache"),
    );
    paths.create_dirs().unwrap();
    fs::write(
        paths.plugin_dir.join("cycle.lua"),
        r#"return {
        name='Cyclic Notifier', id='tests.cyclic-notifier', version='1',
        on_journal=function() core.add_finding({title='start'}) end,
        on_notification=function() core.send_message({kind='notification'}) end,
        on_message=function(m) if m.data.kind=='return' then core.add_finding({title='loop'}) end end
    }"#,
    )
    .unwrap();
    // The original plugin bus excludes the sender. A second identity is needed
    // to produce a real cycle rather than relying on incorrect self-delivery.
    fs::write(paths.plugin_dir.join("relay.lua"),r#"return {name='Relay',id='tests.relay',on_message=function(m) if m.data.kind=='notification' then core.send_message({kind='return'}) end end}"#).unwrap();
    let shared = Arc::new(Mutex::new(Snapshot::default()));
    let settings = Settings {
        popup: false,
        speech: false,
        plugins: json!({"BioInsights":{"GalacticCodexRemote":false},"Hecate":{"RemoteLookups":false}}),
        auto_start: false,
        ..Settings::default()
    };
    let mut engine = Engine::new(settings, paths, shared.clone()).unwrap();
    engine.process(
        JournalEvent::parse(&event(0, json!({"event":"Fileheader"})).to_string()).unwrap(),
    );
    engine.publish();
    {
        let snapshot = shared.lock().unwrap();
        assert_eq!(snapshot.total, 1);
        assert!(snapshot.findings.len() > 1 && snapshot.findings.len() <= 256);
        assert_eq!(snapshot.errors.len(), 1);
        assert!(snapshot.errors[0].contains("host dispatch budget"));
    }
    let first_count = shared.lock().unwrap().findings.len();
    // A later real event gets a fresh allowance. Original Basic grids append
    // repeated rows; the optional persistence deduplication does not gate them.
    engine.process(
        JournalEvent::parse(
            &event(
                1,
                json!({"event":"Location","StarSystem":"Next System","SystemAddress":43}),
            )
            .to_string(),
        )
        .unwrap(),
    );
    engine.publish();
    let snapshot = shared.lock().unwrap();
    assert_eq!(snapshot.total, 2);
    assert_eq!(snapshot.current_system, "Next System");
    assert!(snapshot.findings.len() > first_count && snapshot.findings.len() <= 512);
    assert_eq!(snapshot.errors.len(), 2);
}

#[test]
fn read_completion_publishes_partial_chunk_and_state_callbacks_before_clearing_progress() {
    use super::Engine;
    use observatory_domain::JournalEvent;
    use std::sync::{Arc, Mutex};

    let temporary = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::from_roots(
        temporary.path().join("config"),
        temporary.path().join("data"),
        temporary.path().join("cache"),
    );
    paths.create_dirs().unwrap();
    fs::write(
        paths.plugin_dir.join("completion.lua"),
        r#"return {id='tests.read-completion',name='Read completion',
        on_journal=function(e) core.add_finding{title='Row '..e.N} end,
        on_state_changed=function(s)
            if s.state=='Monitoring' then core.add_finding{title='Read ready'} end
        end}"#,
    )
    .unwrap();
    let shared = Arc::new(Mutex::new(Snapshot::default()));
    let settings = Settings {
        popup: false,
        speech: false,
        plugins: json!({"BioInsights":{"GalacticCodexRemote":false},"Hecate":{"RemoteLookups":false}}),
        auto_start: false,
        ..Settings::default()
    };
    let mut engine = Engine::new(settings, paths, shared.clone()).unwrap();
    engine.state("Reading all journals", true);
    shared.lock().unwrap().read_progress = Some(("journal.log".into(), 0, 1));
    for n in 0..257 {
        engine.process(
            JournalEvent::parse(&json!({"event":"CompletionProbe","N":n}).to_string()).unwrap(),
        );
        if n == 255 {
            engine.publish();
        }
    }
    let partial = shared.lock().unwrap().clone();
    assert_eq!(partial.total, 256);
    assert_eq!(partial.findings.len(), 256);
    assert!(partial.read_progress.is_some());

    engine.state("Read all cancelled", false);
    engine.finish_read(true);
    let ready = shared.lock().unwrap().clone();
    assert_eq!(ready.state, "Monitoring");
    assert!(ready.monitoring);
    assert!(ready.read_progress.is_none());
    assert_eq!(ready.total, 257);
    assert_eq!(ready.last_event, "CompletionProbe");
    assert_eq!(ready.findings.len(), 258);
    assert_eq!(ready.findings.back().unwrap().title, "Read ready");
    assert!(ready.errors.is_empty(), "{:?}", ready.errors);
}

#[test]
fn cancel_read_all_resumes_monitor_without_turning_unread_history_into_live_events() {
    let session = Session::new(&fixture());
    session.start();
    let mut file = OpenOptions::new()
        .append(true)
        .open(&session.journal)
        .unwrap();
    for n in 0..50_000 {
        writeln!(file, "{{\"event\":\"CancellationProbe\",\"N\":{n}}}").unwrap();
    }
    file.sync_all().unwrap();
    session.controller.send(Command::Replay);
    session.wait(|state| state.read_progress.is_some());
    session.controller.send(Command::CancelReplay);
    let after = session.wait(|state| {
        state.read_progress.is_none() && state.state == "Monitoring" && state.monitoring
    });
    assert!(
        after.total < 50_000 + fixture().len() as u64 * 2,
        "ReadAll cancellation was not observed"
    );
    thread::sleep(Duration::from_millis(150));
    assert_eq!(
        session.controller.snapshot.lock().unwrap().total,
        after.total,
        "Historical rows leaked into the live monitor after Cancel"
    );
    session.append(&[json!({"event":"AfterCancelledReadAll"})]);
    let live = session.wait(|state| state.last_event == "AfterCancelledReadAll");
    assert_eq!(live.total, after.total + 1);
    assert!(live.errors.is_empty(), "{:?}", live.errors);
}
