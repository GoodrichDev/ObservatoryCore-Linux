//! Typed host adapters. Builtins have no serialization/IPC overhead. Remote
//! failures are retained for the host error panel; no state-reset fallback.
use super::{Client, WorkerInfo, WorkerKind};
use anyhow::Result;
use observatory_domain::{Finding, JournalEvent};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Mutex,
};

struct Remote {
    client: Mutex<Client>,
    errors: Mutex<Vec<String>>,
    reported: Mutex<BTreeSet<String>>,
    cache: Mutex<BTreeMap<String, Value>>,
    settings: Value,
}
impl Remote {
    fn new(executable: &Path, kind: WorkerKind, settings: Value, directory: &Path) -> Result<Self> {
        let client = Client::spawn(executable, kind, settings.clone(), directory)?;
        Ok(Self {
            client: Mutex::new(client),
            errors: Mutex::new(Vec::new()),
            reported: Mutex::new(BTreeSet::new()),
            cache: Mutex::new(BTreeMap::new()),
            settings,
        })
    }
    fn result<T: DeserializeOwned>(&self, method: &str, args: Value) -> Result<T> {
        self.client
            .lock()
            .expect("native worker client")
            .call(method, args)
    }
    fn value<T: DeserializeOwned + Default>(&self, method: &str, args: Value) -> T {
        match self.result(method, args) {
            Ok(result) => result,
            Err(error) => {
                self.record_error(format!("{error:#}"));
                T::default()
            }
        }
    }
    fn record_error(&self, error: String) {
        let mut reported = self.reported.lock().expect("native worker reported errors");
        if reported.len() < 32 && reported.insert(error.clone()) {
            self.errors
                .lock()
                .expect("native worker errors")
                .push(error);
        }
    }
    /// Keep the last completed projection if a worker crashes. In particular,
    /// a failed state() query must never replace Bio's personal Codex with null.
    fn cached<T: DeserializeOwned + Default>(&self, method: &str, args: Value) -> T {
        let key = format!("{method}:{args}");
        let value = match self.result::<Value>(method, args) {
            Ok(value) => {
                let mut cache = self.cache.lock().expect("native worker projections");
                if cache.len() >= 128 {
                    if let Some(old) = cache.keys().find(|key| !key.starts_with("state:")).cloned()
                    {
                        cache.remove(&old);
                    }
                }
                cache.insert(key.clone(), value.clone());
                value
            }
            Err(error) => {
                self.record_error(format!("{error:#}"));
                self.cache
                    .lock()
                    .expect("native worker projections")
                    .get(&key)
                    .cloned()
                    .unwrap_or_default()
            }
        };
        match serde_json::from_value(value) {
            Ok(value) => value,
            Err(error) => {
                self.record_error(format!(
                    "Invalid {method} native worker projection: {error}"
                ));
                T::default()
            }
        }
    }
    fn errors(&self) -> Vec<String> {
        std::mem::take(&mut *self.errors.lock().expect("native worker errors"))
    }
}
enum Backend<T> {
    Builtin(Box<T>),
    Remote(Box<Remote>),
}
macro_rules! delegate {
    ($(fn $name:ident($($arg:ident:$ty:ty),*) -> $ret:ty => $json:expr;)*) => {$ (
        pub fn $name(&mut self,$($arg:$ty),*) -> $ret {
            match &mut self.backend {
                Backend::Builtin(w) => w.$name($($arg),*),
                Backend::Remote(r) => r.value(stringify!($name),$json),
            }
        }
    )*};
}
macro_rules! query {
    ($(fn $name:ident($($arg:ident:$ty:ty),*) -> $ret:ty => $json:expr;)*) => {$ (
        pub fn $name(&self,$($arg:$ty),*) -> $ret {
            match &self.backend {
                Backend::Builtin(w) => w.$name($($arg),*),
                Backend::Remote(r) => r.cached(stringify!($name),$json),
            }
        }
    )*};
}
macro_rules! common {
    () => {
        pub fn is_external(&self) -> bool {
            matches!(self.backend, Backend::Remote(_))
        }
        pub fn is_healthy(&self) -> bool {
            match &self.backend {
                Backend::Builtin(_) => true,
                Backend::Remote(r) => r.client.lock().expect("native worker client").is_healthy(),
            }
        }
        pub fn worker_info(&self) -> Option<WorkerInfo> {
            match &self.backend {
                Backend::Builtin(_) => None,
                Backend::Remote(r) => Some(
                    r.client
                        .lock()
                        .expect("native worker client")
                        .info()
                        .clone(),
                ),
            }
        }
        pub fn worker_process_id(&self) -> Option<u32> {
            match &self.backend {
                Backend::Builtin(_) => None,
                Backend::Remote(r) => {
                    Some(r.client.lock().expect("native worker client").process_id())
                }
            }
        }
        pub fn take_worker_errors(&mut self) -> Vec<String> {
            match &self.backend {
                Backend::Builtin(_) => Vec::new(),
                Backend::Remote(r) => r.errors(),
            }
        }
    };
}

pub struct BioInsights {
    backend: Backend<crate::bioinsights::BioInsights>,
}
impl BioInsights {
    pub fn new(settings: Value) -> Result<Self> {
        Ok(Self {
            backend: Backend::Builtin(Box::new(crate::bioinsights::BioInsights::new(settings)?)),
        })
    }
    pub fn from_executable(settings: Value, directory: &Path, executable: &Path) -> Result<Self> {
        let remote = Remote::new(executable, WorkerKind::BioInsights, settings, directory)?;
        let state: Value = remote.result("state", Value::Null)?;
        remote
            .cache
            .lock()
            .expect("native worker projections")
            .insert("state:null".into(), state);
        Ok(Self {
            backend: Backend::Remote(Box::new(remote)),
        })
    }
    pub fn default_settings() -> Value {
        crate::bioinsights::default_settings()
    }
    pub fn set_settings(&mut self, settings: Value) -> Result<()> {
        match &mut self.backend {
            Backend::Builtin(w) => w.set_settings(settings),
            Backend::Remote(r) => {
                r.result::<()>("set_settings", json!({"settings":settings}))?;
                r.settings = settings;
                Ok(())
            }
        }
    }
    pub fn restore_legacy_state(&mut self, state: &Value) -> Result<usize> {
        match &mut self.backend {
            Backend::Builtin(w) => w.restore_legacy_state(state),
            Backend::Remote(r) => r.result("restore_legacy_state", json!({"state":state})),
        }
    }
    pub fn handle_http_response(&mut self, response: &Value) -> Result<()> {
        match &mut self.backend {
            Backend::Builtin(w) => w.handle_http_response(response),
            Backend::Remote(r) => r.result("handle_http_response", json!({"response":response})),
        }
    }
    delegate! {
        fn reset() -> () => Value::Null;
        fn set_batch(batch:bool) -> () => json!({"batch":batch});
        fn restore_state(state:&Value) -> () => json!({"state":state});
        fn on_event(event:&JournalEvent) -> Vec<Finding> => json!({"event":event});
        fn on_status(status:&Value) -> Vec<Finding> => json!({"status":status});
        fn on_message(command:&str,payload:&Value) -> Vec<Finding> => json!({"command":command,"payload":payload});
        fn on_peer_message(sender:&str,command:&str,payload:&Value) -> Vec<Finding> => json!({"sender":sender,"command":command,"payload":payload});
        fn take_messages() -> Vec<Value> => Value::Null;
        fn take_http_requests() -> Vec<Value> => Value::Null;
        fn take_diagnostics() -> Vec<Value> => Value::Null;
        fn take_notification_rendering(finding:&Finding) -> Option<u8> => json!({"finding":finding});
    }
    query! {
        fn state() -> Value => Value::Null;
        fn codex_companion() -> Vec<Value> => Value::Null;
        fn unsold_data() -> Vec<Value> => Value::Null;
        fn tools_summary() -> Value => Value::Null;
        fn body_predictions(body:u64) -> Vec<Value> => json!({"body":body});
        fn body_summary(body:u64) -> Value => json!({"body":body});
        fn rows() -> Vec<Value> => Value::Null;
        fn overlay() -> Value => Value::Null;
        fn main_view() -> Value => Value::Null;
        fn browser_route() -> String => Value::Null;
        fn browser_html() -> String => Value::Null;
    }
    pub fn on_events(&mut self, events: &[JournalEvent]) -> Vec<Finding> {
        match &mut self.backend {
            Backend::Builtin(w) => events.iter().flat_map(|e| w.on_event(e)).collect(),
            Backend::Remote(r) => r.value("on_events", json!({"events":events})),
        }
    }
    common!();
}

pub struct Evaluator {
    backend: Backend<crate::evaluator::Evaluator>,
}
impl Evaluator {
    pub fn new(settings: Value) -> Self {
        Self {
            backend: Backend::Builtin(Box::new(crate::evaluator::Evaluator::new(settings))),
        }
    }
    pub fn from_executable(settings: Value, directory: &Path, executable: &Path) -> Result<Self> {
        Ok(Self {
            backend: Backend::Remote(Box::new(Remote::new(
                executable,
                WorkerKind::Evaluator,
                settings,
                directory,
            )?)),
        })
    }
    pub fn set_settings(&mut self, settings: Value) -> Result<()> {
        match &mut self.backend {
            Backend::Builtin(w) => w.set_settings(settings),
            Backend::Remote(r) => {
                r.result::<()>("set_settings", json!({"settings":settings}))?;
                r.settings = settings;
                Ok(())
            }
        }
    }
    delegate! {
        fn reset() -> () => Value::Null;
        fn set_batch(batch:bool) -> () => json!({"batch":batch});
        fn set_preread(preread:bool) -> () => json!({"preread":preread});
        fn on_event(event:&JournalEvent) -> Vec<Finding> => json!({"event":event});
        fn on_status(status:&Value) -> () => json!({"status":status});
        fn on_message(command:&str,payload:&Value) -> () => json!({"command":command,"payload":payload});
        fn on_peer_message(sender:&str,command:&str,payload:&Value) -> () => json!({"sender":sender,"command":command,"payload":payload});
        fn set_bio_summary(body_id:u64,summary:Value) -> () => json!({"body_id":body_id,"summary":summary});
        fn take_messages() -> Vec<Value> => Value::Null;
        fn poll_routes() -> bool => Value::Null;
        fn take_route_findings() -> Vec<Finding> => Value::Null;
        fn take_notification_rendering(finding:&Finding) -> Option<u8> => json!({"finding":finding});
        fn take_notification_payload(finding:&Finding) -> Option<Value> => json!({"finding":finding});
    }
    query! {
        fn rows() -> Vec<Value> => Value::Null;
        fn summary() -> Value => Value::Null;
        fn web_main_html() -> String => Value::Null;
        fn browser_html(page:&str)->String=>json!({"page":page});
    }
    pub fn on_events(&mut self, events: &[JournalEvent]) -> Vec<Finding> {
        match &mut self.backend {
            Backend::Builtin(w) => events.iter().flat_map(|e| w.on_event(e)).collect(),
            Backend::Remote(r) => r.value("on_events", json!({"events":events})),
        }
    }
    common!();
}

pub struct Hecate {
    backend: Backend<crate::hecate::Hecate>,
}
impl Hecate {
    pub fn new(settings: Value) -> Result<Self> {
        Ok(Self {
            backend: Backend::Builtin(Box::new(crate::hecate::Hecate::new(settings)?)),
        })
    }
    pub fn open(settings: Value, directory: &Path) -> Result<Self> {
        Ok(Self {
            backend: Backend::Builtin(Box::new(crate::hecate::Hecate::open(settings, directory)?)),
        })
    }
    pub fn from_executable(settings: Value, directory: &Path, executable: &Path) -> Result<Self> {
        let mut remote = Remote::new(executable, WorkerKind::Hecate, settings, directory)?;
        remote.settings = remote.result("settings", Value::Null)?;
        Ok(Self {
            backend: Backend::Remote(Box::new(remote)),
        })
    }
    pub fn settings(&self) -> &Value {
        match &self.backend {
            Backend::Builtin(w) => w.settings(),
            Backend::Remote(r) => &r.settings,
        }
    }
    pub fn try_set_settings(&mut self, settings: Value) -> Result<()> {
        match &mut self.backend {
            Backend::Builtin(w) => w.try_set_settings(settings),
            Backend::Remote(r) => {
                r.settings = r.result("set_settings", json!({"settings":settings}))?;
                Ok(())
            }
        }
    }
    pub fn set_settings(&mut self, settings: Value) {
        match &mut self.backend {
            Backend::Builtin(w) => w.set_settings(settings),
            Backend::Remote(r) => match r.result("set_settings", json!({"settings":settings})) {
                Ok(settings) => r.settings = settings,
                Err(error) => r
                    .errors
                    .lock()
                    .expect("native worker errors")
                    .push(format!("{error:#}")),
            },
        }
    }
    pub fn register_plugin(&mut self, name: &str, target: Value) -> Result<()> {
        match &mut self.backend {
            Backend::Builtin(w) => w.register_plugin(name, target),
            Backend::Remote(r) => r.result("register_plugin", json!({"name":name,"target":target})),
        }
    }
    pub fn respond_web(&self, request: &str, response: Value) -> Result<()> {
        match &self.backend {
            Backend::Builtin(w) => w.respond_web(request, response),
            Backend::Remote(r) => r.result(
                "respond_web",
                json!({"request":request,"response":response}),
            ),
        }
    }
    pub fn begin_replay(&mut self) -> Result<()> {
        match &mut self.backend {
            Backend::Builtin(w) => w.begin_replay(),
            Backend::Remote(r) => r.result("begin_replay", Value::Null),
        }
    }
    pub fn flush(&mut self) -> Result<()> {
        match &mut self.backend {
            Backend::Builtin(w) => w.flush(),
            Backend::Remote(r) => r.result("flush", Value::Null),
        }
    }
    pub fn handle_message(&mut self, command: &str, payload: &Value) -> Result<Option<Value>> {
        match &mut self.backend {
            Backend::Builtin(w) => w.handle_message(command, payload),
            Backend::Remote(r) => r.result(
                "handle_message",
                json!({"command":command,"payload":payload}),
            ),
        }
    }
    pub fn handle_http_response(&mut self, response: &Value) -> Result<()> {
        match &mut self.backend {
            Backend::Builtin(w) => w.handle_http_response(response),
            Backend::Remote(r) => r.result("handle_http_response", json!({"response":response})),
        }
    }
    pub fn evaluator_total(&self, fid: i64) -> Result<i64> {
        match &self.backend {
            Backend::Builtin(w) => w.evaluator_total(fid),
            Backend::Remote(r) => r.result("evaluator_total", json!({"fid":fid})),
        }
    }
    pub fn evaluator_data(&self, fid: i64) -> Result<Vec<Value>> {
        match &self.backend {
            Backend::Builtin(w) => w.evaluator_data(fid),
            Backend::Remote(r) => r.result("evaluator_data", json!({"fid":fid})),
        }
    }
    pub fn bioinsights_total(&self, fid: i64) -> Result<i64> {
        match &self.backend {
            Backend::Builtin(w) => w.bioinsights_total(fid),
            Backend::Remote(r) => r.result("bioinsights_total", json!({"fid":fid})),
        }
    }
    pub fn bioinsights_data(&self, fid: i64) -> Result<Vec<Value>> {
        match &self.backend {
            Backend::Builtin(w) => w.bioinsights_data(fid),
            Backend::Remote(r) => r.result("bioinsights_data", json!({"fid":fid})),
        }
    }
    delegate! {
        fn reset() -> () => Value::Null;
        fn set_batch(batch:bool) -> () => json!({"batch":batch});
        fn poll_bridge() -> () => Value::Null;
        fn on_event(event:&JournalEvent) -> Vec<Finding> => json!({"event":event});
        fn errors() -> Vec<String> => Value::Null;
        fn take_messages() -> Vec<Value> => Value::Null;
        fn take_http_requests() -> Vec<Value> => Value::Null;
    }
    query! {
        fn server_address() -> Option<(String,u16)> => Value::Null;
        fn rows() -> Vec<Value> => Value::Null;
    }
    pub fn on_events(&mut self, events: &[JournalEvent]) -> Vec<Finding> {
        match &mut self.backend {
            Backend::Builtin(w) => events.iter().flat_map(|e| w.on_event(e)).collect(),
            Backend::Remote(r) => r.value("on_events", json!({"events":events})),
        }
    }
    common!();
}

pub struct StellarOverlay {
    backend: Backend<crate::stellar_overlay::StellarOverlay>,
}
impl StellarOverlay {
    pub fn new(settings: Value) -> Self {
        Self {
            backend: Backend::Builtin(Box::new(crate::stellar_overlay::StellarOverlay::new(
                settings,
            ))),
        }
    }
    pub fn from_executable(settings: Value, directory: &Path, executable: &Path) -> Result<Self> {
        let mut remote = Remote::new(executable, WorkerKind::StellarOverlay, settings, directory)?;
        remote.settings = remote.result("settings", Value::Null)?;
        Ok(Self {
            backend: Backend::Remote(Box::new(remote)),
        })
    }
    pub fn settings(&self) -> &Value {
        match &self.backend {
            Backend::Builtin(w) => w.settings(),
            Backend::Remote(r) => &r.settings,
        }
    }
    pub fn set_settings(&mut self, settings: Value) -> Result<()> {
        match &mut self.backend {
            Backend::Builtin(w) => w.set_settings(settings),
            Backend::Remote(r) => {
                r.settings = r.result("set_settings", json!({"settings":settings}))?;
                Ok(())
            }
        }
    }
    pub fn handle_message(
        &mut self,
        sender: &str,
        command: &str,
        payload: &Value,
    ) -> Result<Option<Value>> {
        match &mut self.backend {
            Backend::Builtin(w) => w.handle_message(sender, command, payload),
            Backend::Remote(r) => {
                let result = r.result(
                    "handle_message",
                    json!({"sender":sender,"command":command,"payload":payload}),
                )?;
                r.settings = r.result("settings", Value::Null)?;
                Ok(result)
            }
        }
    }
    delegate! {
        fn reset() -> () => Value::Null;
        fn set_batch(batch:bool) -> () => json!({"batch":batch});
        fn on_event(event:&JournalEvent) -> () => json!({"event":event});
        fn on_status(status:&Value) -> () => json!({"status":status});
        fn on_notification(notification:&Value) -> () => json!({"notification":notification});
        fn on_notification_at(notification:&Value,now:u64) -> () => json!({"notification":notification,"now":now});
        fn cancel(id:&str) -> () => json!({"id":id});
        fn tick() -> () => Value::Null;
        fn tick_at(now:u64) -> () => json!({"now":now});
        fn set_bio_panel(panel:Value) -> () => json!({"panel":panel});
        fn set_evaluator_panel(panel:Value) -> () => json!({"panel":panel});
    }
    query! {
        fn handles_notifications() -> bool => Value::Null;
        fn snapshot() -> crate::stellar_overlay::OverlaySnapshot => Value::Null;
    }
    pub fn on_events(&mut self, events: &[JournalEvent]) {
        match &mut self.backend {
            Backend::Builtin(w) => events.iter().for_each(|e| w.on_event(e)),
            Backend::Remote(r) => r.value("on_events", json!({"events":events})),
        }
    }
    common!();
}
