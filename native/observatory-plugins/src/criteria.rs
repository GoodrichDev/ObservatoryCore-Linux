//! Native Lua implementation of Observatory Explorer's annotated criteria files.
//! Arrays expose the original .NET zero-based indices and `Count`; the public
//! iterator helpers keep existing criteria independent of that representation.
use crate::explorer::{array, id, signal_count, string, Bodies};
use mlua::{Function, HookTriggers, Lua, MultiValue, Table, Value as LuaValue, VmState};
use observatory_domain::{Finding, JournalEvent};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::SystemTime;

#[path = "criteria_clr.rs"]
mod managed;

pub(crate) fn parent_projection(value: &Value) -> Value {
    managed::project_parent(value)
}

const INSTRUCTION_BUDGET: u64 = 2_000_000;
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug)]
struct Handler {
    kind: String,
    function: String,
    label: String,
}
#[derive(Default)]
struct Notice {
    body: String,
    title: String,
    detail: String,
    body_id: Option<i64>,
}

pub struct Criteria {
    path: PathBuf,
    modified: Option<SystemTime>,
    lua: Lua,
    handlers: Vec<Handler>,
    disabled: HashSet<String>,
    notices: Arc<Mutex<Vec<Notice>>>,
    instructions: Arc<AtomicU64>,
    read_all: bool,
    notification_payloads: Vec<(Finding, Value)>,
    diagnostics: Vec<Value>,
    event_time: String,
    startup_findings: Vec<Finding>,
    raw_scans: HashMap<(u64, u64), Arc<str>>,
    managed: Option<managed::Client>,
    managed_failed: bool,
    managed_history_updates: Vec<Value>,
    managed_event_snapshot: Option<Value>,
}

fn lua_error(error: mlua::Error) -> anyhow::Error {
    anyhow::anyhow!(error.to_string())
}

impl Criteria {
    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        if path.as_os_str().is_empty() {
            return Self::from_source(path, None, "");
        }
        match Self::from_path(path.clone()) {
            Ok(value) => Ok(value),
            Err(error) => {
                let modified = std::fs::metadata(&path)
                    .and_then(|metadata| metadata.modified())
                    .ok();
                let managed_failed = managed::executable().ok().flatten().is_some();
                let mut value = Self::from_source(path, modified, "")?;
                value.managed_failed = managed_failed;
                value.diagnostics.push(json!({"Level":"Error","Message":format!("Failed to load custom criteria: {error}"),"Path":value.path.to_string_lossy()}));
                Ok(value)
            }
        }
    }
    pub fn from_path(path: PathBuf) -> anyhow::Result<Self> {
        if path.is_file() {
            if let Some(executable) = managed::executable()? {
                return Self::with_managed_runtime(path, &executable);
            }
        }
        let source = if path.exists() {
            read_source(&path)?
        } else {
            String::new()
        };
        let modified = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok());
        Self::from_source(path, modified, &source)
    }

    fn from_source(
        path: PathBuf,
        modified: Option<SystemTime>,
        source: &str,
    ) -> anyhow::Result<Self> {
        let lua = Lua::new();
        lua.set_memory_limit(32 * 1024 * 1024).map_err(lua_error)?;
        let instructions = Arc::new(AtomicU64::new(0));
        let counter = instructions.clone();
        lua.set_global_hook(
            HookTriggers::new().every_nth_instruction(1000),
            move |_, _| {
                if counter.fetch_add(1000, Ordering::Relaxed) >= INSTRUCTION_BUDGET {
                    return Err(mlua::Error::RuntimeError(
                        "Criterion exceeded its instruction budget".into(),
                    ));
                }
                Ok(VmState::Continue)
            },
        )
        .map_err(lua_error)?;
        let notices = Arc::new(Mutex::new(Vec::new()));
        let sink = notices.clone();
        lua.globals()
            .set(
                "notify",
                lua.create_function(
                    move |_, (body, title, detail): (String, Option<String>, Option<String>)| {
                        let mut notices = sink.lock().map_err(|_| {
                            mlua::Error::RuntimeError("Notification queue poisoned".into())
                        })?;
                        if notices.len() >= 1024 {
                            return Err(mlua::Error::RuntimeError(
                                "Too many notifications from one criterion".into(),
                            ));
                        }
                        notices.push(Notice {
                            body,
                            title: title.unwrap_or_default(),
                            detail: detail.unwrap_or_default(),
                            body_id: None,
                        });
                        Ok(())
                    },
                )
                .map_err(lua_error)?,
            )
            .map_err(lua_error)?;
        let sink = notices.clone();
        lua.globals()
            .set(
                "notifyForBody",
                lua.create_function(
                    move |_,
                          (body, title, detail, body_id): (
                        String,
                        Option<String>,
                        Option<String>,
                        Option<i64>,
                    )| {
                        let mut notices = sink.lock().map_err(|_| {
                            mlua::Error::RuntimeError("Notification queue poisoned".into())
                        })?;
                        if notices.len() >= 1024 {
                            return Err(mlua::Error::RuntimeError(
                                "Too many notifications from one criterion".into(),
                            ));
                        }
                        notices.push(Notice {
                            body,
                            title: title.unwrap_or_default(),
                            detail: detail.unwrap_or_default(),
                            body_id,
                        });
                        Ok(())
                    },
                )
                .map_err(lua_error)?,
            )
            .map_err(lua_error)?;
        lua.load(HELPERS)
            .set_name("Observatory criteria helpers")
            .exec()
            .map_err(lua_error)?;
        let (handlers, script) = compile_criteria(source)?;
        instructions.store(0, Ordering::Relaxed);
        lua.load(&script)
            .set_name(path.to_string_lossy().as_ref())
            .exec()
            .map_err(lua_error)?;
        let mut startup_findings = Vec::new();
        let mut notification_payloads = Vec::new();
        for notice in notices
            .lock()
            .map_err(|_| anyhow::anyhow!("Notification queue poisoned"))?
            .drain(..)
        {
            let finding = Finding {
                plugin: "Explorer".into(),
                timestamp: String::new(),
                system: String::new(),
                body: notice.body.clone(),
                title: notice.title.clone(),
                detail: notice.detail.clone(),
            };
            notification_payloads.push((finding.clone(),json!({"title":notice.body,"detail":notice.title,"extended_details":notice.detail,"coalescing_id":notice.body_id.unwrap_or(-1),"sender":"Explorer","grid_time":""})));
            startup_findings.push(finding);
        }
        Ok(Self {
            path,
            modified,
            lua,
            handlers,
            disabled: HashSet::new(),
            notices,
            instructions,
            read_all: false,
            notification_payloads,
            diagnostics: Vec::new(),
            event_time: String::new(),
            startup_findings,
            raw_scans: HashMap::new(),
            managed: None,
            managed_failed: false,
            managed_history_updates: Vec::new(),
            managed_event_snapshot: None,
        })
    }
    pub fn set_read_all(&mut self, read_all: bool) {
        if self.read_all != read_all {
            if self.managed.is_some() {
                self.managed_sync_path();
                let rows =
                    self.managed_request(json!({"op":"set_read_all","value":read_all}), None, "");
                self.startup_findings.extend(rows);
                self.read_all = read_all;
                return;
            }
            self.reset();
            self.read_all = read_all;
        }
    }
    pub fn set_path(&mut self, path: PathBuf) {
        if self.path != path {
            self.path = path;
            self.modified = None;
            if self.managed.is_none() {
                self.managed_failed = false;
            }
            if self.managed.is_some() {
                self.managed_sync_path();
            }
        }
    }
    pub fn take_startup_findings(&mut self) -> Vec<Finding> {
        std::mem::take(&mut self.startup_findings)
    }
    pub fn remember_raw(&mut self, event: &JournalEvent) {
        if let Some(raw) = &event.raw_json {
            self.raw_scans.insert(
                (
                    id(&event.data, "SystemAddress").unwrap_or(0),
                    id(&event.data, "BodyID").unwrap_or(0),
                ),
                raw.clone(),
            );
        }
    }
    pub fn raw_for(&self, value: &Value) -> Option<Arc<str>> {
        self.raw_scans
            .get(&(
                id(value, "SystemAddress").unwrap_or(0),
                id(value, "BodyID").unwrap_or(0),
            ))
            .cloned()
    }
    pub fn clear_raw_history(&mut self) {
        self.raw_scans.clear();
        if self.managed.is_some() {
            let rows = self.managed_request(json!({"op":"clear_history"}), None, "");
            self.startup_findings.extend(rows);
        }
    }
    pub fn retain_raw_history(&mut self, keys: &HashSet<(u64, u64)>) {
        self.raw_scans.retain(|key, _| keys.contains(key));
        if self.managed.is_some() {
            let keys = keys
                .iter()
                .map(|(system, body)| json!({"kind":"scan","system_id":system,"body_id":body}))
                .collect::<Vec<_>>();
            let rows = self.managed_request(
                json!({"op":"retain_history","keys":keys,"kinds":["scan"]}),
                None,
                "",
            );
            self.startup_findings.extend(rows);
        }
    }
    pub fn take_notification_payload(&mut self, finding: &Finding) -> Option<Value> {
        self.notification_payloads
            .iter()
            .position(|(row, _)| row == finding)
            .map(|index| self.notification_payloads.remove(index).1)
    }
    pub fn take_diagnostics(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.diagnostics)
    }

    pub fn reset(&mut self) {
        // An init failure may already have run arbitrary global side effects.
        // File edits/new paths or an explicit app restart can retry it; normal
        // journal and read-all transitions must not replay a crashed initializer.
        if self.managed_failed && self.managed.is_none() {
            return;
        }
        if self.managed.is_some() {
            self.managed_sync_path();
            let rows = self.managed_request(json!({"op":"reset"}), None, "");
            self.startup_findings.extend(rows);
            return;
        }
        match Self::from_path(self.path.clone()) {
            Ok(new) => self.replace_program(new),
            Err(error) => self.failed_reload(error),
        }
    }

    fn failed_reload(&mut self, error: anyhow::Error) {
        self.modified = std::fs::metadata(&self.path)
            .and_then(|metadata| metadata.modified())
            .ok();
        self.managed_failed = managed::executable().ok().flatten().is_some();
        self.handlers.clear();
        self.disabled.clear();
        self.diagnostics.push(json!({"Level":"Error","Message":format!("Error Reading Custom Criteria File: {error}"),"Path":self.path.to_string_lossy()}));
    }
    fn replace_program(&mut self, mut new: Self) {
        new.event_time = self.event_time.clone();
        new.read_all = self.read_all;
        new.raw_scans = std::mem::take(&mut self.raw_scans);
        for (_, payload) in &mut new.notification_payloads {
            payload["grid_time"] = json!(self.event_time);
        }
        new.diagnostics.splice(0..0, self.diagnostics.drain(..));
        *self = new;
    }

    pub fn process(
        &mut self,
        event: &JournalEvent,
        system: &Bodies,
        signals: &Bodies,
        current_system: &str,
    ) -> Vec<Finding> {
        let mut out = Vec::new();
        self.notification_payloads
            .retain(|(finding, _)| self.startup_findings.contains(finding));
        let kind = match event.event.as_str() {
            "Scan" => "scan",
            "FSSDiscoveryScan" => "discovery",
            "FSSBodySignals" | "SAASignalsFound" => "bodysignals",
            "FSSAllBodiesFound" => "allbodies",
            "FSDJump" | "CarrierJump" => "jump",
            _ => return out,
        };
        if kind != "scan" {
            self.event_time = crate::communicator::display_timestamp(&event.timestamp);
        }
        if kind == "scan" {
            self.remember_raw(event);
        }
        if self.managed.is_some() {
            self.managed_sync_path();
            out.extend(self.take_startup_findings());
            let address = id(&event.data, "SystemAddress").unwrap_or(0);
            let system = system
                .iter()
                .map(|(body, scan)| {
                    managed::history_entry(
                        scan,
                        address,
                        *body,
                        self.raw_scans.get(&(address, *body)).cloned(),
                    )
                })
                .collect::<Vec<_>>();
            let signals = signals
                .iter()
                .map(|(body, signal)| managed::history_entry(signal, address, *body, None))
                .collect::<Vec<_>>();
            let mut data = event.data.clone();
            if let Some(object) = data.as_object_mut() {
                object.remove("__clr_parent");
                object.remove("__clr_system_key");
            }
            out.extend(self.managed_request(
                json!({
                    "op":"process", "event":data, "event_type":event.event,
                    "raw_json":event.json(), "system":system,
                    "signals":signals,
                }),
                Some(event),
                current_system,
            ));
            return out;
        }
        let changed = std::fs::metadata(&self.path)
            .and_then(|m| m.modified())
            .ok();
        if !self.read_all && changed.is_some() && changed != self.modified {
            match Self::from_path(self.path.clone()) {
                Ok(new) => {
                    self.replace_program(new);
                    if self.managed.is_some() {
                        return self.process(event, system, signals, current_system);
                    }
                }
                Err(error) => {
                    self.failed_reload(error);
                }
            }
        }
        out.extend(self.take_startup_findings());
        for handler in self.handlers.iter().filter(|h| h.kind == kind) {
            if self.disabled.contains(&handler.function) {
                continue;
            }
            self.instructions.store(0, Ordering::Relaxed);
            let result = (|| -> mlua::Result<MultiValue> {
                let function: Function = self.lua.globals().get(handler.function.as_str())?;
                let event_table = journal_table(&self.lua, &event.data)?;
                event_table.set("Json", event.json().as_ref())?;
                if kind == "scan" {
                    let parents = parents_table(&self.lua, &event.data, system, &self.raw_scans)?;
                    let bodies = system_table(&self.lua, system, &self.raw_scans)?;
                    let signal = signals.get(&id(&event.data, "BodyID").unwrap_or(0));
                    let bio = signal
                        .map(|s| signal_count(s, "$SAA_SignalType_Biological;"))
                        .unwrap_or(0);
                    let geo = signal
                        .map(|s| signal_count(s, "$SAA_SignalType_Geological;"))
                        .unwrap_or(0);
                    function.call((event_table, parents, bodies, bio, geo))
                } else if kind == "allbodies" {
                    let parents = self.lua.create_table()?;
                    for (body, scan) in system {
                        parents.set(
                            *body,
                            parents_table(&self.lua, scan, system, &self.raw_scans)?,
                        )?;
                    }
                    function.call((
                        event_table,
                        system_table(&self.lua, system, &self.raw_scans)?,
                        parents,
                    ))
                } else {
                    function.call(event_table)
                }
            })();
            match result {
                Ok(values) if kind == "scan" => {
                    let values: Vec<_> = values.into_iter().collect();
                    let detail =
                        if values.len() == 3 && matches!(values[0], LuaValue::Boolean(true)) {
                            Some((lua_string(&values[1]), lua_string(&values[2])))
                        } else if values.len() == 2 {
                            Some((lua_string(&values[0]), lua_string(&values[1])))
                        } else {
                            None
                        };
                    if let Some((title, detail)) = detail {
                        out.push(Finding {
                            plugin: "Explorer".into(),
                            timestamp: event.timestamp.clone(),
                            system: system_name(event, current_system),
                            body: string(&event.data, "BodyName").into(),
                            title,
                            detail,
                        });
                    }
                }
                Ok(_) => {}
                Err(error) => {
                    self.disabled.insert(handler.function.clone());
                    self.diagnostics.push(json!({"Level":"Error","Message":format!("while processing custom criteria '{}': {error}",handler.label),"Journal":event.data}));
                    if kind == "scan" {
                        out.push(error_finding(
                            event,
                            current_system,
                            &error.to_string(),
                            &event.data.to_string(),
                        ));
                    }
                }
            }
            if let Ok(mut notices) = self.notices.lock() {
                for notice in notices.drain(..) {
                    let finding = Finding {
                        plugin: "Explorer".into(),
                        timestamp: event.timestamp.clone(),
                        system: system_name(event, current_system),
                        body: notice.body.clone(),
                        title: notice.title,
                        detail: notice.detail,
                    };
                    self.notification_payloads.push((finding.clone(),json!({"title":notice.body,"detail":finding.title,"extended_details":finding.detail,"coalescing_id":notice.body_id,"sender":"Explorer","grid_time":self.event_time})));
                    out.push(finding);
                }
            }
        }
        let (mut direct, normal): (Vec<_>, Vec<_>) = out.into_iter().partition(|finding| {
            self.notification_payloads
                .iter()
                .any(|(row, _)| row == finding)
        });
        direct.extend(normal);
        direct
    }
}

fn system_name(event: &JournalEvent, current: &str) -> String {
    let name = string(&event.data, "StarSystem");
    if name.is_empty() {
        current.into()
    } else {
        name.into()
    }
}
fn error_finding(event: &JournalEvent, current: &str, title: &str, detail: &str) -> Finding {
    Finding {
        plugin: "Explorer".into(),
        timestamp: event.timestamp.clone(),
        system: system_name(event, current),
        body: string(&event.data, "BodyName").into(),
        title: title.into(),
        detail: detail.into(),
    }
}
fn read_source(path: &std::path::Path) -> anyhow::Result<String> {
    let metadata = std::fs::metadata(path)?;
    anyhow::ensure!(
        metadata.len() <= MAX_FILE_BYTES,
        "Criteria file exceeds 4 MiB"
    );
    Ok(std::fs::read_to_string(path)?
        .trim_start_matches('\u{feff}')
        .to_owned())
}
fn lua_string(value: &LuaValue) -> String {
    match value {
        LuaValue::String(s) => s.to_string_lossy(),
        LuaValue::Integer(n) => n.to_string(),
        LuaValue::Number(n) => n.to_string(),
        LuaValue::Boolean(b) => if *b { "True" } else { "False" }.into(),
        LuaValue::Nil => String::new(),
        _ => format!("{value:?}"),
    }
}

fn annotation(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    let raw = line
        .strip_prefix("::")
        .or_else(|| line.strip_prefix("---@"))?
        .replace("::", "")
        .replace("---@", "");
    let boundary = raw.find([' ', '=']).unwrap_or(raw.len());
    let kind = raw[..boundary].to_ascii_lowercase();
    let label = raw.get(boundary + 1..).unwrap_or("").to_owned();
    Some((kind, label))
}

fn compile_criteria(source: &str) -> anyhow::Result<(Vec<Handler>, String)> {
    let lines: Vec<_> = source.lines().collect();
    let mut handlers = Vec::new();
    let mut script = String::new();
    let mut globals = String::new();
    let mut i = 0;
    while i < lines.len() {
        if let Some((kind, label)) = annotation(lines[i]) {
            let block = matches!(
                kind.as_str(),
                "global"
                    | "complex"
                    | "criteria"
                    | "allbodies"
                    | "jump"
                    | "bodysignals"
                    | "discovery"
            );
            if matches!(kind.as_str(), "end" | "detail") {
                anyhow::bail!("Unexpected {kind} directive on line {}", i + 1);
            }
            let function = format!("ObservatoryHandler{}", i + 1);
            if block {
                let start = i + 1;
                i += 1;
                while i < lines.len() && !annotation(lines[i]).is_some_and(|a| a.0 == "end") {
                    i += 1;
                }
                anyhow::ensure!(
                    i < lines.len(),
                    "Unterminated {kind} handler beginning on line {start}"
                );
                let body = lines[start..i].join("\n");
                if kind == "global" {
                    globals.push_str(&body);
                    globals.push('\n');
                } else {
                    let (event_kind, args) = match kind.as_str() {
                        "complex" | "criteria" => {
                            ("scan", "scan, parents, system, biosignals, geosignals")
                        }
                        "allbodies" => ("allbodies", "allBodies, system, parentsTable"),
                        "jump" => ("jump", "jump"),
                        "bodysignals" => ("bodysignals", "bodySignals"),
                        "discovery" => ("discovery", "discovery"),
                        _ => unreachable!(),
                    };
                    script.push_str(&format!("function {function}({args})\n{body}\nend\n"));
                    handlers.push(Handler {
                        kind: event_kind.into(),
                        function,
                        label,
                    });
                }
            } else {
                i += 1;
                anyhow::ensure!(
                    i < lines.len(),
                    "Missing criterion expression on line {}",
                    i + 1
                );
                let expression = lines[i];
                let mut detail = "''";
                if i + 1 < lines.len() && annotation(lines[i + 1]).is_some_and(|a| a.0 == "detail")
                {
                    i += 2;
                    anyhow::ensure!(
                        i < lines.len(),
                        "Missing detail expression on line {}",
                        i + 1
                    );
                    detail = lines[i];
                }
                let escaped = lua_quote(&label);
                script.push_str(&format!("function {function}(scan, parents, system, biosignals, geosignals)\nlocal result = {expression}\nlocal detail = ''\nif result then detail = {detail} end\nreturn result, {escaped}, detail\nend\n"));
                handlers.push(Handler {
                    kind: "scan".into(),
                    function,
                    label,
                });
            }
        }
        i += 1;
    }
    script.push_str(&globals);
    Ok((handlers, script))
}
fn lua_quote(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    )
}

fn json_to_lua(lua: &Lua, value: &Value) -> mlua::Result<LuaValue> {
    Ok(match value {
        Value::Null => LuaValue::Nil,
        Value::Bool(v) => LuaValue::Boolean(*v),
        Value::Number(v) => {
            if let Some(i) = v.as_i64() {
                LuaValue::Integer(i)
            } else {
                LuaValue::Number(v.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(v) => LuaValue::String(lua.create_string(v)?),
        Value::Array(values) => {
            let table = lua.create_table()?;
            table.set("Count", values.len())?;
            for (i, value) in values.iter().enumerate() {
                table.set(i, json_to_lua(lua, value)?)?;
            }
            LuaValue::Table(table)
        }
        Value::Object(values) => {
            let table = lua.create_table()?;
            for (key, value) in values {
                table.set(key.as_str(), json_to_lua(lua, value)?)?;
            }
            LuaValue::Table(table)
        }
    })
}
fn dto_schema() -> &'static Value {
    static SCHEMA: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    SCHEMA.get_or_init(|| {
        serde_json::from_str(include_str!("../../fixtures/explorer-dto-schema.json"))
            .expect("installed Framework DTO schema")
    })
}
fn dto_properties(kind: &str) -> std::collections::BTreeMap<String, String> {
    let schema = &dto_schema()["Types"][kind];
    let mut fields = schema["Base"]
        .as_str()
        .map(dto_properties)
        .unwrap_or_default();
    if let Some(properties) = schema["Properties"].as_object() {
        for (name, kind) in properties {
            fields.insert(name.clone(), kind.as_str().unwrap_or("").into());
        }
    }
    fields
}
fn typed_value(lua: &Lua, value: &Value, kind: &str) -> mlua::Result<LuaValue> {
    if let Some(base) = kind.strip_suffix('?') {
        if value.is_null() {
            return Ok(LuaValue::Nil);
        }
        return typed_value(lua, value, base);
    }
    match kind {
        "float" => {
            return Ok(LuaValue::Number(f64::from(
                value.as_f64().unwrap_or(0.0) as f32
            )));
        }
        "double" | "decimal" => return Ok(LuaValue::Number(value.as_f64().unwrap_or(0.0))),
        "int" | "uint" | "long" | "ulong" | "short" | "ushort" | "byte" | "sbyte" => {
            return if value.is_null() {
                Ok(LuaValue::Integer(0))
            } else {
                json_to_lua(lua, value)
            };
        }
        "bool" => return Ok(LuaValue::Boolean(value.as_bool().unwrap_or(false))),
        _ => {}
    }
    if value.is_null() {
        return Ok(LuaValue::Nil);
    }
    if let Some(element) = kind
        .strip_prefix("ImmutableList<")
        .and_then(|v| v.strip_suffix('>'))
    {
        let list = lua.create_table()?;
        if element == "MaterialComposition" && value.is_object() {
            let properties = value.as_object().unwrap();
            list.set("Count", properties.len())?;
            for (index, (name, percent)) in properties.iter().enumerate() {
                list.set(
                    index,
                    typed_value(lua, &json!({"Name":name,"Percent":percent}), element)?,
                )?;
            }
        } else {
            let values = value.as_array().map(Vec::as_slice).unwrap_or(&[]);
            list.set("Count", values.len())?;
            for (index, value) in values.iter().enumerate() {
                list.set(index, typed_value(lua, value, element)?)?;
            }
        }
        return Ok(LuaValue::Table(list));
    }
    if dto_schema()["Types"].get(kind).is_some() {
        let result = lua.create_table()?;
        for (name, field_type) in dto_properties(kind) {
            if matches!(
                name.as_str(),
                "AdditionalProperties" | "TimestampDateTime" | "Json" | "Parent"
            ) {
                continue;
            }
            let key = match name.as_str() {
                "Timestamp" => "timestamp",
                "Event" => "event",
                other => other,
            };
            let value = if kind == "StarPosition" && value.is_array() {
                let index = match key {
                    "x" => 0,
                    "y" => 1,
                    _ => 2,
                };
                &value[index]
            } else {
                &value[key]
            };
            result.set(name, typed_value(lua, value, &field_type)?)?;
        }
        return Ok(LuaValue::Table(result));
    }
    json_to_lua(lua, value)
}
fn journal_table(lua: &Lua, value: &Value) -> mlua::Result<Table> {
    let kind = if value["event"].is_null() && value["PlanetClass"] == "Barycentre" {
        "Scan"
    } else {
        string(value, "event")
    };
    let LuaValue::Table(table) = typed_value(lua, value, kind)? else {
        return lua.create_table();
    };
    table.set("Timestamp", string(value, "timestamp"))?;
    table.set("Event", json_to_lua(lua, &value["event"])?)?;
    table.set("Json", value.to_string())?;
    let properties = dto_properties(kind);
    let additional: serde_json::Map<String, Value> = value
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(key, _)| {
            !matches!(key.as_str(), "timestamp" | "event") && !properties.contains_key(key.as_str())
        })
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    table.set(
        "AdditionalProperties",
        json_to_lua(lua, &json!(additional))?,
    )?;
    if kind == "Scan" && value.get("Parents").is_some() {
        let parents = lua.create_table()?;
        let items = array(value, "Parents");
        parents.set("Count", items.len())?;
        for (index, item) in items.iter().enumerate() {
            if let Some((kind, body)) = item.as_object().and_then(|v| {
                ["Null", "Planet", "Star"].iter().find_map(|kind| {
                    v.get(*kind)
                        .filter(|v| !v.is_null())
                        .map(|value| (*kind, value))
                })
            }) {
                let parent = lua.create_table()?;
                parent.set("Item1", kind)?;
                parent.set("Item2", json_to_lua(lua, body)?)?;
                parents.set(index, parent)?;
            }
        }
        table.set("Parent", parents)?;
    }
    Ok(table)
}
fn raw_table(lua: &Lua, value: &Value, raw: &HashMap<(u64, u64), Arc<str>>) -> mlua::Result<Table> {
    let table = journal_table(lua, value)?;
    if let Some(text) = raw.get(&(
        id(value, "SystemAddress").unwrap_or(0),
        id(value, "BodyID").unwrap_or(0),
    )) {
        table.set("Json", text.as_ref())?;
    }
    Ok(table)
}
fn parents_table(
    lua: &Lua,
    scan: &Value,
    system: &Bodies,
    raw: &HashMap<(u64, u64), Arc<str>>,
) -> mlua::Result<LuaValue> {
    if scan.get("Parents").is_none() {
        return Ok(LuaValue::Nil);
    }
    let parents = lua.create_table()?;
    let list = array(scan, "Parents");
    parents.set("Count", list.len())?;
    for (i, p) in list.iter().enumerate() {
        if let Some((kind, body)) = p.as_object().and_then(|o| o.iter().next()) {
            let parent = lua.create_table()?;
            parent.set("ParentType", kind.as_str())?;
            parent.set("Body", json_to_lua(lua, body)?)?;
            if let Some(scan) = body.as_u64().and_then(|b| system.get(&b)) {
                parent.set("Scan", raw_table(lua, scan, raw)?)?;
            }
            parents.set(i, parent)?;
        }
    }
    Ok(LuaValue::Table(parents))
}
fn system_table(
    lua: &Lua,
    system: &Bodies,
    raw: &HashMap<(u64, u64), Arc<str>>,
) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("Count", system.len())?;
    for (i, scan) in system.values().enumerate() {
        table.set(i, raw_table(lua, scan, raw)?)?;
    }
    Ok(table)
}

const HELPERS: &str = r#"
function nil_iterator() end
local function iterate(list, transform, filter)
    if not list then return nil_iterator end
    local i = -1
    return function()
        while true do
            i = i + 1
            if i >= list.Count then return nil end
            local item = list[i]
            if not filter or filter(item) then return transform and transform(item) or item end
        end
    end
end
function materials(list) return iterate(list,function(m) return {name=m.Name,percent=m.Percent} end) end
function _ringsFiltered(list,filter)
    return iterate(list,function(r) return {name=r.Name,ringclass=r.RingClass,massmt=r.MassMT,innerrad=r.InnerRad,outerrad=r.OuterRad} end,
        function(r) return filter == nil or string.find(r.Name,'^.*'..filter..'$') end)
end
function _hasRingsFiltered(list,filter)
    if list then for i=0,list.Count-1 do if string.find(list[i].Name,'^.*'..filter..'$') then return true end end end
    return false
end
function rings(list) return _ringsFiltered(list,nil) end
function ringsOnly(list) return _ringsFiltered(list,'Ring') end
function beltsOnly(list) return _ringsFiltered(list,'Belt') end
function bodies(list) return iterate(list) end
function allparents(list) return iterate(list,function(p) return {parenttype=p.ParentType,body=p.Body,scan=p.Scan} end) end
function hasBelts(list) return _hasRingsFiltered(list,'Belt') end
function hasRings(list) return _hasRingsFiltered(list,'Ring') end
function isBelt(name) return name ~= nil and string.find(name,'^.*Belt$') end
function isRing(name) return name ~= nil and string.find(name,'^.*Ring$') end
function isStar(scan) return scan.StarType and scan.StarType ~= '' end
function isPlanet(scan) return scan.PlanetClass ~= nil and scan.PlanetClass ~= 'Barycentre' end
function isBarycentre(scan) return scan.PlanetClass == 'Barycentre' end
function hasAtmosphere(scan) return scan.AtmosphereComposition ~= nil end
function hasLandableAtmosphere(scan) return scan.Landable and hasAtmosphere(scan) end
function protect(tbl) return setmetatable({},{__index=tbl,__newindex=function(t,k,v) error('attempting to change constant '..tostring(k)..' to '..tostring(v),2) end}) end
const=protect({SPEED_OF_LIGHT_mps=299792458,GRAVITY_mps2=9.81,ATM_PRESSURE_Pa=101325,DAY_s=86400,HOUR_s=3600})
function distanceAsLs(v) return v/const.SPEED_OF_LIGHT_mps end
function distanceAsKm(v) return v/1000 end
function gravityAsG(v) return v/const.GRAVITY_mps2 end
function pressureAsAtm(v) return v/const.ATM_PRESSURE_Pa end
function periodAsDay(v) return v/const.DAY_s end
function periodAsHour(v) return v/const.HOUR_s end
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installed_dto_schema_preserves_missing_fields_tuple_members_and_typed_objects() {
        let lua = Lua::new();
        let scan=journal_table(&lua,&json!({"event":"Scan","BodyID":7,"Parents":[{"Star":0}],"Radius":18000000.1,"CustomData":"custom"})).unwrap();
        assert!(matches!(
            scan.get::<LuaValue>("Atmosphere").unwrap(),
            LuaValue::Nil
        ));
        assert_eq!(scan.get::<f64>("Radius").unwrap(), 18000000.0);
        let parents: Table = scan.get("Parent").unwrap();
        let first: Table = parents.get(0).unwrap();
        assert_eq!(first.get::<String>("Item1").unwrap(), "Star");
        assert_eq!(first.get::<u64>("Item2").unwrap(), 0);
        let extra: Table = scan.get("AdditionalProperties").unwrap();
        assert_eq!(extra.get::<String>("CustomData").unwrap(), "custom");
        assert!(matches!(
            extra.get::<LuaValue>("BodyID").unwrap(),
            LuaValue::Nil
        ));
        let jump =
            journal_table(&lua, &json!({"event":"FSDJump","StarPos":[1.2,2.3,3.4]})).unwrap();
        assert_eq!(jump.get::<u64>("BoostUsed").unwrap(), 0);
        let pos: Table = jump.get("StarPos").unwrap();
        assert_eq!(pos.get::<f64>("x").unwrap(), 1.2);
        let discovery = journal_table(&lua, &json!({"event":"FSSDiscoveryScan"})).unwrap();
        assert_eq!(discovery.get::<u64>("BodyCount").unwrap(), 0);
        assert_eq!(discovery.get::<f64>("Progress").unwrap(), 0.0);
    }
    #[test]
    fn original_csharp_compiler_and_lua_helpers_differential() {
        let cases: Vec<Value> =
            serde_json::from_str(include_str!("../../fixtures/explorer-lua-oracle.json")).unwrap();
        for case in cases {
            let mut native = criteria(case["Source"].as_str().unwrap());
            let mut original = criteria("");
            for chunk in case["Expected"]["Chunks"].as_array().unwrap() {
                let text = chunk.as_str().unwrap();
                original.lua.load(text).exec().unwrap();
                let first = text.lines().next().unwrap_or("");
                if let Some(rest) = first.strip_prefix("function ") {
                    let name = rest.split_whitespace().next().unwrap();
                    if name.starts_with("Criteria") || name.starts_with("Observatory") {
                        let kind = if first.contains("(scan,") {
                            "scan"
                        } else if first.contains("(allBodies,") {
                            "allbodies"
                        } else if first.contains("(jump)") {
                            "jump"
                        } else if first.contains("(discovery)") {
                            "discovery"
                        } else {
                            "bodysignals"
                        };
                        original.handlers.push(Handler {
                            kind: kind.into(),
                            function: name.into(),
                            label: String::new(),
                        });
                    }
                }
            }
            let mut input = event();
            input.data["Rings"] = json!([{"Name":"A Ring","RingClass":"Rocky","MassMT":2,"InnerRad":3,"OuterRad":4},{"Name":"B Belt","RingClass":"Icy","MassMT":5,"InnerRad":6,"OuterRad":7}]);
            let mut system = Bodies::new();
            system.insert(
                8,
                json!({"BodyID":8,"BodyName":"Second numeric ID","StarType":"G"}),
            );
            system.insert(
                0,
                json!({"BodyID":0,"BodyName":"Last numeric ID","StarType":"G"}),
            );
            let mut signals = Bodies::new();
            signals.insert(2,json!({"Signals":[{"Type":"$SAA_SignalType_Biological;","Count":3},{"Type":"$SAA_SignalType_Geological;","Count":2}]}));
            for kind in [
                "FSDJump",
                "FSSDiscoveryScan",
                "SAASignalsFound",
                "FSSAllBodiesFound",
                "Scan",
                "Scan",
            ] {
                input.event = kind.into();
                input.data["SystemName"] = json!("Sol");
                input.data["event"] = json!(kind);
                assert_eq!(
                    native.process(&input, &system, &signals, "Sol"),
                    original.process(&input, &system, &signals, "Sol"),
                    "{} {kind}",
                    case["Label"]
                );
            }
        }
    }
    fn event() -> JournalEvent {
        JournalEvent {
            raw_json: None,
            event: "Scan".into(),
            timestamp: "2026-10-25T00:00:00Z".into(),
            data: json!({"event":"Scan","timestamp":"2026-10-25T00:00:00Z","StarSystem":"Sol","SystemAddress":1,"BodyID":2,"BodyName":"Earth","Landable":true,"Radius":200000,"Materials":{"Carbon":2.5},"Parents":[{"Star":0}]}),
        }
    }
    fn criteria(source: &str) -> Criteria {
        Criteria::from_source(PathBuf::from("<test>"), None, source).unwrap()
    }
    #[test]
    fn legacy_simple_and_detail() {
        let mut c = criteria(
            "::Description=Tiny 'world'::\nscan.Radius < 300000\n::Detail::\nstring.format('%.0f km', distanceAsKm(scan.Radius))",
        );
        let found = c.process(&event(), &Bodies::new(), &Bodies::new(), "Sol");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].title, "Tiny 'world'");
        assert_eq!(found[0].detail, "200 km");
    }
    #[test]
    fn helpers_and_parent_history() {
        let source = "---@Criteria helpers\nlocal count=0\nfor m in materials(scan.Materials) do if m.name == 'Carbon' then count=count+1 end end\nfor p in allparents(parents) do if p.scan and isStar(p.scan) then count=count+1 end end\nreturn count==2, 'Helpers', tostring(count)\n---@End";
        let mut c = criteria(source);
        let mut system = Bodies::new();
        system.insert(0, json!({"event":"Scan","StarType":"G","BodyID":0}));
        let found = c.process(&event(), &system, &Bodies::new(), "Sol");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].detail, "2");
    }
    #[test]
    fn all_handler_types_and_notify() {
        let mut c = criteria(
            "---@Global\nanswer=42\n---@End\n---@Jump\nnotify(jump.StarSystem, 'Arrived', tostring(answer))\n---@End\n---@BodySignals\nnotifyForBody('ignored','Signals','Found',2)\n---@End",
        );
        let mut e = event();
        e.event = "FSDJump".into();
        e.data["event"] = "FSDJump".into();
        let found = c.process(&e, &Bodies::new(), &Bodies::new(), "Sol");
        assert_eq!(found[0].body, "Sol");
        assert_eq!(found[0].detail, "42");
        e.event = "SAASignalsFound".into();
        let mut system = Bodies::new();
        system.insert(2, json!({"BodyName":"Earth"}));
        assert_eq!(
            c.process(&e, &system, &Bodies::new(), "Sol")[0].body,
            "ignored"
        );
    }
    #[test]
    fn failing_handler_isolated_and_budgeted() {
        let mut c =
            criteria("---@Complex loop\nwhile true do end\n---@End\n::Description=Good::\ntrue");
        let found = c.process(&event(), &Bodies::new(), &Bodies::new(), "Sol");
        assert_eq!(found.len(), 2);
        assert!(found[0].title.contains("instruction budget"));
        assert_eq!(found[1].title, "Good");
        assert_eq!(
            c.process(&event(), &Bodies::new(), &Bodies::new(), "Sol")
                .len(),
            1
        );
    }
    #[test]
    fn malformed_block_has_location() {
        let result = compile_criteria("---@Complex broken\nreturn true");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unterminated complex"));
    }
    #[test]
    fn empty_numeric_properties_and_zero_index() {
        let mut c = criteria(
            "---@Complex\nreturn scan.SurfaceGravity == 0 and scan.Parent[0].Item2 == 0 and scan.Materials.Count == 1, 'Defaults', ''\n---@End",
        );
        assert_eq!(
            c.process(&event(), &Bodies::new(), &Bodies::new(), "Sol")
                .len(),
            1
        );
    }
    #[test]
    fn discovery_and_all_bodies_receive_history() {
        let mut c = criteria(
            "---@Discovery\nnotify(discovery.SystemName, 'Discovery', tostring(discovery.BodyCount))\n---@End\n---@AllBodies\nlocal n=0\nfor body in bodies(system) do n=n+1 end\nnotify(allBodies.SystemName, 'Complete', tostring(n)..':'..parentsTable[2][0].ParentType)\n---@End",
        );
        let mut e = event();
        e.event = "FSSDiscoveryScan".into();
        e.data["event"] = json!("FSSDiscoveryScan");
        e.data["SystemName"] = json!("Sol");
        e.data["BodyCount"] = 3.into();
        let mut system = Bodies::new();
        system.insert(2, event().data);
        let found = c.process(&e, &system, &Bodies::new(), "Sol");
        assert_eq!(found[0].detail, "3");
        e.event = "FSSAllBodiesFound".into();
        e.data["event"] = json!("FSSAllBodiesFound");
        let found = c.process(&e, &system, &Bodies::new(), "Sol");
        assert_eq!(found[0].detail, "1:Star");
    }
    #[test]
    fn coroutine_obeys_instruction_budget() {
        let mut c = criteria(
            "---@Complex coroutine\nlocal f=coroutine.wrap(function() for i=1,10000000 do end end)\nf()\nreturn true,'Must not finish',''\n---@End",
        );
        let found = c.process(&event(), &Bodies::new(), &Bodies::new(), "Sol");
        assert_eq!(found.len(), 1);
        assert!(found[0].title.contains("instruction budget"));
    }
    #[test]
    fn invalid_edit_disables_functions_until_source_is_repaired() {
        let path = std::env::temp_dir().join(format!(
            "observatory-criteria-{}-{}.lua",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, "::Description=Working::\ntrue").unwrap();
        let mut c = Criteria::from_path(path.clone()).unwrap();
        assert_eq!(
            c.process(&event(), &Bodies::new(), &Bodies::new(), "Sol")[0].title,
            "Working"
        );
        std::fs::write(&path, "---@Complex unterminated\ninvalid lua").unwrap();
        // Force a differing mtime marker without depending on the filesystem's timestamp granularity.
        c.modified = Some(std::time::UNIX_EPOCH);
        let found = c.process(&event(), &Bodies::new(), &Bodies::new(), "Sol");
        assert!(found.is_empty());
        assert!(c
            .take_diagnostics()
            .iter()
            .any(
                |diagnostic| ["Message", "Detail"].iter().any(|field| diagnostic[*field]
                    .as_str()
                    .unwrap_or("")
                    .contains("Error Reading Custom Criteria File"))
            ));
        assert!(c
            .process(&event(), &Bodies::new(), &Bodies::new(), "Sol")
            .is_empty());
        std::fs::write(&path, "::Description=Repaired::\ntrue").unwrap();
        c.modified = Some(std::time::UNIX_EPOCH);
        assert_eq!(
            c.process(&event(), &Bodies::new(), &Bodies::new(), "Sol")[0].title,
            "Repaired"
        );
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn lua_receives_source_numeric_types_and_preserves_raw_event() {
        let mut c = criteria(
            "---@Complex single precision\nreturn scan.Radius == 18000000 and scan.RotationPeriod == 12060 and scan.Materials[0].Percent == 2.5 and scan.Rings[0].InnerRad == 1000000.125 and scan.DistanceFromArrivalLS > 10, 'Source types', ''\n---@End",
        );
        let mut e = event();
        e.data["Radius"] = 18000000.1.into();
        e.data["RotationPeriod"] = 12060.0001.into();
        e.data["DistanceFromArrivalLS"] = 10.000000001.into();
        e.data["Materials"] = json!([{"Name":"Carbon","Percent":2.50000001}]);
        e.data["Rings"] = json!([{"Name":"A Ring","InnerRad":1000000.1}]);
        let findings = c.process(&e, &Bodies::new(), &Bodies::new(), "Sol");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].title, "Source types");
        assert_eq!(e.data["Radius"], json!(18000000.1));
        assert_eq!(e.data["Rings"][0]["InnerRad"], json!(1000000.1));
    }
}
