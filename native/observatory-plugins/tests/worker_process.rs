//! Real subprocess transport tests against the same production models used by
//! builtin mode. These are not mock workers or journal interpretation stubs.
use observatory_domain::JournalEvent;
use observatory_plugins::{
    bioinsights, evaluator, hecate, stellar_overlay,
    worker_rpc::{self, WorkerKind},
};
use serde_json::json;
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "observatory-worker-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn executable() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_observatory-native-worker"))
}
fn events() -> Vec<JournalEvent> {
    [
        json!({"event":"LoadGame","FID":"F123","Commander":"Transport QA","GameMode":"Open"}),
        json!({"event":"Location","SystemAddress":18446744073709550001u64,"StarSystem":"Transport System","StarPos":[0,0,0],"BodyID":1,"Body":"Transport System 1","BodyType":"Planet"}),
        json!({"event":"Scan","SystemAddress":18446744073709550001u64,"StarSystem":"Transport System","BodyID":0,"BodyName":"Transport System","StarType":"G","Radius":695700000,"SurfaceTemperature":5778,"StellarMass":1}),
        json!({"event":"Scan","SystemAddress":18446744073709550001u64,"BodyID":1,"BodyName":"Transport System 1","PlanetClass":"Rocky body","MassEM":0.1,"SurfaceGravity":1.0,"Radius":2000000,"SurfaceTemperature":170,"SurfacePressure":10,"Atmosphere":"thin carbon dioxide atmosphere","AtmosphereType":"CarbonDioxide","AtmosphereComposition":[{"Name":"CarbonDioxide","Percent":100}],"DistanceFromArrivalLS":50,"Landable":true,"WasDiscovered":false,"WasMapped":false}),
        json!({"event":"Scan","SystemAddress":18446744073709550001u64,"BodyID":2,"BodyName":"Transport System 2","PlanetClass":"Earthlike body","MassEM":1.0,"Radius":6371000,"DistanceFromArrivalLS":100,"WasDiscovered":false,"WasMapped":false}),
        json!({"event":"FSSBodySignals","BodyID":1,"BodyName":"Transport System 1","SystemAddress":18446744073709550001u64,"Signals":[{"Type":"$SAA_SignalType_Biological;","Count":2}]}),
        json!({"event":"SAASignalsFound","BodyID":1,"BodyName":"Transport System 1","SystemAddress":18446744073709550001u64,"Signals":[{"Type":"$SAA_SignalType_Biological;","Count":2}],"Genuses":[{"Genus":"$Codex_Ent_Stratum_Genus_Name;","Genus_Localised":"Stratum"}]}),
        json!({"event":"Touchdown","OnPlanet":true,"BodyID":1,"Body":"Transport System 1","Latitude":1.25,"Longitude":2.5,"SystemAddress":18446744073709550001u64}),
        json!({"event":"ScanOrganic","ScanType":"Log","Genus":"$Codex_Ent_Stratum_Genus_Name;","Species":"$Codex_Ent_Stratum_Tectonicas_Name;","Variant":"$Codex_Ent_Stratum_Tectonicas_Emerald_Name;","Body":1,"SystemAddress":18446744073709550001u64}),
        json!({"event":"SAAScanComplete","BodyID":1,"BodyName":"Transport System 1","SystemAddress":18446744073709550001u64,"ProbesUsed":2,"EfficiencyTarget":6}),
    ].into_iter().map(|mut value| {value["timestamp"]=json!("2026-10-26T01:02:03Z");JournalEvent::parse(&value.to_string()).unwrap()}).collect()
}

#[test]
fn bio_process_preserves_journals_bus_http_notifications_and_all_ui_projections() {
    let dir = Directory::new();
    let settings = json!({"GalacticCodexRemote":false});
    let mut direct = bioinsights::BioInsights::new(settings.clone()).unwrap();
    let mut remote =
        worker_rpc::BioInsights::from_executable(settings, &dir.0, executable()).unwrap();
    assert!(remote.is_external());
    assert_eq!(
        remote.worker_info().unwrap().plugin,
        WorkerKind::BioInsights
    );
    for peer in ["Hecate", "Evaluator", "StellarOverlay"] {
        assert_eq!(
            remote.on_peer_message(peer, "Ready", &json!({"Version":"1.0"})),
            direct.on_peer_message(peer, "Ready", &json!({"Version":"1.0"}))
        );
    }
    for event in events() {
        let a = direct.on_event(&event);
        let b = remote.on_event(&event);
        assert_eq!(a, b, "{}", event.event);
        for finding in &a {
            assert_eq!(
                direct.take_notification_rendering(finding),
                remote.take_notification_rendering(finding)
            );
        }
    }
    let status = json!({"Flags":2097152,"Flags2":0,"Latitude":1.25,"Longitude":2.5,"PlanetRadius":2000000,"BodyName":"Transport System 1","Altitude":3,"Heading":45});
    assert_eq!(direct.on_status(&status), remote.on_status(&status));
    assert_eq!(direct.take_messages(), remote.take_messages());
    assert_eq!(direct.take_http_requests(), remote.take_http_requests());
    assert_eq!(direct.take_diagnostics(), remote.take_diagnostics());
    assert_eq!(direct.rows(), remote.rows());
    assert_eq!(direct.main_view(), remote.main_view());
    assert_eq!(direct.overlay(), remote.overlay());
    assert_eq!(direct.tools_summary(), remote.tools_summary());
    assert_eq!(direct.body_summary(1), remote.body_summary(1));
    assert_eq!(direct.body_predictions(1), remote.body_predictions(1));
    assert_eq!(direct.codex_companion(), remote.codex_companion());
    assert_eq!(direct.unsold_data(), remote.unsold_data());
    assert_eq!(direct.state(), remote.state());
    assert_eq!(direct.browser_route(), remote.browser_route());
    assert_eq!(direct.browser_html(), remote.browser_html());
    direct
        .set_settings(json!({"GalacticCodexRemote":false,"Theme":"Orange"}))
        .unwrap();
    remote
        .set_settings(json!({"GalacticCodexRemote":false,"Theme":"Orange"}))
        .unwrap();
    assert_eq!(direct.main_view(), remote.main_view());
    assert!(remote.take_worker_errors().is_empty());
}

#[test]
fn evaluator_process_preserves_values_routes_bus_and_notification_payloads() {
    let dir = Directory::new();
    let mut direct = evaluator::Evaluator::new(json!({}));
    let mut remote =
        worker_rpc::Evaluator::from_executable(json!({}), &dir.0, executable()).unwrap();
    direct.set_preread(true);
    remote.set_preread(true);
    direct.set_batch(true);
    remote.set_batch(true);
    let events = events();
    let expected: Vec<_> = events.iter().flat_map(|e| direct.on_event(e)).collect();
    assert_eq!(expected, remote.on_events(&events));
    direct.set_preread(false);
    remote.set_preread(false);
    direct.set_batch(false);
    remote.set_batch(false);
    for finding in &expected {
        assert_eq!(
            direct.take_notification_rendering(finding),
            remote.take_notification_rendering(finding)
        );
        assert_eq!(
            direct.take_notification_payload(finding),
            remote.take_notification_payload(finding)
        );
    }
    direct.on_message(
        "EvaluatorSetTotal",
        &json!({"Total":123456789,"SubtractCurrent":true}),
    );
    remote.on_message(
        "EvaluatorSetTotal",
        &json!({"Total":123456789,"SubtractCurrent":true}),
    );
    assert_eq!(direct.take_messages(), remote.take_messages());
    assert_eq!(direct.poll_routes(), remote.poll_routes());
    assert_eq!(direct.take_route_findings(), remote.take_route_findings());
    assert_eq!(direct.rows(), remote.rows());
    assert_eq!(direct.summary(), remote.summary());
    assert_eq!(direct.web_main_html(), remote.web_main_html());
    remote.set_settings(json!({"ShowValues":false})).unwrap();
    direct.set_settings(json!({"ShowValues":false})).unwrap();
    assert_eq!(direct.rows(), remote.rows());
    assert!(remote.take_worker_errors().is_empty());
}

#[test]
fn hecate_process_preserves_durable_ledgers_and_reopens_them_after_worker_restart() {
    let a = Directory::new();
    let b = Directory::new();
    let settings = json!({"RemoteLookups":false,"ServerSettings":{"EnableWebServer":false}});
    let mut direct = hecate::Hecate::open(settings.clone(), &a.0).unwrap();
    let mut remote =
        worker_rpc::Hecate::from_executable(settings.clone(), &b.0, executable()).unwrap();
    for event in events() {
        assert_eq!(direct.on_event(&event), remote.on_event(&event));
    }
    for (command, payload) in [
        (
            "EvaluatorAdd",
            json!({"FID":"F123","ID64":18446744073709550001u64,"BodyID":1,"Value":987654321,"Name":"Transport System 1","DateTime":"2026-10-26T01:02:03Z"}),
        ),
        (
            "BioInsightsUnsoldAdd",
            json!({"FID":"F123","ID64":18446744073709550001u64,"BodyID":1,"BioName":"Stratum Tectonicas","Species":123,"Value":19010800,"BodyName":"Transport System 1","DateTime":"2026-10-26T01:02:03Z"}),
        ),
        (
            "LoadSystem",
            json!({"ID64":18446744073709550001u64,"DateTime":"2026-10-27T01:02:03Z"}),
        ),
        (
            "EvaluatorGetData",
            json!({"FID":"F123","Response":{"opaque":true}}),
        ),
        ("BioInsightsGetData", json!({"FID":"F123"})),
    ] {
        assert_eq!(
            direct.handle_message(command, &payload).unwrap(),
            remote.handle_message(command, &payload).unwrap()
        );
    }
    assert_eq!(
        direct.evaluator_total(123).unwrap(),
        remote.evaluator_total(123).unwrap()
    );
    assert_eq!(
        direct.bioinsights_total(123).unwrap(),
        remote.bioinsights_total(123).unwrap()
    );
    assert_eq!(direct.rows(), remote.rows());
    assert_eq!(direct.take_messages(), remote.take_messages());
    assert!(remote.take_worker_errors().is_empty());
    remote.flush().unwrap();
    drop(remote);
    let remote = worker_rpc::Hecate::from_executable(settings, &b.0, executable()).unwrap();
    assert_eq!(
        direct.evaluator_data(123).unwrap(),
        remote.evaluator_data(123).unwrap()
    );
    assert_eq!(
        direct.bioinsights_data(123).unwrap(),
        remote.bioinsights_data(123).unwrap()
    );
}

#[test]
fn stellar_process_preserves_clock_domain_queue_radar_and_handler_settings() {
    let dir = Directory::new();
    let mut direct = stellar_overlay::StellarOverlay::new(json!({"DelayBetweenNotificationsMS":0}));
    let mut remote = worker_rpc::StellarOverlay::from_executable(
        json!({"DelayBetweenNotificationsMS":0}),
        &dir.0,
        executable(),
    )
    .unwrap();
    let notification = json!({"Rendering":1,"Title":"Clock domain","Detail":"Visible payload","Guid":"00000000-0000-0000-0000-000000000123","Timeout":5000,"Sender":"Evaluator"});
    direct.on_notification_at(&notification, 800000);
    remote.on_notification_at(&notification, 800000);
    direct.tick_at(800001);
    remote.tick_at(800001);
    let mut a = serde_json::to_value(direct.snapshot()).unwrap();
    let mut b = serde_json::to_value(remote.snapshot()).unwrap();
    a.as_object_mut().unwrap().remove("sampled_ms");
    b.as_object_mut().unwrap().remove("sampled_ms");
    assert_eq!(a, b);
    assert_eq!(a["notifications"][0]["created_ms"], 800000);
    for (command, payload) in [
        (
            "RadarShowPlanet",
            json!({"SystemAddress":42,"BodyID":1,"Radius":2000000}),
        ),
        ("RadarAddVehicle", json!({"ContextID":1,"X":1.0,"Y":2.0})),
        (
            "RegisterPluginCustomColours",
            json!({"TitleColor":"FFFF0000","GetDefaultColors":true}),
        ),
    ] {
        assert_eq!(
            direct
                .handle_message("BioInsights", command, &payload)
                .unwrap(),
            remote
                .handle_message("BioInsights", command, &payload)
                .unwrap()
        );
    }
    assert_eq!(
        serde_json::to_value(direct.snapshot().radar).unwrap(),
        serde_json::to_value(remote.snapshot().radar).unwrap()
    );
    remote.cancel("00000000-0000-0000-0000-000000000123");
    assert!(remote.snapshot().notifications.is_empty());
    remote
        .set_settings(json!({"HandleObservatoryNotifications":false}))
        .unwrap();
    assert!(!remote.handles_notifications());
    assert!(remote.take_worker_errors().is_empty());
}

#[test]
fn metadata_inspection_does_not_initialize_or_bind_worker_services() {
    for kind in [
        WorkerKind::BioInsights,
        WorkerKind::Evaluator,
        WorkerKind::Hecate,
        WorkerKind::StellarOverlay,
    ] {
        let output = std::process::Command::new(executable())
            .args(["--kind", kind.name(), "--hello"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let info: worker_rpc::WorkerInfo = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(info, worker_rpc::WorkerInfo::current(kind));
        assert_eq!(worker_rpc::inspect(executable(), kind).unwrap(), info);
    }
}

#[test]
fn selection_is_independent_and_bad_artifact_never_silently_resets_to_builtin() {
    let dir = Directory::new();
    assert!(worker_rpc::Evaluator::from_executable(
        json!({}),
        &dir.0,
        &dir.0.join("missing-worker")
    )
    .is_err());
    let mut builtin = worker_rpc::Evaluator::new(json!({}));
    assert!(!builtin.is_external());
    let mut external = worker_rpc::BioInsights::from_executable(
        json!({"GalacticCodexRemote":false}),
        &dir.0,
        executable(),
    )
    .unwrap();
    assert!(external.is_external());
    external.on_events(&events());
    builtin.on_events(&events());
    let before = builtin.rows();
    external.reset();
    assert_eq!(before, builtin.rows());
    assert!(!before.is_empty());
}

#[test]
#[cfg(target_os = "linux")]
fn crashed_plugin_keeps_last_personal_state_reports_one_error_and_leaves_other_workers_alive() {
    let dir = Directory::new();
    let mut bio = worker_rpc::BioInsights::from_executable(
        json!({"GalacticCodexRemote":false}),
        &dir.0,
        executable(),
    )
    .unwrap();
    let mut eval = worker_rpc::Evaluator::from_executable(json!({}), &dir.0, executable()).unwrap();
    bio.on_events(&events());
    eval.on_events(&events());
    let personal = bio.state();
    let rows = bio.rows();
    let eval_rows = eval.rows();
    assert!(personal.is_object());
    assert!(!eval_rows.is_empty());
    assert!(std::process::Command::new("kill")
        .args(["-KILL", &bio.worker_process_id().unwrap().to_string()])
        .status()
        .unwrap()
        .success());
    bio.on_status(&json!({"Flags":0}));
    assert!(!bio.is_healthy());
    assert_eq!(personal, bio.state());
    assert_eq!(rows, bio.rows());
    assert_eq!(bio.take_worker_errors().len(), 1);
    bio.on_status(&json!({"Flags":0}));
    assert!(bio.take_worker_errors().is_empty());
    assert_eq!(eval_rows, eval.rows());
    assert!(eval.is_healthy());
}

#[test]
#[cfg(target_os = "linux")]
fn unrelated_elf_is_rejected_by_real_protocol_handshake() {
    let dir = Directory::new();
    let started = std::time::Instant::now();
    assert!(
        worker_rpc::Hecate::from_executable(json!({}), &dir.0, Path::new("/bin/true")).is_err()
    );
    assert!(started.elapsed() < std::time::Duration::from_secs(2));
    assert!(worker_rpc::inspect(Path::new("/bin/true"), WorkerKind::Hecate).is_err());
    assert!(!dir.0.join("hecate.db").exists());
}

#[test]
fn hecate_process_real_http_and_websocket_requests_bridge_through_host_rpc() {
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
        time::{Duration, Instant},
    };
    let dir = Directory::new();
    let mut remote = worker_rpc::Hecate::from_executable(
        json!({"RemoteLookups":false,"ServerSettings":{"EnableWebServer":false}}),
        &dir.0,
        executable(),
    )
    .unwrap();
    let mut port = None;
    for _ in 0..30 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let candidate = listener.local_addr().unwrap().port();
        drop(listener);
        if !(8000..=58999).contains(&candidate) {
            continue;
        }
        if remote.try_set_settings(json!({"ServerSettings":{"EnableWebServer":true,"ServerPort":candidate,"ServerLANIPAddress":""}})).is_ok() {port=Some(candidate);break;}
    }
    let port = port.expect("Available source-valid listener port");
    remote
        .register_plugin("Native QA", json!("native-qa-target"))
        .unwrap();
    let next = |remote: &mut worker_rpc::Hecate, command: &str| {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if let Some(event) = remote
                .take_messages()
                .into_iter()
                .find(|v| v["Command"] == command)
            {
                break event;
            }
            assert!(Instant::now() < deadline, "Missing {command}");
            std::thread::sleep(Duration::from_millis(2));
        }
    };
    let http = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(4)))
            .unwrap();
        write!(stream,"GET /nativeqa/main?roundtrip=1 HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n").unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    });
    let event = next(&mut remote, "WebRequest");
    assert_eq!(event["Target"], "native-qa-target");
    assert!(event["Payload"]["URI"]
        .as_str()
        .unwrap()
        .ends_with("/nativeqa/main?roundtrip=1"));
    remote.respond_web(event["Payload"]["RequestID"].as_str().unwrap(),json!({"ContentType":"application/json","Content":"{\"transport\":\"Δ\"}","StatusCode":201})).unwrap();
    let response = http.join().unwrap();
    assert!(response.starts_with("HTTP/1.1 201"));
    assert!(response.ends_with("{\"transport\":\"Δ\"}"));
    let (mut socket, _) =
        tungstenite::connect(format!("ws://127.0.0.1:{port}/ws/nativeqa").as_str()).unwrap();
    if let tungstenite::stream::MaybeTlsStream::Plain(stream) = socket.get_mut() {
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
    }
    socket
        .send(tungstenite::Message::Text(
            "http://localhost/nativeqa/main".into(),
        ))
        .unwrap();
    let event = next(&mut remote, "WebSocketRequest");
    assert_eq!(event["Target"], "native-qa-target");
    remote
        .handle_message(
            "WebSocketResponse",
            &json!({"ClientID":event["Payload"]["ClientID"],"Json":"{\"actual\":true}"}),
        )
        .unwrap();
    assert_eq!(
        socket.read().unwrap().into_text().unwrap(),
        "{\"actual\":true}"
    );
    socket.close(None).unwrap();
    assert!(remote.is_healthy());
    assert!(remote.take_worker_errors().is_empty());
}
