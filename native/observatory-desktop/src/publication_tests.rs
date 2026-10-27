use super::*;
use std::time::Instant;

fn isolated_engine() -> (tempfile::TempDir, Engine, Arc<Mutex<Snapshot>>) {
    let temporary = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::from_roots(
        temporary.path().join("config"),
        temporary.path().join("data"),
        temporary.path().join("cache"),
    );
    paths.create_dirs().unwrap();
    let snapshot = Arc::new(Mutex::new(Snapshot::default()));
    let engine = Engine::new(Settings {
        popup: false, speech: false,
        plugins: json!({"Hecate":{"RemoteLookups":false,"ServerSettings":{"EnableWebServer":false}},"BioInsights":{"GalacticCodexRemote":false}}),
        ..Default::default()
    }, paths, snapshot.clone()).unwrap();
    (temporary, engine, snapshot)
}
fn journal(data: Value) -> JournalEvent {
    JournalEvent::parse(&data.to_string()).unwrap()
}
fn status(engine: &mut Engine, data: Value) {
    engine.scan(ScanBatch {
        updates: vec![observatory_io::AuxiliaryUpdate {
            name: "Status.json".into(),
            data,
        }],
        ..Default::default()
    });
    engine.publish();
}

#[test]
fn idle_and_position_updates_reuse_tab_data_but_keep_live_state() {
    let (_directory, mut engine, snapshot) = isolated_engine();
    status(
        &mut engine,
        json!({"GuiFocus":0,"Latitude":0.0,"Longitude":0.0,"Heading":0}),
    );
    let before = snapshot.lock().unwrap().clone();
    engine.publish();
    engine.process(journal(json!({"event":"MiningRefined","Type":"Gold"})));
    status(
        &mut engine,
        json!({"GuiFocus":0,"Latitude":0.1,"Longitude":0.2,"Heading":90}),
    );
    let after = snapshot.lock().unwrap().clone();
    assert_eq!(after.total, before.total + 1);
    assert_eq!(after.last_event, "MiningRefined");
    assert_eq!(after.status["Heading"], 90);
    assert_eq!(after.overlay.radar.heading, 90.0);
    assert_eq!(after.overlay.radar.latitude, 0.1);
    assert_eq!(after.overlay.radar.longitude, 0.2);
    assert_eq!(after.worker_revision, before.worker_revision);
    assert!(Arc::ptr_eq(&before.bio_main_view, &after.bio_main_view));
    assert!(Arc::ptr_eq(&before.bio_summary, &after.bio_summary));
    assert!(Arc::ptr_eq(&before.eval_summary, &after.eval_summary));
    assert!(Arc::ptr_eq(
        &before.plugin_rows["BioInsights"],
        &after.plugin_rows["BioInsights"]
    ));
    assert!(Arc::ptr_eq(
        &before.plugin_rows["Evaluator"],
        &after.plugin_rows["Evaluator"]
    ));
    // Focus is visible Evaluator data; it must refresh without rebuilding Bio.
    status(
        &mut engine,
        json!({"GuiFocus":9,"Latitude":0.1,"Longitude":0.2,"Heading":90}),
    );
    let focused = snapshot.lock().unwrap();
    assert_eq!(focused.eval_summary["GuiFocus"], 9);
    assert!(!Arc::ptr_eq(&after.eval_summary, &focused.eval_summary));
    assert!(Arc::ptr_eq(&after.bio_summary, &focused.bio_summary));
}

#[test]
fn body_selection_clear_and_journal_updates_preserve_previous_frames() {
    let (_directory, mut engine, snapshot) = isolated_engine();
    engine.process(journal(
        json!({"event":"Location","SystemAddress":1,"StarSystem":"Test"}),
    ));
    engine.process(journal(json!({"event":"Scan","SystemAddress":1,"BodyID":1,"BodyName":"Test 1","PlanetClass":"Rocky body","BioSignals":1})));
    engine.publish();
    let before = snapshot.lock().unwrap().clone();
    assert_eq!(before.bio_main_view["Groups"][0]["Current"], false);
    status(&mut engine, json!({"BodyName":"Test 1"}));
    let selected = snapshot.lock().unwrap().clone();
    assert_eq!(selected.bio_main_view["Groups"][0]["Current"], true);
    assert_eq!(before.bio_main_view["Groups"][0]["Current"], false);
    assert!(Arc::ptr_eq(&before.bio_summary, &selected.bio_summary));
    engine.process(journal(
        json!({"event":"ReceiveText","Channel":"local","From":"Other","Message":"First message"}),
    ));
    engine.publish();
    let populated = snapshot.lock().unwrap().clone();
    assert_eq!(populated.messages.len(), 1);
    engine.clear("Communicator");
    engine.publish();
    assert!(snapshot.lock().unwrap().messages.is_empty());
    assert_eq!(populated.messages.len(), 1);
    engine.process(journal(
        json!({"event":"ReceiveText","Channel":"local","From":"Other","Message":"Next message"}),
    ));
    engine.publish();
    assert_eq!(snapshot.lock().unwrap().messages[0].message, "Next message");
}

#[test]
fn lua_status_rows_update_without_mutating_previous_frames() {
    let (directory, mut engine, snapshot) = isolated_engine();
    let source = directory.path().join("status.lua");
    std::fs::write(
        &source,
        r#"return {name="Status rows",id="test.status",
        on_status=function(s) core.add_row{heading=s.Heading} end,
        on_message=function() core.clear_rows();core.add_row{reset=true} end}"#,
    )
    .unwrap();
    engine
        .extensions
        .push(Extension::load(&source, None, directory.path()).unwrap());
    status(&mut engine, json!({"Heading":0}));
    let previous = snapshot.lock().unwrap().clone();
    status(&mut engine, json!({"Heading":90}));
    let current = snapshot.lock().unwrap().clone();
    assert_eq!(previous.plugin_rows["Status rows"].len(), 1);
    assert_eq!(current.plugin_rows["Status rows"].len(), 2);
    assert_eq!(current.plugin_rows["Status rows"][1]["heading"], 90);
    assert!(current.worker_revision > previous.worker_revision);
    engine.call_plugins("on_message", &json!({"data":{"target":"test.status"}}));
    let reset = snapshot.lock().unwrap().clone();
    assert_eq!(
        reset.plugin_rows["Status rows"].as_slice(),
        &[json!({"reset":true})]
    );
    assert_eq!(current.plugin_rows["Status rows"].len(), 2);
}

#[test]
fn source_notification_channels_and_native_overrides_are_independent() {
    let (directory, mut engine, snapshot) = isolated_engine();
    engine
        .settings
        .extra
        .insert("disabled_plugins".into(), json!(["StellarOverlay"]));
    engine.settings.popup = true;
    engine.settings.speech = true;
    engine
        .settings
        .extra
        .insert("chime_file".into(), json!("unused-test-chime"));
    for mask in [0, 1, 2, 4] {
        let settings = engine.notification_settings_for(mask);
        assert_eq!(settings.popup, mask == 1);
        assert_eq!(settings.speech, mask == 2);
        assert_eq!(settings.extra.contains_key("chime_file"), mask == 2);
    }
    for (name, popup, audio) in [
        ("ordinary", false, false),
        ("visual", true, false),
        ("vocal", false, true),
    ] {
        let path = directory.path().join(format!("{name}.lua"));
        std::fs::write(&path,format!(r#"return {{name="{name}",id="test.{name}",override_popup={popup},override_audio={audio},
            on_notification=function(n) core.add_row{{rendering=n.rendering}} end}}"#)).unwrap();
        engine
            .extensions
            .push(Extension::load(&path, None, directory.path()).unwrap());
    }
    engine.call_plugins("on_notification", &json!({"rendering":0}));
    assert!(snapshot.lock().unwrap().plugin_rows.is_empty());
    for mask in [1, 2, 4] {
        engine.call_plugins("on_notification", &json!({"rendering":mask}));
    }
    let state = snapshot.lock().unwrap();
    assert_eq!(
        state.plugin_rows["ordinary"].as_slice(),
        &[json!({"rendering":4})]
    );
    assert_eq!(
        state.plugin_rows["visual"].as_slice(),
        &[json!({"rendering":1}), json!({"rendering":4})]
    );
    assert_eq!(
        state.plugin_rows["vocal"].as_slice(),
        &[json!({"rendering":2}), json!({"rendering":4})]
    );
}

/// Opt-in release benchmark; input journals remain read-only and all output is
/// placed in an automatically removed directory. No remote lookup is enabled.
#[test]
#[ignore]
fn profile_publication_and_snapshot_clone() {
    let journal_dir = std::env::var_os("OBSERVATORY_PROFILE_JOURNALS")
        .map(PathBuf::from)
        .expect("Set OBSERVATORY_PROFILE_JOURNALS to the read-only journal directory");
    let temporary = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::from_roots(
        temporary.path().join("config"),
        temporary.path().join("data"),
        temporary.path().join("cache"),
    );
    paths.create_dirs().unwrap();
    let snapshot = Arc::new(Mutex::new(Snapshot::default()));
    let mut engine = Engine::new(Settings {
        popup: false, speech: false,
        plugins: json!({"Hecate":{"RemoteLookups":false},"BioInsights":{"GalacticCodexRemote":false}}),
        ..Default::default()
    }, paths, snapshot.clone()).unwrap();
    let mut reader = JournalTailer::new(journal_dir);
    engine.state("Reading all journals", true);
    let replay_started = Instant::now();
    reader
        .replay_all(&AtomicBool::new(false), |item| {
            if let ReplayItem::Event(event) = item {
                engine.process(event);
            }
        })
        .unwrap();
    engine.state("Idle", false);
    engine.publish();
    let replay_ms = replay_started.elapsed().as_secs_f64() * 1000.0;
    if let Some(path) = std::env::var_os("OBSERVATORY_PROFILE_DUMP") {
        let frame = snapshot.lock().unwrap().clone();
        std::fs::write(path,serde_json::to_vec(&json!({
            "bio":{"main_view":frame.bio_main_view,"summary":frame.bio_summary,"settings":engine.settings.plugins["BioInsights"]},
            "evaluator":{"summary":frame.eval_summary,"rows":frame.plugin_rows["Evaluator"],"settings":engine.settings.plugins["Evaluator"]}
        })).unwrap()).unwrap();
    }
    let iterations = 50u32;
    let started = Instant::now();
    for _ in 0..iterations {
        engine.publish();
    }
    let publish_ms = started.elapsed().as_secs_f64() * 1000.0 / f64::from(iterations);
    let started = Instant::now();
    for _ in 0..iterations {
        let frame = snapshot.lock().unwrap().clone();
        std::hint::black_box(frame);
    }
    let clone_ms = started.elapsed().as_secs_f64() * 1000.0 / f64::from(iterations);
    let started = Instant::now();
    for _ in 0..iterations {
        engine.projections_dirty = ProjectionDirty::all();
        engine.publish();
    }
    let dirty_publish_ms = started.elapsed().as_secs_f64() * 1000.0 / f64::from(iterations);
    let snapshot = snapshot.lock().unwrap();
    println!(
        "{}",
        json!({"events":snapshot.total,"replay_ms":replay_ms,"publish_ms":publish_ms,"dirty_publish_ms":dirty_publish_ms,"snapshot_clone_ms":clone_ms,"metadata_bytes":serde_json::to_vec(&snapshot.plugin_details).unwrap().len(),"bio_summary_bytes":serde_json::to_vec(&snapshot.bio_summary).unwrap().len(),"bio_view_bytes":serde_json::to_vec(&snapshot.bio_main_view).unwrap().len(),"bio_rows":snapshot.plugin_rows["BioInsights"].len(),"evaluator_rows":snapshot.plugin_rows["Evaluator"].len()})
    );
}
