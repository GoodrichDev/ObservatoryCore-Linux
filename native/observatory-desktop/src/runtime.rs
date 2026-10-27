use crate::{
    extensions::{self, Extension},
    http::HttpClient,
    notifications::Notifications,
    store::Store,
};
use anyhow::{Context, Result};
use observatory_domain::{Finding, JournalEvent};
use observatory_io::{ConfigPaths, JournalTailer, ReplayItem, ScanBatch, Settings};
use observatory_plugins::{
    botanist::{Botanist, BotanistRow, SamplerStatus},
    communicator::{Communicator, MessageRow},
    explorer::Explorer,
    stellar_overlay::OverlaySnapshot,
    worker_rpc::{BioInsights, Evaluator, Hecate, StellarOverlay, WorkerInfo},
};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, VecDeque},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

#[derive(Clone, Default)]
pub struct Snapshot {
    pub repaint: Option<eframe::egui::Context>,
    pub state: String,
    pub monitoring: bool,
    pub total: u64,
    pub last_event: String,
    pub findings: Arc<VecDeque<Finding>>,
    pub finding_grid_times: Arc<VecDeque<Option<String>>>,
    pub botanist: Arc<Vec<BotanistRow>>,
    pub sampler: Option<SamplerStatus>,
    pub messages: Arc<Vec<MessageRow>>,
    pub errors: VecDeque<String>,
    pub extensions: Vec<(String, String, bool)>,
    pub plugin_rows: BTreeMap<String, Arc<Vec<Value>>>,
    pub status: Value,
    pub current_system: String,
    pub plugin_details: BTreeMap<String, Arc<Value>>,
    pub bio_main_view: Arc<Value>,
    /// BioInsights-only main-page invalidation, independent of live summary/status.
    pub bio_revision: u64,
    pub bio_summary: Arc<Value>,
    pub bio_browser_html: Arc<String>,
    pub ui_actions: VecDeque<Value>,
    pub eval_summary: Arc<Value>,
    /// Evaluator-only page invalidation; unrelated workers must not remeasure or scroll it.
    pub evaluator_revision: u64,
    /// Revision of immutable worker projections, separate from live overlay/status.
    pub worker_revision: u64,
    pub overlay: OverlaySnapshot,
    pub read_progress: Option<(String, usize, usize)>,
    pub native_updates: Arc<Value>,
    pub native_popups: Vec<crate::native_popup::Popup>,
}

#[derive(Clone)]
pub struct GridExport {
    pub headers: Vec<String>,
    pub rows: Arc<Vec<Vec<String>>>,
}
pub enum Command {
    Start,
    Stop,
    Replay,
    CancelReplay,
    TestNotification(Settings, u8, bool),
    Configure(Settings),
    Export(PathBuf, u8, Option<String>),
    ExportView(PathBuf, String, Arc<GridExport>),
    Clear(String),
    PluginMessage(String, String, Value),
    Shutdown,
}

pub struct Controller {
    pub tx: Sender<Command>,
    pub snapshot: Arc<Mutex<Snapshot>>,
    pub cancel: Arc<AtomicBool>,
    replay_cancel: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}
impl Controller {
    pub fn spawn(settings: Settings, paths: ConfigPaths) -> Result<Self> {
        let store = Store::open(&paths.data_dir.join("findings.sqlite3"))?;
        let initial = Snapshot {
            state: "Idle".into(),
            ..Default::default()
        };

        drop(store);
        let snapshot = Arc::new(Mutex::new(initial));
        let shared = snapshot.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let token = cancel.clone();
        let replay_cancel = Arc::new(AtomicBool::new(false));
        let replay_token = replay_cancel.clone();
        let (tx, rx) = mpsc::channel();
        let handle = thread::Builder::new()
            .name("observatory-journals".into())
            .spawn(move || {
                if let Err(error) = worker(settings, paths, shared.clone(), token, replay_token, rx)
                {
                    let mut state = shared.lock().unwrap();
                    state.state = "Error".into();
                    state.errors.push_back(format!("{error:#}"));
                }
            })?;
        Ok(Self {
            tx,
            snapshot,
            cancel,
            replay_cancel,
            handle: Some(handle),
        })
    }
    pub fn send(&self, command: Command) {
        if matches!(command, Command::CancelReplay) {
            self.replay_cancel.store(true, Ordering::Relaxed);
            self.cancel.store(true, Ordering::Relaxed);
        }
        if matches!(command, Command::Stop | Command::Shutdown) {
            self.replay_cancel.store(false, Ordering::Relaxed);
            self.cancel.store(true, Ordering::Relaxed);
        }
        let _ = self.tx.send(command);
    }
}
impl Drop for Controller {
    fn drop(&mut self) {
        self.send(Command::Shutdown);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

pub fn default_plugin_settings(settings: &mut Settings) {
    if !settings.plugins.is_object() {
        settings.plugins = json!({});
    }
    if let Some(bio) = settings
        .plugins
        .get_mut("BioInsights")
        .and_then(Value::as_object_mut)
    {
        for obsolete in ["NotifyPredictions", "NotifySamples", "OverlayEnabled"] {
            bio.remove(obsolete);
        }
    }
    for (name, defaults) in [
        (
            "Explorer",
            observatory_plugins::explorer::default_settings(),
        ),
        (
            "Botanist",
            observatory_plugins::botanist::default_settings(),
        ),
        (
            "Communicator",
            observatory_plugins::communicator::default_settings(),
        ),
        (
            "BioInsights",
            observatory_plugins::bioinsights::default_settings(),
        ),
        (
            "Evaluator",
            observatory_plugins::evaluator::default_settings(),
        ),
        ("Hecate", observatory_plugins::hecate::default_settings()),
        (
            "StellarOverlay",
            observatory_plugins::stellar_overlay::default_settings(),
        ),
    ] {
        if !settings.plugins[name].is_object() {
            settings.plugins[name] = defaults;
        } else {
            fill_missing_settings(&mut settings.plugins[name], &defaults);
        }
    }
    settings.plugins["StellarOverlay"] =
        StellarOverlay::new(settings.plugins["StellarOverlay"].clone())
            .settings()
            .clone();
}
fn fill_missing_settings(target: &mut Value, defaults: &Value) {
    if let (Some(target), Some(defaults)) = (target.as_object_mut(), defaults.as_object()) {
        for (key, value) in defaults {
            if let Some(existing) = target.get_mut(key) {
                fill_missing_settings(existing, value);
            } else {
                target.insert(key.clone(), value.clone());
            }
        }
    }
}

/// The expensive tab projections change independently of live position/HUD data.
#[derive(Clone, Copy, Default)]
struct ProjectionDirty {
    findings: bool,
    metadata: bool,
    bio: bool,
    bio_summary: bool,
    evaluator: bool,
    botanist: bool,
    messages: bool,
}
impl ProjectionDirty {
    fn all() -> Self {
        Self {
            metadata: true,
            bio: true,
            bio_summary: true,
            evaluator: true,
            findings: true,
            botanist: true,
            messages: true,
        }
    }
}

fn source_theme_name(settings: &Settings) -> String {
    if settings.theme.eq_ignore_ascii_case("dark") {
        "Dark".into()
    } else if settings.theme.eq_ignore_ascii_case("light") {
        "Light".into()
    } else {
        settings.theme.clone()
    }
}
struct Engine {
    explorer: Explorer,
    botanist: Botanist,
    communicator: Communicator,
    bioinsights: BioInsights,
    evaluator: Evaluator,
    hecate: Hecate,
    overlay: StellarOverlay,
    extensions: Vec<Extension>,
    settings: Settings,
    paths: ConfigPaths,
    store: Store,
    notifications: Notifications,
    native_popups: crate::native_popup::Manager,
    findings: VecDeque<Finding>,
    finding_grid_times: VecDeque<Option<String>>,
    bio_browser_enabled: bool,
    processed: u64,
    last_event: String,
    status_dirty: bool,
    updates: crate::native_updates::Updates,
    http: HttpClient,
    snapshot: Arc<Mutex<Snapshot>>,
    batch: bool,
    projections_dirty: ProjectionDirty,
    dispatch_depth: usize,
    dispatch_budget: usize,
    native_pending: VecDeque<(String, Value)>,
    hidden_rows: BTreeMap<String, std::collections::HashSet<String>>,
}
fn load_worker<T>(
    paths: &ConfigPaths,
    name: &str,
    builtin: impl FnOnce() -> Result<T>,
    external: impl FnOnce(&std::path::Path) -> Result<T>,
    info: fn(&T) -> Option<WorkerInfo>,
    errors: &mut Vec<String>,
) -> Result<T> {
    match crate::native_updates::worker_path(paths, name) {
        Ok(Some(path)) => {
            match external(&path).and_then(|worker|{let actual=info(&worker).context("Native worker did not report its version")?;let expected=crate::native_updates::installed(paths,name)?.context("Native worker metadata disappeared")?;anyhow::ensure!(actual.source_version==expected.artifact.version,"Native worker source version differs from installed metadata");Ok(worker)}){Ok(worker)=>return Ok(worker),Err(error)=>errors.push(format!("{name} installed native worker could not start; using bundled worker: {error:#}"))}
        }
        Ok(None) => {}
        Err(error) => errors.push(format!(
            "{name} installed native worker is invalid; using bundled worker: {error:#}"
        )),
    }
    builtin()
}
impl Engine {
    fn new(
        mut settings: Settings,
        paths: ConfigPaths,
        snapshot: Arc<Mutex<Snapshot>>,
    ) -> Result<Self> {
        default_plugin_settings(&mut settings);
        let explorer = Explorer::new(settings.plugins["Explorer"].clone(), None)?;
        let botanist = Botanist::new(settings.plugins["Botanist"].clone());
        let communicator = Communicator::new(settings.plugins["Communicator"].clone());
        let mut startup_errors = Vec::new();
        let mut bioinsights = load_worker(
            &paths,
            "BioInsights",
            || BioInsights::new(settings.plugins["BioInsights"].clone()),
            |exe| {
                BioInsights::from_executable(
                    settings.plugins["BioInsights"].clone(),
                    &paths.data_dir.join("BioInsights"),
                    exe,
                )
            },
            BioInsights::worker_info,
            &mut startup_errors,
        )?;
        let bio_state = paths.data_dir.join("BioInsights/state.json");
        if bio_state.is_file() {
            bioinsights.restore_state(&serde_json::from_reader(std::fs::File::open(&bio_state)?)?);
        }
        let evaluator = load_worker(
            &paths,
            "Evaluator",
            || Ok(Evaluator::new(settings.plugins["Evaluator"].clone())),
            |exe| {
                Evaluator::from_executable(
                    settings.plugins["Evaluator"].clone(),
                    &paths.data_dir.join("Evaluator"),
                    exe,
                )
            },
            Evaluator::worker_info,
            &mut startup_errors,
        )?;
        let mut hecate_settings = settings.plugins["Hecate"].clone();
        if settings
            .extra
            .get("disabled_plugins")
            .and_then(Value::as_array)
            .is_some_and(|plugins| plugins.iter().any(|name| name.as_str() == Some("Hecate")))
        {
            hecate_settings["ServerSettings"] = json!({"EnableWebServer":false});
        }
        let hecate = load_worker(
            &paths,
            "Hecate",
            || Hecate::open(hecate_settings.clone(), &paths.data_dir.join("Hecate")),
            |exe| {
                Hecate::from_executable(
                    hecate_settings.clone(),
                    &paths.data_dir.join("Hecate"),
                    exe,
                )
            },
            Hecate::worker_info,
            &mut startup_errors,
        )?;
        let overlay = load_worker(
            &paths,
            "StellarOverlay",
            || {
                Ok(StellarOverlay::new(
                    settings.plugins["StellarOverlay"].clone(),
                ))
            },
            |exe| {
                StellarOverlay::from_executable(
                    settings.plugins["StellarOverlay"].clone(),
                    &paths.data_dir.join("StellarOverlay"),
                    exe,
                )
            },
            StellarOverlay::worker_info,
            &mut startup_errors,
        )?;
        let store = Store::open(&paths.data_dir.join("findings.sqlite3"))?;
        let (extensions, errors) = extensions::discover_with_context(
            &paths.plugin_dir,
            &settings.plugins,
            &paths.data_dir,
            &json!({"theme_name":source_theme_name(&settings),"theme_details":crate::core_theme::details(&settings),"log_monitor_state":0,"status":{},"cargo":{},"updated_plugins_folder":paths.data_dir.join("native-plugins")}),
        );
        let findings = VecDeque::new();
        let bio_browser_enabled = settings
            .extra
            .get("bioinsights_renderer")
            .and_then(Value::as_str)
            == Some("webkit")
            && !settings.plugins["BioInsights"]["NativeOutput"]
                .as_str()
                .unwrap_or(" Enabled")
                .starts_with("Disabled");
        let mut engine = Self {
            findings,
            finding_grid_times: VecDeque::new(),
            bio_browser_enabled,
            processed: 0,
            last_event: String::new(),
            status_dirty: true,
            explorer,
            botanist,
            communicator,
            bioinsights,
            evaluator,
            hecate,
            overlay,
            extensions,
            updates: crate::native_updates::Updates::new(&paths),
            settings,
            paths,
            store,
            notifications: Notifications::new(),
            native_popups: Default::default(),
            http: HttpClient::new(),
            snapshot,
            batch: false,
            projections_dirty: ProjectionDirty::all(),
            dispatch_depth: 0,
            dispatch_budget: 256,
            native_pending: VecDeque::new(),
            hidden_rows: BTreeMap::new(),
        };
        engine.native_popups.set_settings(&engine.settings);
        engine.notifications.set_settings(&engine.settings);
        for error in errors.into_iter().chain(startup_errors) {
            engine.error(error);
        }
        for plugin in &engine.extensions {
            if engine.enabled(&plugin.name) {
                engine
                    .hecate
                    .register_plugin(&plugin.name, json!(plugin.id))?;
            }
        }
        let peers = [
            ("BioInsights", "1.3.444.1052"),
            ("Evaluator", "1.3.444.637"),
            ("Hecate", "1.3.444.761"),
            ("StellarOverlay", "1.3.228.387"),
        ];
        let mut ready = Vec::new();
        for (sender, version) in peers {
            if !engine.enabled(sender) {
                continue;
            }
            for (target, _) in peers {
                if sender != target && engine.enabled(target) {
                    ready.push((
                        sender.to_owned(),
                        json!({"target":target,"command":"Ready","payload":{"Version":version}}),
                    ));
                }
            }
        }
        engine.native_messages(ready);
        for name in ["BioInsights", "Evaluator", "Hecate", "StellarOverlay"] {
            let external = match name {
                "BioInsights" => engine.bioinsights.is_external(),
                "Evaluator" => engine.evaluator.is_external(),
                "Hecate" => engine.hecate.is_external(),
                _ => engine.overlay.is_external(),
            };
            if external {
                match crate::native_updates::mark_activated(&engine.paths,name){Ok(true) if matches!(name,"BioInsights"|"Evaluator")=>engine.native_messages(vec![("Core".into(),json!({"target":name,"command":"SourceUIAlert","payload":{"Type":"UpdateInstalled","Active":true}}))]),Ok(_)=>{},Err(error)=>engine.error(format!("{name} activation receipt: {error:#}"))}
            }
        }

        engine.updates.configure(&engine.settings);
        engine.explorer_startup();
        Ok(engine)
    }
    fn enabled(&self, name: &str) -> bool {
        !self
            .settings
            .extra
            .get("disabled_plugins")
            .and_then(Value::as_array)
            .is_some_and(|list| list.iter().any(|v| v.as_str() == Some(name)))
    }
    /// Settings change the existing workers. Recreating them discarded live
    /// samples, commander histories and queued notifications on every Save.
    fn configure(&mut self, mut settings: Settings) -> Result<()> {
        default_plugin_settings(&mut settings);
        let changed = |name: &str| settings.plugins[name] != self.settings.plugins[name];
        let prepared_explorer = if changed("Explorer") {
            Some(Explorer::prepare_preferences(
                settings.plugins["Explorer"].clone(),
            )?)
        } else {
            None
        };
        if changed("BioInsights") {
            let _ = BioInsights::new(settings.plugins["BioInsights"].clone())?;
        }
        for plugin in &self.extensions {
            if let Some(value) = settings.plugins.get(&plugin.name) {
                anyhow::ensure!(
                    value.is_object() && serde_json::to_vec(value)?.len() <= 4 * 1024 * 1024,
                    "Invalid {} settings",
                    plugin.name
                );
            }
        }
        let disabled_changed =
            settings.extra.get("disabled_plugins") != self.settings.extra.get("disabled_plugins");
        let hecate_changed = changed("Hecate") || disabled_changed;
        let previous = self.hecate.settings().clone();
        if hecate_changed {
            let mut hecate = settings.plugins["Hecate"].clone();
            if settings
                .extra
                .get("disabled_plugins")
                .and_then(Value::as_array)
                .is_some_and(|list| list.iter().any(|v| v == "Hecate"))
            {
                hecate["ServerSettings"]["EnableWebServer"] = json!(false);
            }
            self.hecate.try_set_settings(hecate)?;
        }
        if let Err(error) = settings.save(&self.paths.settings_file) {
            if hecate_changed {
                self.hecate.try_set_settings(previous)?;
            }
            return Err(error.context("save configuration"));
        }
        if let Some(prepared) = prepared_explorer {
            self.explorer.apply_preferences(prepared);
        }
        if changed("Botanist") {
            self.botanist
                .set_settings(settings.plugins["Botanist"].clone());
            self.projections_dirty.botanist = true;
        }
        if changed("Communicator") {
            self.communicator
                .set_settings(settings.plugins["Communicator"].clone());
            self.projections_dirty.messages = true;
        }
        if changed("BioInsights") {
            self.bioinsights
                .set_settings(settings.plugins["BioInsights"].clone())?;
            self.projections_dirty.bio = true;
            self.projections_dirty.bio_summary = true;
        }
        if changed("Evaluator") {
            self.evaluator
                .set_settings(settings.plugins["Evaluator"].clone())?;
            self.projections_dirty.evaluator = true;
        }
        if changed("StellarOverlay") {
            self.overlay
                .set_settings(settings.plugins["StellarOverlay"].clone())?;
        }
        if disabled_changed {
            self.projections_dirty = ProjectionDirty::all();
        }
        if settings.plugins != self.settings.plugins {
            self.projections_dirty.metadata = true;
        }
        for plugin in &mut self.extensions {
            if let Some(value) = settings.plugins.get(&plugin.name) {
                if self.settings.plugins.get(&plugin.name) != Some(value) {
                    plugin.set_settings(value.clone())?;
                }
            }
        }
        self.native_popups.set_settings(&settings);
        self.notifications.set_settings(&settings);
        self.updates.configure(&settings);
        let changed_theme = self.settings.theme != settings.theme
            || crate::core_theme::details(&self.settings) != crate::core_theme::details(&settings);
        if changed_theme {
            let details = crate::core_theme::details(&settings);
            for plugin in &mut self.extensions {
                if let Err(error) = plugin.theme_changed(&source_theme_name(&settings), &details) {
                    self.snapshot
                        .lock()
                        .unwrap()
                        .errors
                        .push_back(format!("{} theme: {error:#}", plugin.name));
                }
            }
        }
        self.settings = settings;

        Ok(())
    }
    fn error(&self, message: String) {
        use std::io::Write;
        let path = self.paths.data_dir.join("errors.log");
        if std::fs::metadata(&path).is_ok_and(|m| m.len() > 2 * 1024 * 1024) {
            let _ = std::fs::rename(&path, self.paths.data_dir.join("errors.previous.log"));
        }
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(file, "{message}");
        }
        eprintln!("{message}");
        let mut state = self.snapshot.lock().unwrap();
        state.errors.push_back(message);
        while state.errors.len() > 100 {
            state.errors.pop_front();
        }
        let repaint = state.repaint.clone();
        drop(state);
        if let Some(ctx) = repaint {
            ctx.request_repaint();
        }
    }
    fn state(&mut self, state: &str, batch: bool) {
        self.projections_dirty = ProjectionDirty::all();
        self.batch = batch;
        let realtime = {
            let mut snapshot = self.snapshot.lock().unwrap();
            let realtime = if batch || state == "Read all cancelled" {
                snapshot.monitoring
            } else {
                state == "Monitoring"
            };
            snapshot.monitoring = realtime;
            realtime
        };
        let flags = u32::from(realtime)
            | if batch {
                if state == "Preparing current system" {
                    4
                } else {
                    2
                }
            } else if state == "Read all cancelled" {
                8
            } else {
                0
            };
        for plugin in &self.extensions {
            if let Err(error) = plugin.set_context("log_monitor_state", &json!(flags)) {
                self.error(error.to_string());
            }
        }
        self.explorer
            .set_monitor_state(batch, batch && state != "Preparing current system");
        self.explorer_startup();
        self.botanist.set_batch(batch);
        self.communicator.set_batch(batch);
        self.bioinsights.set_batch(batch);
        self.evaluator
            .set_preread(state == "Preparing current system");
        self.evaluator.set_batch(batch);
        self.hecate.set_batch(batch);
        self.overlay.set_batch(batch);
        self.sync_sampler();
        self.snapshot.lock().unwrap().state = state.into();
        self.call_plugins("on_state_changed", &json!({"state":state,"batch":batch}));
    }
    fn finish_read(&mut self, monitoring: bool) {
        self.state(if monitoring { "Monitoring" } else { "Idle" }, false);
        // Replay publishes full chunks while reading. Keep progress visible
        // until the final partial chunk and state callbacks are published, so
        // they cannot appear to arrive later from the resumed live monitor.
        self.publish();
        self.snapshot.lock().unwrap().read_progress = None;
    }
    fn invalidate_event(&mut self, event: &str) {
        // These are the journal branches which mutate the corresponding workers'
        // visible data. Mining/inventory/heartbeat records do not rebuild tabs.
        let dirty = &mut self.projections_dirty;
        if matches!(
            event,
            "LoadGame"
                | "Commander"
                | "Location"
                | "FSDJump"
                | "CarrierJump"
                | "StartJump"
                | "Scan"
                | "ScanBaryCentre"
                | "FSSBodySignals"
                | "SAASignalsFound"
                | "SAAScanComplete"
                | "FSSAllBodiesFound"
                | "SupercruiseExit"
                | "ApproachBody"
                | "Touchdown"
                | "Disembark"
                | "LeaveBody"
                | "SupercruiseEntry"
                | "ScanOrganic"
                | "CodexEntry"
                | "SellOrganicData"
                | "Died"
                | "Shutdown"
        ) {
            dirty.bio = true;
        }
        if matches!(
            event,
            "LoadGame"
                | "Commander"
                | "Location"
                | "FSDJump"
                | "CarrierJump"
                | "ScanOrganic"
                | "CodexEntry"
                | "SellOrganicData"
                | "Died"
                | "Disembark"
                | "Touchdown"
        ) {
            dirty.bio_summary = true;
        }
        if matches!(
            event,
            "LoadGame"
                | "Location"
                | "FSDJump"
                | "CarrierJump"
                | "StartJump"
                | "Shutdown"
                | "Scan"
                | "ScanBaryCentre"
                | "FSSDiscoveryScan"
                | "FSSAllBodiesFound"
                | "FSSBodySignals"
                | "SAASignalsFound"
                | "SAAScanComplete"
                | "ApproachBody"
                | "SupercruiseExit"
                | "ScanOrganic"
                | "Disembark"
                | "SellExplorationData"
                | "MultiSellExplorationData"
                | "Died"
        ) {
            dirty.evaluator = true;
        }
        dirty.botanist |= matches!(
            event,
            "LoadGame"
                | "FSDJump"
                | "Location"
                | "CarrierJump"
                | "ApproachBody"
                | "Touchdown"
                | "Scan"
                | "SAASignalsFound"
                | "ScanOrganic"
                | "LeaveBody"
                | "Shutdown"
        );
        dirty.messages |= matches!(event, "SendText" | "ReceiveText");
    }
    fn process(&mut self, event: JournalEvent) {
        self.dispatch_event(event, true);
    }
    fn dispatch_event(&mut self, mut event: JournalEvent, counted: bool) {
        // LogMonitor sets these before JournalEntry callbacks. A callback may
        // parse/replay another event; the outer event must not overwrite it.
        if counted {
            self.processed += 1;
            self.last_event = if event.event == "JournalBase" {
                "JournalBase".into()
            } else {
                event.data["event"]
                    .as_str()
                    .unwrap_or(&event.event)
                    .to_owned()
            };
        }
        self.invalidate_event(&event.event);
        let previous_system = self.snapshot.lock().unwrap().current_system.clone();
        if matches!(event.event.as_str(), "Location" | "FSDJump" | "CarrierJump")
            && !event.str("StarSystem").is_empty()
        {
            self.snapshot.lock().unwrap().current_system = event.str("StarSystem").to_owned();
        }
        let mut findings = Vec::new();
        if self.enabled("Explorer") {
            let changed_system =
                matches!(event.event.as_str(), "Location" | "FSDJump" | "CarrierJump")
                    && (!matches!(event.event.as_str(), "CarrierJump") || event.boolean("Docked"))
                    && previous_system != event.str("StarSystem");
            if changed_system
                && !self.batch
                && self.settings.plugins["Explorer"]["OnlyShowCurrentSystem"] == true
            {
                self.clear("Explorer");
            }
            findings.extend(self.explorer.on_event(&event));
            if let Some(snapshot) = self.explorer.take_event_snapshot() {
                event.timestamp = snapshot["timestamp"]
                    .as_str()
                    .unwrap_or(&event.timestamp)
                    .into();
                event.data = snapshot;
            }
        }
        if self.enabled("Botanist") {
            self.botanist.on_event(&event);
            self.sync_sampler();
        }
        if self.enabled("Communicator") {
            findings.extend(self.communicator.on_event(&event));
        }
        if self.enabled("Hecate") {
            findings.extend(self.hecate.on_event(&event));
        }
        if self.enabled("BioInsights") {
            findings.extend(self.bioinsights.on_event(&event));
        }
        if event.event == "CodexEntry" {
            if let Err(error) = self.save_bio_state() {
                self.error(format!("BioInsights state: {error:#}"));
            }
        }
        if self.enabled("Evaluator") {
            findings.extend(self.evaluator.on_event(&event));
        }
        if self.enabled("StellarOverlay") {
            self.overlay.on_event(&event);
        }
        self.native_messages(Vec::new());
        for value in self.bioinsights.take_diagnostics() {
            if let Err(error) = self.append_plugin_log("BioInsights", &value) {
                self.error(format!("BioInsights log: {error:#}"));
            }
        }
        self.explorer_diagnostics();
        self.call_plugins("on_journal", &extensions::event_value(&event));
        for mut finding in findings.drain(..) {
            if finding.timestamp.is_empty() {
                finding.timestamp = event.timestamp.clone();
            }
            self.record(finding);
        }
    }
    fn record(&mut self, finding: Finding) {
        let rendering = if finding.plugin == "BioInsights" {
            self.bioinsights
                .take_notification_rendering(&finding)
                .unwrap_or(0)
        } else if finding.plugin == "Evaluator" {
            self.evaluator
                .take_notification_rendering(&finding)
                .unwrap_or(0)
        } else if finding.plugin == "Communicator" {
            self.communicator.notification_rendering()
        } else {
            7
        };
        let mut value = if finding.plugin == "Evaluator" {
            self.evaluator.take_notification_payload(&finding)
        } else if finding.plugin == "Explorer" {
            self.explorer.take_notification_payload(&finding)
        } else {
            None
        }
        .unwrap_or_else(|| serde_json::to_value(&finding).unwrap_or(Value::Null));
        value["rendering"] = json!(rendering);
        let value = crate::core_notification::normalize(value);
        self.record_with_payload(finding, rendering, Some(value.clone()));
    }
    fn record_with_notification(&mut self, finding: Finding, notify: bool) {
        if notify {
            self.record_with_rendering(finding, 7);
        } else {
            self.append_finding(finding, None);
        }
    }
    fn record_with_rendering(&mut self, finding: Finding, rendering: u8) {
        self.record_with_payload(finding, rendering, None);
    }
    fn record_with_payload(&mut self, finding: Finding, rendering: u8, payload: Option<Value>) {
        let mut value =
            payload.unwrap_or_else(|| serde_json::to_value(&finding).unwrap_or(Value::Null));
        value["rendering"] = json!(rendering);
        self.append_finding(
            finding,
            value
                .get("grid_time")
                .and_then(Value::as_str)
                .map(str::to_owned),
        );
        self.dispatch_core_notification(value);
    }
    fn append_finding(&mut self, finding: Finding, grid_time: Option<String>) {
        if let Err(error) = self.store.insert(&finding) {
            self.error(format!("persist result: {error:#}"));
        }
        self.findings.push_back(finding);
        self.finding_grid_times.push_back(grid_time);
        self.projections_dirty.findings = true;
    }
    fn explorer_startup(&mut self) {
        if self.enabled("Explorer") {
            for finding in self.explorer.take_startup_findings() {
                self.record(finding);
            }
        }
        self.explorer_diagnostics();
    }
    fn explorer_diagnostics(&mut self) {
        for value in self.explorer.take_diagnostics() {
            if let Err(error) = self.append_plugin_log("Explorer", &value) {
                self.error(format!("Explorer log: {error:#}"));
            }
        }
    }
    fn dispatch_core_notification(&mut self, value: Value) -> Value {
        let value = crate::core_notification::normalize(value);
        let rendering = value.get("rendering").and_then(Value::as_u64).unwrap_or(7) as u8;
        let action = value["action"].as_str().unwrap_or("show");
        let popup_override = self.enabled("StellarOverlay") && self.overlay.handles_notifications()
            || self
                .extensions
                .iter()
                .any(|p| p.enabled && self.enabled(&p.name) && p.override_popup);
        let audio_override = self
            .extensions
            .iter()
            .any(|p| p.enabled && self.enabled(&p.name) && p.override_audio);
        let route = crate::core_notification::delivery(
            action,
            rendering,
            self.batch,
            self.settings.popup,
            self.settings.speech,
            popup_override,
            audio_override,
        );
        if route.notification {
            let mut show = value.clone();
            show["action"] = json!("show");
            if self.enabled("StellarOverlay") {
                self.overlay.on_notification(&show);
            }
            self.call_plugins("on_notification", &show);
        }
        if route.visual || route.audio {
            let mut settings = self.settings.clone();
            settings.popup = route.visual;
            settings.speech = route.audio;
            if !route.audio {
                settings.extra.remove("chime_file");
            }
            self.native_lifecycle(value.clone(), settings);
        }
        if route.update || route.cancel {
            if route.cancel || self.enabled("StellarOverlay") {
                self.overlay.on_notification(&value);
            }
            self.call_plugins(
                if route.update {
                    "on_notification_update"
                } else {
                    "on_notification_cancel"
                },
                &value,
            );
        }
        value
    }
    fn notification_settings(&self) -> Settings {
        let mut settings = self.settings.clone();
        if self.enabled("StellarOverlay") && self.overlay.handles_notifications() {
            settings.popup = false;
        }
        for plugin in &self.extensions {
            if plugin.enabled && self.enabled(&plugin.name) {
                if plugin.override_popup {
                    settings.popup = false;
                }
                if plugin.override_audio {
                    settings.speech = false;
                    settings.extra.remove("chime_file");
                }
            }
        }
        settings
    }
    #[cfg(test)]
    fn notification_settings_for(&self, rendering: u8) -> Settings {
        let mut settings = self.notification_settings();
        settings.popup &= rendering & 1 != 0;
        settings.speech &= rendering & 2 != 0;
        if rendering & 2 == 0 {
            settings.extra.remove("chime_file");
        }
        settings
    }
    fn append_plugin_log(&self, plugin: &str, value: &Value) -> Result<()> {
        use std::io::Write;
        anyhow::ensure!(
            matches!(
                plugin,
                "Explorer" | "BioInsights" | "Evaluator" | "Hecate" | "StellarOverlay"
            ),
            "Unknown native log owner"
        );
        let folder = self.paths.data_dir.join(plugin);
        std::fs::create_dir_all(&folder)?;
        let basename = if plugin == "StellarOverlay" {
            "Overlay"
        } else {
            plugin
        };
        let file = folder.join(format!("{basename}.log"));
        if std::fs::metadata(&file).is_ok_and(|m| m.len() > 16 * 1024 * 1024) {
            let previous = folder.join(format!("{basename}.previous.log"));
            std::fs::rename(&file, previous)?;
        }
        let mut output = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file)?;
        writeln!(output, "{}", serde_json::to_string(value)?)?;
        Ok(())
    }
    fn native_lifecycle(&mut self, value: Value, mut settings: Settings) {
        let value = crate::core_notification::normalize(value);
        if settings
            .extra
            .get("notification_backend")
            .and_then(Value::as_str)
            != Some("dbus")
        {
            if settings.popup || value["action"] == "cancel" {
                if let Err(error) = self.native_popups.lifecycle(
                    &value,
                    &settings,
                    observatory_plugins::stellar_overlay::now_ms(),
                ) {
                    self.error(format!("native notification: {error:#}"));
                }
            }
            settings.popup = false;
        }
        self.notifications.lifecycle(value, settings);
    }
    fn sync_sampler(&mut self) {
        for value in self.botanist.take_notification_events() {
            self.dispatch_core_notification(value);
        }
    }
    fn call_plugins(&mut self, handler: &str, value: &Value) {
        if self.dispatch_depth == 0 {
            self.dispatch_budget = 256;
        }
        if self.dispatch_depth >= 8 || self.dispatch_budget == 0 {
            if self.dispatch_budget > 0 {
                self.dispatch_budget = 0;
                self.error("Plugin callback cycle exceeded the host dispatch budget".into());
            }
            return;
        }
        self.dispatch_depth += 1;
        self.dispatch_budget -= 1;
        let mut findings = Vec::new();
        let mut errors = Vec::new();
        let mut messages = Vec::new();
        let mut saved = false;
        let mut notifications = Vec::new();
        let mut http_requests = Vec::new();
        for index in 0..self.extensions.len() {
            let name = self.extensions[index].name.clone();
            if !self.enabled(&name) && handler != "on_notification_cancel" {
                continue;
            }
            let plugin = &mut self.extensions[index];
            if matches!(handler, "on_message" | "on_legacy_message") {
                if value["__recipient"]
                    .as_str()
                    .is_some_and(|recipient| recipient != plugin.id)
                {
                    continue;
                }
                let sender_id = value["source"]["id"].as_str().unwrap_or("");
                let sender_name = value["source"]["name"].as_str().unwrap_or("");
                if sender_id == plugin.id || sender_name == plugin.name {
                    continue;
                }
                let target = value["data"]["target"]
                    .as_str()
                    .or_else(|| value["data"]["Target"].as_str())
                    .unwrap_or("");
                if !target.is_empty() && target != plugin.id && target != plugin.name {
                    continue;
                }
            }
            if handler == "on_audio_complete"
                && value["plugin"].as_str() != Some(plugin.name.as_str())
                && value["plugin"].as_str() != Some(plugin.id.as_str())
            {
                continue;
            }
            if handler == "on_http_response"
                && value["__recipient"].as_str() != Some(plugin.id.as_str())
            {
                continue;
            }
            if matches!(handler, "on_notification" | "on_notification_update") {
                let rendering = value["rendering"].as_u64().unwrap_or(7);
                let receives = rendering & 4 != 0
                    || plugin.override_popup && rendering & 1 != 0
                    || plugin.override_audio && rendering & 2 != 0;
                if self.batch && !plugin.accept_batch_notifications || !receives {
                    continue;
                }
            }
            if let Err(error) = plugin.call(handler, value) {
                errors.push(format!("{error:#}"));
                plugin.enabled = false;
                self.projections_dirty.metadata = true;
            }
            let output = plugin.take_output();
            http_requests.extend(
                output
                    .http_requests
                    .into_iter()
                    .map(|value| (plugin.id.clone(), value)),
            );
            findings.extend(output.findings);
            messages.extend(output.messages);
            notifications.extend(
                output
                    .notifications
                    .into_iter()
                    .map(|value| (name.clone(), value)),
            );
            for log in output.logs {
                errors.push(format!("{name}: {log}"));
            }
            if let Some(settings) = output.settings {
                self.settings.plugins[&name] = settings;
                self.projections_dirty.metadata = true;
                saved = true;
            }
            if output.clear_rows || !output.rows.is_empty() {
                // Cloning a large extension table is worker work, not UI-lock
                // work. Existing frames retain the old immutable allocation.
                let previous = self
                    .snapshot
                    .lock()
                    .unwrap()
                    .plugin_rows
                    .get(&name)
                    .cloned();
                let mut rows = if output.clear_rows {
                    Vec::new()
                } else {
                    previous.as_deref().cloned().unwrap_or_default()
                };
                rows.extend(output.rows);
                let retired = {
                    let mut snapshot = self.snapshot.lock().unwrap();
                    snapshot.worker_revision = snapshot.worker_revision.wrapping_add(1);
                    snapshot.plugin_rows.insert(name, Arc::new(rows))
                };
                drop(retired);
            }
        }
        // Replies can themselves emit messages; the outer dispatch depth and
        // budget bound the entire protocol, including notification round trips.
        for message in messages.into_iter().take(100) {
            let sender = message["source"]["name"]
                .as_str()
                .or_else(|| message["source"]["id"].as_str())
                .unwrap_or("Extension")
                .to_owned();
            if message["data"]
                .get("command")
                .or_else(|| message["data"].get("Command"))
                .is_some()
            {
                // Typed/legacy source messages have one routing path. Calling
                // on_message again here used to deliver them twice.
                self.native_messages(vec![(sender, message["data"].clone())]);
            } else {
                self.call_plugins("on_message", &message);
            }
        }
        for error in errors {
            self.error(error);
        }
        for finding in findings {
            if handler == "on_notification" {
                self.record_with_notification(finding, true);
            } else {
                self.record(finding);
            }
        }
        for (name, value) in notifications {
            let value = crate::core_notification::normalize(value);
            if value["action"].as_str() == Some("show") {
                let finding = Finding {
                    plugin: name,
                    timestamp: value["timestamp"].as_str().unwrap_or("").into(),
                    system: value["system"].as_str().unwrap_or("").into(),
                    body: value["body"].as_str().unwrap_or("").into(),
                    title: value["title"].as_str().unwrap_or("").into(),
                    detail: value["detail"].as_str().unwrap_or("").into(),
                };
                self.record_with_notification(finding, false);
            }
            self.dispatch_core_notification(value);
        }
        for (plugin_id, request) in http_requests {
            if let Err(error) = self.http.submit(plugin_id.clone(), request.clone()) {
                let response = json!({"__recipient":plugin_id,"request_id":request["request_id"],"ok":false,"status":null,"error":error});
                self.call_plugins("on_http_response", &response);
            }
        }
        if saved {
            if let Err(error) = self.settings.save(&self.paths.settings_file) {
                self.error(format!("save plugin settings: {error:#}"));
            }
        }
        self.dispatch_depth -= 1;
    }
    fn deserialize_ancillary(&mut self, event_type: &str) {
        if self.batch
            || !matches!(
                event_type,
                "Cargo"
                    | "NavRoute"
                    | "Market"
                    | "Outfitting"
                    | "Shipyard"
                    | "Backpack"
                    | "FCMaterials"
                    | "ModuleInfo"
                    | "ShipLocker"
            )
        {
            return;
        }
        let Ok(directory) =
            observatory_io::resolve_journal_dir(self.settings.journal_dir.as_deref())
        else {
            return;
        };
        let file = if event_type == "ModuleInfo" {
            "ModulesInfo"
        } else {
            event_type
        };
        let filename = format!("{file}.json");
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(50));
            let result = (|| -> Result<JournalEvent> {
                let raw = std::fs::read_to_string(directory.join(&filename))?;
                let data: Value = serde_json::from_str(&raw)?;
                anyhow::ensure!(data.is_object(), "Ancillary JSON must be an object");
                Ok(JournalEvent {
                    event: format!("{event_type}File"),
                    timestamp: data["timestamp"].as_str().unwrap_or("").to_owned(),
                    data,
                    raw_json: Some(raw.into()),
                })
            })();
            if let Ok(event) = result {
                let data = event.data.clone();
                self.dispatch_event(event, false);
                self.call_plugins("on_auxiliary", &json!({"name":filename,"data":data}));
                break;
            }
        }
    }
    fn message_sender(&self, sender: &str, legacy: bool) -> Value {
        if let Some(plugin) = self
            .extensions
            .iter()
            .find(|plugin| plugin.name == sender || plugin.id == sender)
        {
            return json!({"id":if legacy{"00000000-0000-0000-0000-000000000000"}else{&plugin.id},"name":plugin.name,"full_name":plugin.name,"short_name":plugin.name,"version":plugin.version});
        }
        let (id, version) = match sender {
            "BioInsights" => (
                "4d617474-4701-4249-4f49-4e5349474854",
                self.bioinsights.worker_info(),
            ),
            "Evaluator" => (
                "4d617474-4702-0045-5641-4c5541544f52",
                self.evaluator.worker_info(),
            ),
            "Hecate" => (
                "4d617474-4707-0000-0000-484543415445",
                self.hecate.worker_info(),
            ),
            "StellarOverlay" => (
                "4d617474-4703-5354-454c-4c41524f5645",
                self.overlay.worker_info(),
            ),
            _ => ("00000000-0000-0000-0000-000000000000", None),
        };
        let full = crate::about::metadata(sender)["FullName"]
            .as_str()
            .unwrap_or(sender)
            .to_owned();
        json!({"id":if legacy{"00000000-0000-0000-0000-000000000000"}else{id},"name":sender,"full_name":full,"short_name":if legacy{full.as_str()}else{sender},"version":version.map(|info|info.source_version)})
    }
    /// Route original plugin command envelopes on the journal worker. Replies
    /// retain their recipient, and one bounded queue covers native round trips.
    fn native_messages(&mut self, initial: Vec<(String, Value)>) {
        let mut queue = std::mem::take(&mut self.native_pending);
        queue.extend(initial);
        let mut remaining = 256usize;
        loop {
            if self.enabled("BioInsights") {
                queue.extend(
                    self.bioinsights
                        .take_messages()
                        .into_iter()
                        .map(|m| ("BioInsights".into(), m)),
                );
            }
            if self.enabled("Hecate") {
                queue.extend(
                    self.hecate
                        .take_messages()
                        .into_iter()
                        .map(|m| ("Hecate".into(), m)),
                );
            }
            if self.enabled("Evaluator") {
                queue.extend(
                    self.evaluator
                        .take_messages()
                        .into_iter()
                        .map(|m| ("Evaluator".into(), m)),
                );
            }
            let Some((sender, message)) = queue.pop_front() else {
                break;
            };
            if remaining == 0 {
                queue.push_front((sender, message));
                break;
            }
            remaining -= 1;
            let command = message["command"]
                .as_str()
                .or_else(|| message["Command"].as_str())
                .unwrap_or("")
                .to_owned();
            if command.is_empty() {
                continue;
            }
            let raw_target = message["target"]
                .as_str()
                .or_else(|| message["Target"].as_str())
                .unwrap_or("");
            let target = match raw_target.to_ascii_lowercase().as_str() {
                "00000000-0000-0000-0000-000000000000" => "",
                "4d617474-4701-4249-4f49-4e5349474854" => "BioInsights",
                "4d617474-4702-0045-5641-4c5541544f52" => "Evaluator",
                "4d617474-4703-5354-454c-4c41524f5645" => "StellarOverlay",
                "4d617474-4707-0000-0000-484543415445" => "Hecate",
                _ => raw_target,
            }
            .to_owned();
            if target != "Core" && !target.is_empty() && target == sender {
                continue;
            }
            let mut payload = message
                .get("payload")
                .or_else(|| message.get("Payload"))
                .cloned()
                .unwrap_or_else(|| json!({}));
            if target == "Core" {
                match command.as_str() {
                    "OpenSettings" | "OpenAbout" | "FocusPlugin" | "DisplayMessageBox" => {
                        let mut snapshot = self.snapshot.lock().unwrap();
                        if snapshot.ui_actions.len() < 128 {
                            snapshot
                                .ui_actions
                                .push_back(json!({"CoreCommand":command,"Payload":payload}));
                        }
                    }
                    "PluginError" => {
                        let version = self
                            .extensions
                            .iter()
                            .find(|p| p.name == sender || p.id == sender)
                            .map(|p| p.version.as_str())
                            .unwrap_or("");
                        self.error(format!(
                            "from plugin {sender} (v{version}) {}: {}",
                            payload["Context"].as_str().unwrap_or(""),
                            payload["Error"].as_str().unwrap_or("")
                        ));
                    }
                    "DeserializeEvent" => {
                        let json = payload["Json"].as_str().unwrap_or("");
                        match JournalEvent::parse(json) {
                            Ok(mut event) => {
                                event.event = extensions::journal_type(&event.event).0.to_owned();
                                if payload["Replay"].as_bool().unwrap_or(false) {
                                    let event_type = event.event.clone();
                                    self.process(event);
                                    self.deserialize_ancillary(&event_type);
                                } else {
                                    self.processed += 1;
                                    self.last_event = if event.event == "JournalBase" {
                                        "JournalBase".into()
                                    } else {
                                        event.data["event"]
                                            .as_str()
                                            .unwrap_or(&event.event)
                                            .to_owned()
                                    };
                                }
                            }
                            Err(error) => {
                                self.error(format!("{sender} DeserializeEvent: {error:#}"))
                            }
                        }
                    }
                    "Notification" => {
                        payload["plugin"] = json!(sender);
                        self.dispatch_core_notification(payload);
                    }
                    "PlayAudioFile" => {
                        let filename = payload["Filename"]
                            .as_str()
                            .or_else(|| payload["filename"].as_str())
                            .unwrap_or("");
                        let mut options = payload
                            .get("Options")
                            .or_else(|| payload.get("options"))
                            .cloned()
                            .unwrap_or(json!({}));
                        if !options.is_object() {
                            self.error(format!("{sender}: AudioOptions must be an object"));
                            continue;
                        }
                        options["plugin"] = json!(sender);
                        let mut file = PathBuf::from(filename);
                        if let Some(plugin) = self
                            .extensions
                            .iter()
                            .find(|p| p.name == sender || p.id == sender)
                        {
                            let root = self.paths.data_dir.join("storage").join(&plugin.id);
                            if file.is_relative() {
                                file = root.join(&file);
                            }
                            if file.exists()
                                && !(std::fs::canonicalize(&file)
                                    .ok()
                                    .zip(std::fs::canonicalize(&root).ok())
                                    .is_some_and(|(file, root)| file.starts_with(root)))
                            {
                                self.error(format!(
                                    "{sender}: audio files must be inside plugin storage"
                                ));
                                continue;
                            }
                        }
                        if let Err(error) =
                            self.notifications
                                .play_file(&file, &options, &self.settings)
                        {
                            self.error(format!("{sender} audio: {error:#}"));
                        }
                    }
                    "CheckForUpdates" => {
                        let name = payload["Plugin"].as_str().unwrap_or(&sender);
                        self.updates.check(
                            name,
                            payload["AllowBeta"].as_bool().unwrap_or(false),
                            self.settings.plugins[name]["InstallUpdates"]
                                .as_bool()
                                .unwrap_or(true),
                        );
                    }
                    "InstallUpdate" => {
                        self.updates
                            .install(payload["Plugin"].as_str().unwrap_or(&sender));
                    }
                    "InstallNativePlugin" => {
                        let name = payload["Plugin"].as_str().unwrap_or("");
                        if !matches!(
                            name,
                            "BioInsights" | "Evaluator" | "Hecate" | "StellarOverlay"
                        ) {
                            self.error(format!("No bundled native plugin named {name}"));
                            continue;
                        }
                        let mut settings = self.settings.clone();
                        let mut disabled = settings
                            .extra
                            .get("disabled_plugins")
                            .and_then(Value::as_array)
                            .cloned()
                            .unwrap_or_default();
                        disabled.retain(|item| item.as_str() != Some(name));
                        settings
                            .extra
                            .insert("disabled_plugins".into(), json!(disabled));
                        match self.configure(settings) {
                            Ok(()) => {
                                let version = match name {
                                    "BioInsights" => "1.3.444.1052",
                                    "Evaluator" => "1.3.444.637",
                                    "Hecate" => "1.3.444.761",
                                    _ => "1.3.228.387",
                                };
                                for target in
                                    ["BioInsights", "Evaluator", "Hecate", "StellarOverlay"]
                                {
                                    if target != name && self.enabled(target) {
                                        queue.push_back((name.to_owned(),json!({"target":target,"command":"Ready","payload":{"Version":version}})));
                                    }
                                }
                                queue.push_back(("Core".into(),json!({"target":sender,"command":"SourceUIAlert","payload":{"Type":"PluginInstalled","Active":true,"Description":format!("{name} is included in this Linux build and is now enabled.")}})));
                            }
                            Err(error) => self.error(format!("Enable {name}: {error:#}")),
                        }
                    }
                    "BioInsightsUiAction" => {
                        let mut snapshot = self.snapshot.lock().unwrap();
                        if snapshot.ui_actions.len() < 128 {
                            snapshot.ui_actions.push_back(payload);
                        }
                    }
                    "Log" | "OverlayState" | "JournalDatabaseReset" => {
                        if let Err(error) = self.append_plugin_log(&sender, &payload) {
                            self.error(format!("{sender} log: {error:#}"));
                        }
                    }
                    _ => {}
                }
                continue;
            }
            if target == "StellarOverlay" && command == "ShowBitmapNotification" {
                if let Some(filename) = payload["Filename"].as_str() {
                    let image = std::fs::canonicalize(filename);
                    let root = std::fs::canonicalize(&self.paths.data_dir);
                    match (image, root) {
                        (Ok(image), Ok(root)) if image.starts_with(&root) => {
                            payload["Filename"] = json!(image)
                        }
                        _ => {
                            self.error(format!(
                                "{sender}: notification images must exist within application data"
                            ));
                            continue;
                        }
                    }
                }
            }
            let result = if target == "Hecate" && self.enabled("Hecate") {
                if command == "WebServerRegisterPlugin" {
                    self.hecate
                        .register_plugin(&sender, json!(sender))
                        .map(|_| None)
                } else {
                    self.hecate.handle_message(&command, &payload)
                }
            } else if target == "StellarOverlay" && self.enabled("StellarOverlay") {
                self.overlay.handle_message(&sender, &command, &payload)
            } else {
                Ok(None)
            };
            match result {
                Ok(Some(mut reply)) => {
                    reply["Target"] = json!(sender);
                    queue.push_back((target.clone(), reply));
                }
                Err(error) => self.error(format!("{sender} → {target} {command}: {error:#}")),
                _ => {}
            }
            if sender != "BioInsights"
                && (target == "BioInsights" || target.is_empty())
                && self.enabled("BioInsights")
            {
                self.projections_dirty.bio = true;
                self.projections_dirty.bio_summary = true;
                if command == "SetZoomLevel" {
                    self.projections_dirty.metadata = true;
                    if let Some(zoom) = payload["ZoomLevel"].as_u64() {
                        self.settings.plugins["BioInsights"]["ZoomLevel"] =
                            json!(zoom.clamp(50, 600));
                        if let Err(error) = self.settings.save(&self.paths.settings_file) {
                            self.error(format!("BioInsights zoom: {error:#}"));
                        }
                    }
                }
                let findings = self
                    .bioinsights
                    .on_peer_message(&sender, &command, &payload);
                for finding in findings {
                    self.record(finding);
                }
                if matches!(
                    command.as_str(),
                    "MarkVariant" | "WebRequest" | "WebSocketRequest"
                ) {
                    if let Err(error) = self.save_bio_state() {
                        self.error(format!("BioInsights state: {error:#}"));
                    }
                }
            }
            if sender != "Evaluator"
                && (target == "Evaluator" || target.is_empty())
                && self.enabled("Evaluator")
            {
                self.projections_dirty.evaluator = true;
                self.evaluator.on_peer_message(&sender, &command, &payload);
                if command == "SetZoomLevel" {
                    self.projections_dirty.metadata = true;
                    if let Some(zoom) = payload["ZoomLevel"].as_u64() {
                        self.settings.plugins["Evaluator"]["ZoomLevel"] =
                            json!(zoom.clamp(50, 600));
                        if let Err(error) = self.settings.save(&self.paths.settings_file) {
                            self.error(format!("Evaluator zoom: {error:#}"));
                        }
                    }
                }
            }
            // Each receiver gets a fresh Lua table. Legacy source sends also
            // invoke its old callback once before the typed wrapper.
            let legacy = message["legacy"].as_bool().unwrap_or(false);
            let identity = self.message_sender(&sender, legacy);
            if legacy {
                let recipients = self
                    .extensions
                    .iter()
                    .map(|plugin| plugin.id.clone())
                    .collect::<Vec<_>>();
                for recipient in recipients {
                    self.call_plugins("on_legacy_message",&json!({"__recipient":recipient,"source":identity,"data":{"target":target,"message":payload["message"]},"message":payload["message"]}));
                    self.call_plugins(
                        "on_message",
                        &json!({"__recipient":recipient,"source":identity,"data":message}),
                    );
                }
            } else {
                self.call_plugins("on_message", &json!({"source":identity,"data":message}));
            }
        }
        if queue.len() + self.native_pending.len() > 32768 {
            self.error("Native plugin pending message limit exceeded".into());
        } else {
            self.native_pending.extend(queue);
        }
        if self.enabled("Hecate") {
            for error in self.hecate.errors() {
                self.error(error);
            }
            for request in self.hecate.take_http_requests() {
                if let Err(error) = self.http.submit("native:Hecate".into(), request.clone()) {
                    let _ = self.hecate.handle_http_response(
                        &json!({"request_id":request["request_id"],"ok":false,"error":error}),
                    );
                }
            }
        }
        if self.enabled("BioInsights") && !self.batch && self.snapshot.lock().unwrap().monitoring {
            for request in self.bioinsights.take_http_requests() {
                if let Err(error) = self
                    .http
                    .submit("native:BioInsights".into(), request.clone())
                {
                    if let Err(error) = self.bioinsights.handle_http_response(
                        &json!({"request_id":request["request_id"],"ok":false,"error":error}),
                    ) {
                        self.error(format!("BioInsights HTTP: {error:#}"));
                    }
                }
            }
        }
    }
    fn save_bio_state(&self) -> Result<()> {
        anyhow::ensure!(
            self.bioinsights.is_healthy(),
            "BioInsights state save skipped: native worker is unavailable"
        );
        let dir = self.paths.data_dir.join("BioInsights");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("state.json");
        let bytes = serde_json::to_vec(&self.bioinsights.state())?;
        if std::fs::read(&path).is_ok_and(|old| old == bytes) {
            return Ok(());
        }
        let mut file = tempfile::NamedTempFile::new_in(dir)?;
        use std::io::Write;
        file.write_all(&bytes)?;
        file.as_file().sync_all()?;
        file.persist(path)?;
        Ok(())
    }
    fn scan(&mut self, batch: ScanBatch) {
        for issue in batch.errors {
            self.error(format!(
                "{}:{}: {}",
                issue.path.display(),
                issue.line,
                issue.message
            ));
        }
        // Status may change in the same poll as ScanOrganic. Install the newest
        // valid surface coordinates before a sample captures its origin.
        for update in batch.updates {
            if update.name == "Status" || update.name == "Status.json" {
                let previous = self.snapshot.lock().unwrap().status.clone();
                self.status_dirty |= previous != update.data;
                self.projections_dirty.bio |= previous["BodyName"] != update.data["BodyName"];
                self.projections_dirty.evaluator |= previous["GuiFocus"] != update.data["GuiFocus"]
                    || previous["Destination"] != update.data["Destination"];
                self.botanist.on_status(&update.data);
                if self.enabled("BioInsights") {
                    let findings = self.bioinsights.on_status(&update.data);
                    for finding in findings {
                        self.record(finding);
                    }
                }
                if self.enabled("StellarOverlay") {
                    self.overlay.on_status(&update.data);
                }
                if self.enabled("Evaluator") {
                    self.evaluator.on_status(&update.data);
                }
                self.native_messages(Vec::new());
                self.sync_sampler();
                self.snapshot.lock().unwrap().status = update.data.clone();
                for plugin in &self.extensions {
                    if let Err(error) = plugin.set_context("status", &update.data) {
                        self.error(error.to_string());
                    }
                }
                self.call_plugins("on_status", &update.data);
            } else if update.name == "Cargo" || update.name == "Cargo.json" {
                for plugin in &self.extensions {
                    if let Err(error) = plugin.set_context("cargo", &update.data) {
                        self.error(error.to_string());
                    }
                }
                self.call_plugins("on_cargo", &update.data);
            } else {
                self.call_plugins(
                    "on_auxiliary",
                    &json!({"name":update.name,"data":update.data}),
                );
            }
        }
        for event in batch.events {
            self.process(event);
        }
    }
    fn poll_http(&mut self) {
        if self.enabled("Explorer") {
            for finding in self.explorer.poll_custom() {
                self.record(finding);
            }
            self.explorer_diagnostics();
        }
        for event in self.notifications.take_audio_completions() {
            self.call_plugins("on_audio_complete", &event);
            let target = event["plugin"].as_str().unwrap_or("");
            if matches!(
                target,
                "BioInsights" | "Evaluator" | "Hecate" | "StellarOverlay"
            ) {
                self.native_messages(vec![(
                    "Core".into(),
                    json!({"target":target,"command":"PlayAudioFileCompleted","payload":event}),
                )]);
            }
        }
        let failures = self
            .bioinsights
            .take_worker_errors()
            .into_iter()
            .chain(self.evaluator.take_worker_errors())
            .chain(self.hecate.take_worker_errors())
            .chain(self.overlay.take_worker_errors())
            .collect::<Vec<_>>();
        for error in failures {
            self.projections_dirty.metadata = true;
            self.error(format!("Native worker: {error}"));
        }
        for event in self.updates.poll() {
            self.projections_dirty.metadata = true;
            let plugin = event["plugin"].as_str().unwrap_or("");
            match event["state"].as_str().unwrap_or("") {
                "Available" | "Downloaded" => {
                    let kind = if event["state"] == "Available" {
                        "UpdateAvailable"
                    } else {
                        "UpdateDownloaded"
                    };
                    if matches!(plugin, "BioInsights" | "Evaluator") {
                        self.native_messages(vec![("Core".into(),json!({"target":plugin,"command":"SourceUIAlert","payload":{"Type":kind,"Active":true}}))]);
                    }
                }
                "Error" => self.error(format!(
                    "{plugin} native update: {}",
                    event["error"].as_str().unwrap_or("Unknown error")
                )),
                _ => {}
            }
        }

        if self.enabled("Evaluator") {
            if self.evaluator.poll_routes() {
                self.projections_dirty.evaluator = true;
            }
            for finding in self.evaluator.take_route_findings() {
                self.record(finding);
            }
        }
        for diagnostic in self.bioinsights.take_diagnostics() {
            if let Err(error) = self.append_plugin_log("BioInsights", &diagnostic) {
                self.error(format!("BioInsights log: {error:#}"));
            }
        }
        for response in self.http.poll() {
            let mut data = response.data;
            if response.plugin_id == "native:BioInsights" {
                self.projections_dirty.bio = true;
                self.projections_dirty.bio_summary = true;
                match self.bioinsights.handle_http_response(&data) {
                    Ok(()) => {
                        if let Err(error) = self.save_bio_state() {
                            self.error(format!("BioInsights cache: {error:#}"));
                        }
                    }
                    Err(error) => self.error(format!("BioInsights HTTP: {error:#}")),
                }
                continue;
            }
            if response.plugin_id == "native:Hecate" {
                if let Err(error) = self.hecate.handle_http_response(&data) {
                    self.error(format!("Hecate HTTP: {error:#}"));
                }
                continue;
            }
            data["__recipient"] = json!(response.plugin_id);
            self.call_plugins("on_http_response", &data);
        }
        self.native_messages(Vec::new());
        self.overlay.tick();
        self.native_popups
            .tick(observatory_plugins::stellar_overlay::now_ms());
    }
    fn publish(&mut self) {
        let dirty = self.projections_dirty;
        let updates = dirty.metadata.then(|| Arc::new(self.updates.summary()));
        let findings = dirty.findings.then(|| Arc::new(self.findings.clone()));
        let finding_grid_times = dirty
            .findings
            .then(|| Arc::new(self.finding_grid_times.clone()));
        let bio_browser_html = (self.bio_browser_enabled && (dirty.bio || dirty.bio_summary))
            .then(|| Arc::new(self.bioinsights.browser_html()));
        // All JSON building, prediction/catalogue work, row filtering and large
        // allocation/destruction happen outside the render/input thread's lock.
        let botanist = dirty.botanist.then(|| {
            let mut rows = self.botanist.rows();
            if let Some(hidden) = self.hidden_rows.get("Botanist") {
                rows.retain(|row| {
                    !hidden.contains(&serde_json::to_string(row).unwrap_or_default())
                });
            }
            rows
        });
        let messages = dirty.messages.then(|| {
            let mut rows: Vec<MessageRow> = self.communicator.rows().into_iter().cloned().collect();
            if let Some(hidden) = self.hidden_rows.get("Communicator") {
                rows.retain(|row| {
                    !hidden.contains(&serde_json::to_string(row).unwrap_or_default())
                });
            }
            rows
        });
        let metadata = dirty.metadata.then(|| {
            let mut extensions: Vec<_> = self.extensions.iter()
                .map(|p| (p.name.clone(),p.version.clone(),p.enabled && self.enabled(&p.name))).collect();
            let mut details: BTreeMap<String, Value> = self.extensions.iter()
                .map(|p| (p.name.clone(),json!({"id":p.id,"version":p.version,"description":p.description,"author":p.author,"settings":p.settings,"status":if p.enabled{"OK"}else{"Error"},"worker_healthy":p.enabled}))).collect();
            for(name,version,id)in [("Explorer","1.3.0.27422","b11e8725-3f71-4445-9b09-682f9eab8b59"),("Botanist","1.1.3.27422","116d68f6-1c21-49ad-ae33-f0fef619af24"),("Communicator","0.0.2.27422","581b2f03-1c16-4f02-9e20-e3bef5e9518b")]{
                extensions.push((name.into(),version.into(),self.enabled(name)));
                details.insert(name.into(),json!({"id":id,"full_name":format!("Observatory {name}"),"author":"Vithigar","version":version,"role":"Worker","has_ui":true,"settings":self.settings.plugins[name]}));
            }
            for (name, full_name, version, role, has_ui, id) in [
                ("BioInsights","BioInsights","1.3.444.1052","Worker",true,"4d617474-4701-4249-4f49-4e5349474854"),
                ("Evaluator","Evaluator","1.3.444.637","Worker",true,"4d617474-4702-0045-5641-4c5541544f52"),
                ("Hecate","Hecate - Data Vault","1.3.444.761","Worker",false,"4d617474-4707-0000-0000-484543415445"),
                ("StellarOverlay","Stellar Overlay","1.3.228.387","Worker, Notifier",false,"4d617474-4703-5354-454c-4c41524f5645"),
            ] {
                let worker_info=match name{"BioInsights"=>self.bioinsights.worker_info(),"Evaluator"=>self.evaluator.worker_info(),"Hecate"=>self.hecate.worker_info(),_=>self.overlay.worker_info()};
                let version=worker_info.as_ref().map(|info|info.source_version.as_str()).unwrap_or(version);
                extensions.push((name.into(),version.into(),self.enabled(name)));
                details.insert(name.into(),json!({"id":id,"full_name":full_name,"author":"Matt G","version":version,"role":role,"has_ui":has_ui,"settings":self.settings.plugins[name],"worker":worker_info}));
            }
            for name in ["BioInsights","Evaluator","Hecate","StellarOverlay"]{let(pid,healthy)=match name{"BioInsights"=>(self.bioinsights.worker_process_id(),self.bioinsights.is_healthy()),"Evaluator"=>(self.evaluator.worker_process_id(),self.evaluator.is_healthy()),"Hecate"=>(self.hecate.worker_process_id(),self.hecate.is_healthy()),_=>(self.overlay.worker_process_id(),self.overlay.is_healthy())};if let Some(details)=details.get_mut(name){details["worker_process_id"]=json!(pid);details["worker_healthy"]=json!(healthy);details["status"]=json!(if healthy{"OK"}else{"Error"});}}
            for name in ["BioInsights","Evaluator","Hecate","StellarOverlay"]{if let Some(details)=details.get_mut(name){details["about"]=crate::about::metadata(name);details["description"]=details["about"]["Description"].clone();}}
            let mut columns=vec!["BodyName","Sigs","Flags","Genus","Species","Seen","Samples","Type","PossibleVariants","BaseValue","Distance"];
            if self.settings.plugins["BioInsights"]["OutputGridFull"].as_bool()==Some(true) {
                columns.extend(["Body","Grav","Temp","Atmosphere","Pressure","Volcanism"]);
            }
            details.get_mut("BioInsights").unwrap()["columns"]=json!(columns);
            details.get_mut("Evaluator").unwrap()["columns"]=json!(observatory_plugins::evaluator::COLUMNS);
            (extensions,details.into_iter().map(|(k,v)|(k,Arc::new(v))).collect())
        });
        let bio = dirty.bio.then(|| {
            let view = Arc::new(self.bioinsights.main_view());
            let mut rows = self.bioinsights.rows();
            if let Some(hidden) = self.hidden_rows.get("BioInsights") {
                rows.retain(|row| !hidden.contains(&row.to_string()));
            }
            (view, Arc::new(rows))
        });
        let bio_summary = dirty.bio_summary.then(|| {
            let mut summary = self.bioinsights.tools_summary();
            summary["HecateAvailable"] = json!(self.enabled("Hecate"));
            Arc::new(summary)
        });
        let evaluator = dirty.evaluator.then(|| {
            let mut summary = self.evaluator.summary();
            summary["HecateAvailable"] = json!(self.enabled("Hecate"));
            let summary = Arc::new(summary);
            let mut rows = self.evaluator.rows();
            if let Some(hidden) = self.hidden_rows.get("Evaluator") {
                rows.retain(|row| !hidden.contains(&row.to_string()));
            }
            (summary, Arc::new(rows))
        });
        let overlay = if self.enabled("StellarOverlay") {
            self.overlay.snapshot()
        } else {
            OverlaySnapshot::default()
        };
        let sampler = self.botanist.sampler().cloned();
        // Return retired allocations from this scope so their destructors cannot
        // turn an otherwise cheap pointer swap into a long UI mutex stall.
        let retired = {
            let mut state = self.snapshot.lock().unwrap();
            let repaint = state.repaint.clone().filter(|_| {
                state.total != self.processed
                    || self.status_dirty
                    || dirty.findings
                    || dirty.metadata
                    || dirty.bio
                    || dirty.bio_summary
                    || dirty.evaluator
                    || dirty.botanist
                    || dirty.messages
                    || !overlay.notifications.is_empty()
                    || !overlay.alerts.is_empty()
                    || !state.native_popups.is_empty()
                    || !self.native_popups.snapshot().is_empty()
            });
            state.total = self.processed;
            state.last_event = self.last_event.clone();
            if let Some(updates) = updates {
                state.native_updates = updates;
            }
            if let Some(html) = bio_browser_html {
                state.bio_browser_html = html;
            }
            let old_findings = findings.map(|rows| std::mem::replace(&mut state.findings, rows));
            let old_finding_times = finding_grid_times
                .map(|times| std::mem::replace(&mut state.finding_grid_times, times));
            let old_botanist =
                botanist.map(|rows| std::mem::replace(&mut state.botanist, Arc::new(rows)));
            let old_messages =
                messages.map(|rows| std::mem::replace(&mut state.messages, Arc::new(rows)));
            let old_metadata = metadata.map(|(extensions, details)| {
                (
                    std::mem::replace(&mut state.extensions, extensions),
                    std::mem::replace(&mut state.plugin_details, details),
                )
            });
            let old_bio = bio.map(|(view, rows)| {
                state.bio_revision = state.bio_revision.wrapping_add(1);
                (
                    std::mem::replace(&mut state.bio_main_view, view),
                    state.plugin_rows.insert("BioInsights".into(), rows),
                )
            });
            let old_summary =
                bio_summary.map(|summary| std::mem::replace(&mut state.bio_summary, summary));
            let old_eval = evaluator.map(|(summary, rows)| {
                state.evaluator_revision = state.evaluator_revision.wrapping_add(1);
                (
                    std::mem::replace(&mut state.eval_summary, summary),
                    state.plugin_rows.insert("Evaluator".into(), rows),
                )
            });
            if dirty.findings
                || dirty.metadata
                || dirty.bio
                || dirty.bio_summary
                || dirty.evaluator
                || dirty.botanist
                || dirty.messages
            {
                state.worker_revision = state.worker_revision.wrapping_add(1);
            }
            state.sampler = sampler;
            state.overlay = overlay;
            state.native_popups = self.native_popups.snapshot();
            (
                repaint,
                old_findings,
                old_finding_times,
                old_botanist,
                old_messages,
                old_metadata,
                old_bio,
                old_summary,
                old_eval,
            )
        };
        if let Some(repaint) = &retired.0 {
            repaint.request_repaint();
        }
        drop(retired);
        self.status_dirty = false;
        self.projections_dirty = ProjectionDirty::default();
    }
    fn clear(&mut self, plugin: &str) {
        if plugin == "Botanist" {
            self.botanist.clear_visible_rows();
        }
        if plugin == "Communicator" {
            self.communicator.clear_visible_rows();
        }
        match plugin {
            "Botanist" => self.projections_dirty.botanist = true,
            "Communicator" => self.projections_dirty.messages = true,
            "BioInsights" => self.projections_dirty.bio = true,
            "Evaluator" => self.projections_dirty.evaluator = true,
            _ => {}
        }
        // Keep only an Arc clone under the render thread's lock. Serializing a
        // large Lua/native table to track cleared rows happens on this worker.
        let previous_rows = self
            .snapshot
            .lock()
            .unwrap()
            .plugin_rows
            .get(plugin)
            .cloned();
        let values: Vec<String> = if matches!(plugin, "Botanist" | "Communicator") {
            Vec::new()
        } else {
            previous_rows
                .iter()
                .flat_map(|rows| rows.iter())
                .map(|row| serde_json::to_string(row).unwrap_or_default())
                .collect()
        };
        self.hidden_rows
            .entry(plugin.to_owned())
            .or_default()
            .extend(values);
        let retired = {
            let mut snapshot = self.snapshot.lock().unwrap();

            let mut index = 0;
            self.finding_grid_times.retain(|_| {
                let keep = self.findings[index].plugin != plugin;
                index += 1;
                keep
            });
            self.findings.retain(|finding| finding.plugin != plugin);
            self.projections_dirty.findings = true;
            snapshot.worker_revision = snapshot.worker_revision.wrapping_add(1);
            snapshot.plugin_rows.remove(plugin)
        };
        drop(retired);
    }
    fn export(
        &mut self,
        path: &std::path::Path,
        delimiter: u8,
        plugin: Option<&str>,
    ) -> Result<usize> {
        if let Some(extension) = self
            .extensions
            .iter_mut()
            .find(|p| Some(p.name.as_str()) == plugin)
        {
            if let Some(content) = extension.export(&(delimiter as char).to_string())? {
                std::fs::write(path, content)?;
                return Ok(1);
            }
        }
        let rows: Option<(Vec<String>, Vec<Vec<String>>)> = match plugin {
            Some("Botanist") => Some((
                vec![
                    "Body",
                    "Bio signals",
                    "Species",
                    "Genus",
                    "Analysed",
                    "Colony distance",
                ]
                .into_iter()
                .map(str::to_owned)
                .collect(),
                self.botanist
                    .rows()
                    .iter()
                    .map(|r| {
                        vec![
                            r.body.clone(),
                            r.bio_total.to_string(),
                            r.species.clone(),
                            r.genus.clone(),
                            r.analysed.to_string(),
                            r.colony_distance.to_string(),
                        ]
                    })
                    .collect(),
            )),
            Some("Communicator") => Some((
                vec!["Time", "Channel", "From", "Message", "Sent"]
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
                self.communicator
                    .rows()
                    .into_iter()
                    .map(|r| {
                        vec![
                            r.timestamp.clone(),
                            r.channel.clone(),
                            r.from.clone(),
                            r.message.clone(),
                            r.sent.to_string(),
                        ]
                    })
                    .collect(),
            )),
            Some(name) => {
                let (rows, details) = {
                    let snapshot = self.snapshot.lock().unwrap();
                    (
                        snapshot.plugin_rows.get(name).cloned(),
                        snapshot.plugin_details.get(name).cloned(),
                    )
                };
                rows.filter(|r| !r.is_empty()).map(|rows| {
                    let mut keys = std::collections::BTreeSet::new();
                    for row in rows.iter() {
                        if let Some(object) = row.as_object() {
                            keys.extend(object.keys().cloned());
                        }
                    }
                    let keys = details
                        .as_ref()
                        .and_then(|details| details["columns"].as_array())
                        .map(|columns| {
                            columns
                                .iter()
                                .filter_map(Value::as_str)
                                .map(str::to_owned)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_else(|| keys.into_iter().collect::<Vec<_>>());
                    let values = rows
                        .iter()
                        .map(|row| {
                            keys.iter()
                                .map(|key| {
                                    row[key]
                                        .as_str()
                                        .map(str::to_owned)
                                        .unwrap_or_else(|| row[key].to_string())
                                })
                                .collect()
                        })
                        .collect();
                    (keys, values)
                })
            }
            None => None,
        };
        if let Some((headers, rows)) = rows {
            if delimiter == b'\t'
                || path
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("xlsx"))
            {
                return crate::table_export::write(
                    path,
                    plugin.unwrap_or("Observatory"),
                    &headers,
                    &rows,
                );
            }
            let mut writer = csv::WriterBuilder::new()
                .delimiter(delimiter)
                .from_path(path)?;
            writer.write_record(headers)?;
            for row in &rows {
                writer.write_record(row)?;
            }
            writer.flush()?;
            Ok(rows.len())
        } else {
            self.store.export(path, delimiter, plugin)
        }
    }
}

fn worker(
    settings: Settings,
    paths: ConfigPaths,
    snapshot: Arc<Mutex<Snapshot>>,
    cancel: Arc<AtomicBool>,
    replay_cancel: Arc<AtomicBool>,
    rx: Receiver<Command>,
) -> Result<()> {
    let mut engine = Engine::new(settings, paths, snapshot)?;
    engine.call_plugins("on_ready", &json!({}));
    let mut tailer: Option<JournalTailer> = None;
    let mut monitoring = false;
    let mut startup_replay_should_monitor =
        engine.settings.start_read_all && engine.settings.auto_start;
    let startup = if engine.settings.start_read_all {
        Some(Command::Replay)
    } else if engine.settings.auto_start {
        Some(Command::Start)
    } else {
        None
    };
    let mut pending = startup;
    loop {
        let command = pending.take().or_else(|| {
            rx.recv_timeout(if monitoring {
                engine.settings.poll_interval()
            } else {
                Duration::from_millis(250)
            })
            .ok()
        });
        match command {
            Some(Command::Shutdown) => {
                engine.state("Stopped", false);
                engine.call_plugins("on_shutdown", &json!({}));
                return Ok(());
            }
            Some(Command::CancelReplay) => {
                if replay_cancel.swap(false, Ordering::Relaxed) {
                    cancel.store(false, Ordering::Relaxed);
                }
            }
            Some(Command::TestNotification(settings, rendering, direct)) => {
                let (title, detail) = if rendering == 1 && direct {
                    ("Test Popup Notification","Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec at elit maximus, ornare dui nec, accumsan velit. Vestibulum fringilla elit.")
                } else if direct {
                    (
                        "Test Voice Notification",
                        "This is a test of native voice notifications.",
                    )
                } else {
                    (
                        "Notification routing test",
                        "Please check that expected notifications were triggered.",
                    )
                };
                let payload = json!({"title":title,"detail":detail,"rendering":rendering,"plugin":"Core","extended_details":"Ensure all expected content is present."});
                let mut configured = if direct {
                    settings
                } else {
                    let mut current = engine.notification_settings();
                    current.extra = settings.extra;
                    current
                };
                if direct {
                    configured.popup = rendering & 1 != 0;
                    configured.speech = rendering & 2 != 0;
                } else {
                    configured.popup &= rendering & 1 != 0;
                    configured.speech &= rendering & 2 != 0;
                }
                if !direct && engine.enabled("StellarOverlay") {
                    engine.overlay.on_notification(&payload);
                }
                engine.native_lifecycle(payload.clone(), configured);
                if !direct {
                    engine.call_plugins("on_notification", &payload);
                }
            }
            Some(Command::Stop) => {
                monitoring = false;
                engine.state("Idle", false);
            }
            Some(Command::Clear(plugin)) => {
                engine.clear(&plugin);
            }
            Some(Command::PluginMessage(name, command, payload)) => {
                engine.native_messages(vec![(
                    "Core".into(),
                    json!({"target":name,"command":command,"payload":payload}),
                )]);
            }
            Some(Command::Configure(mut settings)) => {
                let resume_monitoring = monitoring;
                let directory_changed = engine.settings.journal_dir != settings.journal_dir;
                default_plugin_settings(&mut settings);
                match engine.configure(settings) {
                    Ok(()) => {
                        if directory_changed {
                            tailer = None;
                            monitoring = false;
                            pending = resume_monitoring.then_some(Command::Start);
                        }
                        if !resume_monitoring {
                            engine.state("Idle", false);
                        }
                    }
                    Err(error) => engine.error(format!("configure: {error:#}")),
                }
            }
            Some(Command::ExportView(path, plugin, view)) => {
                let result = (|| -> Result<usize> {
                    if let Some(extension) = engine.extensions.iter_mut().find(|p| p.name == plugin)
                    {
                        if let Some(content) = extension.export("\t")? {
                            std::fs::write(&path, content)?;
                            return Ok(view.rows.len());
                        }
                    }
                    crate::table_export::write(&path, &plugin, &view.headers, &view.rows)
                })();
                match result {
                    Ok(count) => {
                        engine.snapshot.lock().unwrap().state =
                            format!("Exported {count} rows to {}", path.display())
                    }
                    Err(error) => engine.error(format!("export: {error:#}")),
                }
            }
            Some(Command::Export(path, delimiter, plugin)) => {
                match engine.export(&path, delimiter, plugin.as_deref()) {
                    Ok(count) => {
                        engine.snapshot.lock().unwrap().state =
                            format!("Exported {count} rows to {}", path.display());
                    }
                    Err(error) => engine.error(format!("export: {error:#}")),
                }
            }
            Some(Command::Start) | Some(Command::Replay) => {
                let replay = matches!(command, Some(Command::Replay));
                let resume_after_read = !replay || monitoring || startup_replay_should_monitor;
                if replay {
                    startup_replay_should_monitor = false;
                }
                cancel.store(false, Ordering::Relaxed);
                match observatory_io::resolve_journal_dir(engine.settings.journal_dir.as_deref()) {
                    Ok(dir) => {
                        let first_start = tailer.is_none();
                        if first_start {
                            tailer = Some(JournalTailer::new(dir));
                        }
                        let reader = tailer.as_mut().unwrap();
                        if replay {
                            engine.hidden_rows.clear();
                            engine.findings.clear();
                            engine.finding_grid_times.clear();
                            engine.projections_dirty.findings = true;
                            engine.bioinsights.reset();
                            engine.evaluator.reset();
                            engine.overlay.reset();
                            if engine.enabled("Hecate") {
                                if let Err(error) = engine.hecate.begin_replay() {
                                    engine.error(format!("Hecate read all: {error:#}"));
                                }
                            }
                            engine.state("Reading all journals", true);
                            engine.native_messages(vec![("Core".into(),json!({"target":"","command":"SourceReadAllState","payload":{"State":"Started"}}))]);
                            engine.snapshot.lock().unwrap().read_progress =
                                Some((String::new(), 0, 1));
                            let result = reader.replay_all(&cancel, |item| match item {
                                ReplayItem::Event(event) => {
                                    engine.process(event);
                                    if engine.processed % 256 == 0 {
                                        engine.publish();
                                    }
                                }
                                ReplayItem::Error(issue) => engine.error(format!(
                                    "{}:{}: {}",
                                    issue.path.display(),
                                    issue.line,
                                    issue.message
                                )),
                                ReplayItem::Progress {
                                    path,
                                    completed,
                                    total,
                                } => {
                                    let mut snapshot = engine.snapshot.lock().unwrap();
                                    snapshot.state =
                                        format!("Reading journals {completed}/{total}");
                                    snapshot.read_progress = Some((
                                        path.file_name()
                                            .unwrap_or_default()
                                            .to_string_lossy()
                                            .into_owned(),
                                        completed,
                                        total,
                                    ));
                                }
                            });
                            let read_state = if cancel.load(Ordering::Relaxed) {
                                "Cancelled"
                            } else if result.is_ok() {
                                "Completed"
                            } else {
                                "Required"
                            };
                            if read_state == "Cancelled" {
                                engine.state("Read all cancelled", false);
                            }
                            engine.native_messages(vec![("Core".into(),json!({"target":"","command":"SourceReadAllState","payload":{"State":read_state}}))]);
                            if let Err(error) = result {
                                engine.error(format!("read all: {error:#}"));
                            }
                        } else if first_start {
                            engine.state("Preparing current system", true);
                            engine.scan(reader.preread(&cancel));
                        }
                        if replay && replay_cancel.swap(false, Ordering::Relaxed) {
                            cancel.store(false, Ordering::Relaxed);
                        }
                        monitoring = resume_after_read && !cancel.load(Ordering::Relaxed);
                        if first_start && !replay && !monitoring {
                            tailer = None;
                        }
                        engine.finish_read(monitoring);
                    }
                    Err(error) => {
                        monitoring = false;
                        engine.state("Choose journal directory in Settings", false);
                        engine.error(format!("{error:#}"));
                    }
                }
            }
            None => {}
        }
        if monitoring && !cancel.load(Ordering::Relaxed) {
            if let Some(reader) = tailer.as_mut() {
                engine.scan(reader.scan(&cancel));
            }
        }
        for error in engine.notifications.take_errors() {
            engine.error(error);
        }
        engine.poll_http();
        engine.snapshot.lock().unwrap().monitoring = monitoring;
        for plugin in &engine.extensions {
            if let Err(error) =
                plugin.set_context("log_monitor_state", &json!(if monitoring { 1 } else { 0 }))
            {
                engine.error(error.to_string());
            }
        }
        engine.publish();
    }
}

/// Run the same engine without a display, useful for imports and reproducible diagnostics.
pub fn replay(settings: Settings, paths: ConfigPaths, export: Option<PathBuf>) -> Result<Value> {
    let snapshot = Arc::new(Mutex::new(Snapshot::default()));
    let mut engine = Engine::new(settings, paths, snapshot)?;
    let dir = observatory_io::resolve_journal_dir(engine.settings.journal_dir.as_deref())?;
    let mut reader = JournalTailer::new(dir);
    let cancel = AtomicBool::new(false);
    if engine.enabled("Hecate") {
        engine
            .hecate
            .begin_replay()
            .context("rebuild Hecate vault")?;
    }
    engine.state("Reading all journals", true);
    let summary = reader
        .replay_all(&cancel, |item| match item {
            ReplayItem::Event(event) => engine.process(event),
            ReplayItem::Error(issue) => engine.error(format!(
                "{}:{}: {}",
                issue.path.display(),
                issue.line,
                issue.message
            )),
            ReplayItem::Progress { .. } => {}
        })
        .context("journal replay")?;
    engine.publish();
    engine.hecate.flush()?;
    if let Some(path) = export {
        engine.store.export(&path, b',', None)?;
    }
    let snapshot = engine.snapshot.lock().unwrap();
    Ok(
        json!({"summary":format!("{summary:?}"),"events":snapshot.total,"findings":snapshot.findings,"botanist":snapshot.botanist,"messages":snapshot.messages,"plugins":snapshot.plugin_rows,"plugin_views":{"BioInsights":snapshot.bio_main_view},"bio_summary":snapshot.bio_summary,"eval_summary":snapshot.eval_summary,"overlay":snapshot.overlay,"plugin_details":snapshot.plugin_details,"native_updates":snapshot.native_updates,"errors":snapshot.errors}),
    )
}

pub fn watch(mut settings: Settings, paths: ConfigPaths, seconds: u64) -> Result<Value> {
    settings.auto_start = false;
    settings.start_read_all = false;
    let controller = Controller::spawn(settings, paths)?;
    let cancellation = controller.cancel.clone();
    ctrlc::set_handler(move || {
        cancellation.store(true, Ordering::Relaxed);
    })?;
    controller.send(Command::Start);
    let initialization = std::time::Instant::now();
    let initial_events = loop {
        let snapshot = controller.snapshot.lock().unwrap();
        if snapshot.state == "Monitoring" {
            break snapshot.total;
        }
        if snapshot.state == "Error" || snapshot.state.starts_with("Choose journal") {
            anyhow::bail!("{}: {:?}", snapshot.state, snapshot.errors);
        }
        drop(snapshot);
        anyhow::ensure!(
            initialization.elapsed() < Duration::from_secs(60),
            "journal initialization timed out"
        );
        if controller.cancel.load(Ordering::Relaxed) {
            anyhow::bail!("journal initialization cancelled");
        }
        thread::sleep(Duration::from_millis(25));
    };
    let started = std::time::Instant::now();
    while started.elapsed() < Duration::from_secs(seconds)
        && !controller.cancel.load(Ordering::Relaxed)
    {
        thread::sleep(Duration::from_millis(25));
    }
    controller.send(Command::Stop);
    let snapshot = controller.snapshot.lock().unwrap();
    Ok(
        json!({"initial_events":initial_events,"live_events":snapshot.total.saturating_sub(initial_events),"last_event":snapshot.last_event,"status_loaded":snapshot.status.is_object(),"errors":snapshot.errors,"duration_seconds":started.elapsed().as_secs_f64(),"findings":snapshot.findings,"plugins":snapshot.plugin_rows,"plugin_views":{"BioInsights":snapshot.bio_main_view},"bio_summary":snapshot.bio_summary,"eval_summary":snapshot.eval_summary,"overlay":snapshot.overlay,"plugin_details":snapshot.plugin_details,"native_updates":snapshot.native_updates}),
    )
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod runtime_tests;

#[cfg(test)]
#[path = "native_plugins_tests.rs"]
mod native_plugins_tests;

#[cfg(test)]
#[path = "publication_tests.rs"]
mod publication_tests;

#[cfg(test)]
#[path = "runtime_core_api_tests.rs"]
mod runtime_core_api_tests;

#[cfg(test)]
#[path = "runtime_notification_tests.rs"]
mod runtime_notification_tests;
