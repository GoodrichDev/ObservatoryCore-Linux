//! The built-in Explorer worker. Thresholds and default switches follow the C# worker.
use crate::criteria::Criteria;
use observatory_domain::{Finding, JournalEvent};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub type Bodies = indexmap::IndexMap<u64, Value>;

#[cfg(test)]
#[path = "../tests/clr_explorer_state/mod.rs"]
mod clr_state_tests;

pub fn default_settings() -> Value {
    let mut settings = json!({
        "OnlyShowCurrentSystem": false, "Codex": false, "UncommonSecondary": false,
        "FastOrbit": false, "DiverseLife": false, "GoodFSDBody": false,
        "GreenSystem": false, "GoldSystem": false, "HighValueMappable": false,
        "EnableCustomCriteria": false, "CustomCriteriaFile": ""
    });
    for key in [
        "CloseBinary",
        "CloseOrbit",
        "CloseRing",
        "CollidingBinary",
        "FastRotation",
        "HighEccentricity",
        "LandableAtmosphere",
        "LandableHighG",
        "LandableLarge",
        "LandableRing",
        "LandableTerraformable",
        "Nested",
        "ShepherdMoon",
        "SmallObject",
        "WideRing",
    ] {
        settings[key] = Value::Bool(true);
    }
    let personal = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_default();
    settings["CustomCriteriaFile"] = json!(personal.join("ObservatoryCriteria.lua"));
    settings
}

pub struct Explorer {
    settings: Value,
    systems: HashMap<u64, Bodies>,
    signals: HashMap<u64, Bodies>,
    barycentres: HashMap<u64, Bodies>,
    current_system: String,
    current_address: u64,
    criteria: Option<Criteria>,
    batch: bool,
    monitor_read_all: bool,
    notification_payloads: Vec<(Finding, Value)>,
    direct_criteria_count: usize,
    event_snapshot: Option<Value>,
}

impl Explorer {
    pub fn new(settings: Value, criteria_path: Option<PathBuf>) -> anyhow::Result<Self> {
        Self::from_settings(settings, criteria_path, true)
    }
    pub fn prepare_preferences(settings: Value) -> anyhow::Result<Self> {
        Self::from_settings(settings, None, false)
    }
    fn from_settings(
        settings: Value,
        criteria_path: Option<PathBuf>,
        load: bool,
    ) -> anyhow::Result<Self> {
        let mut merged = default_settings();
        if let Some(obj) = settings.as_object() {
            for (key, value) in obj {
                merged[key] = value.clone();
            }
        }
        if let Some(path) = criteria_path {
            merged["CustomCriteriaFile"] = json!(path);
        }
        let criteria = if load {
            Some(Criteria::load(PathBuf::from(
                merged["CustomCriteriaFile"].as_str().unwrap_or(""),
            ))?)
        } else {
            None
        };
        Ok(Self {
            settings: merged,
            systems: HashMap::new(),
            signals: HashMap::new(),
            barycentres: HashMap::new(),
            current_system: String::new(),
            current_address: 0,
            criteria,
            batch: false,
            monitor_read_all: false,
            notification_payloads: Vec::new(),
            direct_criteria_count: 0,
            event_snapshot: None,
        })
    }

    pub fn reset(&mut self) {
        self.systems.clear();
        self.signals.clear();
        self.barycentres.clear();
        self.current_system.clear();
        self.current_address = 0;
        self.notification_payloads.clear();
        if let Some(criteria) = self.criteria.as_mut() {
            criteria.clear_raw_history();
            criteria.reset();
        }
    }
    pub fn set_batch(&mut self, batch: bool) {
        self.set_monitor_state(batch, batch);
    }
    pub fn set_monitor_state(&mut self, batch: bool, read_all: bool) {
        if batch {
            self.clear_histories();
        }
        self.batch = batch;
        if let Some(criteria) = self.criteria.as_mut() {
            if batch && !self.monitor_read_all && read_all {
                criteria.set_read_all(true);
            } else if !batch && self.monitor_read_all && !read_all {
                criteria.set_read_all(false);
            }
        }
        self.monitor_read_all = read_all;
    }
    fn clear_histories(&mut self) {
        self.systems.clear();
        self.signals.clear();
        self.barycentres.clear();
        if let Some(criteria) = self.criteria.as_mut() {
            criteria.clear_raw_history();
        }
    }
    pub fn take_notification_payload(&mut self, finding: &Finding) -> Option<Value> {
        self.notification_payloads
            .iter()
            .position(|(row, _)| row == finding)
            .map(|index| self.notification_payloads.remove(index).1)
    }
    pub fn take_diagnostics(&mut self) -> Vec<Value> {
        self.criteria
            .as_mut()
            .map(Criteria::take_diagnostics)
            .unwrap_or_default()
    }
    pub fn take_startup_findings(&mut self) -> Vec<Finding> {
        let Some(criteria) = self.criteria.as_mut() else {
            return Vec::new();
        };
        let findings = criteria.take_startup_findings();
        for finding in &findings {
            if let Some(payload) = criteria.take_notification_payload(finding) {
                self.notification_payloads.push((finding.clone(), payload));
            }
        }
        findings
    }
    pub fn apply_preferences(&mut self, prepared: Self) {
        self.settings = prepared.settings;
        if let Some(criteria) = self.criteria.as_mut() {
            criteria.set_path(PathBuf::from(
                self.settings["CustomCriteriaFile"].as_str().unwrap_or(""),
            ));
        }
    }

    fn enabled(&self, key: &str) -> bool {
        self.settings[key].as_bool().unwrap_or(false)
    }

    pub fn on_event(&mut self, event: &JournalEvent) -> Vec<Finding> {
        self.notification_payloads.clear();
        self.event_snapshot = None;
        let data = &event.data;
        let address = id(data, "SystemAddress").unwrap_or(0);
        match event.event.as_str() {
            "Location" | "FSDJump" | "CarrierJump" => {
                if event.event == "CarrierJump" && !boolean(data, "Docked") {
                    return Vec::new();
                }
                let system = string(data, "StarSystem");
                if self.current_system != system
                    && self.enabled("OnlyShowCurrentSystem")
                    && !self.batch
                {
                    self.clear_histories();
                }
                if !system.is_empty() {
                    self.current_system = system.to_owned();
                    self.current_address = address;
                }
            }
            "Scan" => return self.process_scan(event),
            "ScanBaryCentre" => {
                if let Some(body) = id(data, "BodyID") {
                    if !self
                        .barycentres
                        .get(&address)
                        .is_some_and(|b| b.contains_key(&body))
                    {
                        if let Some(criteria) = self.criteria.as_mut() {
                            criteria.remember_raw(event);
                        }
                    }
                    self.barycentres
                        .entry(address)
                        .or_default()
                        .entry(body)
                        .or_insert_with(|| data.clone());
                }
                return Vec::new();
            }
            "FSSBodySignals" | "SAASignalsFound" => {
                if let Some(body) = id(data, "BodyID") {
                    if event.event == "FSSBodySignals" {
                        self.signals
                            .entry(address)
                            .or_default()
                            .entry(body)
                            .or_insert_with(|| data.clone());
                    }
                }
            }
            "CodexEntry" if self.enabled("Codex") && boolean(data, "IsNewEntry") => {
                let body = id(data, "BodyID")
                    .and_then(|body| self.systems.get(&address)?.get(&body))
                    .map(|scan| string(scan, "BodyName"))
                    .unwrap_or_else(|| string(data, "System"));
                let finding = self.finding(
                    event,
                    body,
                    &format!("New {} Codex Entry", string(data, "Category_Localised")),
                    string(data, "Name_Localised"),
                );
                self.notification_payloads.push((finding.clone(),json!({"title":finding.title,"detail":finding.detail,"extended_details":"","sender":"Explorer","coalescing_id":-1})));
                return vec![finding];
            }
            _ => {}
        }
        self.custom(event, address)
    }

    fn custom(&mut self, event: &JournalEvent, address: u64) -> Vec<Finding> {
        self.direct_criteria_count = 0;
        self.event_snapshot = None;
        if !self.enabled("EnableCustomCriteria") {
            return Vec::new();
        }
        if event.event == "FSSAllBodiesFound" && !self.systems.contains_key(&address) {
            return Vec::new();
        }
        if let Some(criteria) = self.criteria.as_mut() {
            let empty = Bodies::new();
            let system = self.systems.get(&address).unwrap_or(&empty);
            let signals = self.signals.get(&address).unwrap_or(&empty);
            let findings = criteria.process(event, system, signals, &self.current_system);
            self.event_snapshot = criteria.take_event_snapshot();
            let typed_event = self.event_snapshot.as_ref().unwrap_or(&event.data);
            for finding in &findings {
                let payload = if let Some(mut payload) = criteria.take_notification_payload(finding)
                {
                    self.direct_criteria_count += 1;
                    let title = payload["title"].as_str().unwrap_or("");
                    if title.starts_with(&self.current_system)
                        && self.systems.contains_key(&self.current_address)
                        && payload["coalescing_id"].as_u64().is_some()
                    {
                        let bary = self
                            .barycentres
                            .get(&self.current_address)
                            .is_some_and(|v| {
                                v.contains_key(&payload["coalescing_id"].as_u64().unwrap())
                            });
                        let (title, title_ssml) =
                            short_body_title(&self.current_system, title, bary);
                        payload["title"] = json!(title);
                        payload["title_ssml"] = json!(title_ssml);
                    }
                    if payload["coalescing_id"].is_null() {
                        payload["coalescing_id"] = json!(-1);
                    }
                    payload
                } else {
                    scan_notification(typed_event, &finding.title, &finding.detail)
                };
                self.notification_payloads.push((finding.clone(), payload));
            }
            let updates = criteria.take_history_updates();
            self.apply_clr_updates(updates);
            findings
        } else {
            Vec::new()
        }
    }

    /// The original Core passes one typed journal object to subsequent workers.
    /// Keep its raw Json separate from properties mutated by custom criteria.
    pub fn take_event_snapshot(&mut self) -> Option<Value> {
        self.event_snapshot.take()
    }

    pub fn poll_custom(&mut self) -> Vec<Finding> {
        let Some(criteria) = self.criteria.as_mut() else {
            return Vec::new();
        };
        let findings = criteria.poll(&self.current_system);
        for finding in &findings {
            if let Some(payload) = criteria.take_notification_payload(finding) {
                self.notification_payloads.push((finding.clone(), payload));
            }
        }
        let updates = criteria.take_history_updates();
        self.apply_clr_updates(updates);
        findings
    }

    fn apply_clr_updates(&mut self, updates: Vec<Value>) {
        for update in updates {
            let (Some(system), Some(body), Some(mut data)) = (
                update["system_id"].as_u64(),
                update["body_id"].as_u64(),
                update
                    .get("event")
                    .filter(|value| value.is_object())
                    .cloned(),
            ) else {
                continue;
            };
            data["__clr_system_key"] = json!(system);
            if update["kind"] == "scan" {
                if let Some(parent) = update.get("parent") {
                    data["__clr_parent"] = crate::criteria::parent_projection(parent);
                }
                if let Some(previous) = self
                    .systems
                    .get_mut(&system)
                    .and_then(|bodies| bodies.get_mut(&body))
                {
                    *previous = data;
                }
            } else if update["kind"] == "signal" {
                if let Some(previous) = self
                    .signals
                    .get_mut(&system)
                    .and_then(|bodies| bodies.get_mut(&body))
                {
                    *previous = data;
                }
            }
        }
    }

    fn finding(&self, event: &JournalEvent, body: &str, title: &str, detail: &str) -> Finding {
        Finding {
            plugin: "Explorer".into(),
            timestamp: event.timestamp.clone(),
            system: match string(&event.data, "StarSystem") {
                "" => self.current_system.clone(),
                s => s.to_owned(),
            },
            body: body.into(),
            title: title.into(),
            detail: detail.into(),
        }
    }

    fn process_scan(&mut self, event: &JournalEvent) -> Vec<Finding> {
        let scan = &event.data;
        let address = id(scan, "SystemAddress").unwrap_or(0);
        let body_id = id(scan, "BodyID").unwrap_or(0);
        if address != 0
            && self
                .systems
                .get(&address)
                .is_some_and(|b| b.contains_key(&body_id))
        {
            return Vec::new();
        }
        if let Some(criteria) = self.criteria.as_mut() {
            criteria.remember_raw(event);
        }
        if address != 0 {
            self.systems
                .entry(address)
                .or_default()
                .insert(body_id, scan.clone());
        } else {
            self.systems.entry(address).or_default();
        }
        if self.systems.len() > 1000 {
            self.systems.retain(|key, _| *key == address);
            if let Some(criteria) = self.criteria.as_mut() {
                let keys: HashSet<(u64, u64)> = self
                    .systems
                    .iter()
                    .chain(self.barycentres.iter())
                    .flat_map(|(address, bodies)| bodies.keys().map(|body| (*address, *body)))
                    .collect();
                criteria.retain_raw_history(&keys);
            }
        }
        // Move the map out briefly so the rules can update notification state
        // without cloning every previously scanned body for each new scan.
        let system = self.systems.remove(&address).unwrap_or_default();
        let mut results = self.check_interest(scan, &system, address);
        let parent_already_scanned =
            first_parent(scan).is_some_and(|(_, parent)| system.contains_key(&parent));
        self.systems.insert(address, system);
        let mut findings = Vec::new();
        if let Some((_, parent)) = first_parent(scan) {
            if let Some(bary) = self
                .barycentres
                .get(&address)
                .and_then(|b| b.get(&parent))
                .cloned()
            {
                if !parent_already_scanned {
                    let synthetic = synthesize_barycentre(&bary, scan);
                    let bary_event = JournalEvent {
                        raw_json: self
                            .criteria
                            .as_ref()
                            .and_then(|criteria| criteria.raw_for(&bary)),
                        event: "Scan".into(),
                        timestamp: string(&synthetic, "timestamp").into(),
                        data: synthetic,
                    };
                    findings.extend(self.process_scan(&bary_event));
                }
            }
        }
        let custom = self.custom(event, address);
        let typed_event = self.event_snapshot.as_ref().map(|data| JournalEvent {
            event: event.event.clone(),
            timestamp: string(data, "timestamp").into(),
            data: data.clone(),
            raw_json: event.raw_json.clone(),
        });
        let event = typed_event.as_ref().unwrap_or(event);
        let scan = &event.data;
        findings.extend(custom.iter().take(self.direct_criteria_count).cloned());
        for (title, detail, system_wide) in results.drain(..) {
            let body = if system_wide {
                string(scan, "StarSystem")
            } else {
                string(scan, "BodyName")
            };
            let finding = self.finding(event, body, &title, &detail);
            self.notification_payloads
                .push((finding.clone(), scan_notification(scan, &title, &detail)));
            findings.push(finding);
        }
        findings.extend(custom.into_iter().skip(self.direct_criteria_count));
        findings
    }

    fn check_interest(
        &mut self,
        scan: &Value,
        system: &Bodies,
        address: u64,
    ) -> Vec<(String, String, bool)> {
        let mut out = Vec::new();
        let mut add = |title: &str, detail: String| out.push((title.to_owned(), detail, false));
        let is_ring = string(scan, "BodyName").contains("Ring");
        // The source framework exposes these fields as System.Single. Keep
        // their arithmetic in f32, including promotion to double for fractional
        // literal comparisons, so threshold and display behavior stay stable.
        let radius = single(scan, "Radius");
        let orbit = single(scan, "SemiMajorAxis");
        let landable = boolean(scan, "Landable");
        let rings = array(scan, "Rings");
        if landable {
            if self.enabled("LandableTerraformable") && !string(scan, "TerraformState").is_empty() {
                add(
                    &format!("Landable and {}", string(scan, "TerraformState")),
                    String::new(),
                );
            }
            if self.enabled("LandableRing") && !rings.is_empty() {
                add("Ringed Landable Body", String::new());
            }
            if self.enabled("LandableAtmosphere") && !string(scan, "Atmosphere").is_empty() {
                add(
                    "Landable with Atmosphere",
                    title_case(string(scan, "Atmosphere")),
                );
            }
            if self.enabled("LandableHighG") && f64::from(single(scan, "SurfaceGravity")) > 29.4 {
                add(
                    "Landable with High Gravity",
                    format!(
                        "Surface gravity: {:.2}g",
                        f64::from(single(scan, "SurfaceGravity")) / 9.81
                    ),
                );
            }
            if self.enabled("LandableLarge") && radius > 18_000_000.0 {
                add(
                    "Landable Large Planet",
                    format!("Radius: {:.0}km", radius / 1000.0),
                );
            }
        }
        if self.enabled("HighValueMappable")
            && (["Earthlike body", "Ammonia world", "Water world"]
                .contains(&string(scan, "PlanetClass"))
                || !string(scan, "TerraformState").is_empty())
        {
            let discovery = if !boolean(scan, "WasMapped") {
                if boolean(scan, "WasDiscovered") {
                    "Unmapped "
                } else {
                    "Undiscovered "
                }
            } else {
                ""
            };
            let terraform = if !string(scan, "TerraformState").is_empty() {
                "Terraformable "
            } else {
                ""
            };
            add(
                "High-Value Body",
                format!(
                    "{discovery}{terraform}{}, {}Ls",
                    title_case(string(scan, "PlanetClass")),
                    grouped(number(scan, "DistanceFromArrivalLS"), 0)
                ),
            );
        }
        if address != 0 && orbit != 0.0 {
            if let Some(parent) = first_parent(scan).and_then(|(_, body)| system.get(&body)) {
                if self.enabled("CloseOrbit") && !is_ring && single(parent, "Radius") * 3.0 > orbit
                {
                    add(
                        "Close Orbit",
                        format!(
                            "Orbital Radius: {}km, Parent Radius: {}km",
                            grouped(orbit / 1000.0, 0),
                            grouped(single(parent, "Radius") / 1000.0, 0)
                        ),
                    );
                }
                if let Some(ring) = array(parent, "Rings").last() {
                    if self.enabled("ShepherdMoon")
                        && !is_ring
                        && single(ring, "OuterRad") > orbit
                        && !string(ring, "Name").contains("Belt")
                    {
                        add(
                            "Shepherd Moon",
                            format!(
                                "Orbit: {}km, Ring Radius: {}km",
                                grouped(orbit / 1000.0, 0),
                                grouped(single(ring, "OuterRad") / 1000.0, 0)
                            ),
                        );
                    }
                }
                if self.enabled("CloseRing") {
                    for ring in array(parent, "Rings") {
                        let separation = (orbit - single(ring, "OuterRad"))
                            .abs()
                            .min((single(ring, "InnerRad") - orbit).abs())
                            - radius;
                        if separation < radius * 10.0 {
                            let kind = if string(ring, "Name").contains("Belt") {
                                "Belt"
                            } else {
                                "Ring"
                            };
                            add(
                                &format!("Close {kind} Proximity"),
                                format!(
                                    "Orbit: {}km, Radius: {}km, Distance from {}: {}km{}",
                                    grouped(orbit / 1000.0, 0),
                                    grouped(radius / 1000.0, 0),
                                    kind.to_lowercase(),
                                    grouped(separation / 1000.0, 0),
                                    if landable { ", Landable" } else { "" }
                                ),
                            );
                        }
                    }
                }
            }
        }
        if self.enabled("DiverseLife") {
            if let Some(signal) = self
                .signals
                .get(&address)
                .and_then(|b| b.get(&id(scan, "BodyID").unwrap_or(0)))
            {
                let count = signal_count(signal, "$SAA_SignalType_Biological;");
                if count > 7 {
                    add("Diverse Life", format!("Biological Signals: {count}"));
                }
            }
        }
        if self.enabled("WideRing") {
            for ring in rings.iter().filter(|r| !string(r, "Name").contains("Belt")) {
                let width = single(ring, "OuterRad") - single(ring, "InnerRad");
                if width > radius * 5.0 {
                    add(
                        "Wide Ring",
                        format!(
                            "{}: Width: {}Ls / {}km, Parent Radius: {}km",
                            string(ring, "Name")
                                .replace(string(scan, "BodyName"), "")
                                .trim(),
                            grouped(width / (299_792_458_u32 as f32), 2),
                            grouped(width / 1000.0, 0),
                            grouped(radius / 1000.0, 0)
                        ),
                    );
                }
            }
        }
        if self.enabled("SmallObject")
            && scan.get("StarType").is_none_or(Value::is_null)
            && scan
                .get("PlanetClass")
                .is_some_and(|class| !class.is_null())
            && string(scan, "PlanetClass") != "Barycentre"
            && radius < 300_000.0
        {
            add(
                "Small Object",
                format!("Radius: {}km", grouped(radius / 1000.0, 0)),
            );
        }
        if self.enabled("HighEccentricity") && f64::from(single(scan, "Eccentricity")) > 0.9 {
            add(
                &format!(
                    "{}Highly Eccentric Orbit",
                    if landable { "Landable with " } else { "" }
                ),
                format!(
                    "Eccentricity: {}",
                    trim_decimal(f64::from(single(scan, "Eccentricity")), 4)
                ),
            );
        }
        let parents = array(scan, "Parents");
        if self.enabled("Nested")
            && !is_ring
            && parents.len() > 1
            && parents[0].get("Planet").is_some()
            && parents[1].get("Planet").is_some()
        {
            add("Nested Moon", String::new());
        }
        let rotation = single(scan, "RotationPeriod");
        if self.enabled("FastRotation")
            && rotation != 0.0
            && !boolean(scan, "TidalLock")
            && rotation.abs() < 28_800.0
            && !is_ring
            && string(scan, "StarType").is_empty()
        {
            add(
                "Non-locked Body with Fast Rotation",
                format!("Period: {} hours", grouped(rotation / 3600.0, 1)),
            );
        }
        let period = single(scan, "OrbitalPeriod");
        if self.enabled("FastOrbit") && period != 0.0 && period.abs() < 28_800.0 && !is_ring {
            add(
                "Fast Orbit",
                format!(
                    "Orbital Period: {} hours",
                    grouped((period / 3600.0).abs(), 1)
                ),
            );
        }
        if (self.enabled("CloseBinary") || self.enabled("CollidingBinary"))
            && f64::from(radius / orbit) > 0.4
        {
            if let Some(("Null", parent)) = first_parent(scan) {
                let partners: Vec<_> = system
                    .iter()
                    .filter(|(body, s)| {
                        **body != id(scan, "BodyID").unwrap_or(0)
                            && first_parent(s).is_some_and(|(_, p)| p == parent)
                    })
                    .map(|(_, s)| s)
                    .collect();
                if partners.len() == 1 {
                    let partner = partners[0];
                    if f64::from(single(partner, "Radius") / single(partner, "SemiMajorAxis")) > 0.4
                    {
                        let colliding = single(partner, "Radius") + radius
                            >= single(partner, "SemiMajorAxis")
                                * (1.0 - single(partner, "Eccentricity"))
                                + orbit * (1.0 - single(scan, "Eccentricity"));
                        let title = if self.enabled("CollidingBinary") && colliding {
                            Some("COLLIDING binary")
                        } else if self.enabled("CloseBinary") {
                            Some("Close binary relative to body size")
                        } else {
                            None
                        };
                        if let Some(title) = title {
                            add(
                                title,
                                format!(
                                    "Orbit: {}km, Radius: {}km, Partner: {}",
                                    grouped((f64::from(orbit) / 1000.0).trunc(), 0),
                                    grouped((f64::from(radius) / 1000.0).trunc(), 0),
                                    string(partner, "BodyName")
                                ),
                            );
                        }
                    }
                }
            }
        }
        if self.enabled("GoodFSDBody") && landable {
            let materials = materials(scan);
            let missing: Vec<_> = BOOST_MATERIALS
                .iter()
                .filter(|m| !materials.contains(&m.to_ascii_lowercase()))
                .collect();
            if missing.len() == 1 {
                add(
                    "5 of 6 Premium Boost Materials",
                    format!("Missing material: {}", missing[0]),
                );
            }
        }
        // The source emits when this scan completes the set, not when a prior scan
        // already completed it. No independent global notification deduplication.
        if (self.enabled("GreenSystem") || self.enabled("GoldSystem"))
            && scan.get("Materials").is_some_and(|v| !v.is_null())
        {
            let found: HashSet<String> = system.values().flat_map(materials).collect();
            let prior: HashSet<String> = system
                .iter()
                .filter(|(body, _)| **body != id(scan, "BodyID").unwrap_or(0))
                .flat_map(|(_, scan)| materials(scan))
                .collect();
            if self.enabled("GreenSystem")
                && BOOST_MATERIALS
                    .iter()
                    .all(|m| found.contains(&m.to_ascii_lowercase()))
                && !BOOST_MATERIALS
                    .iter()
                    .all(|m| prior.contains(&m.to_ascii_lowercase()))
            {
                out.push((
                    "All Premium Boost Materials In System".into(),
                    String::new(),
                    true,
                ));
            }
            if self.enabled("GoldSystem")
                && SURFACE_MATERIALS
                    .iter()
                    .all(|m| found.contains(&m.to_ascii_lowercase()))
                && !SURFACE_MATERIALS
                    .iter()
                    .all(|m| prior.contains(&m.to_ascii_lowercase()))
            {
                out.push((
                    "All Surface Materials In System".into(),
                    String::new(),
                    true,
                ));
            }
        }
        if self.enabled("UncommonSecondary")
            && id(scan, "BodyID").unwrap_or(0) > 0
            && !string(scan, "StarType").trim().is_empty()
            && number(scan, "DistanceFromArrivalLS") > 10.0
            && !["O", "B", "A", "F", "G", "K", "M", "L", "T", "Y", "TTS"]
                .contains(&string(scan, "StarType").to_ascii_uppercase().as_str())
        {
            out.push((
                "Uncommon Secondary Star Type".into(),
                format!(
                    "{}, Distance: {}Ls",
                    star_type_name(string(scan, "StarType")),
                    grouped(number(scan, "DistanceFromArrivalLS"), 0)
                ),
                false,
            ));
        }
        out
    }
}

pub(crate) fn string<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or("")
}
pub(crate) fn number(value: &Value, key: &str) -> f64 {
    value[key]
        .as_f64()
        .or_else(|| match value[key].as_str()? {
            "NaN" => Some(f64::NAN),
            "Infinity" => Some(f64::INFINITY),
            "-Infinity" => Some(f64::NEG_INFINITY),
            _ => None,
        })
        .unwrap_or(0.0)
}
pub(crate) fn single(value: &Value, key: &str) -> f32 {
    number(value, key) as f32
}
pub(crate) fn boolean(value: &Value, key: &str) -> bool {
    value[key].as_bool().unwrap_or(false)
}
pub(crate) fn id(value: &Value, key: &str) -> Option<u64> {
    value[key].as_u64()
}
pub(crate) fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    if key == "Parents" && value.get("__clr_parent").is_some() {
        value["__clr_parent"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[])
    } else {
        value[key].as_array().map(Vec::as_slice).unwrap_or(&[])
    }
}
pub(crate) fn first_parent(value: &Value) -> Option<(&str, u64)> {
    let parent = array(value, "Parents").first()?.as_object()?;
    parent
        .iter()
        .find_map(|(kind, body)| body.as_u64().map(|body| (kind.as_str(), body)))
}
pub(crate) fn signal_count(value: &Value, kind: &str) -> u64 {
    array(value, "Signals")
        .iter()
        .find(|s| string(s, "Type") == kind)
        .and_then(|s| id(s, "Count"))
        .unwrap_or(0)
}
fn materials(scan: &Value) -> HashSet<String> {
    if let Some(materials) = scan["Materials"].as_object() {
        materials.keys().map(|s| s.to_ascii_lowercase()).collect()
    } else {
        array(scan, "Materials")
            .iter()
            .map(|m| string(m, "Name").to_ascii_lowercase())
            .collect()
    }
}
const BOOST_MATERIALS: &[&str] = &[
    "Carbon",
    "Germanium",
    "Arsenic",
    "Niobium",
    "Yttrium",
    "Polonium",
];
const SURFACE_MATERIALS: &[&str] = &[
    "Antimony",
    "Arsenic",
    "Cadmium",
    "Carbon",
    "Chromium",
    "Germanium",
    "Iron",
    "Manganese",
    "Mercury",
    "Molybdenum",
    "Nickel",
    "Niobium",
    "Phosphorus",
    "Polonium",
    "Ruthenium",
    "Selenium",
    "Sulphur",
    "Technetium",
    "Tellurium",
    "Tin",
    "Tungsten",
    "Vanadium",
    "Yttrium",
    "Zinc",
    "Zirconium",
];
fn title_case(value: &str) -> String {
    // TextInfo.ToTitleCase treats punctuation as a word boundary, preserves
    // uppercase acronyms, and lowercases mixed-case words and apostrophe tails.
    let mut result = String::new();
    let mut chars = value.chars().peekable();
    while let Some(first) = chars.next() {
        if !first.is_alphabetic() {
            result.push(first);
            continue;
        }
        let mut word = String::from(first);
        while chars.peek().is_some_and(|c| {
            c.is_alphanumeric() || *c == '\'' || ('\u{0300}'..='\u{036f}').contains(c)
        }) {
            word.push(chars.next().unwrap());
        }
        let lowercase = word.chars().any(char::is_lowercase) || word.contains('\'');
        let mut uppercase = first.to_uppercase();
        let upper = uppercase.next().unwrap_or(first);
        // The source uses simple character casing; it does not expand ß to SS.
        result.push(if uppercase.next().is_none() {
            upper
        } else {
            first
        });
        for character in word.chars().skip(1) {
            result.push(if lowercase {
                character.to_lowercase().next().unwrap_or(character)
            } else {
                character
            });
        }
    }
    result
}
pub(crate) fn grouped(value: impl Into<f64>, precision: usize) -> String {
    let value = value.into();
    let text = format!("{value:.precision$}");
    let (integer, fraction) = text.split_once('.').unwrap_or((&text, ""));
    let (sign, digits) = integer
        .strip_prefix('-')
        .map(|s| ("-", s))
        .unwrap_or(("", integer));
    let mut result = sign.to_owned();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    if precision > 0 {
        result.push('.');
        result.push_str(fraction);
    }
    result
}
fn trim_decimal(value: f64, precision: usize) -> String {
    format!("{value:.precision$}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}
fn star_type_name(star: &str) -> String {
    match star.to_ascii_lowercase().as_str() {
        "b_bluewhitesupergiant" => "B Blue-White Supergiant",
        "a_bluewhitesupergiant" => "A Blue-White Supergiant",
        "f_whitesupergiant" => "F White Supergiant",
        "g_whitesupergiant" => "G White Supergiant",
        "k_orangegiant" => "K Orange Giant",
        "m_redgiant" => "M Red Giant",
        "m_redsupergiant" => "M Red Supergiant",
        "aebe" => "Herbig Ae/Be",
        "w" | "wn" | "wnc" | "wc" | "wo" => "Wolf-Rayet",
        "c" | "cs" | "cn" | "cj" | "ch" | "chd" => "Carbon Star",
        "s" => "S-Type Star",
        "ms" => "MS-Type Star",
        "d" | "da" | "dab" | "dao" | "daz" | "dav" | "db" | "dbz" | "dbv" | "do" | "dov" | "dq"
        | "dc" | "dcv" | "dx" => "White Dwarf",
        "n" => "Neutron Star",
        "h" => "Black Hole",
        "supermassiveblackhole" => "Supermassive Black Hole",
        "x" => "Exotic Star",
        "rogueplanet" => "Rogue Planet",
        "tts" | "t" => "T Tauri Type",
        _ => return format!("{star}-Type Star"),
    }
    .into()
}
fn ordinal_change(ordinal: &str, direction: i32) -> String {
    let Some(last) = ordinal.chars().last() else {
        return String::new();
    };
    let prefix = &ordinal[..ordinal.len() - last.len_utf8()];
    if direction > 0 && (last == 'Z' || last == '9') {
        let changed = ordinal_change(if prefix.is_empty() { " " } else { prefix }, direction);
        let prefix = &changed[..changed.len() - changed.chars().last().unwrap().len_utf8()];
        format!(
            "{}{}",
            prefix,
            char::from_u32(last as u32 - 9).unwrap_or(last)
        )
    } else if direction < 0 && (last == 'A' || last == '0') {
        let changed = ordinal_change(if prefix.is_empty() { " " } else { prefix }, direction);
        let prefix = &changed[..changed.len() - changed.chars().last().unwrap().len_utf8()];
        format!(
            "{}{}",
            prefix,
            char::from_u32(last as u32 + 9).unwrap_or(last)
        )
    } else {
        format!(
            "{prefix}{}",
            char::from_u32((last as i64 + direction as i64) as u32).unwrap_or(last)
        )
    }
}
fn synthesize_barycentre(bary: &Value, child: &Value) -> Value {
    let mut data = bary.clone();
    let system = string(child, "StarSystem");
    let removed = string(child, "BodyName").replace(system, "");
    let affix = removed.trim();
    let name = if affix.is_empty() {
        format!("{} Barycentre", string(child, "BodyName"))
    } else if id(child, "BodyID")
        .unwrap_or(0)
        .checked_sub(id(bary, "BodyID").unwrap_or(0))
        == Some(1)
    {
        format!("{system} {affix}-{}", ordinal_change(affix, 1))
    } else {
        format!("{system} {}-{affix}", ordinal_change(affix, -1))
    };
    data["BodyName"] = name.into();
    data["PlanetClass"] = "Barycentre".into();
    data["Parents"] = array(child, "Parents").iter().skip(1).cloned().collect();
    // ConvertBarycentre creates a Scan and copies its Timestamp and Json, but
    // never assigns JournalBase.Event on the synthetic CLR object.
    data["event"] = Value::Null;
    data
}

fn split_ordinal_ssml(ordinal: &str) -> String {
    ordinal
        .split(' ')
        .map(|part| {
            if part.chars().all(|c| c.is_ascii_digit()) {
                format!(" {part}")
            } else {
                format!("<say-as interpret-as=\"spell-out\">{part}</say-as>")
            }
        })
        .collect()
}
fn short_body_title(system: &str, body: &str, barycentre: bool) -> (String, String) {
    let affix = if body.starts_with(system) {
        body.replace(system, "")
    } else {
        format!(" {body}")
    };
    let mut label = if barycentre { "Barycentre" } else { "Body" };
    let spoken = if affix.is_empty() {
        label = "Primary Star";
        String::new()
    } else if affix.contains("Ring") && affix.len() >= 6 {
        let boundary = affix.len() - 6;
        if affix.is_char_boundary(boundary) && affix.is_char_boundary(boundary + 1) {
            format!(
                "<say-as interpret-as=\"spell-out\">{}</say-as><break strength=\"weak\"/>{}{}",
                &affix[..boundary],
                split_ordinal_ssml(&affix[boundary..boundary + 1]),
                &affix[boundary + 1..]
            )
        } else {
            split_ordinal_ssml(&affix)
        }
    } else if affix.contains(body) {
        body.into()
    } else {
        split_ordinal_ssml(&affix)
    };
    (
        format!("{label}{affix}"),
        format!(
            "<speak version=\"1.0\" xmlns=\"http://www.w3.org/2001/10/synthesis\" xml:lang=\"en-US\"><voice name=\"\">{label} {spoken}</voice></speak>"
        ),
    )
}
fn scan_notification(scan: &Value, description: &str, detail: &str) -> Value {
    let (title, title_ssml) = short_body_title(
        string(scan, "StarSystem"),
        string(scan, "BodyName"),
        string(scan, "PlanetClass") == "Barycentre",
    );
    json!({"title":title,"title_ssml":title_ssml,"detail":description,"extended_details":detail,"sender":"Explorer","coalescing_id":id(scan,"BodyID").unwrap_or(0)})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_json_is_exact_for_current_scan_cached_parent_and_system_body() {
        let path = std::env::temp_dir().join(format!(
            "explorer-raw-{}-{}.lua",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path,"---@Complex\nif scan.BodyID==2 then return true,'Raw',scan.Json..'|'..parents[0].Scan.Json..'|'..system[0].Json end return false,'',''\n---@End").unwrap();
        let mut worker = Explorer::new(
            json!({"EnableCustomCriteria":true,"CustomCriteriaFile":path}),
            None,
        )
        .unwrap();
        let parent = " { \"event\":\"Scan\", \"SystemAddress\":1, \"BodyID\":1, \"BodyName\":\"Test 1\", \"Radius\":1000000 } ";
        let child = "{\"BodyID\":2, \"BodyName\":\"Test 2\",\"SystemAddress\":1,\"Radius\":1000000,\"Parents\":[{\"Planet\":1}],\"event\":\"Scan\"}";
        worker.on_event(&JournalEvent::parse(parent).unwrap());
        let findings = worker.on_event(&JournalEvent::parse(child).unwrap());
        assert_eq!(
            findings.iter().find(|f| f.title == "Raw").unwrap().detail,
            format!("{child}|{parent}|{parent}")
        );
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn source_globals_initialize_even_disabled_and_preferences_keep_lua_state() {
        let path = std::env::temp_dir().join(format!(
            "explorer-source-globals-{}-{}.lua",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path,"---@Global\ncount=40\nnotify('Startup','Global','Initialised')\n---@End\n---@Complex\ncount=count+1 return true,'Counter',tostring(count)\n---@End").unwrap();
        let settings = json!({"CustomCriteriaFile":path,"EnableCustomCriteria":false});
        let mut worker = Explorer::new(settings.clone(), None).unwrap();
        let startup = worker.take_startup_findings();
        assert_eq!(startup.len(), 1);
        let payload = worker.take_notification_payload(&startup[0]).unwrap();
        assert_eq!(payload["title"], "Startup");
        assert_eq!(payload["grid_time"], "");
        let mut enabled = settings;
        enabled["EnableCustomCriteria"] = json!(true);
        worker.apply_preferences(Explorer::prepare_preferences(enabled).unwrap());
        assert!(worker.take_startup_findings().is_empty());
        assert_eq!(worker.on_event(&event(scan(1)))[0].detail, "41");
        worker.set_batch(true);
        assert_eq!(worker.take_startup_findings().len(), 1);
        std::fs::write(&path,"---@Global\ncount=999\n---@End\n---@Complex\ncount=count+1 return true,'Counter',tostring(count)\n---@End").unwrap();
        assert_eq!(worker.on_event(&event(scan(1)))[0].detail, "41");
        worker.set_batch(false);
        assert_eq!(worker.on_event(&event(scan(2)))[0].detail, "1000");
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn installed_explorer_event_history_and_notification_oracle() {
        let cases: Vec<Value> =
            serde_json::from_str(include_str!("../../fixtures/explorer-events-oracle.json"))
                .unwrap();
        for mut case in cases {
            for step in case["Expected"].as_array_mut().unwrap() {
                for row in step["Rows"].as_array_mut().unwrap() {
                    row["Time"] = crate::communicator::tests::localize_oracle_time(&row["Time"]);
                }
            }
            let mut worker = Explorer::new(case["Settings"].clone(), None).unwrap();
            let mut rows = Vec::<Value>::new();
            for (index, step) in case["Events"].as_array().unwrap().iter().enumerate() {
                let old = worker.current_system.clone();
                let mut calls = Vec::new();
                if step["event"] == "Batch" {
                    worker.set_batch(step["Enabled"].as_bool().unwrap());
                    if worker.batch {
                        rows.clear();
                    }
                } else {
                    for finding in worker.on_event(&event(step.clone())) {
                        let payload = worker.take_notification_payload(&finding).unwrap();
                        let (grid_body, grid_title, grid_detail) =
                            (&finding.body, &finding.title, &finding.detail);
                        rows.push(json!({"Time":crate::communicator::display_timestamp(&finding.timestamp),"BodyName":grid_body,"Description":grid_title,"Details":grid_detail}));
                        calls.push(json!({"Title":payload["title"],"TitleSsml":payload.get("title_ssml"),"Detail":payload["detail"],"ExtendedDetails":payload["extended_details"],"Sender":payload["sender"],"CoalescingId":payload["coalescing_id"]}));
                    }
                    if old != worker.current_system
                        && worker.enabled("OnlyShowCurrentSystem")
                        && !worker.batch
                    {
                        rows.clear();
                    }
                }
                assert_eq!(
                    json!({"Rows":rows,"Calls":calls}),
                    case["Expected"][index],
                    "{} step{index}",
                    case["Label"]
                );
            }
        }
    }
    #[test]
    fn installed_default_criteria_differential_oracle() {
        let cases: Vec<Value> =
            serde_json::from_str(include_str!("../../fixtures/explorer-criteria-oracle.json"))
                .unwrap();
        for case in cases {
            let mut worker = Explorer::new(json!({}), None).unwrap();
            worker.settings = case["Settings"].clone();
            let history: Bodies = case["History"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| (id(v, "BodyID").unwrap(), v.clone()))
                .collect();
            worker.signals.insert(
                1,
                case["Signals"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| (id(v, "BodyID").unwrap(), v.clone()))
                    .collect(),
            );
            let actual:Vec<Value>=worker.check_interest(&case["Scan"],&history,1).into_iter().map(|(title,detail,system_wide)|json!({"Title":title,"Detail":detail,"SystemWide":system_wide})).collect();
            assert_eq!(json!(actual), case["Expected"], "{}", case["Label"]);
        }
    }
    fn event(mut value: Value) -> JournalEvent {
        if value.get("timestamp").is_none() {
            value["timestamp"] = "2026-10-25T00:00:00Z".into();
        }
        JournalEvent {
            raw_json: None,
            event: string(&value, "event").into(),
            timestamp: string(&value, "timestamp").into(),
            data: value,
        }
    }
    fn scan(body: u64) -> Value {
        json!({"event":"Scan","BodyID":body,"SystemAddress":1,"StarSystem":"Sol","BodyName":format!("Sol {body}"),"PlanetClass":"Rocky body","Radius":1000000})
    }
    #[test]
    fn thresholds_and_duplicates() {
        let mut e = Explorer::new(json!({}), None).unwrap();
        let mut s = scan(1);
        s["Landable"] = true.into();
        s["SurfaceGravity"] = 29.4.into();
        assert!(e.on_event(&event(s.clone())).is_empty());
        s["BodyID"] = 2.into();
        s["SurfaceGravity"] = 29.401.into();
        let ev = event(s);
        let found = e.on_event(&ev);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].title, "Landable with High Gravity");
        assert!(e.on_event(&ev).is_empty());
    }
    #[test]
    fn parent_ring_and_binary() {
        let mut e = Explorer::new(json!({}), None).unwrap();
        let mut p = scan(1);
        p["Radius"] = 10000000.into();
        p["Rings"] = json!([{"Name":"Sol 1 A Ring","InnerRad":15000000,"OuterRad":20000000}]);
        e.on_event(&event(p));
        let mut s = scan(2);
        s["Parents"] = json!([{"Planet":1}]);
        s["SemiMajorAxis"] = 21000000.into();
        let found = e.on_event(&event(s));
        assert!(found.iter().any(|f| f.title == "Close Orbit"));
        assert!(found.iter().any(|f| f.title == "Close Ring Proximity"));
    }
    #[test]
    fn green_emits_once_and_resets() {
        let mut e = Explorer::new(json!({"GreenSystem":true}), None).unwrap();
        let mut s = scan(1);
        s["Materials"] =
            json!({"Carbon":1,"Germanium":1,"Arsenic":1,"Niobium":1,"Yttrium":1,"Polonium":1});
        let found = e.on_event(&event(s.clone()));
        assert_eq!(
            found
                .iter()
                .filter(|f| f.title == "All Premium Boost Materials In System")
                .count(),
            1
        );
        s["BodyID"] = 2.into();
        assert!(!e
            .on_event(&event(s.clone()))
            .iter()
            .any(|f| f.title == "All Premium Boost Materials In System"));
        e.reset();
        assert!(e
            .on_event(&event(s))
            .iter()
            .any(|f| f.title == "All Premium Boost Materials In System"));
    }
    #[test]
    fn all_independent_checks() {
        let mut e = Explorer::new(
            json!({"FastOrbit":true,"UncommonSecondary":true,"HighValueMappable":true}),
            None,
        )
        .unwrap();
        let mut s = scan(3);
        s["Landable"] = true.into();
        s["TerraformState"] = "Terraformable".into();
        s["Atmosphere"] = "thin argon atmosphere".into();
        s["Radius"] = 19000000.into();
        s["SurfaceGravity"] = 40.into();
        s["RotationPeriod"] = 3600.into();
        s["OrbitalPeriod"] = 3600.into();
        s["Eccentricity"] = 0.95.into();
        s["Parents"] = json!([{"Planet":1},{"Planet":0}]);
        s["Rings"] = json!([{"Name":"Sol 3 A Ring","OuterRad":200000000,"InnerRad":20000000}]);
        let f = e.on_event(&event(s));
        for title in [
            "Landable and Terraformable",
            "Landable with Atmosphere",
            "Landable with High Gravity",
            "Landable Large Planet",
            "High-Value Body",
            "Ringed Landable Body",
            "Wide Ring",
            "Nested Moon",
            "Fast Orbit",
            "Landable with Highly Eccentric Orbit",
            "Non-locked Body with Fast Rotation",
        ] {
            assert!(f.iter().any(|f| f.title == title), "missing {title}");
        }
    }
    #[test]
    fn uncommon_stars_signals_and_materials() {
        let mut e=Explorer::new(json!({"UncommonSecondary":true,"DiverseLife":true,"GoodFSDBody":true,"GoldSystem":true,"Codex":true}),None).unwrap();
        e.on_event(&event(
            json!({"event":"Location","StarSystem":"Sol","SystemAddress":1}),
        ));
        e.on_event(&event(json!({"event":"FSSBodySignals","SystemAddress":1,"BodyID":2,"Signals":[{"Type":"$SAA_SignalType_Biological;","Count":8}]})));
        let mut s = scan(2);
        s["Materials"] = json!({"Carbon":1,"Germanium":1,"Arsenic":1,"Niobium":1,"Yttrium":1});
        s["Landable"] = true.into();
        let found = e.on_event(&event(s));
        assert!(found.iter().any(|f| f.title == "Diverse Life"));
        assert_eq!(
            found
                .iter()
                .find(|f| f.title == "5 of 6 Premium Boost Materials")
                .unwrap()
                .detail,
            "Missing material: Polonium"
        );
        let mut star = scan(3);
        star["StarType"] = "N".into();
        star["DistanceFromArrivalLS"] = 500.into();
        assert!(e
            .on_event(&event(star))
            .iter()
            .any(|f| f.detail == "Neutron Star, Distance: 500Ls"));
        let found=e.on_event(&event(json!({"event":"CodexEntry","SystemAddress":1,"BodyID":2,"System":"Sol","IsNewEntry":true,"Category_Localised":"Organic","Name_Localised":"Aleoida"})));
        assert_eq!(found[0].body, "Sol 2");
        assert_eq!(found[0].title, "New Organic Codex Entry");
        let mut s = scan(4);
        s["Materials"] = Value::Array(
            SURFACE_MATERIALS
                .iter()
                .map(|name| json!({"Name":name,"Percent":1}))
                .collect(),
        );
        assert!(e
            .on_event(&event(s))
            .iter()
            .any(|f| f.title == "All Surface Materials In System"));
    }
    #[test]
    fn close_binary_and_barycentre_reconstruction() {
        let mut e = Explorer::new(json!({"FastOrbit":true}), None).unwrap();
        e.on_event(&event(json!({"event":"ScanBaryCentre","StarSystem":"Sol","SystemAddress":1,"BodyID":1,"OrbitalPeriod":10000,"Parents":[{"Star":0}]})));
        let mut a = scan(2);
        a["BodyName"] = "Sol A".into();
        a["Parents"] = json!([{"Null":1},{"Star":0}]);
        a["Radius"] = 1000000.into();
        a["SemiMajorAxis"] = 1200000.into();
        a["Eccentricity"] = 0.2.into();
        let first = e.on_event(&event(a.clone()));
        assert!(first
            .iter()
            .any(|f| f.title == "Fast Orbit" && f.body == "Sol A-B"));
        a["BodyID"] = 3.into();
        a["BodyName"] = "Sol B".into();
        assert!(e
            .on_event(&event(a))
            .iter()
            .any(|f| f.title == "COLLIDING binary"));
    }
    #[test]
    fn format_values() {
        assert_eq!(grouped(1234567.25, 2), "1,234,567.25");
        assert_eq!(ordinal_change("A", 1), "B");
        assert_eq!(ordinal_change("2", -1), "1");
    }
    #[test]
    fn source_title_case_handles_punctuation_acronyms_and_mixed_case() {
        for (input, expected) in [
            (
                "thin argon, \"trace\" atmosphere",
                "Thin Argon, \"Trace\" Atmosphere",
            ),
            ("hELLo WORLD", "Hello WORLD"),
            ("1abc 2DEF", "1Abc 2DEF"),
            ("earth-LIKE body", "Earth-LIKE Body"),
            ("o'BRIEN", "O'brien"),
            ("hydrogen-rich atmosphere", "Hydrogen-Rich Atmosphere"),
            ("SO2 gas", "SO2 Gas"),
            ("ß test", "ß Test"),
            ("mixed123ABC x-y", "Mixed123abc X-Y"),
        ] {
            assert_eq!(title_case(input), expected);
        }
    }
    #[test]
    fn source_single_precision_thresholds_and_period_formatting() {
        // Outcomes were checked against the unchanged C# DefaultCriteria and
        // original Scan property types using a differential oracle.
        for (field, value, title, expected) in [
            (
                "SurfaceGravity",
                29.4000001,
                "Landable with High Gravity",
                false,
            ),
            (
                "SurfaceGravity",
                29.4000009,
                "Landable with High Gravity",
                true,
            ),
            ("Radius", 18_000_000.1, "Landable Large Planet", false),
            ("Radius", 18_000_002.0, "Landable Large Planet", true),
            ("Radius", 299_999.999, "Small Object", false),
            ("Radius", 299_999.98, "Small Object", true),
            (
                "Eccentricity",
                0.900000001,
                "Landable with Highly Eccentric Orbit",
                false,
            ),
            (
                "Eccentricity",
                0.90000003,
                "Landable with Highly Eccentric Orbit",
                true,
            ),
            (
                "RotationPeriod",
                28_799.999_9,
                "Non-locked Body with Fast Rotation",
                false,
            ),
            (
                "RotationPeriod",
                -28_799.999_9,
                "Non-locked Body with Fast Rotation",
                false,
            ),
            ("OrbitalPeriod", 28_799.999_9, "Fast Orbit", false),
        ] {
            let mut e = Explorer::new(json!({"FastOrbit":true}), None).unwrap();
            let mut s = scan(1);
            s["Landable"] = true.into();
            s[field] = value.into();
            assert_eq!(
                e.on_event(&event(s)).iter().any(|f| f.title == title),
                expected,
                "{field}={value}"
            );
        }
        let mut e = Explorer::new(json!({"FastOrbit":true}), None).unwrap();
        let mut s = scan(1);
        s["RotationPeriod"] = 12060.0001.into();
        s["OrbitalPeriod"] = 12060.0001.into();
        let findings = e.on_event(&event(s));
        assert_eq!(
            findings
                .iter()
                .find(|f| f.title == "Fast Orbit")
                .unwrap()
                .detail,
            "Orbital Period: 3.3 hours"
        );
        assert_eq!(
            findings
                .iter()
                .find(|f| f.title == "Non-locked Body with Fast Rotation")
                .unwrap()
                .detail,
            "Period: 3.3 hours"
        );
    }
    #[test]
    fn binary_ratio_uses_single_division_then_double_comparison() {
        let mut e = Explorer::new(json!({}), None).unwrap();
        let mut partner = scan(1);
        partner["Parents"] = json!([{"Null":9}]);
        partner["SemiMajorAxis"] = 2_000_000.into();
        e.on_event(&event(partner));
        let mut s = scan(2);
        s["Parents"] = json!([{"Null":9}]);
        s["SemiMajorAxis"] = 2_500_000.into();
        let findings = e.on_event(&event(s));
        let binary = findings
            .iter()
            .find(|f| f.title == "Close binary relative to body size")
            .unwrap();
        assert_eq!(
            binary.detail,
            "Orbit: 2,500km, Radius: 1,000km, Partner: Sol 1"
        );
    }
}
