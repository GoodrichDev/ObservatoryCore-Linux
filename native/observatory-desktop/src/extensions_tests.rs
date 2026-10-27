use super::*;

fn script(dir: &Path, name: &str, source: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, source).unwrap();
    path
}

#[test]
fn notification_lifecycle_and_journal_fields_are_real() {
    let dir = tempfile::tempdir().unwrap();
    let path = script(
        dir.path(),
        "test.lua",
        r#"return {name="Test",id="test",on_journal=function(e)
        local id=core.notify{title=e.event,detail=tostring(e.SystemAddress)}
        core.update_notification(id,{detail="Updated",persistent=true});core.cancel_notification(id)
        core.add_finding{title="durable"};core.add_row(e)
    end}"#,
    );
    let mut plugin = Extension::load(&path, None, dir.path()).unwrap();
    plugin
        .call(
            "on_journal",
            &json!({"event":"Scan","SystemAddress":123,"Future":{"flag":true}}),
        )
        .unwrap();
    let out = plugin.take_output();
    assert_eq!(out.findings[0].plugin, "Test");
    assert_eq!(out.notifications.len(), 3);
    assert_eq!(out.notifications[0]["title"], "Scan");
    assert_eq!(out.notifications[0]["id"], out.notifications[2]["id"]);
    assert_eq!(out.notifications[1]["action"], "update");
    assert_eq!(out.notifications[2]["action"], "cancel");
    assert_eq!(out.rows[0]["Future"]["flag"], true);
}

#[test]
fn saved_settings_apply_before_on_load_and_defaults_merge() {
    let dir = tempfile::tempdir().unwrap();
    script(
        dir.path(),
        "test.lua",
        r#"return {name="Test",settings={enabled=false,nested={a=1,b=2}},
        on_load=function() core.add_row(plugin.settings) end}"#,
    );
    let (mut plugins, errors) = discover(
        dir.path(),
        &json!({"Test":{"enabled":true,"nested":{"a":3}}}),
        dir.path(),
    );
    assert!(errors.is_empty(), "{errors:?}");
    let rows = plugins[0].take_output().rows;
    assert_eq!(rows[0]["enabled"], true);
    assert_eq!(rows[0]["nested"], json!({"a":3,"b":2}));
}

#[test]
fn messages_include_host_identity_and_clear_rows_is_explicit() {
    let dir = tempfile::tempdir().unwrap();
    let path = script(
        dir.path(),
        "test.lua",
        r#"return {name="Sender",id="sender",version="1.2.3",
        on_journal=function() core.add_row{old=true};core.clear_rows();core.add_row{new=true};core.send_message{source="forged",hello=42} end}"#,
    );
    let mut plugin = Extension::load(&path, None, dir.path()).unwrap();
    plugin.call("on_journal", &json!({})).unwrap();
    let output = plugin.take_output();
    assert!(output.clear_rows);
    assert_eq!(output.rows, [json!({"new":true})]);
    assert_eq!(
        output.messages[0]["source"],
        json!({"name":"Sender","id":"sender","version":"1.2.3"})
    );
    assert_eq!(output.messages[0]["data"]["hello"], 42);
    assert!(!plugin.take_output().clear_rows);
}

#[test]
fn plugin_storage_persists_and_is_isolated_by_id() {
    let dir = tempfile::tempdir().unwrap();
    let first = script(
        dir.path(),
        "a.lua",
        r#"return {name="A",id="a",on_load=function()
        local count=core.get_storage("count",0)+1;core.set_storage("count",count);core.add_row{count=count} end}"#,
    );
    let second = script(
        dir.path(),
        "b.lua",
        r#"return {name="B",id="b",on_load=function() core.add_row{count=core.get_storage("count",0)} end}"#,
    );
    let mut one = Extension::load(&first, None, dir.path()).unwrap();
    assert_eq!(one.take_output().rows[0]["count"], 1);
    let mut reloaded = Extension::load(&first, None, dir.path()).unwrap();
    assert_eq!(reloaded.take_output().rows[0]["count"], 2);
    let mut other = Extension::load(&second, None, dir.path()).unwrap();
    assert_eq!(other.take_output().rows[0]["count"], 0);
    assert!(dir.path().join("storage/a/state.json").is_file());
}

#[test]
fn failed_callbacks_roll_back_output_storage_settings_then_quarantine() {
    let dir = tempfile::tempdir().unwrap();
    let path = script(
        dir.path(),
        "test.lua",
        r#"return {name="Test",id="test",settings={a=1},
        on_load=function() core.add_row{preserved=true} end,
        on_journal=function() core.clear_rows();core.add_row{discard=true};core.notify{title="discard"};
            core.set_storage("discard",true);core.save_settings{a=2};error("failed") end}"#,
    );
    let mut plugin = Extension::load(&path, None, dir.path()).unwrap();
    assert!(plugin.call("on_journal", &json!({})).is_err());
    assert!(!plugin.enabled);
    let output = plugin.take_output();
    assert_eq!(output.rows, [json!({"preserved":true})]);
    assert!(!output.clear_rows);
    assert!(output.notifications.is_empty());
    assert!(output.settings.is_none());
    assert_eq!(plugin.settings["a"], 1);
    assert!(!dir.path().join("storage/test/state.json").exists());
    assert!(plugin.call("on_journal", &json!({})).is_ok());
    assert!(plugin.take_output().rows.is_empty());
}

#[test]
fn infinite_callbacks_and_coroutines_are_bounded() {
    let dir = tempfile::tempdir().unwrap();
    for (index, body) in [
        "while true do end",
        "coroutine.resume(coroutine.create(function() while true do end end))",
    ]
    .iter()
    .enumerate()
    {
        let path = script(
            dir.path(),
            &format!("{index}.lua"),
            &format!("return {{name='Loop',on_journal=function() {body} end}}"),
        );
        let mut plugin = Extension::load(&path, None, dir.path()).unwrap();
        let start = Instant::now();
        let result = plugin.call("on_journal", &json!({}));
        assert!(start.elapsed() < Duration::from_secs(2));
        assert!(result.is_err());
        assert!(!plugin.enabled);
    }
}

#[test]
fn malformed_sources_ids_storage_and_disallowed_io_preserve_files() {
    let dir = tempfile::tempdir().unwrap();
    for (index, source) in [
        "not lua",
        "return {name='X',id='../other'}",
        "return {name='X',on_load=function() os.execute('true') end}",
    ]
    .iter()
    .enumerate()
    {
        let path = script(dir.path(), &format!("{index}.lua"), source);
        assert!(Extension::load(&path, None, dir.path()).is_err());
    }
    let storage = dir.path().join("storage/bad");
    fs::create_dir_all(&storage).unwrap();
    fs::write(storage.join("state.json"), b"broken").unwrap();
    let path = script(dir.path(), "bad.lua", "return {name='Bad',id='bad'}");
    assert!(Extension::load(&path, None, dir.path()).is_err());
    assert_eq!(fs::read(storage.join("state.json")).unwrap(), b"broken");
}

#[test]
fn reload_reads_new_source_duplicate_ids_skip_on_load() {
    let dir = tempfile::tempdir().unwrap();
    let path = script(
        dir.path(),
        "a.lua",
        "return {name='A',id='same',on_load=function() core.set_storage('value',1) end}",
    );
    let mut first = Extension::load(&path, None, dir.path()).unwrap();
    first.call("on_ready", &json!({})).unwrap();
    script(dir.path(), "a.lua", "return {name='A',id='same',version='2',on_load=function() core.set_storage('value',2) end}");
    script(
        dir.path(),
        "b.lua",
        "return {name='B',id='same',on_load=function() core.set_storage('value',999) end}",
    );
    let (plugins, errors) = discover(dir.path(), &json!({}), dir.path());
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].version, "2");
    assert_eq!(errors.len(), 1);
    let stored: Value =
        serde_json::from_slice(&fs::read(dir.path().join("storage/same/state.json")).unwrap())
            .unwrap();
    assert_eq!(stored["value"], 2);
}

#[test]
fn export_runs_under_budget_and_missing_export_falls_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = script(
        dir.path(),
        "test.lua",
        "return {name='Test',on_export=function(delimiter) return 'a'..delimiter..'b\\n' end}",
    );
    let mut plugin = Extension::load(&path, None, dir.path()).unwrap();
    assert_eq!(plugin.export(";").unwrap().as_deref(), Some("a;b\n"));
    let path = script(dir.path(), "other.lua", "return {name='Other'}");
    let mut plugin = Extension::load(&path, None, dir.path()).unwrap();
    assert!(plugin.export(",").unwrap().is_none());
}

#[test]
fn empty_object_arguments_and_notification_ids_cannot_escape_scope() {
    let dir = tempfile::tempdir().unwrap();
    let path = script(
        dir.path(),
        "test.lua",
        r#"return {name="Test",id="test",on_journal=function()
        core.save_settings{};local id=core.notify{};core.update_notification(id,{});
        core.cancel_notification("other:notification");core.add_finding{}
    end}"#,
    );
    let mut plugin = Extension::load(&path, None, dir.path()).unwrap();
    plugin.call("on_journal", &json!({})).unwrap();
    let output = plugin.take_output();
    assert_eq!(output.settings, Some(json!({})));
    assert_eq!(output.notifications[2]["id"], "test:other:notification");
}

#[test]
fn sample_extension_executes_real_jumps_storage_messages_and_export() {
    let dir = tempfile::tempdir().unwrap();
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../plugins/sample.lua");
    let mut plugin = Extension::load(&path, None, dir.path()).unwrap();
    plugin.call("on_ready", &json!({})).unwrap();
    plugin
        .call("on_state_changed", &json!({"batch":false}))
        .unwrap();
    plugin.call("on_journal",&json!({"event":"FSDJump","timestamp":"2026-10-25T00:00:00Z","StarSystem":"Sol","JumpDist":12.5})).unwrap();
    let output = plugin.take_output();
    assert_eq!(output.rows.len(), 1);
    assert_eq!(output.rows[0]["System"], "Sol");
    assert_eq!(output.messages[0]["data"]["system"], "Sol");
    assert_eq!(output.notifications.len(), 1);
    assert!(plugin.export(",").unwrap().unwrap().contains("Sol,12.5"));
    plugin.call("on_shutdown", &json!({})).unwrap();
    assert_eq!(plugin.take_output().notifications[0]["action"], "cancel");
}

#[test]
fn settings_and_context_assignment_do_not_run_plugin_metamethods() {
    let dir = tempfile::tempdir().unwrap();
    let path = script(
        dir.path(),
        "test.lua",
        r#"return {name="Test",on_load=function()
        plugin.settings=nil;setmetatable(plugin,{__newindex=function() core.log("reentered") end});
        core.save_settings{ok=true};core.status=nil;
        setmetatable(core,{__newindex=function() error("context must be raw") end})
    end}"#,
    );
    let mut plugin = Extension::load(&path, None, dir.path()).unwrap();
    plugin.set_context("status", &json!({"Flags":1})).unwrap();
    let output = plugin.take_output();
    assert_eq!(output.settings, Some(json!({"ok":true})));
    assert!(output.logs.is_empty());
}

#[test]
fn long_generated_scoped_ids_remain_usable_for_update() {
    let plugin_id = "a".repeat(128);
    let local_id = "b".repeat(256);
    let scoped = notification_id(&plugin_id, &local_id).unwrap();
    assert_eq!(notification_id(&plugin_id, &scoped).unwrap(), scoped);
}

#[test]
fn json_helpers_round_trip_arrays_objects_unicode_and_explicit_null() {
    let dir = tempfile::tempdir().unwrap();
    let path = script(
        dir.path(),
        "json.lua",
        r#"
        local initial=core.json_decode('{"empty_array":[],"empty_object":{},"null_value":null,"array":[null,true,12.5],"text":"日本語 🚀"}')
        return {name="JSON",on_journal=function()
            assert(initial.null_value==core.null)
            core.add_row{encoded=core.json_encode(initial),nil_encoded=core.json_encode(nil)}
        end}
    "#,
    );
    let mut plugin = Extension::load(&path, None, dir.path()).unwrap();
    plugin.call("on_journal", &json!({})).unwrap();
    let output = plugin.take_output();
    let decoded: Value = serde_json::from_str(output.rows[0]["encoded"].as_str().unwrap()).unwrap();
    assert_eq!(
        decoded,
        json!({"empty_array":[],"empty_object":{},"null_value":null,"array":[null,true,12.5],"text":"日本語 🚀"})
    );
    assert_eq!(output.rows[0]["nil_encoded"], "null");
}

#[test]
fn json_helpers_reject_malformed_input_cycles_depth_and_nonfinite_values() {
    let dir = tempfile::tempdir().unwrap();
    let path = script(
        dir.path(),
        "json.lua",
        r#"return {name="JSON",on_journal=function()
        assert(not pcall(core.json_decode,'{"broken":'))
        assert(not pcall(core.json_decode,string.rep('[',150)..'0'..string.rep(']',150)))
        local recursive={};recursive.self=recursive;assert(not pcall(core.json_encode,recursive))
        local root={};local item=root;for i=1,150 do item.next={};item=item.next end
        assert(not pcall(core.json_encode,root))
        assert(not pcall(core.json_encode,math.huge))
        assert(not pcall(core.json_encode,function() end))
        local mixed={1,2};mixed.name="hidden";assert(not pcall(core.json_encode,mixed))
        local marked=core.json_decode('[]');marked.name="hidden";assert(not pcall(core.json_encode,marked))
        core.add_row{ok=true}
    end}"#,
    );
    let mut plugin = Extension::load(&path, None, dir.path()).unwrap();
    plugin.call("on_journal", &json!({})).unwrap();
    assert_eq!(plugin.take_output().rows[0]["ok"], true);
}

#[test]
fn json_helpers_enforce_text_and_encoded_output_limits() {
    let dir = tempfile::tempdir().unwrap();
    let path = script(dir.path(), "json.lua", "return {name='JSON'}");
    let plugin = Extension::load(&path, None, dir.path()).unwrap();
    let core: Table = plugin.lua.globals().get("core").unwrap();
    let decode: Function = core.get("json_decode").unwrap();
    let encode: Function = core.get("json_encode").unwrap();
    let oversized = plugin
        .lua
        .create_string(vec![b' '; MAX_OUTPUT + 1])
        .unwrap();
    let error = decode
        .call::<mlua::Value>(oversized)
        .unwrap_err()
        .to_string();
    assert!(error.contains("JSON input exceeds 4 MiB"));
    let escapes = plugin
        .lua
        .create_string(vec![b'\\'; MAX_OUTPUT / 2 + 1])
        .unwrap();
    assert!(encode
        .call::<String>(escapes)
        .unwrap_err()
        .to_string()
        .contains("JSON output exceeds 4 MiB"));
    let malformed = decode
        .call::<mlua::Value>("{\"secret\":not-json}")
        .unwrap_err()
        .to_string();
    assert!(malformed.contains("Invalid JSON at line"));
    assert!(!malformed.contains("secret"));
}

#[test]
fn json_encoder_bounds_repeated_table_expansion() {
    let dir = tempfile::tempdir().unwrap();
    let path = script(
        dir.path(),
        "json.lua",
        r#"return {name="JSON",on_journal=function()
        local value={leaf=true};for i=1,25 do value={left=value,right=value} end
        local ok,error=pcall(core.json_encode,value)
        assert(not ok);assert(string.find(tostring(error),'limit'))
    end}"#,
    );
    let mut plugin = Extension::load(&path, None, dir.path()).unwrap();
    // Exercise the helper outside the instruction hook's wall-clock budget;
    // traversal itself must stop rather than expanding the 2^25 shared tree.
    let core: Table = plugin.lua.globals().get("core").unwrap();
    let encode: Function = core.get("json_encode").unwrap();
    let mut value = plugin.lua.create_table().unwrap();
    for _ in 0..25 {
        let next = plugin.lua.create_table().unwrap();
        next.raw_set("left", value.clone()).unwrap();
        next.raw_set("right", value).unwrap();
        value = next;
    }
    assert!(encode
        .call::<String>(value)
        .unwrap_err()
        .to_string()
        .contains("limit"));
    // Normal callbacks remain usable after an explicitly handled helper error.
    plugin.call("on_ready", &json!({})).unwrap();
}
