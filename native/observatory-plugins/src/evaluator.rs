//! Native Evaluator worker, ported from installed Evaluator 1.3.444.637.
//! Values are estimates until sold; discovery/mapping state belongs to each commander.
#[cfg(test)]
mod dispatch_tests;
mod model;
mod notifications;
mod overlay;
pub mod pricing;
mod route_worker;
mod routing;
mod solver;
mod ui_alerts;
mod web;
mod web_ui;

use model::{flag, Body, System};
use observatory_domain::{Finding, JournalEvent};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const VERSION: &str = "1.3.444.637";

/// Installed AboutInfo values from Evaluator constructor, without host-added prose.
pub fn about_info() -> Value {
    json!({"FullName":"Evaluator","ShortName":"Evaluator","AuthorName":"Matt G","Description":"Evaluator will calculate the potential value of bodies after FSSing them then direct you to which ones are worth mapping with the DSS.","Links":[{"Text":"Website","Url":"https://edjp.colacube.net/observatory"},{"Text":"Buy me a coffee","Url":"https://ko-fi.com/explorationtools"}]})
}

pub const COLUMNS: &[&str] = &[
    "Body",
    "Type",
    "Distance",
    "Gravity",
    "Terraformable",
    "Map",
    "Order",
    "Done",
    "CurrentValue",
    "PotentialMaxValue",
];

pub fn default_settings() -> Value {
    let mut settings = json!({"MinimumBodyValue":10000,"MinimumValueForWorthMapping":1000000,
        "MaxDistanceELW":0,"MaxDistanceWW":0,"MaxDistanceAW":0,"MaxDistanceAtmospheric":0,"MaxDistanceOther":0,
        "IncludeAtmosphericBio":false,"SetTargetedInFSSWorthMapping":true,
        "BIHighValueWorthMapping":false,"BIPersonalCodexWorthMapping":false,"BIGalacticCodexWorthMapping":false,"BIMarkedWorthMapping":false,
        "InstallUpdates":true,"InstallBetas":false,"MaxSystems":1000,"ZoomLevel":100,"ButtonSize":"  Normal","NativeOutput":" Enabled",
        "ExpandOutput":true,"ShowHeaders":true,"ShowTotal":false,"PopoutAlwaysShow":true,"PopoutShowNoFocus":true,
        "PopoutShowLeftPanel":true,"PopoutShowRightPanel":true,"PopoutShowDSSMode":true,"PopoutShowRolePanel":true,
        "PopoutShowSysMap":true,"PopoutShowFSSMode":true,"PopoutOpacity":100,
        "Notifications":{"BodiesWorthMapping":{"Popup":true,"Voice":true,"Plugin":false},
            "EfficientRoute":{"Popup":true,"Voice":true,"Plugin":false},
            "DSSPersistent":{"Popup":true,"Voice":false,"Plugin":false},
            "AlwaysPersistent":{"Popup":false,"Voice":false,"Plugin":false},"PersistentPositionX":80,"PersistentPositionY":25},
        "ThemeSettings":{"ThemeIndex":1,"ThemeMode":0,"ThemeFullName":["Default","Elite Orange"],"ThemeFileName":["theme.json","theme-orange.json"]}});
    for (key, value) in [
        ("TitleColorHex", "FFFF7C00"),
        ("ColorHex", "FFFF6300"),
        ("BorderColorHex", "FF83FF8C"),
        ("BackgroundColorHex", "B0000000"),
    ] {
        settings[key] = json!(value);
    }
    settings["UseCustomNotificationColours"] = json!(false);
    settings["VerboseLogging"] = json!(false);
    for key in [
        "AlwaysShowMain",
        "AlwaysShowELW",
        "AlwaysShowWW",
        "AlwaysShowAW",
        "AlwaysShowTerraformables",
        "AlwaysShowAtmospherics",
        "AlwaysShowAtmosphericsBios",
        "AlwaysShowNonAtmosphericsBios",
        "AlwaysShowBIHighValue",
        "AlwaysShowBIPersonalCodex",
        "AlwaysShowBIGalacticCodex",
        "AlwaysShowBIMarked",
    ] {
        settings[key] = json!(false);
    }
    settings
}

pub struct Evaluator {
    settings: Value,
    ui_alerts: BTreeMap<u8, String>,
    systems: BTreeMap<(String, u64), System>,
    current: u64,
    current_visit: u64,
    next_visit: u64,
    preread: bool,
    commander: String,
    fid: String,
    odyssey: bool,
    batch: bool,
    filter: String,
    messages: Vec<Value>,
    timestamp: String,
    // Hecate can include older systems outside the bounded on-screen history.
    historical_total: i64,
    unsold_data: Value,
    emitted_values: BTreeMap<(String, u64, u64), i64>,
    pending_web: Vec<(String, Value)>,
    gui_focus: u64,
    overlay_payload: Value,
    persistent_open: bool,
    system_complete_or_mapped: bool,
    transport_mode: u8,
    default_notification_colours: Value,
    ready_peers: BTreeMap<String, String>,
    server_address: String,
    server_port: String,
    route_worker: Option<route_worker::Worker>,
    deferred_route_events: Vec<((String, u64), JournalEvent)>,
    route_findings: Vec<Finding>,
    notification_renderings: Vec<(Finding, u8)>,
    notification_payloads: Vec<(Finding, Value)>,
}
impl Evaluator {
    pub fn set_settings(&mut self, settings: Value) -> anyhow::Result<()> {
        if !settings.is_object() {
            anyhow::bail!("Evaluator settings must be an object");
        }
        let replacement = Self::new(settings);
        self.settings = replacement.settings;
        self.register_colours();
        self.recalculate();
        Ok(())
    }
    pub fn new(settings: Value) -> Self {
        let mut defaults = default_settings();
        if let Some(values) = settings.as_object() {
            for (key, value) in values {
                defaults[key] = value.clone();
            }
        }
        Self {
            settings: defaults,
            ui_alerts: BTreeMap::new(),
            systems: BTreeMap::new(),
            current: 0,
            current_visit: 0,
            next_visit: 0,
            preread: false,
            commander: String::new(),
            fid: "F0".into(),
            odyssey: false,
            batch: false,
            filter: "Default".into(),
            messages: Vec::new(),
            timestamp: String::new(),
            historical_total: 0,
            unsold_data: json!([]),
            emitted_values: BTreeMap::new(),
            pending_web: Vec::new(),
            gui_focus: 0,
            overlay_payload: Value::Null,
            persistent_open: false,
            system_complete_or_mapped: false,
            transport_mode: 0,
            default_notification_colours: Value::Null,
            ready_peers: BTreeMap::new(),
            server_address: String::new(),
            server_port: String::new(),
            route_worker: None,
            deferred_route_events: Vec::new(),
            route_findings: Vec::new(),
            notification_renderings: Vec::new(),
            notification_payloads: Vec::new(),
        }
    }
    pub fn reset(&mut self) {
        let settings = self.settings.clone();
        *self = Self::new(settings);
        self.queue(
            "Core",
            "Notification",
            json!({"id":"native.evaluator.remaining","action":"cancel","plugin":"Evaluator"}),
        );
    }
    pub fn set_preread(&mut self, preread: bool) {
        self.preread = preread;
    }
    pub fn set_batch(&mut self, batch: bool) {
        let finished = self.batch && !batch;
        self.batch = batch;
        if finished {
            self.register_colours();
            self.recalculate();
            self.queue(
                "Hecate",
                "EvaluatorGetTotal",
                json!({"FID":self.fid_number(),"SubtractCurrent":true}),
            );
            self.queue("Hecate", "WebServerRegisterPlugin", json!({}));
            self.queue("Hecate","WebSocketBroadcast",json!({"Plugin":"evaluator","Json":json!({"event":"reloadpage","page":"/evaluator/main"}).to_string()}));
        }
    }
    pub fn on_status(&mut self, status: &Value) {
        let previous = self.gui_focus;
        self.gui_focus = status["GuiFocus"].as_u64().unwrap_or(0);
        if flag(&self.settings, "SetTargetedInFSSWorthMapping") && self.gui_focus == 9 {
            let destination = &status["Destination"];
            if destination["System"].as_u64() == Some(self.current) {
                if let Some(id) = destination["Body"].as_u64() {
                    if let Some(body) = self
                        .systems
                        .get_mut(&self.key())
                        .and_then(|system| system.bodies.get_mut(&id))
                        .filter(|body| !body.star)
                    {
                        if body.override_mapping != Some(true) {
                            body.override_mapping = Some(true);
                            self.recalculate();
                        }
                        self.request_overlay(true);
                    }
                }
            }
        }
        let notifications = &self.settings["Notifications"];
        if notifications["DSSPersistent"]["Popup"]
            .as_bool()
            .unwrap_or(true)
        {
            if self.gui_focus == 10 {
                self.request_overlay(false);
            } else if previous == 10 && notifications["AlwaysPersistent"]["Popup"] != true {
                self.close_overlay();
            }
        }
    }
    fn register_colours(&mut self) {
        if flag(&self.settings, "UseCustomNotificationColours") {
            self.queue("StellarOverlay","RegisterPluginCustomColours",json!({"TitleColor":self.settings["TitleColorHex"],"TextColor":self.settings["ColorHex"],"BackgroundColor":self.settings["BackgroundColorHex"],"BorderColor":self.settings["BorderColorHex"],"GetDefaultColors":true}));
        } else {
            self.queue("StellarOverlay", "UnregisterPluginCustomColours", json!({}));
        }
    }
    fn log(&mut self, message: String) {
        self.queue(
            "Core",
            "Log",
            json!({"Plugin":"Evaluator","Message":message}),
        );
    }
    fn key(&self) -> (String, u64) {
        (
            self.fid.clone(),
            if self.current_visit == 0 {
                self.current
            } else {
                self.current_visit
            },
        )
    }
    fn receive_total(total: i64, current: i64, subtract: bool) -> i64 {
        if subtract && total >= current {
            total - current
        } else {
            total
        }
    }
    fn fid_number(&self) -> u64 {
        self.fid.trim_start_matches('F').parse().unwrap_or(0)
    }
    fn queue(&mut self, target: &str, command: &str, payload: Value) {
        self.messages
            .push(json!({"target":target,"command":command,"payload":payload}));
    }
    pub fn take_messages(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.messages)
    }
    fn location(&mut self, event: &JournalEvent) {
        let address = event.id("SystemAddress").unwrap_or(self.current);
        if address == 0 {
            return;
        }
        if let Some(worker) = &mut self.route_worker {
            worker.cancel();
        }
        self.deferred_route_events.clear();
        self.system_complete_or_mapped = false;
        self.expire_ui_alerts();
        if !self.batch {
            self.systems.retain(|(fid, _), _| fid != &self.fid);
        }
        self.current = address;
        self.next_visit += 1;
        self.current_visit = self.next_visit;
        let name = if event.str("StarSystem").is_empty() {
            event.str("SystemName")
        } else {
            event.str("StarSystem")
        };
        self.systems.insert(
            self.key(),
            System {
                name: name.into(),
                address,
                visit: self.current_visit,
                date: event.timestamp.clone(),
                active: true,
                ..System::default()
            },
        );
        self.emitted_values
            .retain(|(fid, system, _), _| fid != &self.fid || *system != address);
        self.queue(
            "Hecate",
            "EvaluatorSysAdd",
            json!({"FID":self.fid_number(),"ID64":address,"Name":name,"DateTime":event.timestamp}),
        );
        self.queue(
            "Hecate",
            "EvaluatorGetTotal",
            json!({"FID":self.fid_number(),"SubtractCurrent":false}),
        );
        let max = self.settings["MaxSystems"].as_u64().unwrap_or(1000).max(1) as usize;
        while self
            .systems
            .iter()
            .filter(|((fid, _), _)| fid == &self.fid)
            .count()
            > max.saturating_add(1)
        {
            let oldest = self
                .systems
                .iter()
                .filter(|(key, _)| key.0 == self.fid && **key != self.key())
                .min_by_key(|(key, _)| key.1)
                .map(|(key, _)| key.clone());
            if let Some(key) = oldest {
                self.systems.remove(&key);
            } else {
                break;
            }
        }
    }
    fn recalculate(&mut self) {
        // Original defers Orrery/route work until FinishedReadAll/FinishedPreRead.
        if self.batch {
            return;
        }
        let key = self.key();
        if let Some(sys) = self.systems.get_mut(&key) {
            let count = sys
                .bodies
                .values()
                .filter(|body| {
                    !body.visited() && (body.worth(&self.settings) || body.worth_visiting())
                })
                .count();
            if count >= 13 {
                self.route_worker
                    .get_or_insert_with(route_worker::Worker::new)
                    .submit(
                        key,
                        sys.clone(),
                        self.settings.clone(),
                        self.timestamp.clone(),
                    );
            } else {
                if let Some(worker) = &mut self.route_worker {
                    worker.cancel();
                }
                routing::update(sys, &self.settings, &self.timestamp);
            }
        }
        if !self.route_pending() {
            self.flush_route_notifications(&self.key());
        }
    }
    fn route_pending(&self) -> bool {
        self.route_worker
            .as_ref()
            .is_some_and(route_worker::Worker::pending)
    }
    pub fn poll_routes(&mut self) -> bool {
        let Some((key, route)) = self
            .route_worker
            .as_mut()
            .and_then(route_worker::Worker::poll)
        else {
            return false;
        };
        if key != self.key() {
            return false;
        }
        if let Some(system) = self.systems.get_mut(&key) {
            system.route = route;
            for body in system.bodies.values_mut() {
                body.order = system
                    .route
                    .iter()
                    .position(|id| *id == body.id)
                    .map_or(0, |index| index + 1);
            }
        }
        self.flush_route_notifications(&key);
        true
    }
    fn flush_route_notifications(&mut self, key: &(String, u64)) {
        for (event_key, event) in std::mem::take(&mut self.deferred_route_events) {
            if &event_key == key {
                let first = event.event == "FSSAllBodiesFound";
                if let Some(detail) = self.route_notification_detail(first, false) {
                    let findings = self.notify(
                        &event,
                        if first {
                            String::new()
                        } else {
                            event.str("BodyName").into()
                        },
                        if first {
                            "System Discovery Complete"
                        } else {
                            "Mapping Complete"
                        }
                        .into(),
                        detail,
                        "EfficientRoute",
                    );
                    if !findings.is_empty()
                        && self.settings["Notifications"]["AlwaysPersistent"]["Popup"] == true
                    {
                        self.request_overlay(!first);
                    }
                    self.route_findings.extend(findings);
                }
            }
        }
    }
    pub fn take_route_findings(&mut self) -> Vec<Finding> {
        std::mem::take(&mut self.route_findings)
    }
    fn ledger(&mut self) {
        let Some(sys) = self.systems.get(&self.key()) else {
            return;
        };
        if !sys.active {
            return;
        }
        let mut entries = Vec::new();
        for body in sys.bodies.values() {
            entries.push((
                body.id,
                body.name.clone(),
                body.current()
                    + if Some(body.id) == sys.main {
                        sys.main_bonus()
                    } else {
                        0
                    },
            ));
        }
        if sys.scanned_bonus() > 0 {
            entries.push((50000, "Fully Scanned Bonus".into(), sys.scanned_bonus()));
        }
        if sys.mapped_bonus() > 0 {
            entries.push((50001, "Fully Mapped Bonus".into(), sys.mapped_bonus()));
        }
        let address = sys.address;
        let date = sys.date.clone();
        let fid = self.fid_number();
        let mut changed = false;
        for (body, name, value) in entries {
            if self
                .emitted_values
                .insert((self.fid.clone(), address, body), value)
                == Some(value)
            {
                continue;
            }
            changed = true;
            self.queue("Hecate","EvaluatorAdd",json!({"FID":fid,"ID64":address,"BodyID":body,"DateTime":date,"Value":value,"Name":name}));
        }
        if changed {
            self.queue(
                "Hecate",
                "EvaluatorGetTotal",
                json!({"FID":fid,"SubtractCurrent":true}),
            );
        }
    }
    fn notify(
        &mut self,
        event: &JournalEvent,
        body: String,
        title: String,
        detail: String,
        category: &str,
    ) -> Vec<Finding> {
        let options = &self.settings["Notifications"][category];
        let rendering = u8::from(options["Popup"].as_bool().unwrap_or(true))
            | (u8::from(options["Voice"].as_bool().unwrap_or(true)) << 1)
            | (u8::from(options["Plugin"].as_bool().unwrap_or(false)) << 2);
        if rendering == 0 || (self.batch && (category == "BodiesWorthMapping" || !self.preread)) {
            return Vec::new();
        }
        let system = self
            .systems
            .get(&self.key())
            .map_or(String::new(), |s| s.name.clone());
        let display_title = if category == "BodiesWorthMapping" {
            notifications::body_name(&body, &system, false, false)
        } else {
            title
        };
        let title_ssml = if category == "BodiesWorthMapping" {
            notifications::body_name(&body, &system, true, true)
        } else {
            String::new()
        };
        let detail_ssml = if category == "EfficientRoute" {
            self.route_notification_detail(event.event == "FSSAllBodiesFound", true)
                .map(|s| notifications::ssml(&s))
                .unwrap_or_default()
        } else {
            String::new()
        };
        let mut payload = json!({"plugin":"Evaluator","rendering":rendering,"title":display_title,"detail":detail,"title_ssml":title_ssml,"detail_ssml":detail_ssml});
        if category == "BodiesWorthMapping" {
            payload["coalescing_id"] = event.data["BodyID"].clone();
        }
        let finding = Finding {
            plugin: "Evaluator".into(),
            timestamp: event.timestamp.clone(),
            system,
            body,
            title: display_title,
            detail,
        };
        self.notification_renderings
            .push((finding.clone(), rendering));
        self.notification_payloads.push((finding.clone(), payload));
        vec![finding]
    }
    pub fn take_notification_rendering(&mut self, finding: &Finding) -> Option<u8> {
        self.notification_renderings
            .iter()
            .position(|(value, _)| value == finding)
            .map(|index| self.notification_renderings.remove(index).1)
    }
    pub fn on_event(&mut self, e: &JournalEvent) -> Vec<Finding> {
        self.timestamp = e.timestamp.clone();
        if flag(&self.settings, "VerboseLogging") {
            self.log(format!("JournalEvent - {}", e.event));
        }
        let mut findings = Vec::new();
        match e.event.as_str() {
            "LoadGame" => {
                self.fid = if e.str("FID").is_empty() {
                    "F0".into()
                } else {
                    e.str("FID").into()
                };
                self.commander = e.str("Commander").into();
                if e.str("FID").is_empty() {
                    self.queue("Hecate", "GetFID", json!({"Name":self.commander}));
                }
                self.odyssey = e.boolean("Odyssey");
            }
            "Location" | "FSDJump" | "CarrierJump" => {
                if e.event == "Location" {
                    self.transport_mode = if e.boolean("InSRV") {
                        1
                    } else if e.boolean("OnFoot") {
                        2
                    } else {
                        0
                    };
                }
                let changes_system = e.id("SystemAddress").is_some_and(|id| id != self.current);
                if e.event == "FSDJump"
                    || (changes_system
                        && (e.event == "Location" || e.boolean("Docked") || e.boolean("OnFoot")))
                {
                    self.location(e);
                }
            }
            "StartJump" => {
                if e.str("JumpType") == "Supercruise" {
                    if self.settings["Notifications"]["AlwaysPersistent"]["Popup"] == true {
                        self.request_overlay(false);
                    }
                } else {
                    self.close_overlay();
                    if !self.batch {
                        self.queue(
                            "Hecate",
                            "EvaluatorGetTotal",
                            json!({"FID":self.fid_number(),"SubtractCurrent":false}),
                        );
                    }
                }
            }
            "Shutdown" | "SupercruiseExit" => self.close_overlay(),
            "Scan" => {
                if e.id("SystemAddress").is_some_and(|id| id != self.current) {
                    return findings;
                }
                let key = self.key();
                if let Some(sys) = self.systems.get_mut(&key) {
                    if sys.active {
                        let id = e.id("BodyID").unwrap_or(0);
                        let extra = sys.extras.remove(&id).unwrap_or_default();
                        if let Some(mut body) = Body::from_scan(e, self.odyssey, extra) {
                            body.extra.is_high_value = body.potential() as f64
                                >= model::number(&self.settings, "MinimumValueForWorthMapping");
                            if !sys.bodies.contains_key(&id) {
                                if body.star
                                    && matches!(
                                        body.name
                                            .strip_prefix(&sys.name)
                                            .unwrap_or(&body.name)
                                            .trim(),
                                        "" | "A" | "0"
                                    )
                                {
                                    sys.main = Some(id);
                                }
                                let worth = body.worth(&self.settings);
                                let name = body.name.clone();
                                sys.orbits.insert(id, e.data.clone());
                                sys.body_order.push(id);
                                sys.bodies.insert(id, body);
                                if worth {
                                    findings = self.notify(
                                        e,
                                        name.clone(),
                                        name,
                                        "Meets Mapping Criteria".into(),
                                        "BodiesWorthMapping",
                                    );
                                }
                            }
                        }
                    }
                }
                self.recalculate();
                self.ledger();
            }
            "ScanBaryCentre" => {
                if e.id("SystemAddress").is_some_and(|id| id != self.current) {
                    return findings;
                }
                if let (Some(sys), Some(id)) = (self.systems.get_mut(&self.key()), e.id("BodyID")) {
                    sys.orbits.insert(id, e.data.clone());
                }
                self.recalculate();
            }
            "FSSDiscoveryScan" | "FSSAllBodiesFound" => {
                let current = e.id("SystemAddress").is_none_or(|id| id == self.current);
                if current {
                    if e.event == "FSSDiscoveryScan" {
                        if let Some(sys) = self.systems.get_mut(&self.key()) {
                            sys.total = e.id("BodyCount").unwrap_or(sys.total as u64) as usize;
                        }
                    } else {
                        self.system_complete_or_mapped = true;
                        self.recalculate();
                        self.ledger();
                    }
                }
                if e.event == "FSSAllBodiesFound" {
                    if self.route_pending() {
                        self.deferred_route_events.push((self.key(), e.clone()));
                        return findings;
                    }
                    if let Some(detail) = self.route_notification_detail(true, false) {
                        findings = self.notify(
                            e,
                            String::new(),
                            "System Discovery Complete".into(),
                            detail,
                            "EfficientRoute",
                        );
                        if !findings.is_empty()
                            && self.settings["Notifications"]["AlwaysPersistent"]["Popup"] == true
                        {
                            self.request_overlay(false);
                        }
                    }
                }
            }
            "FSSBodySignals" | "SAASignalsFound" => {
                if e.id("SystemAddress").is_some_and(|id| id != self.current) {
                    return findings;
                }
                let id = e.id("BodyID").unwrap_or(0);
                if let Some(sys) = self.systems.get_mut(&self.key()) {
                    let extra = if let Some(body) = sys.bodies.get_mut(&id) {
                        &mut body.extra
                    } else {
                        sys.extras.entry(id).or_default()
                    };
                    if let Some(signals) = e.data["Signals"].as_array() {
                        for signal in signals {
                            let kind = signal["Type"].as_str().unwrap_or("");
                            let count = signal["Count"].as_u64().unwrap_or(0);
                            if kind.contains("Biological") {
                                extra.bio_count = Some(count);
                            }
                            if kind.contains("Geological") {
                                extra.geo_count = Some(count);
                            }
                        }
                    }
                    if flag(&self.settings, "IncludeAtmosphericBio") {
                        if let Some(body) = sys.bodies.get_mut(&id) {
                            let max = model::number(&self.settings, "MaxDistanceAtmospheric");
                            if body.atmospheric
                                && body.extra.bio_count.is_some_and(|v| v > 0)
                                && (max == 0.0 || body.distance <= max)
                            {
                                body.override_mapping = Some(true);
                            }
                        }
                    }
                }
                self.recalculate();
            }
            "SAAScanComplete" => {
                if e.id("SystemAddress").is_some_and(|id| id != self.current) {
                    return findings;
                }
                if let Some(body) = self
                    .systems
                    .get_mut(&self.key())
                    .and_then(|s| s.bodies.get_mut(&e.id("BodyID").unwrap_or(0)))
                {
                    body.mapped = true;
                    body.efficient = e.number("ProbesUsed") <= e.number("EfficiencyTarget");
                    body.extra.is_high_value |= body.potential() as f64
                        >= model::number(&self.settings, "MinimumValueForWorthMapping");
                }
                self.system_complete_or_mapped = true;
                if let Some(sys) = self.systems.get_mut(&self.key()) {
                    sys.from = e.id("BodyID");
                }
                self.recalculate();
                self.ledger();
                if self.route_pending() {
                    self.deferred_route_events.push((self.key(), e.clone()));
                    return findings;
                }
                findings = self.notify(
                    e,
                    e.str("BodyName").into(),
                    "Mapping Complete".into(),
                    self.route_notification_detail(false, false)
                        .unwrap_or_default(),
                    "EfficientRoute",
                );
                if !findings.is_empty()
                    && self.settings["Notifications"]["AlwaysPersistent"]["Popup"] == true
                {
                    self.request_overlay(true);
                }
            }
            "Embark" => {
                self.transport_mode = if e.boolean("SRV") { 1 } else { 0 };
            }
            "Disembark" => self.transport_mode = 2,
            "ScanOrganic" => self.odyssey = true,
            "ApproachBody" => {
                if let Some(sys) = self.systems.get_mut(&self.key()) {
                    sys.from = e.id("BodyID");
                }
                if self.transport_mode == 0 && self.system_complete_or_mapped {
                    self.recalculate();
                    self.request_overlay(true);
                }
            }
            "SellExplorationData" | "MultiSellExplorationData" | "Died" => {
                if e.event == "Died" && self.transport_mode != 0 {
                    return findings;
                }
                self.historical_total = 0;
                self.unsold_data = json!([]);
                self.queue(
                    "Hecate",
                    "EvaluatorDelete",
                    json!({"FID":self.fid_number()}),
                );
            }
            _ => {}
        }
        findings
    }
    fn route_description(&self) -> String {
        let Some(sys) = self.systems.get(&self.key()) else {
            return String::new();
        };
        if sys.route.is_empty() {
            "No more bodies to map".into()
        } else {
            format!(
                "Next: {}",
                sys.route
                    .iter()
                    .filter_map(|id| sys.bodies.get(id))
                    .map(|b| b.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
    pub fn set_bio_summary(&mut self, body_id: u64, summary: Value) {
        let payload = json!({"ID64":summary["SystemAddress"].as_u64().unwrap_or(self.current),"BodyID":body_id,
            "IsHighValue":summary["HighValue"].as_bool().or_else(||summary["high_value"].as_bool()).unwrap_or(false),
            "IsPCodex":summary["PersonalCodex"].as_bool().unwrap_or(false),
            "IsGCodex":summary["GalacticCodex"].as_bool().unwrap_or(false),"IsMarked":summary["Marked"].as_bool().unwrap_or(false)});
        self.on_message("SetBioInsightsFlags", &payload);
    }
    pub fn on_peer_message(&mut self, sender: &str, command: &str, payload: &Value) {
        let bio = sender == "BioInsights"
            || sender.eq_ignore_ascii_case("4d617474-4701-4249-4f49-4e5349474854");
        let hecate = sender == "Hecate"
            || sender.eq_ignore_ascii_case("4d617474-4707-0000-0000-484543415445");
        if command == "Ready" {
            if !sender.is_empty()
                && sender != "Evaluator"
                && !sender.eq_ignore_ascii_case("4d617474-4702-0045-5641-4c5541544f52")
            {
                self.ready_peers.insert(
                    sender.to_owned(),
                    payload["Version"].as_str().unwrap_or("").to_owned(),
                );
            }
            return;
        }
        if (command == "SetBioInsightsFlags" && !bio)
            || ((command == "GetFIDResponse" || command == "SetServerAddress") && !hecate)
        {
            self.log(format!("[{sender}] called {command}"));
            return;
        }
        self.on_message(command, payload);
    }
    pub fn on_message(&mut self, command: &str, payload: &Value) {
        let required: &[&str] = match command {
            "SetBioInsightsFlags" => &["IsHighValue", "IsPCodex", "IsGCodex", "IsMarked"],
            "SetWorthVisiting" => &["IsWorthMapping"],
            "SetBodyVisited" => &["IsVisited"],
            _ => &[],
        };
        if !required.is_empty()
            && (payload["ID64"].as_u64().is_none()
                || payload["BodyID"]
                    .as_u64()
                    .is_none_or(|id| id > i32::MAX as u64)
                || required.iter().any(|key| !payload[*key].is_boolean()))
        {
            self.log(format!(
                "{command} called with missing or invalid source keys"
            ));
            return;
        }
        let id = payload["ID64"].as_u64().unwrap_or(self.current);
        if matches!(
            command,
            "SetWorthMapping" | "SetBioInsightsFlags" | "SetWorthVisiting" | "SetBodyVisited"
        ) && id != self.current
        {
            return;
        }
        let body_id = payload["BodyID"].as_u64();
        match command {
            "Ready" => {}
            "SourceUIAlert" => self.set_ui_alert(payload),
            "SourceReadAllState" => self.read_all_state(payload["State"].as_str().unwrap_or("")),
            "DismissUIAlert" => {
                if let Some(id) = payload["ID"].as_u64().and_then(|id| u8::try_from(id).ok()) {
                    self.dismiss_ui_alert(id);
                }
            }
            "GetFIDResponse" => {
                if let Some(fid) = payload["FID"].as_u64().filter(|fid| *fid > 0) {
                    self.fid = format!("F{fid}");
                }
            }
            "SetServerAddress" => {
                if let (Some(address), Some(port)) = (
                    payload["ServerAddress"].as_str(),
                    payload["ServerPort"].as_str(),
                ) {
                    self.server_address = address.into();
                    self.server_port = port.into();
                }
            }
            "RegisterPluginCustomColoursResponse" => {
                if ["TitleColor", "TextColor", "BackgroundColor", "BorderColor"]
                    .iter()
                    .all(|k| payload[*k].is_string())
                {
                    self.default_notification_colours = payload.clone();
                }
            }
            "LegacyPluginMessage" => {
                let payload = payload.get("message").unwrap_or(payload);
                let command = payload
                    .as_str()
                    .or_else(|| payload.get(0).and_then(Value::as_str))
                    .or_else(|| payload["Command"].as_str())
                    .unwrap_or("UNDEFINED");
                self.log(format!(
                    "called legacy HandlePluginMessage [CMD: {command}]"
                ));
            }
            "Unsold" => self.queue(
                "Hecate",
                "EvaluatorGetData",
                json!({"FID":self.fid_number()}),
            ),
            "EvaluatorGetDataResponse" => {
                self.unsold_data = payload["Data"].clone();
                for (command, mut request) in std::mem::take(&mut self.pending_web) {
                    request["_VaultReady"] = json!(true);
                    self.web_request(&command, &request);
                }
            }
            "WebRequest" | "WebSocketRequest" => self.web_request(command, payload),
            "SetZoomLevel" => {
                self.settings["ZoomLevel"] =
                    json!(payload["ZoomLevel"].as_u64().unwrap_or(100).clamp(50, 600))
            }
            "EvaluatorSetTotal" => {
                if let Some(total) = payload["Total"].as_i64() {
                    let current = self.systems.get(&self.key()).map_or(0, System::current);
                    self.historical_total =
                        Self::receive_total(total, current, payload["SubtractCurrent"] == true);
                }
            }
            "SetFilter" => self.filter = payload["Filter"].as_str().unwrap_or("Default").into(),
            "FindQuickestRoute" => self.recalculate(),
            "SetBioInsightsFlags" | "SetWorthVisiting" | "SetBodyVisited" | "SetWorthMapping" => {
                if let (Some((_, sys)), Some(body_id)) = (
                    self.systems
                        .iter_mut()
                        .rev()
                        .find(|((fid, _), sys)| fid == &self.fid && sys.address == id),
                    body_id,
                ) {
                    let body = sys.bodies.get_mut(&body_id);
                    if command == "SetWorthMapping" {
                        if let Some(body) = body {
                            body.override_mapping = payload["IsWorthMapping"].as_bool();
                        }
                    } else {
                        let extra = body
                            .map(|b| &mut b.extra)
                            .unwrap_or_else(|| sys.extras.entry(body_id).or_default());
                        match command {
                            "SetWorthVisiting" => {
                                extra.worth_visiting =
                                    payload["IsWorthMapping"].as_bool().unwrap_or(false)
                            }
                            "SetBodyVisited" => {
                                extra.visited = payload["IsVisited"].as_bool().unwrap_or(false)
                            }
                            _ => {
                                extra.high_value = payload["IsHighValue"]
                                    .as_bool()
                                    .or_else(|| payload["HighValue"].as_bool())
                                    .unwrap_or(false);
                                extra.personal = payload["IsPCodex"]
                                    .as_bool()
                                    .or_else(|| payload["PersonalCodex"].as_bool())
                                    .unwrap_or(false);
                                extra.galactic = payload["IsGCodex"]
                                    .as_bool()
                                    .or_else(|| payload["GalacticCodex"].as_bool())
                                    .unwrap_or(false);
                                extra.marked = payload["IsMarked"]
                                    .as_bool()
                                    .or_else(|| payload["Marked"].as_bool())
                                    .unwrap_or(false);
                            }
                        }
                    }
                }
                if command == "SetWorthMapping"
                    || (command != "SetBioInsightsFlags" && self.system_complete_or_mapped)
                {
                    self.recalculate();
                    self.request_overlay(true);
                }
            }
            _ => {}
        }
    }
    pub fn rows(&self) -> Vec<Value> {
        let mut systems: Vec<_> = self
            .systems
            .iter()
            .filter(|((fid, _), _)| fid == &self.fid)
            .map(|(_, s)| s)
            .collect();
        systems.sort_by_key(|system| system.visit);
        systems
            .into_iter()
            .flat_map(|sys| {
                sys.bodies
                    .values()
                    .filter(move |b| match self.filter.as_str() {
                        "All" => true,
                        "MappingOnly" => {
                            b.visible(&self.settings, sys.main)
                                && (b.mapped || b.worth(&self.settings) || b.worth_visiting())
                        }
                        _ => b.visible(&self.settings, sys.main),
                    })
                    .map(move |b| b.row(sys, &self.settings))
            })
            .collect()
    }
    pub fn summary(&self) -> Value {
        let systems:Vec<_>=self.systems.iter().filter(|((fid,_),_)|fid==&self.fid).map(|(_,sys)|json!({"ID64":sys.address,"VisitID":sys.visit,"System":sys.name,"Current":sys.address==self.current,"TotalBodies":sys.total,"ScannedBodies":sys.bodies.len(),"SystemValue":sys.current(),"PotentialValue":sys.potential(&self.settings),"Active":sys.active,"DateTime":sys.date,"Route":sys.route})).collect();
        let total = self.systems.get(&self.key()).map_or(0, System::current);
        json!({"Commander":self.commander,"FID":self.fid,"SystemAddress":self.current,"Systems":systems,"UnsoldTotal":self.historical_total+total,"UnsoldData":self.unsold_data,"ZoomLevel":self.settings["ZoomLevel"],"Route":self.route_description(),"Filter":self.filter,"Odyssey":self.odyssey,"GuiFocus":self.gui_focus,"IsPreReading":self.preread,"TransportMode":self.transport_mode,"DefaultNotificationColours":self.default_notification_colours,"ReadyPeers":self.ready_peers,"UIAlerts":self.ui_alerts()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(value: Value) -> JournalEvent {
        JournalEvent::parse(&value.to_string()).unwrap()
    }
    fn worker() -> Evaluator {
        let mut w = Evaluator::new(json!({"MinimumBodyValue":0}));
        w.on_event(&event(
            json!({"event":"LoadGame","FID":"F123","Commander":"Test","Odyssey":true}),
        ));
        w.on_event(&event(json!({"event":"Location","StarSystem":"Test System","SystemAddress":18446744073709551614_u64})));
        w
    }
    #[test]
    fn ready_records_peer_identity_without_registration_or_total_side_effects() {
        let mut worker = Evaluator::new(json!({}));
        worker.on_peer_message("Hecate", "Ready", &json!({"Version":"1.3.444.761"}));
        assert!(worker.take_messages().is_empty());
        assert_eq!(worker.summary()["ReadyPeers"]["Hecate"], "1.3.444.761");
        worker.on_peer_message("Evaluator", "Ready", &json!({"Version":VERSION}));
        assert_eq!(worker.ready_peers.len(), 1);
        worker.on_peer_message("Other", "GetFIDResponse", &json!({"FID":999}));
        assert_eq!(worker.fid, "F0");
    }
    #[test]
    fn original_value_symbol_is_sticky_and_only_recomputed_with_values() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../../fixtures/evaluator-value-flag-oracle.json"
        ))
        .unwrap();
        for case in oracle.as_array().unwrap() {
            let mut worker = worker();
            worker.settings["MinimumValueForWorthMapping"] = case["first"].clone();
            worker.on_event(&event(json!({"event":"Scan","BodyID":1,"BodyName":"Test System 1","PlanetClass":"Earthlike body","MassEM":1})));
            assert_eq!(worker.rows()[0]["IsHighValue"], case["initial"]);
            worker.settings["MinimumValueForWorthMapping"] = case["next"].clone();
            assert_eq!(worker.rows()[0]["IsHighValue"], case["afterSettings"]);
            worker.on_event(&event(json!({"event":"SAAScanComplete","BodyID":1})));
            assert_eq!(worker.rows()[0]["IsHighValue"], case["afterUpdate"]);
        }
    }
    #[test]
    fn displayed_visits_and_total_updates_match_independent_source_state_cases() {
        let oracle: Value =
            serde_json::from_str(include_str!("../../fixtures/evaluator-state-oracle.json"))
                .unwrap();
        for case in oracle["visits"].as_array().unwrap() {
            let mut worker = Evaluator::new(json!({"MaxSystems":case["maximum"]}));
            worker.fid = "F123".into();
            worker.set_preread(case["preread"] == true);
            worker.set_batch(case["batch"] == true);
            for address in [1, 2, 1] {
                worker.on_event(&event(json!({"event":"FSDJump","StarSystem":format!("System {address}"),"SystemAddress":address})));
            }
            let addresses: Vec<_> = worker.systems.values().map(|s| s.address).collect();
            assert_eq!(json!(addresses), case["addresses"], "{case}");
            let bus: Vec<Value> = worker
                .take_messages()
                .into_iter()
                .filter(|message| message["command"] == "EvaluatorSysAdd")
                .map(|message| message["command"].clone())
                .collect();
            assert_eq!(json!(bus), case["messages"]);
        }
        for case in oracle["totals"].as_array().unwrap() {
            let current = case["current"].as_i64().unwrap();
            let actual = Evaluator::receive_total(
                case["total"].as_i64().unwrap(),
                current,
                case["flag"] == 1,
            );
            assert_eq!(actual, case["unsold"].as_i64().unwrap());
            assert_eq!(actual + current, case["display"].as_i64().unwrap());
        }
    }
    #[test]
    fn larger_routes_finish_off_dispatch_thread_and_defer_accurate_route_notifications() {
        let mut worker = worker();
        worker.set_batch(true);
        for id in 1..=14 {
            worker.on_event(&event(json!({"event":"Scan","BodyID":id,"BodyName":format!("Test System {id}"),"PlanetClass":"Earthlike body","MassEM":1})));
        }
        assert!(worker.systems[&worker.key()].route.is_empty());
        let started = std::time::Instant::now();
        worker.set_batch(false);
        assert!(started.elapsed() < std::time::Duration::from_millis(250));
        assert!(worker.route_pending());
        assert!(worker.on_event(&event(json!({"event":"SAAScanComplete","BodyID":1,"BodyName":"Test System 1","ProbesUsed":2,"EfficiencyTarget":6}))).is_empty());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
        while worker.route_pending() && std::time::Instant::now() < deadline {
            worker.poll_routes();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(!worker.route_pending());
        let route = worker.systems[&worker.key()].route.clone();
        assert_eq!(route.len(), 13);
        assert!(!route.contains(&1));
        let findings = worker.take_route_findings();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].title, "Mapping Complete");
        assert_eq!(
            findings[0].detail,
            format!("Next body to map is {}", route[0])
        );
        assert!(worker.take_notification_payload(&findings[0]).is_some());
        // A late completion for another commander cannot overwrite current state.
        worker.recalculate();
        worker.on_event(&event(
            json!({"event":"LoadGame","FID":"F999","Commander":"Other"}),
        ));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while worker.route_pending() && std::time::Instant::now() < deadline {
            worker.poll_routes();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(worker.rows().is_empty());
        assert_eq!(worker.summary()["FID"], "F999");
    }
    #[test]
    fn shrinking_pending_route_flushes_prior_mapping_notification() {
        let mut worker = worker();
        worker.set_batch(true);
        for id in 1..=14 {
            worker.on_event(&event(json!({"event":"Scan","BodyID":id,"BodyName":format!("Test System {id}"),"PlanetClass":"Earthlike body","MassEM":1})));
        }
        worker.set_batch(false);
        assert!(worker.on_event(&event(json!({"event":"SAAScanComplete","BodyID":1,"BodyName":"Test System 1","ProbesUsed":2,"EfficiencyTarget":6}))).is_empty());
        assert_eq!(worker.deferred_route_events.len(), 1);
        let now=worker.on_event(&event(json!({"event":"SAAScanComplete","BodyID":2,"BodyName":"Test System 2","ProbesUsed":2,"EfficiencyTarget":6})));
        assert!(!worker.route_pending());
        assert!(worker.deferred_route_events.is_empty());
        let delayed = worker.take_route_findings();
        assert_eq!(delayed.len(), 1);
        assert_eq!(now.len(), 1);
        assert_eq!(delayed[0].body, "Test System 1");
        assert_eq!(now[0].body, "Test System 2");
        assert_eq!(delayed[0].detail, now[0].detail);
        assert!(!worker.systems[&worker.key()].route.contains(&1));
        assert!(!worker.systems[&worker.key()].route.contains(&2));
    }
    #[test]
    fn plugin_flags_for_previous_system_do_not_mutate_read_all_history() {
        let mut worker = worker();
        worker.set_batch(true);
        worker.on_event(&event(json!({"event":"Scan","BodyID":1,"BodyName":"First 1","PlanetClass":"Earthlike body","MassEM":1})));
        let old_key = worker.key();
        let old_id = worker.current;
        worker.on_event(&event(
            json!({"event":"Location","SystemAddress":1234,"StarSystem":"Second"}),
        ));
        worker.on_event(&event(json!({"event":"Scan","BodyID":1,"BodyName":"Second 1","PlanetClass":"Earthlike body","MassEM":1})));
        for command in [
            "SetWorthMapping",
            "SetBodyVisited",
            "SetWorthVisiting",
            "SetBioInsightsFlags",
        ] {
            worker.on_message(command,&json!({"ID64":old_id,"BodyID":1,"IsWorthMapping":true,"IsVisited":true,"IsHighValue":true,"IsPCodex":true,"IsGCodex":true,"IsMarked":true}));
        }
        let body = &worker.systems[&old_key].bodies[&1];
        assert_eq!(body.override_mapping, None);
        assert!(!body.extra.visited);
        assert!(!body.extra.worth_visiting);
        assert!(!body.extra.high_value);
        assert!(!body.extra.personal);
        assert!(!body.extra.galactic);
        assert!(!body.extra.marked);
    }
    #[test]
    fn settings_changes_preserve_history_commander_routes_and_vault_data() {
        let mut w = worker();
        w.on_event(&event(json!({"event":"Scan","BodyID":9,"BodyName":"Test System 9","PlanetClass":"Earthlike body","MassEM":1})));
        w.on_message(
            "EvaluatorGetDataResponse",
            &json!({"Data":[{"Name":"Previous","Value":421}]}),
        );
        let before = w.summary();
        let emitted = w.emitted_values.clone();
        w.set_settings(json!({"ZoomLevel":165,"ShowTotal":true}))
            .unwrap();
        let after = w.summary();
        for key in [
            "Commander",
            "FID",
            "SystemAddress",
            "Systems",
            "UnsoldTotal",
            "UnsoldData",
        ] {
            assert_eq!(before[key], after[key], "{key}");
        }
        assert_eq!(emitted, w.emitted_values);
        assert_eq!(after["ZoomLevel"], 165);
        let settings = w.settings.clone();
        assert!(w.set_settings(json!([])).is_err());
        assert_eq!(w.settings, settings);
    }
    #[test]
    fn notification_masks_and_source_metadata_preserve_each_delivery_selection() {
        for mask in 0..8u8 {
            let mut w = worker();
            w.settings["Notifications"]["BodiesWorthMapping"] =
                json!({"Popup":mask&1!=0,"Voice":mask&2!=0,"Plugin":mask&4!=0});
            let findings=w.on_event(&event(json!({"event":"Scan","BodyID":9,"BodyName":"Test System 9","PlanetClass":"Earthlike body","MassEM":1})));
            if mask == 0 {
                assert!(findings.is_empty());
                continue;
            }
            assert_eq!(findings.len(), 1);
            let finding = &findings[0];
            assert_eq!(w.take_notification_rendering(finding), Some(mask));
            let payload = w.take_notification_payload(finding).unwrap();
            assert_eq!(payload["rendering"], mask);
            assert_eq!(payload["coalescing_id"], 9);
            assert_eq!(payload["title"], "Body 9");
            assert_eq!(payload["detail"], "Meets Mapping Criteria");
            assert!(payload["title_ssml"].as_str().unwrap().contains("cardinal"));
        }
    }
    #[test]
    fn display_filters_match_all_3600_independent_original_updateui_cases() {
        let oracle: Value =
            serde_json::from_str(include_str!("../../fixtures/evaluator-ui-oracle.json")).unwrap();
        for case in oracle["cases"].as_array().unwrap() {
            let mut settings = json!({"MinimumBodyValue":case[1]});
            if let Some(flag) = case[6].as_str().filter(|s| !s.is_empty()) {
                settings[flag] = json!(true);
            }
            let mut worker = Evaluator::new(settings);
            worker.fid = "F1".into();
            worker.current = 42;
            worker.filter = case[2].as_str().unwrap().into();
            let scan = event(
                json!({"event":"Scan","BodyName":"Oracle 1","BodyID":1,"PlanetClass":case[0],"MassEM":1,"WasDiscovered":true,"WasMapped":true,"Landable":true,"SurfaceGravity":1,"SurfacePressure":0}),
            );
            let mut body = Body::from_scan(&scan, true, model::Extra::default()).unwrap();
            body.mapped = case[3].as_bool().unwrap();
            body.override_mapping = case[4].as_bool();
            body.extra.visited = case[5].as_bool().unwrap();
            let system = System {
                name: "Oracle".into(),
                address: 42,
                total: 2,
                active: true,
                bodies: BTreeMap::from([(1, body)]),
                ..Default::default()
            };
            worker.systems.insert(worker.key(), system);
            assert_eq!(
                !worker.rows().is_empty(),
                case[7].as_bool().unwrap(),
                "sourcecase {case}"
            );
        }
    }
    #[test]
    fn discovery_mapping_sale_and_commander_isolation() {
        let mut w = worker();
        w.on_event(&event(json!({"event":"FSSDiscoveryScan","BodyCount":2})));
        w.on_event(&event(json!({"event":"Scan","BodyID":0,"BodyName":"Test System","StarType":"G","StellarMass":1,"WasDiscovered":false})));
        let findings=w.on_event(&event(json!({"event":"Scan","BodyID":1,"BodyName":"Test System 1","PlanetClass":"Earthlike body","MassEM":1,"WasDiscovered":false,"WasMapped":false})));
        assert_eq!(findings.len(), 1);
        assert_eq!(w.rows().len(), 2);
        let before = w.summary()["UnsoldTotal"].as_i64().unwrap();
        w.on_event(&event(
            json!({"event":"SAAScanComplete","BodyID":1,"ProbesUsed":4,"EfficiencyTarget":6}),
        ));
        assert!(w.summary()["UnsoldTotal"].as_i64().unwrap() > before);
        assert_eq!(w.summary()["Systems"][0]["Route"], json!([]));
        assert!(w
            .take_messages()
            .iter()
            .any(|m| m["payload"]["BodyID"] == 50001));
        w.on_event(&event(
            json!({"event":"SellExplorationData","Systems":["Test System"]}),
        ));
        assert_eq!(w.summary()["UnsoldTotal"], w.systems[&w.key()].current());
        w.on_event(&event(
            json!({"event":"LoadGame","FID":"F456","Commander":"Other"}),
        ));
        assert_eq!(w.rows().len(), 0);
    }
    #[test]
    fn malformed_external_flag_payloads_do_not_clear_existing_source_state() {
        let mut w = worker();
        w.on_event(&event(json!({"event":"Scan","BodyID":2,"BodyName":"Test System 2","PlanetClass":"Icy body","MassEM":1})));
        let full = json!({"ID64":w.current,"BodyID":2,"IsHighValue":true,"IsPCodex":true,"IsGCodex":true,"IsMarked":true});
        w.on_message("SetBioInsightsFlags", &full);
        for missing in [
            "ID64",
            "BodyID",
            "IsHighValue",
            "IsPCodex",
            "IsGCodex",
            "IsMarked",
        ] {
            let mut invalid = full.clone();
            for key in ["IsHighValue", "IsPCodex", "IsGCodex", "IsMarked"] {
                invalid[key] = json!(false);
            }
            invalid.as_object_mut().unwrap().remove(missing);
            w.on_message("SetBioInsightsFlags", &invalid);
            let extra = &w.systems[&w.key()].bodies[&2].extra;
            assert!(
                extra.high_value && extra.personal && extra.galactic && extra.marked,
                "missing {missing}"
            );
        }
    }
    #[test]
    fn biological_flags_precede_scan_and_manual_override_wins() {
        let mut w = worker();
        w.settings["BIMarkedWorthMapping"] = json!(true);
        w.on_message("SetBioInsightsFlags", &json!({"ID64":w.current,"BodyID":2,"IsHighValue":false,"IsPCodex":false,"IsGCodex":false,"IsMarked":true}));
        w.on_event(&event(json!({"event":"Scan","BodyID":2,"BodyName":"Test System 2","PlanetClass":"Icy body","MassEM":1,"WasDiscovered":true,"WasMapped":true})));
        assert_eq!(w.rows()[0]["Map"], true);
        w.on_message(
            "SetWorthMapping",
            &json!({"BodyID":2,"IsWorthMapping":false}),
        );
        assert_eq!(w.rows()[0]["Map"], false);
        w.on_event(&event(json!({"event":"Died"})));
        assert_eq!(w.summary()["UnsoldTotal"], w.systems[&w.key()].current());
        assert!(w
            .take_messages()
            .iter()
            .any(|m| m["command"] == "EvaluatorDelete"));
    }
    #[test]
    fn restart_preread_upserts_do_not_double_count_existing_vault_values() {
        fn pump(worker: &mut Evaluator, vault: &mut crate::hecate::Hecate) {
            for _ in 0..10 {
                let messages = worker.take_messages();
                if messages.is_empty() {
                    return;
                }
                for message in messages {
                    if message["target"] == "Hecate" {
                        if let Some(reply) = vault
                            .handle_message(
                                message["command"].as_str().unwrap(),
                                &message["payload"],
                            )
                            .unwrap()
                        {
                            worker
                                .on_message(reply["Command"].as_str().unwrap(), &reply["Payload"]);
                        }
                    }
                }
            }
            panic!("Evaluator message loop failed to settle");
        }
        let mut vault = crate::hecate::Hecate::new(json!({"RemoteLookups":false})).unwrap();
        let entries = [
            json!({"event":"LoadGame","FID":"F77","Commander":"Replay","Odyssey":true}),
            json!({"event":"Location","SystemAddress":42,"StarSystem":"Replay System"}),
            json!({"event":"Scan","BodyID":0,"BodyName":"Replay System","StarType":"G","StellarMass":1}),
            json!({"event":"Scan","BodyID":1,"BodyName":"Replay System 1","PlanetClass":"Earthlike body","MassEM":1}),
        ];
        let mut first = Evaluator::new(json!({}));
        for mut entry in entries.clone() {
            entry["timestamp"] = json!("2026-01-01T00:00:00Z");
            first.on_event(&event(entry));
            pump(&mut first, &mut vault);
        }
        let total = first.summary()["UnsoldTotal"].as_i64().unwrap();
        assert_eq!(total, vault.evaluator_total(77).unwrap());
        let mut restarted = Evaluator::new(json!({}));
        restarted.set_batch(true);
        for mut entry in entries {
            entry["timestamp"] = json!("2026-01-01T00:00:00Z");
            restarted.on_event(&event(entry));
            pump(&mut restarted, &mut vault);
        }
        assert_eq!(restarted.summary()["UnsoldTotal"], total);
        assert_eq!(vault.evaluator_total(77).unwrap(), total);
    }
}
