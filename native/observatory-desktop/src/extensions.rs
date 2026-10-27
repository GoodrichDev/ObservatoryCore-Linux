//! Reloadable native Lua extensions with bounded callbacks and host-managed I/O.
use anyhow::{ensure, Context, Result};
use mlua::{Function, HookTriggers, Lua, LuaSerdeExt, Table, VmState};
use observatory_domain::{Finding, JournalEvent};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant, SystemTime},
};

#[path = "core_api.rs"]
mod core_api;

const MAX_SOURCE: usize = 4 * 1024 * 1024;
const MAX_STORAGE: usize = 1024 * 1024;
const MAX_VALUE: usize = 256 * 1024;
const MAX_OUTPUT: usize = 4 * 1024 * 1024;
const MAX_JSON_NODES: usize = 262_144;
static NEXT_NOTIFICATION: AtomicU64 = AtomicU64::new(1);
static NEXT_HTTP_REQUEST: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Default)]
pub struct PluginOutput {
    pub findings: Vec<Finding>,
    pub notifications: Vec<Value>,
    pub messages: Vec<Value>,
    pub http_requests: Vec<Value>,
    pub rows: Vec<Value>,
    pub clear_rows: bool,
    pub logs: Vec<String>,
    pub settings: Option<Value>,
    bytes: usize,
}
impl PluginOutput {
    fn reserve(&mut self, value: &Value) -> mlua::Result<()> {
        let bytes = serde_json::to_vec(value)
            .map_err(mlua::Error::external)?
            .len();
        let count = self.findings.len()
            + self.notifications.len()
            + self.messages.len()
            + self.http_requests.len()
            + self.rows.len()
            + self.logs.len();
        if bytes > MAX_VALUE || self.bytes + bytes > MAX_OUTPUT || count >= 10_000 {
            return Err(mlua::Error::runtime("plugin output limit exceeded"));
        }
        self.bytes += bytes;
        Ok(())
    }
}
#[derive(Clone, Default)]
struct HostState {
    output: PluginOutput,
    storage: BTreeMap<String, Value>,
    storage_dirty: bool,
    active: bool,
    name: String,
    id: String,
    version: String,
    controls: BTreeMap<usize, core_api::RegisteredControl>,
}
pub struct Extension {
    pub name: String,
    pub id: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub override_popup: bool,
    pub override_audio: bool,
    pub accept_batch_notifications: bool,
    pub enabled: bool,
    pub settings: Value,
    lua: Lua,
    host: Arc<Mutex<HostState>>,
    storage_file: PathBuf,
}
impl Extension {
    #[cfg(test)]
    pub fn load(path: &Path, saved: Option<Value>, storage: &Path) -> Result<Self> {
        let mut extension = Self::prepare(path, saved, None, storage)?;
        extension.call("on_load", &json!({"settings":extension.settings}))?;
        Ok(extension)
    }
    fn prepare(
        path: &Path,
        saved: Option<Value>,
        registry: Option<&Value>,
        storage: &Path,
    ) -> Result<Self> {
        let source = String::from_utf8(read_bounded(path, MAX_SOURCE)?)
            .context("plugin source is not UTF-8")?;
        let lua = Lua::new();
        lua.set_memory_limit(32 * 1024 * 1024)?;
        let host = Arc::new(Mutex::new(HostState::default()));
        install_core(&lua, &host)?;
        for name in [
            "io", "os", "package", "dofile", "loadfile", "require", "debug",
        ] {
            lua.globals().set(name, mlua::Value::Nil)?;
        }
        let budget = set_budget(&lua)?;
        let plugin: Table = lua
            .load(&source)
            .set_name(path.to_string_lossy())
            .eval()
            .context("plugin must return a table")?;
        ensure!(
            !budget.load(Ordering::Relaxed),
            "plugin execution budget exceeded"
        );
        let name = plugin
            .get::<String>("name")
            .context("plugin must have a name")?;
        ensure!(
            !name.trim().is_empty() && name.len() <= 128,
            "invalid plugin name"
        );
        let id = plugin
            .get::<Option<String>>("id")?
            .unwrap_or_else(|| default_id(&name));
        ensure!(safe_id(&id), "invalid plugin id: use ASCII letters, digits, dots, underscores or hyphens, starting with a letter or digit");
        let version = plugin
            .get::<Option<String>>("version")?
            .unwrap_or_else(|| "0.0.0".into());
        ensure!(version.len() <= 128, "plugin version is too long");
        let description = plugin
            .get::<Option<String>>("description")?
            .unwrap_or_default();
        let author = plugin.get::<Option<String>>("author")?.unwrap_or_default();
        let override_popup = plugin
            .get::<Option<bool>>("override_popup")?
            .unwrap_or(false);
        let override_audio = plugin
            .get::<Option<bool>>("override_audio")?
            .unwrap_or(false);
        let accept_batch_notifications = plugin
            .get::<Option<bool>>("accept_batch_notifications")?
            .unwrap_or(false);
        let defaults = plugin
            .get::<Option<mlua::Value>>("settings")?
            .map(|v| lua.from_value::<Value>(v))
            .transpose()?
            .unwrap_or(json!({}));
        let mut settings = if defaults == json!([]) {
            json!({})
        } else {
            defaults
        };
        ensure!(settings.is_object(), "plugin settings must be an object");
        let saved = saved.or_else(|| {
            registry
                .and_then(|v| v.get(&name).or_else(|| v.get(&id)))
                .cloned()
        });
        if let Some(saved) = saved {
            ensure!(saved.is_object(), "saved plugin settings must be an object");
            merge_settings(&mut settings, saved);
        }
        ensure!(
            serde_json::to_vec(&settings)?.len() <= MAX_VALUE,
            "plugin settings are too large"
        );
        plugin.set("settings", lua.to_value(&settings)?)?;
        lua.globals().set("plugin", plugin)?;
        let storage_file = storage.join("storage").join(&id).join("state.json");
        fs::create_dir_all(storage_file.parent().unwrap())
            .context("Cannot create plugin storage folder")?;
        let storage_folder = format!(
            "{}{}",
            storage_file.parent().unwrap().to_string_lossy(),
            std::path::MAIN_SEPARATOR
        );
        {
            let mut state = host.lock().unwrap();
            state.name = name.clone();
            state.id = id.clone();
            state.version = version.clone();
            state.storage = read_storage(&storage_file)?;
        }
        let core: Table = lua.globals().get("core")?;
        core.set("metadata",lua.to_value(&json!({"name":name,"id":id,"version":version,"description":description,"author":author}))?)?;
        core.set("PluginStorageFolder", storage_folder.clone())?;
        core.set(
            "UpdatedPluginsFolder",
            storage.join("plugins").to_string_lossy().to_string(),
        )?;
        core.set("plugin_storage_folder", storage_folder)?;
        core.set(
            "updated_plugins_folder",
            storage.join("plugins").to_string_lossy().to_string(),
        )?;
        core.set(
            "storage_path",
            storage_file.parent().unwrap().to_string_lossy().to_string(),
        )?;
        Ok(Self {
            name,
            id,
            version,
            description,
            author,
            override_popup,
            override_audio,
            accept_batch_notifications,
            enabled: true,
            settings,
            lua,
            host,
            storage_file,
        })
    }
    /// Failed callbacks discard their output/storage changes and quarantine the
    /// instance. Output from earlier successful callbacks remains available.
    fn transaction<T>(&mut self, action: impl FnOnce(&Lua) -> Result<T>) -> Result<T> {
        let before = self.host.lock().unwrap().clone();
        self.host.lock().unwrap().active = true;
        let result = (|| {
            let budget = set_budget(&self.lua)?;
            let result = action(&self.lua)?;
            ensure!(
                !budget.load(Ordering::Relaxed),
                "plugin execution budget exceeded"
            );
            let state = self.host.lock().unwrap();
            if state.storage_dirty {
                save_storage(&self.storage_file, &state.storage)?;
            }
            Ok(result)
        })();
        let mut state = self.host.lock().unwrap();
        if result.is_err() {
            *state = before;
            self.enabled = false;
        } else {
            state.storage_dirty = false;
        }
        state.active = false;
        result
    }
    pub fn call(&mut self, handler: &str, value: &Value) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        let context = format!("{}: {handler}", self.name);
        self.transaction(|lua| {
            let plugin: Table = lua.globals().get("plugin")?;
            if let Some(function) = plugin.get::<Option<Function>>(handler)? {
                function.call::<()>(lua.to_value(value)?).context(context)?;
            }
            Ok(())
        })
    }
    pub fn set_settings(&mut self, value: Value) -> Result<()> {
        ensure!(value.is_object(), "plugin settings must be an object");
        ensure!(
            serde_json::to_vec(&value)?.len() <= MAX_VALUE,
            "plugin settings are too large"
        );
        self.transaction(|lua| {
            let plugin: Table = lua.globals().get("plugin")?;
            plugin.raw_set("settings", lua.to_value(&value)?)?;
            Ok(())
        })?;
        self.settings = value;
        Ok(())
    }
    pub fn export(&mut self, delimiter: &str) -> Result<Option<String>> {
        if !self.enabled {
            return Ok(None);
        }
        self.transaction(|lua| {
            let plugin: Table = lua.globals().get("plugin")?;
            match plugin.get::<Option<Function>>("on_export")? {
                Some(function) => {
                    let text: Option<String> = function.call(delimiter)?;
                    if let Some(text) = &text {
                        ensure!(
                            text.len() <= MAX_OUTPUT,
                            "plugin export byte limit exceeded"
                        );
                    }
                    Ok(text)
                }
                None => Ok(None),
            }
        })
    }
    pub fn set_context(&self, name: &str, value: &Value) -> Result<()> {
        let core = self.lua.globals().raw_get::<Table>("core")?;
        core.raw_set(name, self.lua.to_value(value)?)?;
        match name {
            "log_monitor_state" => {
                let flags = value.as_u64().unwrap_or(0);
                core.raw_set("CurrentLogMonitorState", flags)?;
                core.raw_set("IsLogMonitorBatchReading", flags & 6 != 0)?;
                core.raw_set("is_batch_reading", flags & 6 != 0)?;
            }
            "storage_path" => {
                let path = value.as_str().unwrap_or_default();
                let source = if path.is_empty() || path.ends_with(std::path::MAIN_SEPARATOR) {
                    path.to_string()
                } else {
                    format!("{path}{}", std::path::MAIN_SEPARATOR)
                };
                core.raw_set("PluginStorageFolder", source.clone())?;
                core.raw_set("plugin_storage_folder", source)?;
            }
            "updated_plugins_folder" => {
                core.raw_set("UpdatedPluginsFolder", self.lua.to_value(value)?)?
            }
            _ => {}
        }
        Ok(())
    }
    pub fn set_host_context(&self, context: &Value) -> Result<()> {
        for (key, value) in context
            .as_object()
            .context("Core context must be an object")?
        {
            self.set_context(key, value)?;
        }
        Ok(())
    }
    pub fn theme_changed(&mut self, name: &str, details: &Value) -> Result<()> {
        self.set_context("theme_name", &json!(name))?;
        self.set_context("theme_details", details)?;
        let host = self.host.clone();
        self.transaction(|lua| Ok(core_api::apply_registered(lua, &host)?))?;
        self.call("on_theme_changed", &json!({"Name":name,"Details":details}))
    }
    pub fn take_output(&mut self) -> PluginOutput {
        let mut out = std::mem::take(&mut self.host.lock().unwrap().output);
        for finding in &mut out.findings {
            finding.plugin = self.name.clone();
        }
        for notification in &mut out.notifications {
            notification["plugin"] = json!(self.name);
        }
        for message in &mut out.messages {
            *message = json!({"source":{"name":self.name,"id":self.id,"version":self.version},"data":message.clone()});
        }
        if let Some(settings) = &out.settings {
            self.settings = settings.clone();
        }
        out
    }
}
fn active(state: &mut HostState) -> mlua::Result<()> {
    if state.active {
        Ok(())
    } else {
        Err(mlua::Error::runtime(
            "core output and storage functions are available inside callbacks",
        ))
    }
}
fn object_from_table(lua: &Lua, table: Table) -> mlua::Result<Value> {
    let mut value: Value = lua.from_value(mlua::Value::Table(table))?;
    if value == json!([]) {
        value = json!({});
    }
    if !value.is_object() {
        return Err(mlua::Error::runtime("expected a table with named fields"));
    }
    Ok(value)
}
fn install_core(lua: &Lua, host: &Arc<Mutex<HostState>>) -> mlua::Result<()> {
    let core = lua.create_table()?;
    core.set(
        "json_decode",
        lua.create_function(|lua, text: mlua::String| {
            if text.as_bytes().len() > MAX_OUTPUT {
                return Err(mlua::Error::runtime("JSON input exceeds 4 MiB"));
            }
            let value: Value =
                serde_json::from_slice(text.as_bytes().as_ref()).map_err(|error| {
                    mlua::Error::runtime(format!(
                        "Invalid JSON at line {}, column {}",
                        error.line(),
                        error.column()
                    ))
                })?;
            validate_decoded_json(&value, &mut 0)?;
            lua.to_value(&value)
        })?,
    )?;
    core.set(
        "json_encode",
        lua.create_function(|lua, value: mlua::Value| {
            validate_json_value(
                &value,
                0,
                &mut 0,
                &mut HashSet::new(),
                lua.array_metatable().to_pointer() as usize,
            )?;
            let mut output = JsonBuffer(Vec::new());
            serde_json::to_writer(&mut output, &value.to_serializable())
                .map_err(|error| mlua::Error::runtime(format!("Cannot encode JSON: {error}")))?;
            String::from_utf8(output.0).map_err(mlua::Error::external)
        })?,
    )?;
    let shared = host.clone();
    core.set(
        "http_request",
        lua.create_function(move |lua, table: Table| {
            let mut value = object_from_table(lua, table)?;
            let mut state = shared.lock().unwrap();
            active(&mut state)?;
            let supplied = match value.get("request_id") {
                None => {
                    let epoch = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos();
                    format!(
                        "http-{epoch:x}-{}",
                        NEXT_HTTP_REQUEST.fetch_add(1, Ordering::Relaxed)
                    )
                }
                Some(Value::String(id)) => id.clone(),
                _ => return Err(mlua::Error::runtime("HTTP request_id must be a string")),
            };
            let id = notification_id(&state.id, &supplied)
                .map_err(|_| mlua::Error::runtime("Invalid HTTP request_id"))?;
            value["request_id"] = json!(id);
            crate::http::validate_request(&value).map_err(mlua::Error::runtime)?;
            state.output.reserve(&value)?;
            state.output.http_requests.push(value);
            Ok(id)
        })?,
    )?;
    let shared = host.clone();
    core.set("play_audio_file",lua.create_function(move|lua,(filename,options):(String,Option<Table>)|{
        if filename.len()>4096{return Err(mlua::Error::runtime("Audio filename is too long"));}
        let mut options=options.map(|table|object_from_table(lua,table)).transpose()?.unwrap_or(json!({}));let mut state=shared.lock().unwrap();active(&mut state)?;
        let epoch=SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_nanos();let supplied=options["request_id"].as_str().map(str::to_owned).unwrap_or_else(||format!("audio-{epoch:x}-{}",NEXT_NOTIFICATION.fetch_add(1,Ordering::Relaxed)));let id=notification_id(&state.id,&supplied)?;options["request_id"]=json!(id);let message=json!({"target":"Core","command":"PlayAudioFile","payload":{"Filename":filename,"Options":options}});state.output.reserve(&message)?;state.output.messages.push(message);Ok(id)
    })?)?;
    let shared = host.clone();
    core.set(
        "notify",
        lua.create_function(move |lua, table: Table| {
            let mut value = object_from_table(lua, table)?;
            let mut state = shared.lock().unwrap();
            active(&mut state)?;
            let supplied = value["id"].as_str().map(str::to_owned).unwrap_or_else(|| {
                let epoch = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos();
                format!(
                    "{epoch:x}-{}",
                    NEXT_NOTIFICATION.fetch_add(1, Ordering::Relaxed)
                )
            });
            let id = notification_id(&state.id, &supplied)?;
            value["action"] = json!("show");
            value["id"] = json!(id);
            state.output.reserve(&value)?;
            state.output.notifications.push(value);
            Ok(id)
        })?,
    )?;
    for (name, action) in [
        ("update_notification", "update"),
        ("cancel_notification", "cancel"),
    ] {
        let shared = host.clone();
        core.set(
            name,
            lua.create_function(move |lua, (id, table): (String, Option<Table>)| {
                let mut value: Value = match table {
                    Some(t) => object_from_table(lua, t)?,
                    None => json!({}),
                };
                let mut state = shared.lock().unwrap();
                active(&mut state)?;
                value["action"] = json!(action);
                value["id"] = json!(notification_id(&state.id, &id)?);
                state.output.reserve(&value)?;
                state.output.notifications.push(value);
                Ok(())
            })?,
        )?;
    }
    let shared = host.clone();
    core.set(
        "add_finding",
        lua.create_function(move |lua, table: Table| {
            let value = object_from_table(lua, table)?;
            let mut state = shared.lock().unwrap();
            active(&mut state)?;
            state.output.reserve(&value)?;
            state.output.findings.push(finding_from_value(&value));
            Ok(())
        })?,
    )?;
    for (name, message) in [("add_row", false), ("send_message", true)] {
        let shared = host.clone();
        core.set(
            name,
            lua.create_function(move |lua, value: mlua::Value| {
                let value: Value = lua.from_value(value)?;
                let mut state = shared.lock().unwrap();
                active(&mut state)?;
                state.output.reserve(&value)?;
                if message {
                    state.output.messages.push(value);
                } else {
                    state.output.rows.push(value);
                }
                Ok(())
            })?,
        )?;
    }
    let shared = host.clone();
    core.set(
        "clear_rows",
        lua.create_function(move |_, ()| {
            let mut state = shared.lock().unwrap();
            active(&mut state)?;
            state.output.rows.clear();
            state.output.clear_rows = true;
            Ok(())
        })?,
    )?;
    let shared = host.clone();
    core.set(
        "log",
        lua.create_function(move |_, message: String| {
            let mut state = shared.lock().unwrap();
            active(&mut state)?;
            if message.len() > 4096 || state.output.logs.len() >= 100 {
                return Err(mlua::Error::runtime("plugin log limit exceeded"));
            }
            state.output.reserve(&json!(message))?;
            state.output.logs.push(message);
            Ok(())
        })?,
    )?;
    let shared = host.clone();
    core.set(
        "save_settings",
        lua.create_function(move |lua, value: mlua::Value| {
            let mut value: Value = lua.from_value(value)?;
            if value == json!([]) {
                value = json!({});
            }
            if !value.is_object() {
                return Err(mlua::Error::runtime("settings must be an object"));
            }
            let mut state = shared.lock().unwrap();
            active(&mut state)?;
            state.output.reserve(&value)?;
            // Avoid re-entering plugin __newindex code while holding the host lock.
            lua.globals()
                .raw_get::<Table>("plugin")?
                .raw_set("settings", lua.to_value(&value)?)?;
            state.output.settings = Some(value);
            Ok(())
        })?,
    )?;
    let shared = host.clone();
    core.set(
        "get_storage",
        lua.create_function(move |lua, (key, fallback): (String, Option<mlua::Value>)| {
            validate_key(&key)?;
            let mut state = shared.lock().unwrap();
            active(&mut state)?;
            match state.storage.get(&key) {
                Some(value) => lua.to_value(value),
                None => Ok(fallback.unwrap_or(mlua::Value::Nil)),
            }
        })?,
    )?;
    let shared = host.clone();
    core.set(
        "set_storage",
        lua.create_function(move |lua, (key, value): (String, mlua::Value)| {
            validate_key(&key)?;
            let value: Value = lua.from_value(value)?;
            let mut state = shared.lock().unwrap();
            active(&mut state)?;
            let mut next = state.storage.clone();
            next.insert(key, value);
            validate_storage(&next).map_err(mlua::Error::external)?;
            state.storage = next;
            state.storage_dirty = true;
            Ok(())
        })?,
    )?;
    let shared = host.clone();
    core.set(
        "delete_storage",
        lua.create_function(move |_, key: String| {
            validate_key(&key)?;
            let mut state = shared.lock().unwrap();
            active(&mut state)?;
            if state.storage.remove(&key).is_some() {
                state.storage_dirty = true;
            }
            Ok(())
        })?,
    )?;
    core.set("version", env!("CARGO_PKG_VERSION"))?;
    core.set("api_version", 1)?;
    core.set("null", lua.to_value(&Value::Null)?)?;
    core.set("status", lua.create_table()?)?;
    core.set("cargo", lua.create_table()?)?;
    lua.globals().set("core", core)?;
    core_api::install(lua, host)?;
    Ok(())
}

/// Avoid building an unbounded Rust JSON tree from repeated Lua table aliases.
/// serde then streams directly to JsonBuffer with its own recursive-table guard.
fn validate_json_value(
    value: &mlua::Value,
    depth: usize,
    nodes: &mut usize,
    path: &mut HashSet<usize>,
    array_metatable: usize,
) -> mlua::Result<()> {
    *nodes += 1;
    if depth > 128 || *nodes > MAX_JSON_NODES {
        return Err(mlua::Error::runtime(
            "JSON nesting or value count exceeds the limit",
        ));
    }
    match value {
        mlua::Value::Table(table) => {
            let identity = table.to_pointer() as usize;
            if !path.insert(identity) {
                return Err(mlua::Error::runtime("Cannot encode a recursive JSON table"));
            }
            let length = table.raw_len();
            let array = length > 0
                || table
                    .metatable()
                    .is_some_and(|meta| meta.to_pointer() as usize == array_metatable);
            let mut entries = 0;
            for pair in table.clone().pairs::<mlua::Value, mlua::Value>() {
                let (key, value) = pair?;
                if array
                    && !matches!(&key, mlua::Value::Integer(index) if *index > 0 && (*index as usize) <= length)
                {
                    return Err(mlua::Error::runtime(
                        "JSON cannot encode a table mixing array entries and object fields",
                    ));
                }
                entries += 1;
                validate_json_value(&key, depth + 1, nodes, path, array_metatable)?;
                validate_json_value(&value, depth + 1, nodes, path, array_metatable)?;
            }
            if array && entries != length {
                return Err(mlua::Error::runtime(
                    "JSON arrays cannot contain missing entries; use core.null",
                ));
            }
            path.remove(&identity);
        }
        mlua::Value::Number(number) if !number.is_finite() => {
            return Err(mlua::Error::runtime(
                "JSON cannot encode a non-finite number",
            ));
        }
        mlua::Value::String(text) => {
            text.to_str()?;
        }
        _ => {}
    }
    Ok(())
}

fn validate_decoded_json(value: &Value, nodes: &mut usize) -> mlua::Result<()> {
    *nodes += 1;
    if *nodes > MAX_JSON_NODES {
        return Err(mlua::Error::runtime("JSON value count exceeds the limit"));
    }
    match value {
        Value::Array(values) => {
            for value in values {
                validate_decoded_json(value, nodes)?;
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                validate_decoded_json(value, nodes)?;
            }
        }
        _ => {}
    }
    Ok(())
}

struct JsonBuffer(Vec<u8>);
impl Write for JsonBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.0.len().saturating_add(bytes.len()) > MAX_OUTPUT {
            return Err(std::io::Error::other("JSON output exceeds 4 MiB"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub fn finding_from_value(value: &Value) -> Finding {
    let string = |key: &str| value[key].as_str().unwrap_or("").to_owned();
    Finding {
        plugin: string("plugin"),
        timestamp: string("timestamp"),
        system: string("system"),
        body: string("body"),
        title: string("title"),
        detail: string("detail"),
    }
}
fn notification_id(plugin_id: &str, supplied: &str) -> mlua::Result<String> {
    if supplied.is_empty() || supplied.len() > 512 || supplied.chars().any(char::is_control) {
        return Err(mlua::Error::runtime("invalid notification id"));
    }
    let prefix = format!("{plugin_id}:");
    let id = if supplied.starts_with(&prefix) {
        supplied.into()
    } else {
        format!("{prefix}{supplied}")
    };
    if id.len() > 512 {
        return Err(mlua::Error::runtime("notification id is too long"));
    }
    Ok(id)
}
fn validate_key(key: &str) -> mlua::Result<()> {
    if key.is_empty() || key.len() > 128 || key.chars().any(char::is_control) {
        Err(mlua::Error::runtime(
            "invalid storage key (use 1-128 bytes without control characters)",
        ))
    } else {
        Ok(())
    }
}
fn validate_storage(storage: &BTreeMap<String, Value>) -> Result<()> {
    ensure!(storage.len() <= 1024, "plugin storage key limit exceeded");
    for key in storage.keys() {
        validate_key(key)?;
    }
    ensure!(
        serde_json::to_vec(storage)?.len() <= MAX_STORAGE,
        "plugin storage byte limit exceeded"
    );
    Ok(())
}
fn read_bounded(path: &Path, max: usize) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    File::open(path)?
        .take(max as u64 + 1)
        .read_to_end(&mut data)?;
    ensure!(
        data.len() <= max,
        "{} exceeds its {max} byte limit",
        path.display()
    );
    Ok(data)
}
fn read_storage(path: &Path) -> Result<BTreeMap<String, Value>> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let storage = serde_json::from_slice(&read_bounded(path, MAX_STORAGE)?)
        .with_context(|| format!("invalid plugin storage {}", path.display()))?;
    validate_storage(&storage)?;
    Ok(storage)
}
fn save_storage(path: &Path, storage: &BTreeMap<String, Value>) -> Result<()> {
    validate_storage(storage)?;
    let parent = path.parent().context("plugin storage has no parent")?;
    fs::create_dir_all(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer(&mut temporary, storage)?;
    temporary.write_all(b"\n")?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    #[cfg(unix)]
    File::open(parent)?.sync_all()?;
    Ok(())
}
fn safe_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id.as_bytes()[0].is_ascii_alphanumeric()
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'-'))
}
fn default_id(name: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in name.as_bytes() {
        hash = (hash ^ *byte as u64).wrapping_mul(0x100000001b3);
    }
    let slug: String = name
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphanumeric() {
                Some(c.to_ascii_lowercase())
            } else if matches!(c, ' ' | '-' | '_') {
                Some('-')
            } else {
                None
            }
        })
        .take(64)
        .collect();
    format!("plugin-{}-{hash:016x}", slug.trim_matches('-'))
}
fn merge_settings(defaults: &mut Value, saved: Value) {
    match (defaults, saved) {
        (Value::Object(defaults), Value::Object(saved)) => {
            for (key, value) in saved {
                if let Some(current) = defaults.get_mut(&key) {
                    merge_settings(current, value);
                } else {
                    defaults.insert(key, value);
                }
            }
        }
        (defaults, saved) => *defaults = saved,
    }
}
fn set_budget(lua: &Lua) -> mlua::Result<Arc<AtomicBool>> {
    let start = Instant::now();
    let instructions = Arc::new(AtomicU64::new(0));
    let exhausted = Arc::new(AtomicBool::new(false));
    let signal = exhausted.clone();
    lua.set_global_hook(
        HookTriggers::new().every_nth_instruction(1000),
        move |_, _| {
            if start.elapsed() > Duration::from_millis(100)
                || instructions.fetch_add(1000, Ordering::Relaxed) >= 5_000_000
            {
                signal.store(true, Ordering::Relaxed);
                Err(mlua::Error::runtime("plugin execution budget exceeded"))
            } else {
                Ok(VmState::Continue)
            }
        },
    )?;
    Ok(exhausted)
}
#[cfg(test)]
pub fn discover(dir: &Path, saved: &Value, storage: &Path) -> (Vec<Extension>, Vec<String>) {
    discover_with_context(dir, saved, storage, &json!({}))
}
pub fn discover_with_context(
    dir: &Path,
    saved: &Value,
    storage: &Path,
    context: &Value,
) -> (Vec<Extension>, Vec<String>) {
    let mut plugins = Vec::new();
    let mut errors = Vec::new();
    let mut files = match fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .collect::<Vec<_>>(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return (plugins, errors),
        Err(error) => return (plugins, vec![format!("{}: {error}", dir.display())]),
    };
    files.sort();
    let mut names = HashSet::new();
    let mut ids = HashSet::new();
    for path in files
        .into_iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("lua"))
    {
        match Extension::prepare(&path, None, Some(saved), storage) {
            Ok(mut plugin) => {
                if !names.insert(plugin.name.clone()) || !ids.insert(plugin.id.clone()) {
                    errors.push(format!(
                        "duplicate plugin name or id: {} ({})",
                        plugin.name, plugin.id
                    ));
                    continue;
                }
                if let Err(error) = plugin.set_host_context(context) {
                    errors.push(format!("{}: {error:#}", path.display()));
                    continue;
                }
                match plugin.call("on_load", &json!({"settings":plugin.settings})) {
                    Ok(()) => plugins.push(plugin),
                    Err(error) => errors.push(format!("{}: {error:#}", path.display())),
                }
            }
            Err(error) => errors.push(format!("{}: {error:#}", path.display())),
        }
    }
    (plugins, errors)
}
pub fn journal_type(event: &str) -> (&'static str, &'static str) {
    core_api::journal_type(event)
}
pub fn compare_grid_text(left: &str, right: &str) -> std::cmp::Ordering {
    core_api::compare_grid_text(left, right)
}
pub fn sort_grid_rows(rows: &mut Vec<&Value>, column: &str, ascending: bool) {
    core_api::sort_grid_rows(rows, column, ascending)
}
pub fn event_value(event: &JournalEvent) -> Value {
    event.data.clone()
}

#[cfg(test)]
#[path = "extensions_tests.rs"]
mod tests;

#[cfg(test)]
mod http_tests {
    use super::*;

    #[test]
    fn http_staging_is_scoped_and_returns_without_network_io() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("http.lua");
        fs::write(&path,r#"return {name="HTTP Test",id="test.http",on_journal=function()
            local id=core.http_request{url="https://example.invalid/resource",request_id="one",headers={Accept="application/json"}}
            core.add_row{id=id}
        end}"#).unwrap();
        let mut plugin = Extension::load(&path, None, temp.path()).unwrap();
        plugin.call("on_journal", &json!({})).unwrap();
        let output = plugin.take_output();
        assert_eq!(output.http_requests.len(), 1);
        assert_eq!(output.http_requests[0]["request_id"], "test.http:one");
        assert_eq!(output.rows[0]["id"], "test.http:one");
    }

    #[test]
    fn failed_callback_discards_staged_http_requests() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("http.lua");
        fs::write(
            &path,
            r#"return {name="HTTP Test",id="test.http",on_journal=function()
            core.http_request{url="https://example.invalid/resource"};error("discard")
        end}"#,
        )
        .unwrap();
        let mut plugin = Extension::load(&path, None, temp.path()).unwrap();
        assert!(plugin.call("on_journal", &json!({})).is_err());
        assert!(plugin.take_output().http_requests.is_empty());
    }
}
