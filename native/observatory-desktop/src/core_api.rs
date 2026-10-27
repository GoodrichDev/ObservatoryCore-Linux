//! Native Core API equivalents. GUI operations become ordered host commands;
//! typed control descriptors retain source theme callback/property semantics.
use super::*;
#[derive(Clone)]
pub(super) struct RegisteredControl {
    pub table: Table,
    pub apply: Option<Function>,
}
fn context(lua: &Lua, key: &str, default: Value) -> mlua::Result<mlua::Value> {
    let core: Table = lua.globals().raw_get("core")?;
    let value: mlua::Value = core.raw_get(key)?;
    if value == mlua::Value::Nil {
        lua.to_value(&default)
    } else {
        Ok(value)
    }
}
fn queue(host: &Arc<Mutex<HostState>>, command: &str, payload: Value) -> mlua::Result<()> {
    let mut state = host.lock().unwrap();
    active(&mut state)?;
    let message = json!({"target":"Core","command":command,"payload":payload});
    state.output.reserve(&message)?;
    state.output.messages.push(message);
    Ok(())
}
fn group_id() -> String {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let count = NEXT_NOTIFICATION.fetch_add(1, Ordering::Relaxed) as u128;
    let raw = format!("{:032x}", time.rotate_left(32) ^ count);
    format!(
        "{}-{}-4{}-8{}-{}",
        &raw[..8],
        &raw[8..12],
        &raw[13..16],
        &raw[17..20],
        &raw[20..]
    )
}
fn apply_control(
    lua: &Lua,
    control: &RegisteredControl,
    seen: &mut HashSet<usize>,
) -> mlua::Result<()> {
    let key = control.table.to_pointer() as usize;
    if !seen.insert(key) {
        return Err(mlua::Error::runtime("Control hierarchy contains a cycle"));
    }
    if let Some(callback) = &control.apply {
        if !callback.call::<bool>(control.table.clone())? {
            seen.remove(&key);
            return Ok(());
        }
    }
    let details: Table = match context(lua, "theme_details", json!({}))? {
        mlua::Value::Table(t) => t,
        _ => lua.create_table()?,
    };
    let kind = control
        .table
        .get::<Option<String>>("type")?
        .or(control.table.get::<Option<String>>("Type")?)
        .unwrap_or_else(|| "Control".into());
    // The source themes top-level MenuStrip items before the strip itself;
    // unlike child Controls, these items do not use the ColourButton exclusion.
    if kind == "MenuStrip" {
        if let Some(items) = control.table.get::<Option<Table>>("items")? {
            for item in items.sequence_values::<Table>() {
                apply_control(
                    lua,
                    &RegisteredControl {
                        table: item?,
                        apply: control.apply.clone(),
                    },
                    seen,
                )?;
            }
        }
    }
    let props = control
        .table
        .get::<Option<Table>>("properties")?
        .unwrap_or(lua.create_table()?);
    if props.is_empty() {
        props.set("BackColor", lua.to_value(&Value::Null)?)?;
        props.set("ForeColor", lua.to_value(&Value::Null)?)?;
    }
    let keys: Vec<String> = props
        .clone()
        .pairs::<String, mlua::Value>()
        .filter_map(|pair| pair.ok().map(|(key, _)| key))
        .filter(|key| key.ends_with("Color"))
        .collect();
    for property in keys {
        let mut color: mlua::Value = details.raw_get(format!("{kind}.{property}"))?;
        if color == mlua::Value::Nil {
            color = details.raw_get(format!("Default.{property}"))?;
        }
        if color != mlua::Value::Nil {
            props.raw_set(property, color)?;
        }
    }
    control.table.raw_set("properties", props)?;
    if let Some(children) = control.table.get::<Option<Table>>("controls")? {
        for child in children.sequence_values::<Table>() {
            let child = child?;
            if child.get::<Option<String>>("Name")?.as_deref() != Some("ColourButton") {
                apply_control(
                    lua,
                    &RegisteredControl {
                        table: child,
                        apply: control.apply.clone(),
                    },
                    seen,
                )?;
            }
        }
    }
    seen.remove(&key);
    Ok(())
}
pub(super) fn apply_registered(lua: &Lua, host: &Arc<Mutex<HostState>>) -> mlua::Result<()> {
    let controls: Vec<_> = host.lock().unwrap().controls.values().cloned().collect();
    for control in controls {
        apply_control(lua, &control, &mut HashSet::new())?;
    }
    Ok(())
}
pub(super) fn install(lua: &Lua, host: &Arc<Mutex<HostState>>) -> mlua::Result<()> {
    let core: Table = lua.globals().get("core")?;
    core.set("Version", env!("CARGO_PKG_VERSION"))?;
    core.set("CurrentLogMonitorState", 0)?;
    core.set("IsLogMonitorBatchReading", false)?;
    let http = lua.create_table()?;
    http.set("request", core.get::<Function>("http_request")?)?;
    core.set("HttpClient", http)?;
    for (name, key, default) in [
        ("get_status", "status", json!({})),
        ("get_cargo", "cargo", json!({})),
        ("get_current_theme_name", "theme_name", json!("Dark")),
        ("get_current_theme_details", "theme_details", json!({})),
        ("get_log_monitor_state", "log_monitor_state", json!(0)),
        (
            "get_plugin_storage_folder",
            "PluginStorageFolder",
            json!(""),
        ),
        (
            "get_updated_plugins_folder",
            "updated_plugins_folder",
            json!(""),
        ),
    ] {
        core.set(
            name,
            lua.create_function(move |lua, ()| {
                let value = context(lua, key, default.clone())?;
                if key == "PluginStorageFolder" {
                    if let mlua::Value::String(path) = &value {
                        let path = path.to_str()?;
                        if !path.is_empty() {
                            fs::create_dir_all(path.as_ref()).map_err(mlua::Error::external)?;
                        }
                    }
                }
                Ok(value)
            })?,
        )?;
    }
    core.set(
        "is_log_monitor_batch_reading",
        lua.create_function(|lua, ()| {
            let flags: u32 = lua.from_value(context(lua, "log_monitor_state", json!(0))?)?;
            Ok(flags & 6 != 0)
        })?,
    )?;
    for (name, command) in [
        ("open_settings", "OpenSettings"),
        ("open_about", "OpenAbout"),
        ("focus_plugin", "FocusPlugin"),
    ] {
        let host = host.clone();
        core.set(
            name,
            lua.create_function(move |_, plugin: Option<String>| {
                let plugin = plugin.unwrap_or_else(|| host.lock().unwrap().name.clone());
                queue(&host, command, json!({"Plugin":plugin}))
            })?,
        )?;
    }
    let shared = host.clone();
    core.set(
        "display_message_box",
        lua.create_function(move |_, (title, message): (String, String)| {
            queue(
                &shared,
                "DisplayMessageBox",
                json!({"Title":title,"Message":message}),
            )
        })?,
    )?;
    let shared = host.clone();
    core.set(
        "get_plugin_error_logger",
        lua.create_function(move |lua, ()| {
            let host = shared.clone();
            lua.create_function(move |_, (error, context): (String, Option<String>)| {
                queue(
                    &host,
                    "PluginError",
                    json!({"Error":error,"Context":context.unwrap_or_default()}),
                )
            })
        })?,
    )?;
    let shared = host.clone();
    core.set(
        "deserialize_event",
        lua.create_function(move |lua, (json, replay): (String, Option<bool>)| {
            if json.len() > MAX_OUTPUT {
                return Err(mlua::Error::runtime("Journal event exceeds 4 MiB"));
            }
            let event = JournalEvent::parse(&json).map_err(mlua::Error::external)?;
            // The source parameter is named replay but is passed directly as invoke.
            queue(
                &shared,
                "DeserializeEvent",
                json!({"Json":json,"Replay":replay.unwrap_or(false)}),
            )?;
            let(kind,full_name)=journal_type(&event.event);
            let mut record=event.data.clone();
            record["Json"]=json!(event.json());
            record["Event"]=json!(event.event);
            record["Timestamp"]=event.data.get("timestamp").cloned().unwrap_or(Value::Null);
            lua.to_value(&json!({"journalType":kind,"journalTypeName":full_name,"journalEvent":record,"event":event.event}))
        })?,
    )?;
    // Core GUI operations emitted by this producer are marshalled by the host.
    let shared = host.clone();
    core.set(
        "send_plugin_message",
        lua.create_function(
            move |lua, (first, second): (mlua::Value, Option<mlua::Value>)| {
                let (target, message) = match second {
                    Some(message) => (
                        lua.from_value::<String>(first)?,
                        lua.from_value::<Value>(message)?,
                    ),
                    None => (String::new(), lua.from_value::<Value>(first)?),
                };
                let envelope = if message
                    .get("command")
                    .or_else(|| message.get("Command"))
                    .is_some()
                {
                    let mut envelope = message;
                    envelope["target"] = json!(target);
                    envelope
                } else {
                    json!({"target":target,"command":"LegacyPluginMessage","payload":{"message":message},"legacy":true})
                };
                let mut host = shared.lock().unwrap();
                active(&mut host)?;
                host.output.reserve(&envelope)?;
                host.output.messages.push(envelope);
                Ok(())
            },
        )?,
    )?;
    core.set(
        "execute_on_ui_thread",
        lua.create_function(|_, callback: Function| callback.call::<()>(()))?,
    )?;
    let shared = host.clone();
    core.set(
        "add_grid_items",
        lua.create_function(move |lua, (items, grouped): (Table, Option<bool>)| {
            let rows: Vec<Value> = lua.from_value(mlua::Value::Table(items))?;
            let group = grouped.unwrap_or(false).then(group_id);
            let mut host = shared.lock().unwrap();
            active(&mut host)?;
            for mut row in rows {
                if let Some(group) = &group {
                    if !row.is_object() {
                        row = json!({});
                    }
                    row["ObservatoryListViewGroupID"] = json!(group);
                }
                host.output.reserve(&row)?;
                host.output.rows.push(row);
            }
            Ok(())
        })?,
    )?;
    let shared = host.clone();
    core.set(
        "set_grid_items",
        lua.create_function(move |lua, items: Table| {
            let rows: Vec<Value> = lua.from_value(mlua::Value::Table(items))?;
            let mut host = shared.lock().unwrap();
            active(&mut host)?;
            host.output.rows.clear();
            host.output.clear_rows = true;
            for row in rows {
                host.output.reserve(&row)?;
                host.output.rows.push(row);
            }
            Ok(())
        })?,
    )?;
    let shared = host.clone();
    core.set(
        "register_control",
        lua.create_function(move |lua, (table, apply): (Table, Option<Function>)| {
            {
                let mut host = shared.lock().unwrap();
                active(&mut host)?;
            }
            if let Some(callback) = &apply {
                if !callback.call::<bool>(table.clone())? {
                    return Ok(());
                }
            }
            let key = table.to_pointer() as usize;
            {
                let host = shared.lock().unwrap();
                if host.controls.contains_key(&key) {
                    return Err(mlua::Error::runtime("Control is already registered"));
                }
            }
            let control = RegisteredControl { table, apply };
            shared.lock().unwrap().controls.insert(key, control.clone());
            apply_control(lua, &control, &mut HashSet::new())
        })?,
    )?;
    let shared = host.clone();
    core.set(
        "unregister_control",
        lua.create_function(move |_, control: Option<Table>| {
            let mut host = shared.lock().unwrap();
            active(&mut host)?;
            if let Some(control) = control {
                host.controls.remove(&(control.to_pointer() as usize));
            }
            Ok(())
        })?,
    )?;
    for (alias, original) in [
        ("add_grid_item", "add_row"),
        ("clear_grid", "clear_rows"),
        ("SendNotification", "notify"),
        ("UpdateNotification", "update_notification"),
        ("CancelNotification", "cancel_notification"),
        ("GetStatus", "get_status"),
        ("GetCargo", "get_cargo"),
        ("GetCurrentThemeName", "get_current_theme_name"),
        ("GetCurrentThemeDetails", "get_current_theme_details"),
        ("GetPluginErrorLogger", "get_plugin_error_logger"),
        ("OpenSettings", "open_settings"),
        ("OpenAbout", "open_about"),
        ("FocusPlugin", "focus_plugin"),
        ("DisplayMessageBox", "display_message_box"),
        ("DeserializeEvent", "deserialize_event"),
        ("ExecuteOnUIThread", "execute_on_ui_thread"),
        ("RegisterControl", "register_control"),
        ("UnregisterControl", "unregister_control"),
        ("AddGridItem", "add_grid_item"),
        ("AddGridItems", "add_grid_items"),
        ("SetGridItems", "set_grid_items"),
        ("ClearGrid", "clear_grid"),
        ("SaveSettings", "save_settings"),
        ("SendPluginMessage", "send_plugin_message"),
        ("PlayAudioFile", "play_audio_file"),
    ] {
        core.set(alias, core.get::<Function>(original)?)?;
    }
    Ok(())
}

// DefaultSorter.NaturalCompare uses one UTF-16 index and the longest digit
// run for both sides, including its observable leading-zero suffix behavior.
/// Sort original insertion-order rows using the source grid's group-leader
/// algorithm. Followers are appended in reverse original group order.
pub(super) fn journal_type(event: &str) -> (&'static str, &'static str) {
    static TYPES: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    let metadata = TYPES.get_or_init(|| {
        serde_json::from_str(include_str!("../../fixtures/core-framework-metadata.json"))
            .expect("pinned framework type metadata")
    });
    let kind = metadata["journalTypes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .find(|name| name.eq_ignore_ascii_case(event))
        .unwrap_or("JournalBase");
    (
        kind,
        metadata["journalTypeNames"][kind]
            .as_str()
            .expect("pinned framework full type name"),
    )
}
pub(super) fn sort_grid_rows(rows: &mut Vec<&Value>, column: &str, ascending: bool) {
    let original = rows.clone();
    let mut groups: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    let mut heads = Vec::new();
    for (index, row) in original.iter().enumerate() {
        if let Some(group) = row["ObservatoryListViewGroupID"].as_str() {
            let group_rows = groups.entry(group).or_default();
            if group_rows.is_empty() {
                heads.push(index);
            }
            group_rows.push(index);
        } else {
            heads.push(index);
        }
    }
    let text: Vec<std::borrow::Cow<'_, str>> = original
        .iter()
        .map(|row| match &row[column] {
            Value::String(text) => std::borrow::Cow::Borrowed(text.as_str()),
            Value::Null => std::borrow::Cow::Borrowed(""),
            value => std::borrow::Cow::Owned(value.to_string()),
        })
        .collect();
    heads.sort_by(|&left, &right| {
        let order = compare_grid_text(&text[left], &text[right]);
        if ascending {
            order
        } else {
            order.reverse()
        }
    });
    rows.clear();
    for index in heads {
        let head = original[index];
        rows.push(head);
        if let Some(group) = head["ObservatoryListViewGroupID"].as_str() {
            rows.extend(
                groups[group]
                    .iter()
                    .skip(1)
                    .rev()
                    .map(|&index| original[index]),
            );
        }
    }
}
pub(super) fn compare_grid_text(left: &str, right: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    fn digit(c: u16) -> Option<u16> {
        const STARTS: &[u16] = &[
            0x30, 0x660, 0x6f0, 0x7c0, 0x966, 0x9e6, 0xa66, 0xae6, 0xb66, 0xbe6, 0xc66, 0xce6,
            0xd66, 0xde6, 0xe50, 0xed0, 0xf20, 0x1040, 0x1090, 0x17e0, 0x1810, 0x1946, 0x19d0,
            0x1a80, 0x1a90, 0x1b50, 0x1bb0, 0x1c40, 0x1c50, 0xa620, 0xa8d0, 0xa900, 0xa9d0, 0xa9f0,
            0xaa50, 0xabf0, 0xff10,
        ];
        STARTS
            .iter()
            .find_map(|start| c.checked_sub(*start).filter(|n| *n < 10))
    }
    if left.is_ascii() && right.is_ascii() {
        let (left, right) = (left.as_bytes(), right.as_bytes());
        let mut index = 0;
        while index <= left.len() && index <= right.len() {
            if index == left.len() || index == right.len() {
                return left.len().cmp(&right.len());
            }
            if left[index].is_ascii_digit() && right[index].is_ascii_digit() {
                let a = left[index..]
                    .iter()
                    .take_while(|c| c.is_ascii_digit())
                    .count();
                let b = right[index..]
                    .iter()
                    .take_while(|c| c.is_ascii_digit())
                    .count();
                let count = a.max(b);
                for offset in 0..count {
                    let a = if offset < count - a {
                        b'0'
                    } else {
                        left[index + offset - (count - a)]
                    };
                    let b = if offset < count - b {
                        b'0'
                    } else {
                        right[index + offset - (count - b)]
                    };
                    let order = a.cmp(&b);
                    if order != Ordering::Equal {
                        return order;
                    }
                }
                index += count;
            } else {
                let order = left[index].cmp(&right[index]);
                if order != Ordering::Equal {
                    return order;
                }
                index += 1;
            }
        }
        return Ordering::Equal;
    }
    let left: Vec<u16> = left.encode_utf16().collect();
    let right: Vec<u16> = right.encode_utf16().collect();
    let mut i = 0;
    while i <= left.len() && i <= right.len() {
        if i == left.len() || i == right.len() {
            return left.len().cmp(&right.len());
        }
        if digit(left[i]).is_some() && digit(right[i]).is_some() {
            let a: Vec<_> = left[i..].iter().copied().map_while(digit).collect();
            let b: Vec<_> = right[i..].iter().copied().map_while(digit).collect();
            let count = a.len().max(b.len());
            let a = std::iter::repeat_n(0, count - a.len()).chain(a);
            let b = std::iter::repeat_n(0, count - b.len()).chain(b);
            let order = a.cmp(b);
            if order != Ordering::Equal {
                return order;
            }
            i += count;
        } else {
            let order = left[i].cmp(&right[i]);
            if order != Ordering::Equal {
                return order;
            }
            i += 1;
        }
    }
    Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;
    fn script(dir: &Path, source: &str) -> PathBuf {
        let path = dir.join("core.lua");
        fs::write(&path, source).unwrap();
        path
    }
    #[test]
    fn deserialize_event_base_json_retains_original_spacing_and_order() {
        let dir = tempfile::tempdir().unwrap();
        let path = script(
            dir.path(),
            r#"return {name='Json',on_load=function()local args=core.DeserializeEvent(' { "BodyID":42, "event":"Scan" } ');core.add_row{json=args.journalEvent.Json,event=args.journalEvent.Event,timestamp=args.journalEvent.Timestamp}end}"#,
        );
        let mut plugin = Extension::load(&path, None, dir.path()).unwrap();
        assert_eq!(
            plugin.take_output().rows[0],
            json!({"json":r#" { "BodyID":42, "event":"Scan" } "#,"event":"Scan","timestamp":null})
        );
    }
    #[test]
    fn framework_event_type_full_names_match_every_pinned_metadata_entry() {
        let metadata: Value =
            serde_json::from_str(include_str!("../../fixtures/core-framework-metadata.json"))
                .unwrap();
        for name in metadata["journalTypes"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
        {
            assert_eq!(
                journal_type(&name.to_lowercase()),
                (name, metadata["journalTypeNames"][name].as_str().unwrap())
            );
        }
        assert_eq!(
            journal_type("CargoFile"),
            ("CargoFile", "Observatory.Framework.Files.CargoFile")
        );
        assert_eq!(journal_type("FutureEvent").0, "JournalBase");
    }
    #[test]
    fn all_source_group_leader_and_reversed_follower_captures_match() {
        let fixtures: Value =
            serde_json::from_str(include_str!("../../fixtures/core-grid-oracle.json")).unwrap();
        for case in fixtures.as_array().unwrap() {
            let rows: Vec<Value> = case["input"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| {
                    let mut out = json!({"Id":row["Id"],"Title":row["Text"]});
                    if row["Group"] != "" {
                        out["ObservatoryListViewGroupID"] = row["Group"].clone();
                    }
                    out
                })
                .collect();
            for (key, ascending) in [("ascending", true), ("descending", false), ("third", true)] {
                let mut sorted: Vec<_> = rows.iter().collect();
                sort_grid_rows(&mut sorted, "Title", ascending);
                assert_eq!(
                    json!(sorted
                        .iter()
                        .map(|row| row["Id"].clone())
                        .collect::<Vec<_>>()),
                    case[key]
                );
            }
        }
    }
    #[test]
    fn grid_comparator_matches_all_900_original_defaultsorter_pairs() {
        let cases: Value =
            serde_json::from_str(include_str!("../../fixtures/core-sort-oracle.json")).unwrap();
        for case in cases.as_array().unwrap() {
            let order = compare_grid_text(
                case["left"].as_str().unwrap(),
                case["right"].as_str().unwrap(),
            );
            let actual = match order {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            };
            assert_eq!(actual, case["order"].as_i64().unwrap(), "{case}");
        }
    }
    #[test]
    fn core_export_bytes_and_xlsx_text_cells_match_original_exporthandler() {
        use base64::Engine as _;
        use std::io::Read;
        let source: Value =
            serde_json::from_str(include_str!("../../fixtures/core-export-oracle.json")).unwrap();
        let headers: Vec<String> = serde_json::from_value(source["headers"].clone()).unwrap();
        let rows: Vec<Vec<String>> = serde_json::from_value(source["rows"].clone()).unwrap();
        let folder = tempfile::tempdir().unwrap();
        let csv = folder.path().join("output.csv");
        crate::table_export::write(&csv, "Synthetic", &headers, &rows).unwrap();
        assert_eq!(
            fs::read(csv).unwrap(),
            base64::engine::general_purpose::STANDARD
                .decode(source["csv"].as_str().unwrap())
                .unwrap()
        );
        let xlsx = folder.path().join("output.xlsx");
        crate::table_export::write(&xlsx, "Synthetic", &headers, &rows).unwrap();
        let mut zip = zip::ZipArchive::new(File::open(xlsx).unwrap()).unwrap();
        assert_eq!(zip.len(), source["xlsx"].as_object().unwrap().len());
        let mut native = String::new();
        zip.by_name("xl/worksheets/sheet1.xml")
            .unwrap()
            .read_to_string(&mut native)
            .unwrap();
        let original = source["xlsx"]["xl/worksheets/sheet1.xml"].as_str().unwrap();
        let native = roxmltree::Document::parse(&native).unwrap();
        let original = roxmltree::Document::parse(original).unwrap();
        let cells = |document: &roxmltree::Document| {
            document
                .descendants()
                .filter(|n| n.has_tag_name("c"))
                .map(|n| {
                    (
                        n.attribute("r").unwrap().to_owned(),
                        n.attribute("t").unwrap().to_owned(),
                        n.descendants()
                            .find(|c| c.has_tag_name("t"))
                            .unwrap()
                            .text()
                            .unwrap_or("")
                            .to_owned(),
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(cells(&native), cells(&original));
    }
    #[test]
    fn contexts_are_available_during_on_load_and_flags_match_pinned_framework() {
        let dir = tempfile::tempdir().unwrap();
        script(
            dir.path(),
            r#"return {name='Core API',on_load=function()
      local e=core.deserialize_event('{"event":"scan","BodyID":4,"Future":{"preserved":true}}')
      local u=core.DeserializeEvent('{"event":"UnknownFuture","Extra":9}',true)
      core.add_row{theme=core.GetCurrentThemeName(),details=core.GetCurrentThemeDetails(),status=core.GetStatus(),cargo=core.GetCargo(),state=core.CurrentLogMonitorState,batch=core.IsLogMonitorBatchReading,parsed=e,unknown=u}
    end,on_state_changed=function()core.add_row{state=core.get_log_monitor_state(),batch=core.is_log_monitor_batch_reading()}end}"#,
        );
        let (mut plugins, errors) = discover_with_context(
            dir.path(),
            &json!({}),
            dir.path(),
            &json!({"theme_name":"Survey","theme_details":{"Default.BackColor":{"R":3,"G":4,"B":5}},"status":{"Flags":42},"cargo":{"Count":9},"log_monitor_state":4}),
        );
        assert!(errors.is_empty(), "{errors:?}");
        let plugin = &mut plugins[0];
        let output = plugin.take_output();
        let row = &output.rows[0];
        assert_eq!(row["theme"], "Survey");
        assert_eq!(row["status"]["Flags"], 42);
        assert_eq!(row["cargo"]["Count"], 9);
        assert_eq!(row["state"], 4);
        assert_eq!(row["batch"], true);
        assert_eq!(row["parsed"]["journalType"], "Scan");
        assert_eq!(row["parsed"]["journalEvent"]["Future"]["preserved"], true);
        assert_eq!(row["unknown"]["journalType"], "JournalBase");
        assert_eq!(output.messages[0]["data"]["payload"]["Replay"], false);
        assert_eq!(output.messages[1]["data"]["payload"]["Replay"], true);
        for flags in 0..16 {
            plugin
                .set_context("log_monitor_state", &json!(flags))
                .unwrap();
            plugin.call("on_state_changed", &json!({})).unwrap();
            let out = plugin.take_output();
            assert_eq!(out.rows[0]["state"], flags);
            assert_eq!(out.rows[0]["batch"], flags & 6 != 0);
        }
    }
    #[test]
    fn source_storage_property_has_trailing_separator_and_exists_before_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = script(
            dir.path(),
            r#"return {name='Storage',id='source.storage',on_load=function()core.add_row{property=core.PluginStorageFolder,query=core.get_plugin_storage_folder(),joined=core.PluginStorageFolder..'cache.json'}end}"#,
        );
        let mut plugin = Extension::load(&path, None, dir.path()).unwrap();
        let out = plugin.take_output();
        let parent = dir.path().join("storage/source.storage");
        assert!(parent.is_dir());
        assert_eq!(Path::new(out.rows[0]["property"].as_str().unwrap()), parent);
        assert_eq!(out.rows[0]["property"], out.rows[0]["query"]);
        assert_eq!(
            Path::new(out.rows[0]["joined"].as_str().unwrap()),
            parent.join("cache.json")
        );
    }
    #[test]
    fn grouped_add_uses_one_hidden_group_per_call_and_set_replaces() {
        let dir = tempfile::tempdir().unwrap();
        let path = script(
            dir.path(),
            r#"return {name='Grid',on_load=function()core.AddGridItems({{a=1},{a=2}},true);core.add_grid_items({{a=3}},true)end,on_journal=function()core.SetGridItems({{b='one'},{b='two'}})end}"#,
        );
        let mut plugin = Extension::load(&path, None, dir.path()).unwrap();
        let out = plugin.take_output();
        assert_eq!(out.rows.len(), 3);
        assert_eq!(
            out.rows[0]["ObservatoryListViewGroupID"],
            out.rows[1]["ObservatoryListViewGroupID"]
        );
        assert_ne!(
            out.rows[0]["ObservatoryListViewGroupID"],
            out.rows[2]["ObservatoryListViewGroupID"]
        );
        assert_eq!(
            out.rows[0]["ObservatoryListViewGroupID"]
                .as_str()
                .unwrap()
                .len(),
            36
        );
        plugin.call("on_journal", &json!({})).unwrap();
        let out = plugin.take_output();
        assert!(out.clear_rows);
        assert_eq!(
            out.rows,
            json!([{"b":"one"},{"b":"two"}]).as_array().unwrap().clone()
        );
    }
    #[test]
    fn source_ui_and_message_overloads_keep_order_caller_and_target() {
        let dir = tempfile::tempdir().unwrap();
        let path = script(
            dir.path(),
            r#"return {name='Caller',id='caller',on_load=function()
      core.open_settings();core.OpenAbout('Evaluator');core.focus_plugin('BioInsights');core.display_message_box('Title','Body')
      local log=core.GetPluginErrorLogger();log('failure','context');core.execute_on_ui_thread(function()core.open_about()end)
      core.SendPluginMessage('Evaluator',{Command='SetWorthVisiting',Payload={ID64=42,BodyID=1,IsWorthMapping=true}})
      core.send_plugin_message('Hecate',{'legacy','values'})
    end}"#,
        );
        let mut plugin = Extension::load(&path, None, dir.path()).unwrap();
        let out = plugin.take_output();
        let commands: Vec<_> = out
            .messages
            .iter()
            .map(|m| {
                m["data"]["command"]
                    .as_str()
                    .or_else(|| m["data"]["Command"].as_str())
                    .unwrap()
            })
            .collect();
        assert_eq!(
            commands,
            [
                "OpenSettings",
                "OpenAbout",
                "FocusPlugin",
                "DisplayMessageBox",
                "PluginError",
                "OpenAbout",
                "SetWorthVisiting",
                "LegacyPluginMessage"
            ]
        );
        assert_eq!(out.messages[0]["data"]["payload"]["Plugin"], "Caller");
        assert_eq!(out.messages[6]["data"]["target"], "Evaluator");
        assert_eq!(out.messages[7]["data"]["target"], "Hecate");
        assert!(out.messages.iter().all(|m| m["source"]["id"] == "caller"));
    }
    #[test]
    fn menu_items_are_themed_before_strip_properties_and_child_controls() {
        let dir = tempfile::tempdir().unwrap();
        script(
            dir.path(),
            r#"local trace={};local root={type='MenuStrip',Name='strip',properties={BackColor='old'},items={{type='ToolStripMenuItem',Name='ColourButton',properties={BackColor='old'}}},controls={{type='Button',Name='child',properties={BackColor='old'}}}}
return {name='Menus',on_load=function()core.RegisterControl(root,function(control)table.insert(trace,{name=control.Name,strip=root.properties.BackColor});return true end);core.add_row{trace=trace,strip=root.properties,menu=root.items[1].properties,child=root.controls[1].properties}end}"#,
        );
        let (mut plugins, errors) = discover_with_context(
            dir.path(),
            &json!({}),
            dir.path(),
            &json!({"theme_details":{"Default.BackColor":{"R":7}}}),
        );
        assert!(errors.is_empty(), "{errors:?}");
        let out = plugins[0].take_output();
        let row = &out.rows[0];
        let trace = row["trace"].as_array().unwrap();
        assert_eq!(
            trace
                .iter()
                .map(|v| v["name"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["strip", "strip", "ColourButton", "child"]
        );
        assert_eq!(trace[2]["strip"], "old");
        assert_eq!(trace[3]["strip"]["R"], 7);
        assert_eq!(row["menu"]["BackColor"]["R"], 7);
    }
    #[test]
    fn theme_callbacks_gate_registration_recurse_skip_colour_button_and_unregister() {
        let dir = tempfile::tempdir().unwrap();
        script(
            dir.path(),
            r#"local count=0;local root={type='Button',Name='root',properties={BackColor=core.null,ForeColor=core.null},controls={{type='Label',Name='child',properties={ForeColor=core.null}},{type='Button',Name='ColourButton',properties={BackColor='keep'}}}}
    return {name='Theme',on_load=function()core.register_control(root,function(control)count=count+1;return true end);core.register_control({Name='reject'},function()count=count+1;return false end)end,
    on_journal=function(e)core.add_row{count=count,root=root.properties,child=root.controls[1].properties,excluded=root.controls[2].properties};if e.unregister then core.unregister_control(root)end end}"#,
        );
        let (mut plugins, errors) = discover_with_context(
            dir.path(),
            &json!({}),
            dir.path(),
            &json!({"theme_name":"Dark","theme_details":{"Default.ForeColor":{"R":1},"Default.BackColor":{"R":2},"Button.BackColor":{"R":3}}}),
        );
        assert!(errors.is_empty(), "{errors:?}");
        let p = &mut plugins[0];
        p.call("on_journal", &json!({})).unwrap();
        let out = p.take_output();
        assert_eq!(out.rows[0]["count"], 4);
        assert_eq!(out.rows[0]["root"]["BackColor"]["R"], 3);
        assert_eq!(out.rows[0]["child"]["ForeColor"]["R"], 1);
        assert_eq!(out.rows[0]["excluded"]["BackColor"], "keep");
        p.theme_changed(
            "Light",
            &json!({"Default.ForeColor":{"R":9},"Default.BackColor":{"R":8}}),
        )
        .unwrap();
        p.call("on_journal", &json!({"unregister":true})).unwrap();
        let out = p.take_output();
        assert_eq!(out.rows[0]["count"], 6);
        assert_eq!(out.rows[0]["root"]["BackColor"]["R"], 8);
        assert_eq!(out.rows[0]["child"]["ForeColor"]["R"], 9);
        p.theme_changed("Other", &json!({"Default.BackColor":{"R":99}}))
            .unwrap();
        p.call("on_journal", &json!({})).unwrap();
        let out = p.take_output();
        assert_eq!(out.rows[0]["count"], 6);
        assert_eq!(out.rows[0]["root"]["BackColor"]["R"], 8);
    }
}
