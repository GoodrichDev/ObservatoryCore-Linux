//! Source PluginCore/PluginEventHandler integration through the real Engine.
use super::*;
fn engine(scripts: &[(&str, &str)], disabled: &[&str]) -> (tempfile::TempDir, Engine) {
    let directory = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::from_roots(
        directory.path().join("config"),
        directory.path().join("data"),
        directory.path().join("cache"),
    );
    paths.create_dirs().unwrap();
    for (name, source) in scripts {
        std::fs::write(paths.plugin_dir.join(format!("{name}.lua")), source).unwrap();
    }
    let mut settings = Settings {
        popup: false,
        speech: false,
        plugins: json!({"BioInsights":{"GalacticCodexRemote":false},"Hecate":{"RemoteLookups":false}}),
        ..Default::default()
    };
    settings
        .extra
        .insert("disabled_plugins".into(), json!(disabled));
    let engine = Engine::new(settings, paths, Arc::new(Mutex::new(Snapshot::default()))).unwrap();
    (directory, engine)
}
#[test]
fn typed_messages_deliver_once_exclude_sender_disabled_and_clone_each_payload() {
    let (_temp, mut engine) = engine(
        &[
            (
                "a",
                r#"return {name='Sender',id='source.sender',version='1.2.3',on_ready=function()core.SendPluginMessage{Command='Ping',Payload={value='original'}}end,on_message=function()core.add_row{self_received=true}end}"#,
            ),
            (
                "b",
                r#"return {name='Receiver',id='source.receiver',on_message=function(m)if m.data.Command=='Ping'then core.add_row{sender=m.source.id,version=m.source.version,value=m.data.Payload.value};m.data.Payload.value='changed';core.SendPluginMessage('source.sender',{Command='Reply',Payload={value=42}})end end}"#,
            ),
            (
                "c",
                r#"return {name='Other',id='source.other',on_message=function(m)if m.data.Command=='Ping'then core.add_row{value=m.data.Payload.value}end end}"#,
            ),
            (
                "d",
                r#"return {name='Disabled',id='source.disabled',on_message=function()core.add_row{received=true}end}"#,
            ),
        ],
        &["Disabled"],
    );
    engine.call_plugins("on_ready", &json!({}));
    engine.publish();
    let state = engine.snapshot.lock().unwrap();
    assert_eq!(
        state.plugin_rows["Receiver"].as_ref(),
        &vec![json!({"sender":"source.sender","version":"1.2.3","value":"original"})]
    );
    assert_eq!(
        state.plugin_rows["Other"].as_ref(),
        &vec![json!({"value":"original"})]
    );
    // A single Reply from Receiver is the only event Sender may receive.
    assert_eq!(state.plugin_rows["Sender"].len(), 1);
    assert!(!state.plugin_rows.contains_key("Disabled"));
    assert!(state.errors.is_empty(), "{:?}", state.errors);
}
#[test]
fn legacy_message_invokes_old_callback_then_one_typed_wrapper_with_empty_guid() {
    let (_temp, mut engine) = engine(
        &[
            (
                "a",
                r#"return {name='Legacy Sender',id='legacy.sender',version='2.4.0',on_ready=function()core.SendPluginMessage('Receiver',{legacy_value=17})end,on_message=function()core.add_row{self_received=true}end}"#,
            ),
            (
                "b",
                r#"return {name='Receiver',id='receiver',on_legacy_message=function(m)core.add_row{kind='old',value=m.message.legacy_value,sender=m.source.name,version=m.source.version}end,on_message=function(m)core.add_row{kind='typed',command=m.data.command,value=m.data.payload.message.legacy_value,guid=m.source.id}end}"#,
            ),
        ],
        &[],
    );
    engine.call_plugins("on_ready", &json!({}));
    engine.publish();
    let state = engine.snapshot.lock().unwrap();
    assert_eq!(
        state.plugin_rows["Receiver"].as_ref(),
        &vec![
            json!({"kind":"old","value":17,"sender":"Legacy Sender","version":"2.4.0"}),
            json!({"kind":"typed","command":"LegacyPluginMessage","value":17,"guid":"00000000-0000-0000-0000-000000000000"})
        ]
    );
    assert!(!state.plugin_rows.contains_key("Legacy Sender"));
}
#[test]
fn source_core_commands_reach_ui_and_parse_only_does_not_dispatch() {
    let (_temp, mut engine) = engine(
        &[
            (
                "a",
                r#"return {name='Caller',id='caller',on_ready=function()core.OpenSettings();core.OpenAbout('Evaluator');core.FocusPlugin('BioInsights');core.DisplayMessageBox('Title','Body');core.DeserializeEvent('{"event":"ParseOnly"}');core.DeserializeEvent('{"event":"Replay"}',true)end}"#,
            ),
            (
                "b",
                r#"return {name='Observer',id='observer',on_journal=function(e)core.add_row{event=e.event}end}"#,
            ),
        ],
        &[],
    );
    engine.call_plugins("on_ready", &json!({}));
    engine.publish();
    assert_eq!(engine.processed, 2);
    let state = engine.snapshot.lock().unwrap();
    assert_eq!(
        state.plugin_rows["Observer"].as_ref(),
        &vec![json!({"event":"Replay"})]
    );
    assert_eq!(
        state
            .ui_actions
            .iter()
            .map(|v| v["CoreCommand"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "OpenSettings",
            "OpenAbout",
            "FocusPlugin",
            "DisplayMessageBox"
        ]
    );
    assert_eq!(state.ui_actions[0]["Payload"]["Plugin"], "Caller");
    assert_eq!(
        state.ui_actions[3]["Payload"],
        json!({"Title":"Title","Message":"Body"})
    );
}
#[test]
fn core_theme_queries_use_source_builtin_names() {
    for (name, expected) in [("dark", "Dark"), ("LIGHT", "Light"), ("Survey", "Survey")] {
        let settings = Settings {
            theme: name.into(),
            ..Default::default()
        };
        assert_eq!(source_theme_name(&settings), expected);
    }
}

#[test]
fn legacy_callback_pairs_complete_per_recipient_before_the_next_plugin() {
    let (_temp, mut engine) = engine(
        &[
            (
                "a",
                r#"return {name='Emitter',id='emitter',on_ready=function()core.SendPluginMessage{legacy=true}end}"#,
            ),
            (
                "b",
                r#"return {name='B',id='b',on_legacy_message=function()core.SendPluginMessage('Recorder',{Command='Trace',Payload={value='B-old'}})end,on_message=function(m)if m.data.command=='LegacyPluginMessage'then core.SendPluginMessage('Recorder',{Command='Trace',Payload={value='B-new'}})end end}"#,
            ),
            (
                "c",
                r#"return {name='C',id='c',on_legacy_message=function()core.SendPluginMessage('Recorder',{Command='Trace',Payload={value='C-old'}})end,on_message=function(m)if m.data.command=='LegacyPluginMessage'then core.SendPluginMessage('Recorder',{Command='Trace',Payload={value='C-new'}})end end}"#,
            ),
            (
                "z",
                r#"return {name='Recorder',id='recorder',on_message=function(m)if m.data.Command=='Trace'then core.add_row{value=m.data.Payload.value}end end}"#,
            ),
        ],
        &[],
    );
    engine.call_plugins("on_ready", &json!({}));
    engine.publish();
    let state = engine.snapshot.lock().unwrap();
    assert_eq!(
        state.plugin_rows["Recorder"].as_ref(),
        &vec![
            json!({"value":"B-old"}),
            json!({"value":"B-new"}),
            json!({"value":"C-old"}),
            json!({"value":"C-new"})
        ]
    );
}

#[test]
fn manual_deserialize_reads_ancillary_after_event_without_counting_file_and_skips_batch() {
    let (temp, mut engine) = engine(
        &[(
            "observer",
            r#"return {name='Observer',id='observer',on_journal=function(e)core.add_row{event=e.event,count=e.Count}end,on_auxiliary=function(e)core.add_row{file=e.name,count=e.data.Count}end}"#,
        )],
        &[],
    );
    let journals = temp.path().join("journals");
    std::fs::create_dir_all(&journals).unwrap();
    engine.settings.journal_dir = Some(journals.clone());
    std::fs::write(
        journals.join("Cargo.json"),
        r#" { "event":"Cargo", "Count":99 } "#,
    )
    .unwrap();
    engine.native_messages(vec![("Observer".into(),json!({"target":"Core","command":"DeserializeEvent","payload":{"Json":r#"{"event":"Cargo","Count":1}"#,"Replay":true}}))]);
    engine.publish();
    assert_eq!(engine.processed, 1);
    assert_eq!(engine.last_event, "Cargo");
    assert_eq!(
        engine.snapshot.lock().unwrap().plugin_rows["Observer"].as_ref(),
        &vec![
            json!({"event":"Cargo","count":1}),
            json!({"event":"Cargo","count":99}),
            json!({"file":"Cargo.json","count":99})
        ]
    );
    engine.state("Preparing current system", true);
    engine.native_messages(vec![("Observer".into(),json!({"target":"Core","command":"DeserializeEvent","payload":{"Json":r#"{"event":"Cargo","Count":2}"#,"Replay":true}}))]);
    engine.publish();
    assert_eq!(engine.processed, 2);
    assert_eq!(
        engine.snapshot.lock().unwrap().plugin_rows["Observer"].len(),
        4
    );
}
#[test]
fn nested_deserialize_keeps_last_event_and_counters_in_source_callback_order() {
    let (_temp, mut engine) = engine(
        &[(
            "observer",
            r#"return {name='Observer',id='observer',on_journal=function(e)if e.event=='LoadGame'then core.DeserializeEvent('{"event":"Fileheader"}')end end}"#,
        )],
        &[],
    );
    engine.process(JournalEvent::parse(r#"{"event":"LoadGame"}"#).unwrap());
    assert_eq!(engine.processed, 2);
    assert_eq!(engine.last_event, "Fileheader");
}

#[test]
fn unrelated_worker_publication_does_not_invalidate_evaluator_page_layout_or_scroll() {
    let (_directory, mut engine) = engine(&[], &[]);
    engine.publish();
    let first = engine.snapshot.lock().unwrap().clone();
    engine.projections_dirty.findings = true;
    engine.publish();
    let unrelated = engine.snapshot.lock().unwrap().clone();
    assert_ne!(unrelated.worker_revision, first.worker_revision);
    assert_eq!(unrelated.evaluator_revision, first.evaluator_revision);
    assert!(Arc::ptr_eq(&unrelated.eval_summary, &first.eval_summary));
    assert_eq!(unrelated.bio_revision, first.bio_revision);
    assert!(Arc::ptr_eq(&unrelated.bio_main_view, &first.bio_main_view));
    engine.projections_dirty.evaluator = true;
    engine.projections_dirty.bio = true;
    engine.publish();
    let changed = engine.snapshot.lock().unwrap().clone();
    assert_ne!(changed.evaluator_revision, unrelated.evaluator_revision);
    assert_ne!(changed.bio_revision, unrelated.bio_revision);
    assert!(!Arc::ptr_eq(
        &changed.bio_main_view,
        &unrelated.bio_main_view
    ));
    assert!(!Arc::ptr_eq(&changed.eval_summary, &unrelated.eval_summary));
}

#[test]
fn initialized_native_peers_publish_source_ready_versions_without_self_or_disabled_peer() {
    let (_directory, active) = engine(&[], &[]);
    let summary = active.evaluator.summary();
    assert_eq!(
        summary["ReadyPeers"],
        json!({"BioInsights":"1.3.444.1052","Hecate":"1.3.444.761","StellarOverlay":"1.3.228.387"})
    );
    assert_eq!(summary["FID"], "F0");
    let (_directory, disabled) = engine(&[], &["Hecate"]);
    let summary = disabled.evaluator.summary();
    assert_eq!(
        summary["ReadyPeers"],
        json!({"BioInsights":"1.3.444.1052","StellarOverlay":"1.3.228.387"})
    );
    assert_eq!(summary["FID"], "F0");
}
