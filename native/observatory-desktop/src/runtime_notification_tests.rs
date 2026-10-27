use super::*;

#[test]
fn failing_extension_publishes_error_status_and_unchecked_enabled_metadata() {
    let (_folder, mut engine) = engine(false);
    let path = engine.paths.plugin_dir.join("broken.lua");
    std::fs::write(&path,"return {name='Broken worker',id='broken.worker',on_journal=function() error('independent error status fixture') end}").unwrap();
    engine
        .extensions
        .push(Extension::load(&path, None, &engine.paths.data_dir).unwrap());
    engine.process(JournalEvent::parse("{\"event\":\"Probe\"}").unwrap());
    engine.publish();
    let state = engine.snapshot.lock().unwrap();
    assert_eq!(state.plugin_details["Broken worker"]["status"], "Error");
    assert_eq!(
        state.plugin_details["Broken worker"]["worker_healthy"],
        false
    );
    assert!(
        !state
            .extensions
            .iter()
            .find(|p| p.0 == "Broken worker")
            .unwrap()
            .2
    );
}

fn engine(popup: bool) -> (tempfile::TempDir, Engine) {
    let folder = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::from_roots(
        folder.path().join("config"),
        folder.path().join("data"),
        folder.path().join("cache"),
    );
    paths.create_dirs().unwrap();
    std::fs::write(paths.plugin_dir.join("observer.lua"),r#"return {id='notify.observer',name='Observer',accept_batch_notifications=true,
      on_notification=function(n) core.add_row({event='show',title=n.title,id=n.id,ssml=n.title_ssml}) end,
      on_notification_update=function(n) core.add_row({event='update',title=n.title,id=n.id}) end,
      on_notification_cancel=function(n) core.add_row({event='cancel',id=n.id}) end}"#).unwrap();
    let mut settings = Settings {
        popup,
        speech: false,
        plugins: json!({"BioInsights":{"GalacticCodexRemote":false},"Hecate":{"RemoteLookups":false,"ServerSettings":{"EnableWebServer":false}}}),
        ..Default::default()
    };
    settings.extra.insert(
        "disabled_plugins".into(),
        json!(["BioInsights", "Evaluator", "Hecate", "StellarOverlay"]),
    );
    let engine = Engine::new(settings, paths, Arc::new(Mutex::new(Snapshot::default()))).unwrap();
    (folder, engine)
}
fn rows(engine: &mut Engine) -> Vec<Value> {
    engine.publish();
    engine
        .snapshot
        .lock()
        .unwrap()
        .plugin_rows
        .get("Observer")
        .map(|rows| rows.as_ref().clone())
        .unwrap_or_default()
}

#[test]
fn extension_notifications_use_shared_source_delivery_including_batch_and_pascal_case_fields() {
    let (_folder, mut engine) = engine(true);
    let script = engine.paths.plugin_dir.join("producer.lua");
    std::fs::write(&script, r#"return {id='source.producer',name='Producer',on_journal=function(e)
      if e.event=='Probe' then core.SendNotification{Title='Source batch',Detail='Body',Rendering=4} end
    end}"#).unwrap();
    engine
        .extensions
        .push(Extension::load(&script, None, &engine.paths.data_dir).unwrap());
    engine.state("Reading all journals", true);
    engine.process(JournalEvent::parse(r#"{"event":"Probe"}"#).unwrap());
    let actual = rows(&mut engine);
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0]["title"], "Source batch");
    assert!(engine.native_popups.snapshot().is_empty());
}

#[test]
fn source_state_queries_are_current_during_callbacks_and_cancellation_keeps_realtime_flag() {
    let (_folder, mut engine) = engine(false);
    let script = engine.paths.plugin_dir.join("states.lua");
    std::fs::write(
        &script,
        r#"return {id='source.states',name='States',on_state_changed=function(s)
      core.add_row{state=core.get_log_monitor_state(),batch=core.is_log_monitor_batch_reading()}
    end}"#,
    )
    .unwrap();
    engine
        .extensions
        .push(Extension::load(&script, None, &engine.paths.data_dir).unwrap());
    for (state, batch) in [
        ("Monitoring", false),
        ("Reading all journals", true),
        ("Read all cancelled", false),
        ("Monitoring", false),
        ("Idle", false),
    ] {
        engine.state(state, batch);
    }
    engine.publish();
    assert_eq!(
        engine.snapshot.lock().unwrap().plugin_rows["States"].as_ref(),
        &vec![
            json!({"state":1,"batch":false}),
            json!({"state":3,"batch":true}),
            json!({"state":9,"batch":false}),
            json!({"state":1,"batch":false}),
            json!({"state":0,"batch":false}),
        ]
    );
}

#[test]
fn source_state_is_published_before_explorer_global_notifications_on_batch_edges() {
    let (_folder, mut engine) = engine(false);
    let observer = engine.paths.plugin_dir.join("state-observer.lua");
    std::fs::write(
        &observer,
        r#"return {id='source.state.observer',name='State observer',accept_batch_notifications=true,
      on_notification=function() core.add_row{state=core.get_log_monitor_state()} end}"#,
    )
    .unwrap();
    engine
        .extensions
        .push(Extension::load(&observer, None, &engine.paths.data_dir).unwrap());
    let criteria = engine.paths.data_dir.join("global-state-fixture.lua");
    std::fs::write(
        &criteria,
        "---@Global\nnotify('Source state','Global','Initialised')\n---@End",
    )
    .unwrap();
    engine.explorer = Explorer::new(
        json!({"CustomCriteriaFile":criteria,"EnableCustomCriteria":false}),
        None,
    )
    .unwrap();
    engine.explorer_startup();
    engine.state("Monitoring", false);
    engine.state("Reading all journals", true);
    engine.state("Read all cancelled", false);
    engine.publish();
    assert_eq!(
        engine.snapshot.lock().unwrap().plugin_rows["State observer"].as_ref(),
        &vec![json!({"state":0}), json!({"state":3}), json!({"state":9}),]
    );
}

#[test]
fn source_plugin_channel_survives_batch_and_update_cancel_are_separate_events() {
    let (_folder, mut engine) = engine(false);
    engine.state("Reading all journals", true);
    let first = engine.dispatch_core_notification(json!({"Title":"Batch","Rendering":4}));
    assert_eq!(rows(&mut engine).len(), 1);
    engine.dispatch_core_notification(
        json!({"Guid":first["id"],"Action":"update","Title":"Suppressed","Rendering":7}),
    );
    assert_eq!(rows(&mut engine).len(), 1);
    engine.dispatch_core_notification(json!({"Guid":first["id"],"Action":"cancel","Rendering":0}));
    assert_eq!(rows(&mut engine)[1]["event"], "cancel");
    engine.state("Idle", false);
    engine.dispatch_core_notification(
        json!({"Guid":first["id"],"Action":"update","Title":"Live","Rendering":4}),
    );
    let actual = rows(&mut engine);
    assert_eq!(actual.len(), 4);
    assert_eq!(actual[2]["event"], "show");
    assert_eq!(actual[3]["event"], "update");
    assert_eq!(actual[2]["id"], first["id"]);
}

#[test]
fn source_update_visual_ignores_global_toggle_and_audio_only_update_leaves_popup_unchanged() {
    let (_folder, mut engine) = engine(true);
    let value =
        engine.dispatch_core_notification(json!({"title":"Original","rendering":1,"timeout":0}));
    assert_eq!(engine.native_popups.snapshot()[0].title, "Original");
    engine.settings.popup = false;
    engine.dispatch_core_notification(
        json!({"id":value["id"],"action":"update","title":"Visual update","rendering":1}),
    );
    assert_eq!(engine.native_popups.snapshot()[0].title, "Visual update");
    engine.dispatch_core_notification(
        json!({"id":value["id"],"action":"update","title":"Audio only","rendering":2}),
    );
    assert_eq!(engine.native_popups.snapshot()[0].title, "Visual update");
    engine.dispatch_core_notification(json!({"id":value["id"],"action":"cancel"}));
    assert!(engine.native_popups.snapshot().is_empty());
}

#[test]
fn explorer_payload_uses_source_body_title_ssml_and_grid_rows_preserve_duplicates() {
    let (_folder, mut engine) = engine(false);
    engine.process(JournalEvent::parse(r#"{"event":"Scan","timestamp":"2026-10-25T00:00:00Z","StarSystem":"Test","SystemAddress":1,"BodyID":5,"BodyName":"Test 1 a","PlanetClass":"Rocky body","Radius":100000}"#).unwrap());
    let actual = rows(&mut engine);
    assert_eq!(actual[0]["title"], "Body 1 a");
    assert!(actual[0]["ssml"].as_str().unwrap().contains("spell-out"));
    let finding = engine.findings.back().unwrap().clone();
    engine.batch = true;
    engine.record_with_notification(finding.clone(), false);
    engine.record_with_notification(finding, false);
    assert_eq!(engine.findings.len(), 3);
    assert_eq!(engine.finding_grid_times.len(), 3);
    engine.clear("Explorer");
    assert!(engine.finding_grid_times.is_empty());
}

#[test]
fn botanist_rerunning_log_cancels_old_source_id_and_clear_repopulates_only_on_grid_events() {
    let (_folder, mut engine) = engine(true);
    for line in [
        r#"{"event":"LoadGame","Odyssey":true}"#,
        r#"{"event":"SAASignalsFound","SystemAddress":1,"BodyID":5,"BodyName":"Test 5","Signals":[{"Type":"$SAA_SignalType_Biological;","Count":1}]}"#,
        r#"{"event":"ScanOrganic","SystemAddress":1,"Body":5,"Species_Localised":"Tussock","Genus":"$Codex_Ent_Tussocks_Genus_Name;","ScanType":"Log"}"#,
    ] {
        engine.process(JournalEvent::parse(line).unwrap());
    }
    let first = engine.native_popups.snapshot()[0].id.clone();
    let log=JournalEvent::parse(r#"{"event":"ScanOrganic","SystemAddress":1,"Body":5,"Species_Localised":"Tussock","Genus":"$Codex_Ent_Tussocks_Genus_Name;","ScanType":"Log"}"#).unwrap();
    engine.process(log.clone());
    let popup = engine.native_popups.snapshot();
    assert_eq!(popup.len(), 1);
    assert_ne!(popup[0].id, first);
    engine.clear("Botanist");
    engine.publish();
    assert!(engine.snapshot.lock().unwrap().botanist.is_empty());
    engine.process(log);
    engine.publish();
    assert_eq!(engine.snapshot.lock().unwrap().botanist.len(), 1);
}

#[test]
fn communicator_clear_hides_old_rows_but_allows_equal_new_rows_and_filter_restores_source_history()
{
    let (_folder, mut engine) = engine(false);
    let input = JournalEvent::parse(
        r#"{"event":"ReceiveText","Channel":"wing","From":"Pilot","Message":"Same"}"#,
    )
    .unwrap();
    engine.process(input.clone());
    engine.clear("Communicator");
    engine.process(input);
    engine.publish();
    assert_eq!(engine.snapshot.lock().unwrap().messages.len(), 1);
    let mut settings = engine.settings.clone();
    settings.plugins["Communicator"]["ShowWing"] = json!(false);
    engine.configure(settings).unwrap();
    let mut settings = engine.settings.clone();
    settings.plugins["Communicator"]["ShowWing"] = json!(true);
    engine.configure(settings).unwrap();
    engine.publish();
    assert_eq!(engine.snapshot.lock().unwrap().messages.len(), 2);
}
