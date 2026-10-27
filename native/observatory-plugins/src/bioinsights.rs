//! Native BioInsights: installed Codex rules, FSS/DSS predictions, observed
//! biology, Vista values, first footfall and three sample surface guidance.
//! All game input stays local. See bioinsights/PROVENANCE.md for data origin.
mod companion;
mod feed;
mod knowledge;
mod legacy;
mod main_view;
mod notifications;
mod radar;
mod rules;
mod symbol_css;
mod ui_alerts;
mod web;
mod web_ui;
use anyhow::{bail, Result};
use observatory_domain::{Finding, JournalEvent};
use rules::{compact, name, number, text, Predicted};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
pub const VERSION: &str = "1.3.444.1052";
pub use legacy::decode_legacy_codex;

/// Canonical Frontier token for a Codex or community genus name.
pub fn codex_genus_id(genus: &str) -> Option<String> {
    rules::genus_key(genus).map(|key| format!("${key};"))
}

pub fn default_settings() -> Value {
    let mut settings: Value =
        serde_json::from_str(include_str!("bioinsights/settings-defaults.json"))
            .expect("installed BioInsights settings constructor");
    // Native I/O policy lets offline/replay tools suppress the original public
    // Galactic Codex lookup. It does not alter installed plugin defaults.
    settings["GalacticCodexRemote"] = json!(true);
    settings["ShowHorizons"] = json!(true);
    settings["ShowUnlikely"] = json!(true);
    settings["MinimumValue"] = json!(0);
    settings["Notifications"] = notifications::defaults();
    settings
}

#[derive(Clone, Debug, Default)]
struct SpeciesState {
    variant: String,
    seen: bool,
    analysed: bool,
    samples: Vec<[f64; 2]>,
    sample_count: u8,
    unsold: bool,
    value: u64,
    marked: bool,
}
#[derive(Clone, Debug, Default)]
struct BodyState {
    scan: Value,
    bio_count: Option<u64>,
    mapped: bool,
    genera: BTreeSet<String>,
    observed: BTreeMap<String, SpeciesState>,
    footfall: bool,
    visited: bool,
}
#[derive(Clone, Debug, Default)]
struct SystemState {
    name: String,
    population: u64,
    position: Option<[f64; 3]>,
    bodies: BTreeMap<u64, BodyState>,
    complete: bool,
    star_hint: String,
}

impl SystemState {
    fn rule_bodies(&self) -> BTreeMap<u64, Value> {
        let mut bodies: BTreeMap<_, _> = self
            .bodies
            .iter()
            .map(|(id, b)| (*id, b.scan.clone()))
            .collect();
        if !self.star_hint.is_empty() {
            bodies.insert(
                u64::MAX,
                json!({"BodyID":0,"StarType":self.star_hint,"_Hint":true}),
            );
        }
        bodies
    }
}

pub struct BioInsights {
    settings: Value,
    systems: BTreeMap<u64, SystemState>,
    current_system: u64,
    current_body: Option<u64>,
    active_sample: Option<(u64, u64, String)>,
    position: Option<[f64; 2]>,
    surface_body: String,
    radius: f64,
    timestamp: String,
    messages: Vec<Value>,
    batch: bool,
    commander: String,
    commander_name: String,
    presentation_order: Vec<u64>,
    personal_codex: BTreeSet<(u64, String, String)>,
    marked_variants: BTreeSet<(String, String)>,
    vault_unsold: Vec<Value>,
    vault_total: Option<u64>,
    pending_web: Vec<(String, Value)>,
    gui_focus: u64,
    alert_body: Option<u64>,
    last_body_alert: Option<Value>,
    galactic: feed::Galactic,
    http_requests: Vec<Value>,
    announced: BTreeSet<(u64, u64, String)>,
    predictions_cache: std::sync::Mutex<BTreeMap<(u64, u64), Vec<Predicted>>>,
    alert_cache: std::sync::Mutex<BTreeMap<(u64, u64), Value>>,
    notification_renderings: Vec<(Finding, u8)>,
    notifications_allowed: bool,
    distance_notified: bool,
    diagnostics: Vec<Value>,
    commander_states: BTreeMap<String, Value>,
    distance_cache: Value,
    browser_route: Vec<String>,
    browser_cache: std::sync::Mutex<Option<String>>,
    web_server: String,
    seen_plugins: BTreeMap<String, String>,
    overlay_defaults: Value,
    ui_alerts: BTreeMap<u8, String>,
    next_star_hint: String,
    hecate_nag_skipped: bool,
}
impl BioInsights {
    pub fn new(settings: Value) -> Result<Self> {
        if !settings.is_object() && !settings.is_null() {
            bail!("BioInsights settings must be an object");
        }
        let mut merged = default_settings();
        if let Some(settings) = settings.as_object() {
            for (k, v) in settings {
                merged[k] = v.clone();
            }
        }
        let http_requests = if merged
            .get("GalacticCodexRemote")
            .and_then(Value::as_bool)
            .unwrap_or(true)
        {
            vec![feed::request()]
        } else {
            Vec::new()
        };
        Ok(Self {
            settings: merged,
            systems: BTreeMap::new(),
            current_system: 0,
            current_body: None,
            active_sample: None,
            position: None,
            surface_body: String::new(),
            radius: 0.0,
            timestamp: String::new(),
            messages: Vec::new(),
            batch: false,
            commander: String::new(),
            commander_name: String::new(),
            presentation_order: Vec::new(),
            personal_codex: BTreeSet::new(),
            marked_variants: BTreeSet::new(),
            vault_unsold: Vec::new(),
            vault_total: None,
            pending_web: Vec::new(),
            gui_focus: 0,
            alert_body: None,
            last_body_alert: None,
            galactic: feed::Galactic::default(),
            http_requests,
            announced: BTreeSet::new(),
            predictions_cache: std::sync::Mutex::new(BTreeMap::new()),
            alert_cache: std::sync::Mutex::new(BTreeMap::new()),
            notification_renderings: Vec::new(),
            notifications_allowed: true,
            distance_notified: false,
            diagnostics: Vec::new(),
            commander_states: BTreeMap::new(),
            distance_cache: Value::Null,
            browser_route: vec!["main".into()],
            browser_cache: std::sync::Mutex::new(None),
            web_server: String::new(),
            seen_plugins: BTreeMap::new(),
            ui_alerts: BTreeMap::new(),
            next_star_hint: String::new(),
            hecate_nag_skipped: false,
            overlay_defaults: json!({"TitleColorHex":"FF3ABBF6","ColorHex":"FF1A9BE6","BackgroundColorHex":"B0000000","BorderColorHex":"FFB0D0FF"}),
        })
    }
    pub fn default_settings() -> Value {
        default_settings()
    }
    /// Applying preferences must not reconstruct journal history or a partial
    /// organic sample. Source Settings assignment preserves the worker too.
    pub fn set_settings(&mut self, settings: Value) -> Result<()> {
        let replacement = Self::new(settings)?;
        let remote_was = self.setting("GalacticCodexRemote");
        self.settings = replacement.settings;
        if self.setting("GalacticCodexRemote") && !remote_was {
            self.http_requests.extend(replacement.http_requests);
        }
        if !self.setting("GalacticCodexRemote") {
            self.http_requests.clear();
        }
        self.predictions_cache
            .get_mut()
            .expect("prediction cache")
            .clear();
        self.alert_cache.get_mut().expect("alert cache").clear();
        *self.browser_cache.get_mut().expect("browser cache") = None;
        self.last_body_alert = None;
        self.register_overlay_colours();
        Ok(())
    }
    pub fn reset(&mut self) {
        *self.browser_cache.get_mut().expect("browser cache") = None;
        self.alert_cache.get_mut().expect("alert cache").clear();
        self.store_commander();
        self.systems.clear();
        self.current_system = 0;
        self.current_body = None;
        self.active_sample = None;
        self.position = None;
        self.surface_body.clear();
        self.radius = 0.0;
        self.timestamp.clear();
        self.next_star_hint.clear();
        self.messages.clear();
        self.diagnostics.clear();
        self.commander.clear();
        self.commander_name.clear();
        self.presentation_order.clear();
        self.vault_unsold.clear();
        self.vault_total = None;
        self.pending_web.clear();
        self.gui_focus = 0;
        self.alert_body = None;
        self.last_body_alert = None;
        self.announced.clear();
        self.notification_renderings.clear();
        self.distance_notified = false;
        self.predictions_cache
            .get_mut()
            .expect("prediction cache")
            .clear();
    }
    pub fn set_batch(&mut self, batch: bool) {
        let finished = self.batch && !batch;
        self.batch = batch;
        if finished {
            if !self.hecate_nag_skipped && !self.seen_plugins.contains_key("Hecate") {
                self.browser_route = vec!["hecatenag".into()];
                *self.browser_cache.get_mut().expect("browser cache") = None;
            }
            self.resume_distance_cache();
            self.request_system_knowledge(&self.timestamp.clone());
            self.request_unsold_total();
            self.register_overlay_colours();
            self.message("Hecate", "WebServerRegisterPlugin", json!({}));
        }
    }
    fn setting(&self, key: &str) -> bool {
        self.settings
            .get(key)
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }
    fn system_id(&self, event: &JournalEvent) -> u64 {
        event.id("SystemAddress").unwrap_or(self.current_system)
    }
    fn body_id(&self, event: &JournalEvent, system: u64) -> Option<u64> {
        event
            .id("BodyID")
            .or_else(|| {
                let body = event.str("BodyName");
                self.systems
                    .get(&system)?
                    .bodies
                    .iter()
                    .find(|(_, b)| text(&b.scan, "BodyName") == body && !body.is_empty())
                    .map(|(id, _)| *id)
            })
            .or(self
                .current_body
                .filter(|_| event.str("BodyName").is_empty()))
    }
    fn finding(&self, system: u64, body: u64, title: String, detail: String) -> Finding {
        let sys = self.systems.get(&system);
        Finding {
            plugin: "BioInsights".to_owned(),
            timestamp: self.timestamp.clone(),
            system: sys.map(|s| s.name.clone()).unwrap_or_default(),
            body: sys
                .and_then(|s| s.bodies.get(&body))
                .map(|b| text(&b.scan, "BodyName").to_owned())
                .unwrap_or_default(),
            title,
            detail,
        }
    }
    fn message(&mut self, target: &str, command: &str, payload: Value) {
        if target == "StellarOverlay"
            && command.starts_with("Radar")
            && command != "RadarHide"
            && !self.setting("StellarOverlayUseRadar")
        {
            return;
        }
        // Replay suppresses presentation output, while downstream analysis and
        // vault mutations must see the same state transitions as live reading.
        if !self.batch
            || target == "Hecate" && command.starts_with("BioInsights")
            || target == "Evaluator" && command == "SetBioInsightsFlags"
        {
            self.messages
                .push(json!({"target":target,"command":command,"payload":payload}));
        }
    }
    pub fn take_messages(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.messages)
    }
    /// User marks and personal Codex are separate from journal-derived rows and
    /// Hecate's unsold biology ledger. The host writes this snapshot atomically.
    pub fn state(&self) -> Value {
        let mut commanders = self.commander_states.clone();
        if !self.commander.is_empty() {
            commanders.insert(self.commander.clone(), self.commander_snapshot());
        }
        json!({"version":2,"CurrentCommander":self.commander,"Commanders":commanders,"MarkedVariants":self.marked_variants,"PersonalCodex":self.personal_codex,"GalacticCodex":self.galactic.data})
    }
    /// Full installed catalogue for the in-place Codex Companion pages.
    pub fn codex_companion(&self) -> Vec<Value> {
        companion::project(self)
    }
    /// Typed rows for the native unsold-data dialog; the vault remains owner.
    pub fn unsold_data(&self) -> Vec<Value> {
        self.vault_unsold.iter().map(|row| {
            let species=row.get(3).and_then(Value::as_u64).and_then(|id|rules::codex().genus.values().find_map(|g|g.species.keys().find(|s|species_number(s)==id))).map(String::as_str).unwrap_or("");
            let (g,s)=rules::species_key(species).unwrap_or(("",""));let base=row.get(4).and_then(Value::as_u64).unwrap_or(0);let bonus=row.get(5).and_then(Value::as_bool).unwrap_or(true);
            json!({"ID":row.get(0),"DateTime":row.get(1),"BodyName":row.get(2),"Genus":name(g),"Species":name(s),"FullName":companion::full_name(g,s),"SpeciesID":species,"BaseValue":base,"FirstFootfall":bonus,"Value":base*if bonus{5}else{1}})
        }).collect()
    }
    pub fn tools_summary(&self) -> Value {
        let regions: Value = serde_json::from_str(include_str!("bioinsights/region-names.json"))
            .expect("installed region names");
        json!({"CodexCompanion":self.codex_companion(),"Unsold":self.unsold_data(),"Overlay":self.overlay(),"Regions":regions,"GalacticGenerated":self.galactic.data["Generated"],"OverlayDefaults":self.overlay_defaults,"ReadyPeers":self.seen_plugins,"BrowserRoute":self.browser_route(),"UIAlerts":self.ui_alerts()})
    }
    pub fn restore_state(&mut self, state: &Value) {
        *self.browser_cache.get_mut().expect("browser cache") = None;
        self.alert_cache.get_mut().expect("alert cache").clear();
        let mut scoped = false;
        if let Some(commanders) = state.get("Commanders").and_then(Value::as_object) {
            self.commander_states = commanders
                .iter()
                .filter(|(fid, v)| legacy::valid_fid(fid) && v.is_object())
                .map(|(fid, v)| (fid.clone(), v.clone()))
                .collect();
        }
        if let Some(fid) = state
            .get("CurrentCommander")
            .and_then(Value::as_str)
            .filter(|fid| legacy::valid_fid(fid))
        {
            self.commander = fid.to_owned();
            if let Some(saved) = self.commander_states.get(fid).cloned() {
                self.restore_commander(&saved);
                scoped = true;
            }
        }
        if let Some(data) = state.get("GalacticCodex").filter(|v| !v.is_null()) {
            if let Ok(galactic) = feed::Galactic::parse(data.clone()) {
                self.galactic = galactic;
            }
        }
        if let Some(marks) = state.get("MarkedVariants").filter(|_| !scoped) {
            if let Ok(marks) = serde_json::from_value::<BTreeSet<(String, String)>>(marks.clone()) {
                self.marked_variants = marks
                    .into_iter()
                    .filter(|(s, _)| rules::species_key(s).is_some())
                    .collect();
            }
        }
        if let Some(entries) = state.get("PersonalCodex").filter(|_| !scoped) {
            if let Ok(entries) =
                serde_json::from_value::<BTreeSet<(u64, String, String)>>(entries.clone())
            {
                self.personal_codex = entries
                    .into_iter()
                    .filter(|(_, s, _)| rules::species_key(s).is_some())
                    .collect();
            }
        }
    }
    fn is_marked(&self, species: &str, variant: &str) -> bool {
        self.marked_variants
            .contains(&(species.to_owned(), String::new()))
            || self
                .marked_variants
                .contains(&(species.to_owned(), variant.to_owned()))
    }
    fn has_personal_codex_prediction(&self, system: u64, predictions: &[Predicted]) -> bool {
        let regions = self
            .systems
            .get(&system)
            .and_then(|s| s.position)
            .map(rules::regions_at)
            .unwrap_or(0);
        predictions.iter().filter(|p| p.region_matches).any(|p| {
            p.variants.iter().any(|(variant, _, _, _)| {
                let any = !self
                    .personal_codex
                    .iter()
                    .any(|(_, species, colour)| species == &p.species && colour == variant);
                let current = !self.personal_codex.iter().any(|(region, species, colour)| {
                    *region < 64
                        && regions & (1 << region) != 0
                        && species == &p.species
                        && colour == variant
                });
                self.setting("PersonalCodexPossibleAny") && any
                    || self.setting("PersonalCodexPossibleCurrent") && current
            })
        })
    }

    pub fn on_event(&mut self, event: &JournalEvent) -> Vec<Finding> {
        *self.browser_cache.get_mut().expect("browser cache") = None;
        let prior_system = self.current_system;
        self.alert_cache.get_mut().expect("alert cache").clear();
        self.timestamp = event.timestamp.clone();
        let invalidated = self.system_id(event);
        let previous_radiant = if event.event == "Scan" && !self.batch && self.notifications_allowed
        {
            self.radiant_stars(invalidated)
        } else {
            BTreeMap::new()
        };
        self.predictions_cache
            .get_mut()
            .expect("prediction cache")
            .retain(|(system, _), _| *system != invalidated);
        let mut findings = Vec::new();
        match event.event.as_str() {
            "LoadGame" => {
                let name = if event.str("Commander").is_empty() {
                    event.str("Name")
                } else {
                    event.str("Commander")
                };
                if !name.is_empty() {
                    self.commander_name = name.to_owned();
                }
                if !event.str("FID").is_empty() {
                    self.activate_commander(event.str("FID"));
                }
                if !self.commander.is_empty() {
                    self.message(
                        "Hecate",
                        "BioInsightsGetData",
                        json!({"FID":self.commander}),
                    );
                    self.request_unsold_total();
                }
            }
            "Location" | "FSDJump" | "CarrierJump" => {
                let id = self.system_id(event);
                if id != self.current_system {
                    self.expire_ui_alerts();
                }
                if id != self.current_system || !self.presentation_order.contains(&id) {
                    self.presentation_order.retain(|system| *system != id);
                    self.presentation_order.push(id);
                }
                if id != self.current_system {
                    self.position = None;
                    self.current_body = None;
                    self.active_sample = None;
                    self.alert_body = None;
                }
                self.current_system = id;
                let sys = self.systems.entry(id).or_default();
                if !self.next_star_hint.is_empty() {
                    sys.star_hint = std::mem::take(&mut self.next_star_hint);
                }
                sys.population = event.data["Population"].as_u64().unwrap_or(0);
                if !event.str("StarSystem").is_empty() {
                    sys.name = event.str("StarSystem").to_owned();
                }
                if let Some(p) = event
                    .data
                    .get("StarPos")
                    .and_then(Value::as_array)
                    .filter(|p| p.len() == 3)
                {
                    if let (Some(x), Some(y), Some(z)) =
                        (p[0].as_f64(), p[1].as_f64(), p[2].as_f64())
                    {
                        sys.position = Some([x, y, z]);
                    }
                }
                if event.str("BodyType") == "Planet" {
                    self.current_body = event.id("BodyID");
                    self.surface_body = event.str("Body").to_owned();
                }
                let max = self
                    .settings
                    .get("MaximumSystemsReadAll")
                    .and_then(Value::as_u64)
                    .or(Some(100))
                    .filter(|n| *n > 0)
                    .unwrap_or(1_000_000) as usize;
                while self.systems.len() > max {
                    if let Some(old) = self.presentation_order.iter().copied().find(|k| {
                        *k != id
                            && !self.systems[k]
                                .bodies
                                .values()
                                .any(|b| b.observed.values().any(|s| s.unsold))
                    }) {
                        self.systems.remove(&old);
                        self.presentation_order.retain(|id| *id != old);
                    } else {
                        break;
                    }
                }
                if !self.batch {
                    self.request_system_knowledge(&event.timestamp);
                }
            }
            "StartJump" => {
                self.next_star_hint = event.str("StarClass").to_owned();
                self.position = None;
                self.current_body = None;
                self.active_sample = None;
                self.message("StellarOverlay", "RadarHide", json!({}));
            }
            "Scan" | "ScanBaryCentre" => {
                let id = self.system_id(event);
                if let Some(body) = event.id("BodyID") {
                    let sys = self.systems.entry(id).or_default();
                    if sys.name.is_empty() && !event.str("StarSystem").is_empty() {
                        sys.name = event.str("StarSystem").to_owned();
                    }
                    let state = sys.bodies.entry(body).or_default();
                    let was_mapped = state
                        .scan
                        .get("WasMapped")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    state.scan = event.data.clone();
                    if was_mapped {
                        state.scan["WasMapped"] = json!(true);
                    }
                    if state
                        .scan
                        .get("BioSignals")
                        .and_then(Value::as_u64)
                        .is_some()
                    {
                        state.bio_count = state.scan["BioSignals"].as_u64();
                    }
                    // New stellar or companion scans change system-wide and parent
                    // requirements, so predictions are calculated from current data.
                }
            }
            "FSSBodySignals" | "SAASignalsFound" => {
                let id = self.system_id(event);
                if let Some(body) = self.body_id(event, id) {
                    if id == self.current_system {
                        self.alert_body = Some(body);
                    }
                    let sys = self.systems.entry(id).or_default();
                    let state = sys.bodies.entry(body).or_default();
                    if !event.str("BodyName").is_empty() {
                        state.scan["BodyName"] = json!(event.str("BodyName"));
                    }
                    state.scan["BodyID"] = json!(body);
                    if let Some(signals) = event.data.get("Signals").and_then(Value::as_array) {
                        state.bio_count = Some(
                            signals
                                .iter()
                                .find(|s| compact(text(s, "Type")).contains("biological"))
                                .and_then(|bio| bio.get("Count"))
                                .and_then(Value::as_u64)
                                .unwrap_or(0),
                        );
                    }
                    if let Some(genera) = event.data.get("Genuses").and_then(Value::as_array) {
                        state.genera = genera
                            .iter()
                            .filter_map(|g| rules::genus_key(text(g, "Genus")).map(str::to_owned))
                            .collect();
                        state.mapped = true;
                    }
                    self.publish_body(id, body);
                }
            }
            "SAAScanComplete" => {
                let id = self.system_id(event);
                if let Some(body) = self.body_id(event, id) {
                    self.systems
                        .entry(id)
                        .or_default()
                        .bodies
                        .entry(body)
                        .or_default()
                        .mapped = true;
                    self.publish_body(id, body);
                }
            }
            "FSSAllBodiesFound" => {
                let id = self.system_id(event);
                self.systems.entry(id).or_default().complete = true;
            }
            "SupercruiseExit" | "ApproachBody" | "Touchdown" | "Disembark"
                if event.event != "Disembark"
                    || event.boolean("OnPlanet") && !event.boolean("OnStation") =>
            {
                let id = self.system_id(event);
                let body = self.body_id(event, id);
                if let Some(body) = body {
                    self.current_system = id;
                    self.current_body = Some(body);
                    if !event.str("BodyName").is_empty() {
                        self.surface_body = event.str("BodyName").to_owned();
                    }
                    let sys = self.systems.entry(id).or_default();
                    let state = sys.bodies.entry(body).or_default();
                    if !self.surface_body.is_empty() {
                        state.scan["BodyName"] = json!(self.surface_body);
                    }
                    state.visited = true;
                    if event.event == "Disembark"
                        && event.boolean("OnPlanet")
                        && !state
                            .scan
                            .get("WasFootfalled")
                            .and_then(Value::as_bool)
                            .unwrap_or(true)
                    {
                        state.footfall = true;
                    }
                    if let Some(first) = event.data.get("FirstFootfall").and_then(Value::as_bool) {
                        state.footfall = first;
                    }
                    let radius = number(&state.scan, "Radius");
                    if radius > 0.0 {
                        self.radius = radius;
                    }
                    self.update_position(&event.data);
                }
            }
            "LeaveBody" | "SupercruiseEntry" => {
                self.current_body = None;
                self.position = None;
                self.surface_body.clear();
            }
            "ScanOrganic" => findings.extend(self.organic(event)),
            "CodexEntry" => findings.extend(self.codex_entry(event)),
            "SellOrganicData" => {
                self.vault_unsold.clear();
                self.vault_total = Some(0);
                if let Some(bio) = event
                    .data
                    .get("BioData")
                    .and_then(Value::as_array)
                    .filter(|a| !a.is_empty())
                {
                    let species: BTreeSet<_> = bio
                        .iter()
                        .filter_map(|b| {
                            rules::species_key(text(b, "Species")).map(|(_, s)| s.to_owned())
                        })
                        .collect();
                    for sys in self.systems.values_mut() {
                        for b in sys.bodies.values_mut() {
                            for (key, s) in &mut b.observed {
                                if species.contains(key) {
                                    s.unsold = false;
                                }
                            }
                        }
                    }
                } else {
                    for sys in self.systems.values_mut() {
                        for b in sys.bodies.values_mut() {
                            for s in b.observed.values_mut() {
                                s.unsold = false;
                            }
                        }
                    }
                }
                if !self.commander.is_empty() {
                    self.message(
                        "Hecate",
                        "BioInsightsUnsoldDelete",
                        json!({"FID":self.commander}),
                    );
                }
            }
            "Died" => {
                self.vault_unsold.clear();
                self.vault_total = Some(0);
                for sys in self.systems.values_mut() {
                    for b in sys.bodies.values_mut() {
                        for s in b.observed.values_mut() {
                            s.unsold = false;
                        }
                    }
                }
                self.active_sample = None;
                if !self.commander.is_empty() {
                    self.message(
                        "Hecate",
                        "BioInsightsUnsoldDelete",
                        json!({"FID":self.commander}),
                    );
                }
            }
            "Shutdown" => {
                self.active_sample = None;
                self.position = None;
            }
            _ => {}
        }
        if matches!(
            event.event.as_str(),
            "Scan" | "FSSBodySignals" | "SAASignalsFound"
        ) {
            if let Some(body) = self.body_id(event, invalidated) {
                findings.extend(self.prediction_notifications(invalidated, body));
            }
        }
        if event.event == "Scan" {
            findings.extend(self.radiant_notifications(invalidated, previous_radiant));
        }
        self.sync_overlay();
        self.radar_event(event, prior_system);
        if event.event == "ScanOrganic" && !self.batch && self.notifications_allowed {
            self.cache_distance_sample();
        }
        if event.event == "ScanOrganic" && event.str("ScanType") == "Analyse" {
            self.request_unsold_total();
        }
        self.capture_diagnostic(event);
        findings
    }
    /// Local log entries are drained by the host, away from UI rendering.
    /// DebugBody selects a scan case-insensitively, as ProcessBScan does.
    /// SendDebugData does not submit local logs: the installed submission path
    /// accepts a separate ErrorReport queue, which is empty in this version.
    pub fn take_diagnostics(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.diagnostics)
    }
    fn capture_diagnostic(&mut self, event: &JournalEvent) {
        let selected = text(&self.settings, "DebugBody");
        let debug = event.event == "Scan"
            && !selected.is_empty()
            && selected.eq_ignore_ascii_case(event.str("BodyName"));
        if !debug && !self.setting("VerboseLogging") {
            return;
        }
        let mut entry = json!({"Timestamp":event.timestamp,"Level":if debug{"Debug"}else{"Verbose"},"Operation":event.event,"Event":event.data});
        if debug {
            if let Some(body) = self.body_id(event, self.system_id(event)) {
                entry["Predictions"] = json!(self
                    .predictions(self.system_id(event), body)
                    .iter()
                    .map(prediction_json)
                    .collect::<Vec<_>>());
                entry["Summary"] = self.summary(self.system_id(event), body);
            }
        }
        if self.diagnostics.len() == 256 {
            self.diagnostics.remove(0);
        }
        self.diagnostics.push(entry);
    }
    fn update_position(&mut self, v: &Value) {
        if let (Some(lat), Some(lon)) = (
            v.get("Latitude").and_then(Value::as_f64),
            v.get("Longitude").and_then(Value::as_f64),
        ) {
            if lat.is_finite() && lon.is_finite() && lat.abs() <= 90.0 && lon.abs() <= 180.0 {
                self.position = Some([lat, lon]);
            }
        }
    }
    pub fn on_status(&mut self, status: &Value) -> Vec<Finding> {
        self.gui_focus = status.get("GuiFocus").and_then(Value::as_u64).unwrap_or(0);
        self.position = None;
        self.update_position(status);
        if let Some(radius) = status
            .get("PlanetRadius")
            .and_then(Value::as_f64)
            .filter(|r| *r > 0.0)
        {
            self.radius = radius;
        }
        if let Some(body) = status.get("BodyName").and_then(Value::as_str) {
            if body != self.surface_body {
                *self.browser_cache.get_mut().expect("browser cache") = None;
                self.surface_body = body.to_owned();
                self.current_body = self.systems.get(&self.current_system).and_then(|s| {
                    s.bodies
                        .iter()
                        .find(|(_, b)| text(&b.scan, "BodyName") == body)
                        .map(|(id, _)| *id)
                });
            }
        }
        self.sync_overlay();
        self.distance_notification().into_iter().collect()
    }
    fn predictions(&self, system: u64, body: u64) -> Vec<Predicted> {
        let Some(state) = self.systems.get(&system).and_then(|s| s.bodies.get(&body)) else {
            return Vec::new();
        };
        self.all_predictions(system, body)
            .into_iter()
            .filter(|p| eligible(state, p))
            .collect()
    }
    fn all_predictions(&self, system: u64, body: u64) -> Vec<Predicted> {
        if let Some(cached) = self
            .predictions_cache
            .lock()
            .expect("prediction cache")
            .get(&(system, body))
        {
            return cached.clone();
        }
        let Some(sys) = self.systems.get(&system) else {
            return Vec::new();
        };
        let Some(state) = sys.bodies.get(&body) else {
            return Vec::new();
        };
        if !text(&state.scan, "StarType").is_empty() || state.scan.get("PlanetClass").is_none() {
            return Vec::new();
        }
        let bodies = sys.rule_bodies();
        let regions = sys.position.map(rules::regions_at).unwrap_or(0);
        let spheres = sys.position.map(rules::spheres_at).unwrap_or_default();
        let context = rules::Context {
            scan: &state.scan,
            bodies: &bodies,
            regions,
            spheres: &spheres,
            complete: sys.complete,
        };
        let predictions: Vec<_> = context
            .predict(self.setting("ShowHorizons"))
            .into_iter()
            .filter(|p| {
                (state.bio_count != Some(0) || !state.observed.is_empty())
                    && (self.setting("ShowUnlikely") || p.region_matches)
                    && p.value
                        >= self
                            .settings
                            .get("MinimumValue")
                            .and_then(Value::as_u64)
                            .unwrap_or(0)
            })
            .collect();
        self.predictions_cache
            .lock()
            .expect("prediction cache")
            .insert((system, body), predictions.clone());
        predictions
    }
    fn organic(&mut self, event: &JournalEvent) -> Vec<Finding> {
        let system = self.system_id(event);
        let Some(body) = self.body_id(event, system) else {
            return Vec::new();
        };
        let Some((genus_key, species_key)) = rules::species_key(event.str("Species")) else {
            return Vec::new();
        };
        self.current_system = system;
        self.current_body = Some(body);
        self.update_position(&event.data);
        let genus = &rules::codex().genus[genus_key];
        let species = &genus.species[species_key];
        let kind = event.str("ScanType");
        let position = self.position;
        let sys = self.systems.entry(system).or_default();
        let b = sys.bodies.entry(body).or_default();
        b.genera.insert(genus_key.to_owned());
        let state = b.observed.entry(species_key.to_owned()).or_default();
        state.seen = true;
        state.value = species.base_value;
        let variant = event.str("Variant_Localised");
        state.variant = if !variant.is_empty() {
            variant.rsplit(" - ").next().unwrap_or(variant).to_owned()
        } else {
            variant_colour(event.str("Variant"))
        };
        let mut abandoned = None;
        match kind {
            "Log" => {
                let next = (system, body, species_key.to_owned());
                if self
                    .active_sample
                    .as_ref()
                    .is_some_and(|active| active != &next)
                {
                    abandoned = self.active_sample.take();
                }
                if !state.analysed {
                    state.samples.clear();
                    state.sample_count = 1;
                    if let Some(p) = position {
                        state.samples.push(p);
                    }
                }
                self.active_sample = Some(next);
            }
            "Sample" => {
                if !state.analysed {
                    state.sample_count = (state.sample_count + 1).clamp(2, 2);
                    if let Some(p) = position {
                        if !state.samples.contains(&p) {
                            state.samples.push(p);
                        }
                    }
                }
                self.active_sample = Some((system, body, species_key.to_owned()));
            }
            "Analyse" => {
                state.sample_count = 3;
                state.analysed = true;
                state.unsold = true;
                self.active_sample = None;
            }
            _ => {}
        }
        if let Some((s, b, key)) = abandoned {
            if let Some(state) = self
                .systems
                .get_mut(&s)
                .and_then(|s| s.bodies.get_mut(&b))
                .and_then(|b| b.observed.get_mut(&key))
            {
                state.sample_count = 0;
                state.samples.clear();
            }
        }
        let label = format!("{} {}", name(genus_key), name(species_key));
        if kind == "Analyse" && !self.commander.is_empty() {
            let body_name = self
                .systems
                .get(&system)
                .and_then(|s| s.bodies.get(&body))
                .map(|b| text(&b.scan, "BodyName").to_owned())
                .unwrap_or_default();
            self.message("Hecate","BioInsightsUnsoldAdd",json!({"FID":self.commander,"ID64":system,"BodyID":body,"BioName":label,"DateTime":event.timestamp,"Species":species_number(species_key),"Value":species.base_value,"BodyName":body_name}));
            if !self.batch {
                self.message(
                    "Hecate",
                    "BioInsightsGetData",
                    json!({"FID":self.commander}),
                );
            }
        }
        self.message(
            "StellarOverlay",
            "RadarSetContextID",
            json!({"ContextID":genus_number(genus_key),"Radius":genus.minimum_sample_distance,"SampleCount":if kind=="Sample"{2}else if kind=="Analyse"{3}else{1}}),
        );
        self.message("StellarOverlay","RadarAddBiological",json!({"UseCurrentPosition":true,"ContextID":genus_number(genus_key),"Radius":genus.minimum_sample_distance,"Label":label,"MeasureDistance":kind!="Analyse"}));
        self.publish_body(system, body);
        self.distance_notified = false;
        let mut findings: Vec<_> = self
            .confirmed_species_notification(system, body, species_key)
            .into_iter()
            .collect();
        findings.extend(self.sample_notification(system, body, kind, species_key));
        findings
    }
    fn codex_entry(&mut self, event: &JournalEvent) -> Vec<Finding> {
        let system = self.system_id(event);
        let Some(body) = self.body_id(event, system) else {
            return Vec::new();
        };
        let raw = event.str("Name");
        let matched = rules::species_key(raw)
            .or_else(|| {
                let token = rules::token(raw);
                rules::codex().genus.iter().find_map(|(g, genus)| {
                    genus
                        .species
                        .keys()
                        .find(|s| {
                            token.starts_with(&rules::token(s).trim_end_matches("_name").to_owned())
                        })
                        .map(|s| (g.as_str(), s.as_str()))
                })
            })
            .or_else(|| {
                let id = event.id("EntryID")?.to_string();
                let species_id = *rules::codex().entry_id_species.get(&id)?;
                let ids: BTreeMap<String, u64> =
                    serde_json::from_str(include_str!("bioinsights/species-ids.json")).ok()?;
                let key = ids.iter().find(|(_, v)| **v == species_id)?.0;
                rules::species_key(key)
            });
        if let Some((genus_key, key)) = matched {
            let state = self
                .systems
                .entry(system)
                .or_default()
                .bodies
                .entry(body)
                .or_default();
            state.genera.insert(genus_key.to_owned());
            let observed = state.observed.entry(key.to_owned()).or_default();
            observed.seen = true;
            observed.value = rules::codex().genus[genus_key].species[key].base_value;
            observed.variant = variant_colour(raw);
            self.personal_codex.insert((
                event.id("RegionID").unwrap_or(0),
                key.to_owned(),
                observed.variant.clone(),
            ));
            return self
                .confirmed_species_notification(system, body, key)
                .into_iter()
                .collect();
        }
        Vec::new()
    }
    fn radar_planet(&mut self, system: u64, body: u64) {
        if self.batch || system != self.current_system {
            return;
        }
        let Some(state) = self.systems.get(&system).and_then(|s| s.bodies.get(&body)) else {
            return;
        };
        if state.scan["Landable"] != true || state.bio_count.unwrap_or(0) == 0 {
            return;
        }
        let genera: BTreeMap<_, _> = self
            .systems
            .get(&system)
            .and_then(|s| s.bodies.get(&body))
            .map(|b| {
                b.genera
                    .iter()
                    .map(|g| (genus_number(g).to_string(), name(g)))
                    .collect()
            })
            .unwrap_or_default();
        self.message(
            "StellarOverlay",
            "RadarSetPlanet",
            json!({"ID64":system,"BodyID":body,"RadiusKM":self.radius/1000.0,"Genuses":genera}),
        );
        let ranges: BTreeMap<_, _> = rules::codex()
            .genus
            .iter()
            .map(|(g, v)| (genus_number(g).to_string(), v.minimum_sample_distance))
            .collect();
        self.message(
            "StellarOverlay",
            "RadarSetColonyRanges",
            json!({"Ranges":ranges}),
        );
    }
    fn publish_body(&mut self, system: u64, body: u64) {
        let summary = self.summary(system, body);
        let predictions = self.predictions(system, body);
        let high = self.meets_value_threshold(
            "MinBodyValueMillions",
            summary["BaseMinimumValue"].as_u64().unwrap_or(0),
        ) || predictions
            .iter()
            .any(|p| self.meets_value_threshold("MinIndividualBiologicalValueMillions", p.value));
        let marked = predictions.iter().any(|p| {
            p.variants
                .iter()
                .any(|(colour, _, _, _)| self.is_marked(&p.species, colour))
        });
        let personal = self.has_personal_codex_prediction(system, &predictions);
        let (regional, never) = self.galactic_flags(system, &predictions);
        self.message("Evaluator","SetBioInsightsFlags",json!({"ID64":system,"BodyID":body,"IsHighValue":high,"IsPCodex":personal,"IsMarked":marked,"IsGCodex":regional||never}));
    }
    pub fn body_predictions(&self, body: u64) -> Vec<Value> {
        self.predictions(self.current_system, body)
            .iter()
            .map(prediction_json)
            .collect()
    }
    pub fn body_summary(&self, body: u64) -> Value {
        self.summary(self.current_system, body)
    }
    fn summary(&self, system: u64, body: u64) -> Value {
        let Some(state) = self.systems.get(&system).and_then(|s| s.bodies.get(&body)) else {
            return Value::Null;
        };
        let predictions = self.predictions(system, body);
        let mut genus_values: BTreeMap<&str, (u64, u64)> = BTreeMap::new();
        for p in &predictions {
            if p.region_matches {
                let range = genus_values.entry(&p.genus).or_insert((u64::MAX, 0));
                range.0 = range.0.min(p.value);
                range.1 = range.1.max(p.value);
            }
        }
        for (key, observed) in &state.observed {
            if let Some((g, _)) = rules::species_key(key) {
                genus_values.insert(g, (observed.value, observed.value));
            }
        }
        let count = state.bio_count.map(|n| n as usize).unwrap_or(usize::MAX);
        let mut minimum: Vec<_> = genus_values.values().map(|r| r.0).collect();
        minimum.sort_unstable();
        let mut maximum: Vec<_> = genus_values.values().map(|r| r.1).collect();
        maximum.sort_unstable_by(|a, b| b.cmp(a));
        let base_min: u64 = minimum.into_iter().take(count).sum();
        let base_max: u64 = maximum.into_iter().take(count).sum();
        let multiplier = if state.footfall { 5 } else { 1 };
        let analysed: u64 = state
            .observed
            .values()
            .filter(|s| s.analysed)
            .map(|s| s.value * multiplier)
            .sum();
        let unsold: u64 = state
            .observed
            .values()
            .filter(|s| s.unsold)
            .map(|s| s.value * multiplier)
            .sum();
        let (regional, never) = self.galactic_flags(system, &predictions);
        json!({"SystemAddress":system,"ID64":system,"BodyID":body,"BodyName":text(&state.scan,"BodyName"),"Signals":state.bio_count,"Mapped":state.mapped,"Genuses":state.genera.iter().map(|g|name(g)).collect::<Vec<_>>(),"FirstFootfall":state.footfall,"WasFootfalled":state.scan.get("WasFootfalled").and_then(Value::as_bool),"MinimumValue":base_min*multiplier,"MaximumValue":base_max*multiplier,"BaseMinimumValue":base_min,"BaseMaximumValue":base_max,"AnalysedValue":analysed,"UnsoldValue":unsold,"Predictions":predictions.iter().map(prediction_json).collect::<Vec<_>>(),"BIOFlags":if state.footfall{"FF"}else{""},"IsGCodex":regional||never,"IsGalacticCodexEntry":regional,"IsGalacticCodexEntryNeverSeen":never,"GalacticGenerated":self.galactic.data["Generated"]})
    }
    pub fn rows(&self) -> Vec<Value> {
        let mut rows = Vec::new();
        for (system, sys) in &self.systems {
            for (body, state) in &sys.bodies {
                if self.setting("HideAutoscans") && text(&state.scan, "ScanType") == "AutoScan" {
                    continue;
                }
                let mut predictions = self.all_predictions(*system, *body);
                for key in state.observed.keys() {
                    if !predictions.iter().any(|p| &p.species == key) {
                        if let Some((g, s)) = rules::species_key(key) {
                            let genus = &rules::codex().genus[g];
                            let species = &genus.species[s];
                            predictions.push(Predicted {
                                genus: g.to_owned(),
                                species: s.to_owned(),
                                value: species.base_value,
                                distance: genus.minimum_sample_distance,
                                horizons: genus.is_horizon,
                                variants: Vec::new(),
                                region_matches: true,
                                provisional: false,
                            });
                        }
                    }
                }
                for p in predictions {
                    let observed = state.observed.get(&p.species);
                    let analysed = observed.is_some_and(|s| s.analysed);
                    let seen = observed.is_some_and(|s| s.seen);
                    let ruled_out = !eligible(state, &p);
                    if ruled_out && self.setting("HideNotPresent") {
                        continue;
                    }
                    let variants = observed
                        .filter(|s| !s.variant.is_empty())
                        .map(|s| s.variant.clone())
                        .unwrap_or_else(|| {
                            p.variants
                                .iter()
                                .map(|v| v.0.clone())
                                .collect::<Vec<_>>()
                                .join(", ")
                        });
                    let sample_count = observed.map(|s| s.sample_count).unwrap_or(0);
                    let marked = observed.is_some_and(|s| s.marked)
                        || self.is_marked(&p.species, "")
                        || p.variants
                            .iter()
                            .any(|(colour, _, _, _)| self.is_marked(&p.species, colour));
                    let (regional, never) = self.galactic_flags(*system, std::slice::from_ref(&p));
                    let flags = [
                        if state.footfall { "FF" } else { "" },
                        if !p.region_matches { "U" } else { "" },
                        if p.provisional { "?" } else { "" },
                        if marked { "M" } else { "" },
                        if never {
                            "GC!"
                        } else if regional {
                            "GC"
                        } else {
                            ""
                        },
                    ]
                    .into_iter()
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                    rows.push(json!({"System":sys.name,"SystemAddress":system,"BodyID":body,"BodyName":text(&state.scan,"BodyName"),"Body":text(&state.scan,"PlanetClass"),"Sigs":state.bio_count,"Signals":state.bio_count,"Flags":flags,"Genus":name(&p.genus),"GenusID":p.genus,"Species":name(&p.species),"SpeciesID":p.species,"Seen":if ruled_out{"Not present"}else if seen{"Seen"}else{""},"Samples":if analysed{"Complete".to_owned()}else if sample_count>0{format!("{sample_count}/3")}else{String::new()},"Type":if p.horizons{"Horizons"}else{"Odyssey"},"PossibleVariants":variants,"Variant":variants,"BaseValue":p.value,"Value":p.value*if state.footfall{5}else{1},"FirstFootfall":state.footfall,"Analysed":analysed,"Unsold":observed.is_some_and(|s|s.unsold),"Distance":number(&state.scan,"DistanceFromArrivalLS"),"MinimumSampleDistance":p.distance,"Grav":number(&state.scan,"SurfaceGravity")/9.798,"Temp":number(&state.scan,"SurfaceTemperature"),"Atmosphere":text(&state.scan,"Atmosphere"),"Pressure":number(&state.scan,"SurfacePressure")/101325.0,"Volcanism":text(&state.scan,"Volcanism"),"Likely":p.region_matches,"Provisional":p.provisional,"IsGCodex":regional||never,"IsGalacticCodexEntryNeverSeen":never}));
                }
            }
        }
        rows
    }
    pub fn overlay(&self) -> Value {
        let mut value = self
            .current_body
            .map(|body| self.body_summary(body))
            .filter(|v| !v.is_null())
            .unwrap_or_else(|| json!({}));
        let unsold: u64 = self
            .systems
            .values()
            .flat_map(|s| s.bodies.values())
            .map(|b| {
                b.observed
                    .values()
                    .filter(|s| s.unsold)
                    .map(|s| s.value * if b.footfall { 5 } else { 1 })
                    .sum::<u64>()
            })
            .sum();
        value["UnsoldTotal"] = json!(self.vault_total.unwrap_or(unsold));
        if let Some((system, body, species)) = &self.active_sample {
            if self.current_system == *system && self.current_body == Some(*body) {
                if let Some(s) = self
                    .systems
                    .get(system)
                    .and_then(|s| s.bodies.get(body))
                    .and_then(|b| b.observed.get(species))
                {
                    let genus = rules::species_key(species).map(|(g, _)| g).unwrap_or("");
                    let separation = rules::codex()
                        .genus
                        .get(genus)
                        .map(|g| g.minimum_sample_distance)
                        .unwrap_or(100);
                    let distances: Vec<_> = self
                        .position
                        .filter(|_| self.radius > 0.0)
                        .map(|position| {
                            s.samples
                                .iter()
                                .map(|sample| great_circle(position, *sample, self.radius))
                                .collect()
                        })
                        .unwrap_or_default();
                    value["Sampler"] = json!({"Species":format!("{} {}",name(genus),name(species)),"SpeciesID":species,"SampleCount":s.sample_count,"NextSample":(s.sample_count+1).min(3),"ColonyDistance":separation,"Distances":distances,"Ready":!distances.is_empty()&&distances.iter().all(|d|*d>=separation as f64)});
                }
            }
        }
        value
    }
    fn notification_style(&self, payload: &mut Value, dss: bool) {
        payload["SourceBitmap"] = json!("BioInsights");
        payload["AlertSettings"] =
            json!({"ThemeSettings":self.settings["ThemeSettings"],"Theme":self.settings["Theme"]});
        let integer = |key: &str, default: u64| {
            self.settings
                .get(key)
                .and_then(Value::as_u64)
                .unwrap_or(default)
        };
        payload["XPos"] = json!(integer(
            if dss {
                "DSSNotificationXPos"
            } else {
                "FSSNotificationXPos"
            },
            2
        ));
        payload["YPos"] = json!(integer(
            if dss {
                "DSSNotificationYPos"
            } else {
                "FSSNotificationYPos"
            },
            if dss { 98 } else { 2 }
        ));
        payload["TTLms"] = json!(if dss {
            86_400_000
        } else {
            integer("FSSNotificationTTL", 6000)
        });
        payload["Opacity"] = json!(integer("FSSNotificationOpacity", 80));
        payload["FontSize"] = json!(integer("FSSNotificationFontSize", 12));
        payload["BorderWidth"] = json!(integer("FSSNotificationBorderWidth", 2));
        payload["XPadding"] = json!(3);
        payload["YPadding"] = json!(3);
        payload["EntryTransition"] = json!(2);
        payload["ExitTransition"] = json!(2);
        payload["ForceShow"] = json!(false);
        if self.setting("UseCustomNotificationColours") {
            for key in [
                "ColorHex",
                "TitleColorHex",
                "BorderColorHex",
                "BackgroundColorHex",
            ] {
                payload[key] = self.settings[key].clone();
            }
        } else {
            let original = self.settings["ThemeSettings"]["ThemeIndex"]
                .as_u64()
                .map(|index| index == 0)
                .unwrap_or(self.settings["Theme"] == "Original");
            let source = if original {
                include_str!("bioinsights/theme.json")
            } else {
                include_str!("bioinsights/theme-orange.json")
            };
            let theme: Value = serde_json::from_str(source).expect("original BioInsights theme");
            for (target, key) in [
                ("ColorHex", "DefaultText"),
                ("TitleColorHex", "TitleText"),
                ("BorderColorHex", "Border"),
                ("BackgroundColorHex", "DefaultBackground"),
            ] {
                let color = self.settings["ThemeSettings"]["Colors"]
                    .get(key)
                    .filter(|_| self.settings["ThemeSettings"]["ThemeMode"].as_u64() == Some(1))
                    .unwrap_or(&theme["Colors"][key]);
                payload[target] = json!(format!(
                    "FF{:02X}{:02X}{:02X}",
                    color["R"].as_u64().unwrap_or(0),
                    color["G"].as_u64().unwrap_or(0),
                    color["B"].as_u64().unwrap_or(0)
                ));
            }
        }
    }
    fn sync_overlay(&mut self) {
        if self.batch {
            return;
        }
        let body = if self.gui_focus == 9 {
            self.alert_body.or(self.current_body)
        } else {
            self.current_body.or(self.alert_body)
        };
        let mut body_panel = None;
        if (self.gui_focus == 9 && self.setting("StellarOverlaySendFSSNotification"))
            || (self.gui_focus == 10 && self.setting("StellarOverlaySendDSSNotification"))
        {
            if let Some(body) = body {
                let predictions = self.predictions(self.current_system, body);
                let mut panel = self.summary(self.current_system, body);
                let high = self.meets_value_threshold(
                    "MinBodyValueMillions",
                    panel["BaseMinimumValue"].as_u64().unwrap_or(0),
                ) || predictions.iter().any(|p| {
                    self.meets_value_threshold("MinIndividualBiologicalValueMillions", p.value)
                });
                let marked = predictions.iter().any(|p| {
                    p.variants
                        .iter()
                        .any(|(variant, _, _, _)| self.is_marked(&p.species, variant))
                });
                let (regional, never) = self.galactic_flags(self.current_system, &predictions);
                if !panel.is_null()
                    && !predictions.is_empty()
                    && (high
                        || marked
                        || self.has_personal_codex_prediction(self.current_system, &predictions)
                        || regional
                        || never)
                {
                    panel["Key"] = json!("BIFSSNoti");
                    panel["ImageKey"] = json!("BIFSSNoti");
                    panel["Title"] = panel["BodyName"].clone();
                    panel["VisualAlert"] = self.visual_alert(self.current_system, body);
                    self.notification_style(&mut panel, self.gui_focus == 10);
                    body_panel = Some(panel);
                }
            }
        }
        if body_panel != self.last_body_alert {
            if let Some(panel) = &body_panel {
                self.message("StellarOverlay", "ShowBitmapNotification", panel.clone());
            } else if self.last_body_alert.is_some() {
                self.message(
                    "StellarOverlay",
                    "HideBitmapNotification",
                    json!({"Key":"BIFSSNoti","ImageKey":"BIFSSNoti"}),
                );
            }
            self.last_body_alert = body_panel;
        }
    }
    /// Native UI commands and locally supplied Hecate scan results.
    pub fn on_message(&mut self, command: &str, payload: &Value) -> Vec<Finding> {
        *self.browser_cache.get_mut().expect("browser cache") = None;
        self.alert_cache.get_mut().expect("alert cache").clear();
        match command {
            "RegisterPluginCustomColoursResponse" => {
                if ["TitleColor", "TextColor", "BackgroundColor", "BorderColor"]
                    .into_iter()
                    .all(|k| payload[k].is_string())
                {
                    for (source, target) in [
                        ("TitleColor", "TitleColorHex"),
                        ("TextColor", "ColorHex"),
                        ("BackgroundColor", "BackgroundColorHex"),
                        ("BorderColor", "BorderColorHex"),
                    ] {
                        self.overlay_defaults[target] = payload[source].clone();
                    }
                }
            }
            "SourceUIAlert" => self.set_ui_alert(payload),
            "SourceReadAllState" => self.read_all_state(text(payload, "State")),
            "DismissUIAlert" => {
                if let Some(id) = payload["ID"].as_u64().and_then(|id| u8::try_from(id).ok()) {
                    self.dismiss_ui_alert(id);
                }
            }
            "InstallHecate" => self.message(
                "Core",
                "InstallNativePlugin",
                json!({"Plugin":"Hecate","Dependency":true}),
            ),
            "BrowserNavigate" => self.browser_navigate(payload),
            "SetServerAddress" => {
                let server = text(payload, "ServerAddress");
                let port = payload["ServerPort"]
                    .as_str()
                    .map(str::to_owned)
                    .or_else(|| payload["ServerPort"].as_u64().map(|n| n.to_string()))
                    .unwrap_or_default();
                if !server.is_empty() && !port.is_empty() {
                    self.web_server = format!("{server}:{port}");
                }
            }
            "LoadSystemResponse" | "SpanshSystemResponse" => {
                self.load_system_knowledge(command, payload)
            }
            "BioInsightsGetTotalResponse" => {
                if self.matches_fid(&payload["FID"]) {
                    if let Some(total) = payload["Total"].as_u64() {
                        self.vault_total = Some(total);
                    }
                }
            }
            "TestOverlay" => companion::overlay_test(self, payload),
            "WipeUnsoldData" if payload["Confirmed"] == true && !self.commander.is_empty() => {
                self.message(
                    "Hecate",
                    "BioInsightsUnsoldDelete",
                    json!({"FID":self.commander}),
                );
                self.vault_unsold.clear();
                for system in self.systems.values_mut() {
                    for body in system.bodies.values_mut() {
                        for species in body.observed.values_mut() {
                            species.unsold = false;
                        }
                    }
                }
                self.message(
                    "Hecate",
                    "BioInsightsGetData",
                    json!({"FID":self.commander}),
                );
                self.message(
                    "Hecate",
                    "BioInsightsGetTotal",
                    json!({"FID":self.commander}),
                );
            }
            "RefreshUnsold" | "ShowUnsold" => {
                if !self.commander.is_empty() {
                    self.message(
                        "Hecate",
                        "BioInsightsGetData",
                        json!({"FID":self.commander}),
                    );
                }
            }
            "SetBonus" => {
                if let Some(id) = payload.get("ID").and_then(Value::as_u64) {
                    let bonus = payload
                        .get("Value")
                        .and_then(Value::as_bool)
                        .unwrap_or(true);
                    self.message(
                        "Hecate",
                        "BioInsightsSetBonus",
                        json!({"ID":id,"Value":bonus}),
                    );
                    if !self.commander.is_empty() {
                        self.message(
                            "Hecate",
                            "BioInsightsGetData",
                            json!({"FID":self.commander}),
                        );
                    }
                }
            }
            "BioInsightsGetDataResponse" => {
                if let Some(data) = payload.get("Data").and_then(Value::as_array) {
                    self.vault_unsold = data.clone();
                    self.vault_total = Some(
                        data.iter()
                            .map(|r| {
                                r[4].as_u64().unwrap_or(0)
                                    * if r[5].as_bool().unwrap_or(true) { 5 } else { 1 }
                            })
                            .sum(),
                    );
                }
                let pending = std::mem::take(&mut self.pending_web);
                for (command, mut request) in pending {
                    request["_VaultReady"] = json!(true);
                    self.web_request(&command, &request);
                }
            }
            "WebRequest" | "WebSocketRequest" => {
                self.web_request(command, payload);
            }
            "SetZoomLevel" => {
                if let Some(zoom) = payload["ZoomLevel"].as_u64() {
                    self.settings["ZoomLevel"] = json!(zoom.clamp(50, 600));
                }
            }
            "VirtualScan" | "Scan" => {
                if let Ok(event) = JournalEvent::parse(&payload.to_string()) {
                    // Source SendNotification accepts current-commander journal
                    // events; cached/support scans still update the view quietly.
                    let previous = self.notifications_allowed;
                    self.notifications_allowed = false;
                    let findings = self.on_event(&event);
                    self.notifications_allowed = previous;
                    return findings;
                }
            }
            "MarkVariant" => {
                if let Some((_, species)) = rules::species_key(text(payload, "SpeciesID")) {
                    let variant = text(payload, "Variant").to_owned();
                    let key = (species.to_owned(), variant);
                    let marked = payload
                        .get("Marked")
                        .and_then(Value::as_bool)
                        .unwrap_or(!self.marked_variants.contains(&key));
                    if marked {
                        self.marked_variants.insert(key);
                    } else {
                        self.marked_variants.remove(&key);
                    }
                    let system = payload
                        .get("SystemAddress")
                        .and_then(Value::as_u64)
                        .unwrap_or(self.current_system);
                    if let Some(body) = payload
                        .get("BodyID")
                        .and_then(Value::as_u64)
                        .or(self.current_body)
                    {
                        self.publish_body(system, body);
                    }
                }
            }
            _ => {}
        }
        Vec::new()
    }
}
fn prediction_json(p: &Predicted) -> Value {
    json!({"Genus":name(&p.genus),"GenusID":p.genus,"Species":name(&p.species),"SpeciesID":p.species,"BaseValue":p.value,"MinimumSampleDistance":p.distance,"Horizons":p.horizons,"RegionMatches":p.region_matches,"Provisional":p.provisional,"Description":species_description(&p.genus,&p.species),"Variants":p.variants.iter().map(|(colour,reason,likely,show)|json!({"Colour":colour,"Reason":reason,"Likely":*likely==0,"Likelihood":match likely{0=>"Likely",1=>"Maybe",_=>"Unlikely"},"ShowInUI":show})).collect::<Vec<_>>()})
}
fn eligible(state: &BodyState, p: &Predicted) -> bool {
    (!state.mapped || state.genera.is_empty() || state.genera.contains(&p.genus))
        && !state.observed.iter().any(|(species, s)| {
            s.seen
                && *species != p.species
                && rules::species_key(species).is_some_and(|(genus, _)| genus == p.genus)
        })
}
fn species_number(species: &str) -> u64 {
    static IDS: std::sync::OnceLock<BTreeMap<String, u64>> = std::sync::OnceLock::new();
    *IDS.get_or_init(|| {
        serde_json::from_str(include_str!("bioinsights/species-ids.json"))
            .expect("bundled species IDs")
    })
    .get(species)
    .unwrap_or(&0)
}
fn genus_number(genus: &str) -> u8 {
    const IDS: [&str; 21] = [
        "Codex_Ent_Aleoids_Genus_Name",
        "Codex_Ent_Bacterial_Genus_Name",
        "Codex_Ent_Cactoid_Genus_Name",
        "Codex_Ent_Clypeus_Genus_Name",
        "Codex_Ent_Conchas_Genus_Name",
        "Codex_Ent_Electricae_Genus_Name",
        "Codex_Ent_Fonticulus_Genus_Name",
        "Codex_Ent_Shrubs_Genus_Name",
        "Codex_Ent_Fumerolas_Genus_Name",
        "Codex_Ent_Fungoids_Genus_Name",
        "Codex_Ent_Osseus_Genus_Name",
        "Codex_Ent_Recepta_Genus_Name",
        "Codex_Ent_Stratum_Genus_Name",
        "Codex_Ent_Tubus_Genus_Name",
        "Codex_Ent_Tussocks_Genus_Name",
        "Codex_Ent_Brancae_Name",
        "Codex_Ent_Ground_Struct_Ice_Name",
        "Codex_Ent_Vents_Name",
        "Codex_Ent_Cone_Name",
        "Codex_Ent_Sphere_Name",
        "Codex_Ent_Tube_Name",
    ];
    IDS.iter()
        .position(|g| *g == genus)
        .map(|n| n as u8 + 1)
        .unwrap_or(0)
}
fn species_description(genus: &str, species: &str) -> String {
    let label = format!("{} {}", name(genus), name(species));
    include_str!("bioinsights/species-description.tsv")
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .find(|(s, _)| *s == label)
        .map(|(_, d)| d.to_owned())
        .unwrap_or_default()
}
fn variant_colour(raw: &str) -> String {
    let raw = raw.trim_matches(['$', ';']);
    rules::COLOURS
        .iter()
        .find(|colour| {
            raw.to_ascii_lowercase()
                .ends_with(&format!("_{}", colour.to_ascii_lowercase()))
        })
        .map(|s| s.to_string())
        .unwrap_or_default()
}
fn great_circle([lat, lon]: [f64; 2], [lat2, lon2]: [f64; 2], radius: f64) -> f64 {
    let a = ((lat2 - lat).to_radians() / 2.0).sin().powi(2)
        + lat.to_radians().cos()
            * lat2.to_radians().cos()
            * ((lon2 - lon).to_radians() / 2.0).sin().powi(2);
    radius * 2.0 * a.clamp(0.0, 1.0).sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(value: Value) -> JournalEvent {
        JournalEvent::parse(&value.to_string()).unwrap()
    }
    fn planet() -> Value {
        json!({"event":"Scan","SystemAddress":42,"BodyID":1,"BodyName":"Test A 1","PlanetClass":"High metal content body","Atmosphere":"thin carbon dioxide atmosphere","AtmosphereType":"CarbonDioxide","AtmosphereComposition":[{"Name":"CarbonDioxide","Percent":100.0}],"SurfaceTemperature":180.0,"SurfaceGravity":1.0,"SurfacePressure":100.0,"MassEM":0.1,"Radius":1000000.0,"SemiMajorAxis":149597870700.0,"DistanceFromArrivalLS":499.0,"Parents":[{"Star":0}],"Volcanism":"","WasFootfalled":false})
    }
    fn worker() -> BioInsights {
        let mut w = BioInsights::new(json!({})).unwrap();
        w.on_event(&event(json!({"event":"Location","StarSystem":"Test","SystemAddress":42,"StarPos":[0.0,0.0,0.0]})));
        w.on_event(&event(json!({"event":"Scan","SystemAddress":42,"BodyID":0,"BodyName":"Test A","StarType":"F","SurfaceTemperature":6500.0,"Radius":695700000.0,"DistanceFromArrivalLS":0.0})));
        w.on_event(&event(planet()));
        w.on_event(&event(json!({"event":"FSSBodySignals","SystemAddress":42,"BodyID":1,"BodyName":"Test A 1","Signals":[{"Type":"$SAA_SignalType_Biological;","Count":2}]})));
        w
    }
    #[test]
    fn jump_star_hint_is_provisional_until_real_stellar_scan() {
        let mut bio = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        let event = |value: Value| JournalEvent::parse(&value.to_string()).unwrap();
        bio.on_event(&event(
            json!({"event":"StartJump","JumpType":"Hyperspace","StarClass":"M"}),
        ));
        bio.on_event(&event(
            json!({"event":"FSDJump","SystemAddress":42,"StarSystem":"Source","StarPos":[0,0,0]}),
        ));
        bio.on_event(&event(json!({"event":"Scan","SystemAddress":42,"BodyID":1,"BodyName":"Source 1","Parents":[{"Star":0}],"PlanetClass":"Rocky body","Landable":true,"AtmosphereType":"CarbonDioxide","Atmosphere":"thin carbon dioxide atmosphere","SurfaceTemperature":180,"SurfacePressure":1000,"SurfaceGravity":1,"SemiMajorAxis":149597870700.0})));
        let bacterium = |rows: Vec<Predicted>| {
            rows.into_iter()
                .find(|p| p.species == "Codex_Ent_Bacterial_01_Name")
                .unwrap()
        };
        let before = bacterium(bio.predictions(42, 1));
        assert_eq!(before.variants[0].0, "Teal");
        assert_eq!(before.variants[0].2, 1);
        bio.on_event(&event(json!({"event":"Scan","SystemAddress":42,"BodyID":0,"StarType":"G","SurfaceTemperature":5800,"Radius":700000000,"AbsoluteMagnitude":4.83})));
        let after = bacterium(bio.predictions(42, 1));
        assert_eq!(after.variants[0].0, "Emerald");
        assert_eq!(after.variants[0].2, 0);
    }
    #[test]
    fn installed_species_values_and_parent_colours_are_preserved() {
        let w = worker();
        let p = w.body_predictions(1);
        let tectonicas = p.iter().find(|p| p["Species"] == "Tectonicas").unwrap();
        assert_eq!(tectonicas["BaseValue"], 19_010_800);
        assert_eq!(tectonicas["MinimumSampleDistance"], 500);
        assert_eq!(tectonicas["Variants"][0]["Colour"], "Emerald");
        assert_eq!(tectonicas["Variants"][0]["Likely"], true);
        assert!(w
            .rows()
            .iter()
            .any(|r| r["BodyName"] == "Test A 1" && r["Species"] == "Tectonicas"));
    }
    #[test]
    fn environmental_bounds_and_dss_confirmed_genera_change_predictions() {
        let mut w = worker();
        let mut p = planet();
        p["SurfaceTemperature"] = json!(450.0);
        w.on_event(&event(p));
        assert!(!w
            .body_predictions(1)
            .iter()
            .any(|p| p["Species"] == "Tectonicas"));
        w.on_event(&event(planet()));
        w.on_event(&event(json!({"event":"SAASignalsFound","SystemAddress":42,"BodyID":1,"Signals":[{"Type":"$SAA_SignalType_Biological;","Count":1}],"Genuses":[{"Genus":"$Codex_Ent_Bacterial_Genus_Name;"}]})));
        assert!(w
            .body_predictions(1)
            .iter()
            .all(|p| p["Genus"] == "Bacterium"));
    }
    fn organic(kind: &str, species: &str) -> JournalEvent {
        event(
            json!({"event":"ScanOrganic","SystemAddress":42,"BodyID":1,"ScanType":kind,"Genus":"$Codex_Ent_Stratum_Genus_Name;","Species":species,"Variant":"$Codex_Ent_Stratum_07_Green;"}),
        )
    }
    #[test]
    fn samples_distance_first_footfall_unsold_and_sale_work_together() {
        let mut w = worker();
        w.on_event(&event(json!({"event":"Disembark","SystemAddress":42,"BodyID":1,"BodyName":"Test A 1","OnPlanet":true})));
        w.on_status(
            &json!({"BodyName":"Test A 1","Latitude":0.0,"Longitude":0.0,"PlanetRadius":1000000.0}),
        );
        w.on_event(&organic("Log", "$Codex_Ent_Stratum_07_Name;"));
        assert_eq!(w.overlay()["Sampler"]["SampleCount"], 1);
        assert_eq!(w.overlay()["Sampler"]["Ready"], false);
        w.on_status(&json!({"BodyName":"Test A 1","Latitude":0.04,"Longitude":0.0,"PlanetRadius":1000000.0}));
        assert_eq!(w.overlay()["Sampler"]["Ready"], true);
        w.on_event(&organic("Sample", "$Codex_Ent_Stratum_07_Name;"));
        assert_eq!(w.overlay()["Sampler"]["SampleCount"], 2);
        w.on_event(&organic("Analyse", "$Codex_Ent_Stratum_07_Name;"));
        assert!(w.overlay().get("Sampler").is_none());
        assert_eq!(w.overlay()["UnsoldTotal"], 95_054_000);
        assert!(w.rows().iter().any(|r| r["Species"] == "Tectonicas"
            && r["Samples"] == "Complete"
            && r["FirstFootfall"] == true));
        let msgs = w.take_messages();
        assert!(msgs.iter().any(|m| m["command"] == "RadarAddBiological"));
        w.on_event(&event(json!({"event":"SellOrganicData","BioData":[{"Species":"$Codex_Ent_Stratum_07_Name;","Value":19010800,"Bonus":76043200}]})));
        assert_eq!(w.overlay()["UnsoldTotal"], 0);
        assert!(w
            .rows()
            .iter()
            .any(|r| r["Species"] == "Tectonicas" && r["Analysed"] == true));
    }
    #[test]
    fn switching_species_discards_incomplete_samples_but_preserves_sightings() {
        let mut w = worker();
        w.on_event(&organic("Log", "$Codex_Ent_Stratum_07_Name;"));
        w.on_event(&organic("Log", "$Codex_Ent_Bacterial_01_Name;"));
        let row = w
            .rows()
            .into_iter()
            .find(|r| r["Species"] == "Tectonicas")
            .unwrap();
        assert_eq!(row["Samples"], "");
        assert_eq!(row["Seen"], "Seen");
    }
    #[test]
    fn codex_sightings_add_species_not_in_environmental_predictions() {
        let mut w = worker();
        w.on_event(&event(json!({"event":"CodexEntry","SystemAddress":42,"BodyID":1,"Name":"$Codex_Ent_Tussocks_01_White;","RegionID":18})));
        assert!(w
            .rows()
            .iter()
            .any(|r| r["SpeciesID"] == "Codex_Ent_Tussocks_01_Name"
                && r["Seen"] == "Seen"
                && r["Variant"] == "White"));
    }
    #[test]
    fn missing_surface_location_does_not_report_zero_distance_and_batch_is_quiet() {
        let mut w = worker();
        w.set_batch(true);
        assert!(w
            .on_event(&organic("Log", "$Codex_Ent_Stratum_07_Name;"))
            .is_empty());
        w.on_status(&json!({"BodyName":"Test A 1"}));
        assert_eq!(w.overlay()["Sampler"]["Distances"], json!([]));
        w.take_messages();
        w.on_event(&organic("Sample", "$Codex_Ent_Stratum_07_Name;"));
        assert!(w
            .take_messages()
            .iter()
            .all(|message| message["target"] == "Evaluator"
                && message["command"] == "SetBioInsightsFlags"));
    }
    #[test]
    fn codex_and_user_marks_survive_native_state_reload() {
        let mut worker = worker();
        worker.on_message(
            "MarkVariant",
            &json!({"SpeciesID":"Codex_Ent_Stratum_07_Name","Variant":"Emerald","Marked":true}),
        );
        worker.on_event(&event(json!({"event":"CodexEntry","SystemAddress":42,"BodyID":1,"Name":"$Codex_Ent_Stratum_07_Emerald;","RegionID":18})));
        let snapshot = worker.state();
        let mut restored = BioInsights::new(json!({})).unwrap();
        restored.restore_state(&snapshot);
        assert_eq!(restored.state(), snapshot);
        assert!(restored.is_marked("Codex_Ent_Stratum_07_Name", "Emerald"));
        restored.reset();
        assert_eq!(restored.state(), snapshot);
    }
    #[test]
    fn journal_replay_persists_unsold_mutations_without_radar_or_notifications() {
        let mut worker = worker();
        worker.set_batch(true);
        worker.on_event(&event(json!({"event":"LoadGame","FID":"F12345"})));
        worker.take_messages();
        assert!(worker
            .on_event(&organic("Analyse", "$Codex_Ent_Stratum_07_Name;"))
            .is_empty());
        let messages = worker.take_messages();
        assert_eq!(messages.len(), 2);
        let persisted = messages
            .iter()
            .find(|message| message["command"] == "BioInsightsUnsoldAdd")
            .unwrap();
        assert_eq!(persisted["target"], "Hecate");
        assert_eq!(persisted["payload"]["FID"], "F12345");
        assert_eq!(persisted["payload"]["Value"], 19010800);
        assert!(persisted["payload"]["Species"].as_u64().unwrap() > 0);
        assert!(messages
            .iter()
            .any(|message| message["target"] == "Evaluator"
                && message["command"] == "SetBioInsightsFlags"));
        assert!(messages
            .iter()
            .all(|message| message["target"] == "Hecate" || message["target"] == "Evaluator"));
        worker.on_event(&event(json!({"event":"SellOrganicData"})));
        assert_eq!(
            worker.take_messages()[0]["command"],
            "BioInsightsUnsoldDelete"
        );
    }
    #[test]
    fn fss_overlay_uses_source_lifetime_and_hides_without_refreshing_each_poll() {
        let mut worker = worker();
        worker.take_messages();
        worker.on_status(&json!({"GuiFocus":9}));
        let messages = worker.take_messages();
        let shown = messages
            .iter()
            .find(|m| m["command"] == "ShowBitmapNotification")
            .unwrap();
        assert_eq!(shown["payload"]["ImageKey"], "BIFSSNoti");
        assert_eq!(shown["payload"]["TTLms"], 6000);
        assert_eq!(shown["payload"]["XPos"], 2);
        assert_eq!(shown["payload"]["YPos"], 2);
        assert_eq!(shown["payload"]["FontSize"], 12);
        assert_eq!(shown["payload"]["ColorHex"], "FFFF6F00");
        worker.on_status(&json!({"GuiFocus":9}));
        assert!(worker.take_messages().is_empty());
        worker.on_status(&json!({"GuiFocus":10}));
        let shown = worker.take_messages().pop().unwrap();
        assert_eq!(shown["payload"]["TTLms"], 86_400_000);
        assert_eq!(shown["payload"]["YPos"], 98);
        worker.on_status(&json!({"GuiFocus":0}));
        assert!(worker
            .take_messages()
            .iter()
            .any(|m| m["command"] == "HideBitmapNotification"
                && m["payload"]["ImageKey"] == "BIFSSNoti"));
    }
    #[test]
    fn native_codex_companion_contains_every_species_and_variant() {
        let worker = worker();
        let catalogue = worker.codex_companion();
        assert_eq!(catalogue.len(), 117);
        assert_eq!(
            catalogue
                .iter()
                .map(|s| s["Variants"].as_array().unwrap().len())
                .sum::<usize>(),
            890
        );
    }
    #[test]
    fn batch_replay_preserves_cached_codex_and_marked_analysis_flags() {
        let mut source = worker();
        source.handle_http_response(&json!({"request_id":"bioinsights:galactic","ok":true,"body":json!({"Version":"1.0","Generated":"2026-10-26T00:00:00Z","Seen":{"$Codex_Ent_Stratum_07_Name;":{"Entry":{"Emerald":[23]}}}}).to_string()})).unwrap();
        let mut cached = source.state();
        let personal: Vec<_> = source
            .body_predictions(1)
            .iter()
            .flat_map(|prediction| {
                prediction["Variants"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|variant| json!([18, prediction["SpeciesID"], variant["Colour"]]))
            })
            .collect();
        cached["PersonalCodex"] = json!(personal);
        cached["MarkedVariants"] = json!([["Codex_Ent_Stratum_07_Name", "Emerald"]]);
        let mut replay = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        replay.restore_state(&cached);
        replay.set_batch(true);
        for value in [
            json!({"event":"Location","StarSystem":"Test","SystemAddress":42,"StarPos":[0,0,0]}),
            json!({"event":"Scan","SystemAddress":42,"BodyID":0,"BodyName":"Test A","StarType":"F","SurfaceTemperature":6500.0,"Radius":695700000.0}),
            planet(),
            json!({"event":"FSSBodySignals","SystemAddress":42,"BodyID":1,"Signals":[{"Type":"$SAA_SignalType_Biological;","Count":2}]}),
        ] {
            assert!(replay.on_event(&event(value)).is_empty());
        }
        replay.on_status(&json!({"GuiFocus":9}));
        let messages = replay.take_messages();
        assert_eq!(messages.len(), 1);
        let flags = &messages[0];
        assert_eq!(flags["target"], "Evaluator");
        assert_eq!(flags["command"], "SetBioInsightsFlags");
        assert_eq!(flags["payload"]["ID64"], 42);
        assert_eq!(flags["payload"]["BodyID"], 1);
        assert_eq!(flags["payload"]["IsHighValue"], true);
        assert_eq!(flags["payload"]["IsMarked"], true);
        assert_eq!(flags["payload"]["IsGCodex"], true);
        assert_eq!(flags["payload"]["IsPCodex"], false);
    }
}
