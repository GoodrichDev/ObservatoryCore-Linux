//! Biological signal tracking and live three-sample distance guidance.
use observatory_domain::{Finding, JournalEvent};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub fn default_settings() -> Value {
    json!({"OverlayEnabled": true, "OverlayIsSticky": true})
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BotanistRow {
    pub system_address: u64,
    pub body_id: u64,
    pub body: String,
    pub bio_total: u64,
    pub species: String,
    pub genus: String,
    pub analysed: bool,
    pub colony_distance: u32,
    #[serde(default)]
    pub first_on_body: bool,
}

impl BotanistRow {
    pub fn grid_values(&self) -> Vec<String> {
        let samples = self.species != "(NO SAMPLES TAKEN)";
        vec![
            if self.first_on_body {
                self.body.clone()
            } else {
                String::new()
            },
            if self.first_on_body {
                self.bio_total.to_string()
            } else {
                String::new()
            },
            self.species.clone(),
            if self.analysed {
                "✓".into()
            } else {
                String::new()
            },
            if samples {
                format!("{}m", self.colony_distance)
            } else {
                String::new()
            },
        ]
    }
}

#[derive(Clone, Debug, Default)]
struct Planet {
    body: String,
    bio_total: u64,
    species: Vec<(String, Sample)>,
}

#[derive(Clone, Debug)]
struct Sample {
    genus: String,
    analysed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Location {
    latitude: f64,
    longitude: f64,
}

impl Location {
    fn from_status(status: &Value) -> Option<Self> {
        if status.is_null() {
            return None;
        }
        let latitude = status["Latitude"].as_f64().unwrap_or(0.0);
        let longitude = status["Longitude"].as_f64().unwrap_or(0.0);
        if (latitude, longitude) == (200.0, 200.0) {
            None
        } else {
            Some(Self {
                latitude,
                longitude,
            })
        }
    }
}

/// The UI renders this as a persistent panel and may replace it on each status
/// update. It is cleared on analysis, leaving the body, jumping or shutdown.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SamplerStatus {
    pub species: String,
    pub sample: u8,
    pub colony_distance: u32,
    pub first_distance: Option<f64>,
    pub second_distance: Option<f64>,
    pub sticky: bool,
}

impl SamplerStatus {
    pub fn detail(&self) -> String {
        let format_distance = |distance: f64| crate::explorer::grouped(distance, 0) + "m";
        let first = self.first_distance.map(format_distance).unwrap_or_default();
        let second = self
            .second_distance
            .map(format_distance)
            .unwrap_or_default();
        let both = self.first_distance.is_some() && self.second_distance.is_some();
        format!(
            "Sample {} of 3\nColony distance: {}m\nCurrent distance{}: {}{}{}",
            self.sample,
            self.colony_distance,
            if both { "s" } else { "" },
            first,
            if both { " - " } else { "" },
            second
        )
    }

    /// Whether the commander is far enough from every previous sample.
    pub fn ready(&self) -> bool {
        self.first_distance
            .is_some_and(|distance| distance >= self.colony_distance as f64)
            && (self.sample < 2
                || self
                    .second_distance
                    .is_some_and(|distance| distance >= self.colony_distance as f64))
    }
}

pub struct Botanist {
    settings: Value,
    planets: Vec<((u64, u64), Planet)>,
    odyssey_loaded: bool,
    system: String,
    system_address: u64,
    current_body: String,
    current_body_id: Option<u64>,
    status: Value,
    first_location: Option<Location>,
    second_location: Option<Location>,
    sample_radius: f64,
    sampler: Option<SamplerStatus>,
    batch: bool,
    last_timestamp: String,
    sampler_id: Option<String>,
    notification_events: Vec<Value>,
    hidden_ui: bool,
}

impl Botanist {
    pub fn new(settings: Value) -> Self {
        Self {
            settings,
            planets: Vec::new(),
            odyssey_loaded: false,
            system: String::new(),
            system_address: 0,
            current_body: String::new(),
            current_body_id: None,
            status: Value::Null,
            first_location: None,
            second_location: None,
            sample_radius: 0.0,
            sampler: None,
            batch: false,
            last_timestamp: String::new(),
            sampler_id: None,
            notification_events: Vec::new(),
            hidden_ui: false,
        }
    }

    pub fn reset(&mut self) {
        let settings = self.settings.clone();
        let batch = self.batch;
        *self = Self::new(settings);
        self.batch = batch;
    }

    pub fn settings(&self) -> &Value {
        &self.settings
    }

    pub fn set_settings(&mut self, settings: Value) {
        self.settings = settings;
    }

    pub fn set_batch(&mut self, batch: bool) {
        if self.batch && !batch {
            self.hidden_ui = false;
        }
        self.batch = batch;
    }
    pub fn clear_visible_rows(&mut self) {
        self.hidden_ui = true;
    }

    pub fn take_notification_events(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.notification_events)
    }

    pub fn sampler(&self) -> Option<&SamplerStatus> {
        self.sampler.as_ref()
    }

    pub fn rows(&self) -> Vec<BotanistRow> {
        let mut rows = Vec::new();
        if self.batch || self.hidden_ui {
            return rows;
        }
        for ((system_address, body_id), planet) in &self.planets {
            if planet.species.is_empty() {
                rows.push(BotanistRow {
                    system_address: *system_address,
                    body_id: *body_id,
                    body: planet.body.clone(),
                    bio_total: planet.bio_total,
                    species: "(NO SAMPLES TAKEN)".into(),
                    genus: String::new(),
                    analysed: false,
                    colony_distance: 0,
                    first_on_body: true,
                });
            } else {
                for (index, (species, sample)) in planet.species.iter().enumerate() {
                    rows.push(BotanistRow {
                        system_address: *system_address,
                        body_id: *body_id,
                        body: planet.body.clone(),
                        bio_total: planet.bio_total,
                        species: species.clone(),
                        genus: sample.genus.clone(),
                        analysed: sample.analysed,
                        colony_distance: colony_distance(&sample.genus),
                        first_on_body: index == 0,
                    });
                }
            }
        }
        rows
    }

    pub fn on_event(&mut self, event: &JournalEvent) -> Vec<Finding> {
        self.last_timestamp = event.timestamp.clone();
        if !self.batch
            && matches!(
                event.event.as_str(),
                "SAASignalsFound" | "FSSBodySignals" | "ScanOrganic"
            )
        {
            self.hidden_ui = false;
        }
        match event.event.as_str() {
            "LoadGame" => self.odyssey_loaded = event.boolean("Odyssey"),
            "FSDJump" | "Location" | "CarrierJump" => {
                if let Some(address) = event.id("SystemAddress") {
                    self.system_address = address;
                }
                if !event.str("StarSystem").is_empty() {
                    self.system = event.str("StarSystem").to_owned();
                }
                if event.event != "Location" {
                    self.current_body.clear();
                    self.current_body_id = None;
                    self.clear_sampler();
                }
            }
            "ApproachBody" | "Touchdown" => {
                self.remember_body(event);
            }
            "FSSBodySignals" | "SAASignalsFound" => self.signals_found(event),
            "ScanOrganic" => return self.scan_organic(event),
            "LeaveBody" | "Shutdown" => self.clear_sampler(),
            _ => {}
        }
        Vec::new()
    }

    pub fn on_status(&mut self, status: &Value) -> Vec<Finding> {
        self.status = status.clone();
        self.refresh_distances();
        self.queue_sampler("update");
        self.sampler
            .as_ref()
            .map(|sampler| vec![self.finding(sampler)])
            .unwrap_or_default()
    }

    fn setting(&self, key: &str, default: bool) -> bool {
        self.settings
            .get(key)
            .and_then(Value::as_bool)
            .unwrap_or(default)
    }

    fn remember_body(&mut self, event: &JournalEvent) {
        if let Some(address) = event.id("SystemAddress") {
            self.system_address = address;
        }
        if let Some(id) = event.id("BodyID") {
            self.current_body_id = Some(id);
        }
        let name = if !event.str("BodyName").is_empty() {
            event.str("BodyName")
        } else {
            event.str("Body")
        };
        if !name.is_empty() {
            self.current_body = name.to_owned();
        }
    }

    fn signals_found(&mut self, event: &JournalEvent) {
        if !self.odyssey_loaded {
            return;
        }
        let Some(signals) = event.data.get("Signals").and_then(Value::as_array) else {
            return;
        };
        let Some(total) = signals
            .iter()
            .find(|signal| signal["Type"] == "$SAA_SignalType_Biological;")
            .and_then(|signal| signal["Count"].as_u64())
        else {
            return;
        };
        let key = (
            event.id("SystemAddress").unwrap_or(self.system_address),
            event.id("BodyID").unwrap_or(0),
        );
        if !self.planets.iter().any(|(existing, _)| *existing == key) {
            self.planets.push((
                key,
                Planet {
                    body: event.str("BodyName").into(),
                    bio_total: total,
                    species: Vec::new(),
                },
            ));
        }
    }

    fn scan_organic(&mut self, event: &JournalEvent) -> Vec<Finding> {
        let key = (
            event.id("SystemAddress").unwrap_or(0),
            event.id("Body").unwrap_or(0),
        );
        let Some(species) = event.data.get("Species_Localised").and_then(Value::as_str) else {
            return Vec::new();
        };
        let genus = english_genus(event.str("Genus"), event.str("Genus_Localised"));
        let Some(index) = self
            .planets
            .iter()
            .position(|(existing, _)| *existing == key)
        else {
            self.planets.push((
                key,
                Planet {
                    body: String::new(),
                    bio_total: 0,
                    species: vec![(
                        species.into(),
                        Sample {
                            genus,
                            analysed: false,
                        },
                    )],
                },
            ));
            return Vec::new();
        };
        let scan_type = event.str("ScanType");
        if scan_type == "Analyse" {
            if let Some((_, sample)) = self.planets[index]
                .1
                .species
                .iter_mut()
                .find(|(name, _)| name == species)
            {
                sample.analysed = true;
            }
            self.clear_sampler();
            self.first_location = None;
            self.second_location = None;
            return Vec::new();
        }
        if !matches!(scan_type, "Log" | "Sample") {
            return Vec::new();
        }
        if !self.planets[index]
            .1
            .species
            .iter()
            .any(|(name, _)| name == species)
        {
            self.planets[index].1.species.push((
                species.into(),
                Sample {
                    genus,
                    analysed: false,
                },
            ));
        }
        if self.batch || !self.setting("OverlayEnabled", true) {
            return Vec::new();
        }
        let location = Location::from_status(&self.status);
        if scan_type == "Log" {
            self.first_location = location;
            self.second_location = None;
            self.clear_sampler();
        } else {
            self.second_location = location;
        }
        self.sample_radius = self.status["PlanetRadius"].as_f64().unwrap_or(0.0);
        if self.sampler_id.is_none() {
            static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
            self.sampler_id = Some(format!(
                "core.botanist-sampler-{}",
                NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
        }
        self.sampler = Some(SamplerStatus {
            species: species.into(),
            sample: if scan_type == "Log" { 1 } else { 2 },
            colony_distance: colony_distance(&english_genus(event.str("Genus"), "")),
            first_distance: None,
            second_distance: None,
            sticky: self.setting("OverlayIsSticky", true),
        });
        self.refresh_distances();
        self.queue_sampler(
            if !self.setting("OverlayIsSticky", true) || scan_type == "Log" {
                "show"
            } else {
                "update"
            },
        );
        vec![self.finding(self.sampler.as_ref().unwrap())]
    }

    fn queue_sampler(&mut self, action: &str) {
        if let (Some(sampler), Some(id)) = (&self.sampler, &self.sampler_id) {
            self.notification_events.push(json!({"id":id,"action":action,"title":sampler.species,"detail":sampler.detail(),"timeout":if sampler.sticky{0}else{-1},"rendering":1,"sender":"Botanist"}));
        }
    }

    fn refresh_distances(&mut self) {
        if let Some(sampler) = self.sampler.as_mut() {
            let current = Location::from_status(&self.status);
            let distance = |start: Option<Location>| -> Option<f64> {
                let start = start?;
                let end = current?;
                Some({
                    botanist_distance(
                        (start.latitude, start.longitude),
                        (end.latitude, end.longitude),
                        self.sample_radius,
                    )
                })
            };
            sampler.first_distance = distance(self.first_location);
            sampler.second_distance = distance(self.second_location);
        }
    }

    fn finding(&self, sampler: &SamplerStatus) -> Finding {
        Finding {
            plugin: "Botanist".into(),
            timestamp: self.last_timestamp.clone(),
            system: self.system.clone(),
            body: self.current_body.clone(),
            title: sampler.species.clone(),
            detail: sampler.detail(),
        }
    }

    fn clear_sampler(&mut self) {
        if let Some(id) = self.sampler_id.take() {
            self.notification_events
                .push(json!({"id":id,"action":"cancel"}));
        }
        self.sampler = None;
    }
}

/// The original Botanist uses this exact degree conversion constant.
fn botanist_distance(first: (f64, f64), second: (f64, f64), radius: f64) -> f64 {
    let lat1 = first.0 * 0.0174533;
    let delta_lat = (second.0 - first.0) * 0.0174533;
    let delta_lon = (second.1 - first.1) * 0.0174533;
    let h =
        (delta_lat / 2.0).sin().powi(2) + lat1.cos() * lat1.cos() * (delta_lon / 2.0).sin().powi(2);
    2.0 * radius * h.sqrt().asin()
}

/// Shared spherical distance used by the other workers. Botanist's original
/// approximation is isolated above because its source arithmetic differs.
pub fn great_circle_distance(first: (f64, f64), second: (f64, f64), radius: f64) -> f64 {
    let lat1 = first.0.to_radians();
    let lat2 = second.0.to_radians();
    let h = ((second.0 - first.0).to_radians() / 2.0).sin().powi(2)
        + lat1.cos() * lat2.cos() * ((second.1 - first.1).to_radians() / 2.0).sin().powi(2);
    2.0 * radius * h.clamp(0.0, 1.0).sqrt().asin()
}

pub fn english_genus(identifier: &str, localized: &str) -> String {
    match identifier {
        "$Codex_Ent_Aleoids_Genus_Name;" => "Aleoida",
        "$Codex_Ent_Bacterial_Genus_Name;" => "Bacterium",
        "$Codex_Ent_Cactoid_Genus_Name;" => "Cactoida",
        "$Codex_Ent_Clepeus_Genus_Name;;" | "$Codex_Ent_Clypeus_Genus_Name;" => "Clypeus",
        "$Codex_Ent_Conchas_Genus_Name;" => "Concha",
        "$Codex_Ent_Electricae_Genus_Name;" => "Electricae",
        "$Codex_Ent_Fonticulus_Genus_Name;" => "Fonticulua",
        "$Codex_Ent_Shrubs_Genus_Name;" => "Frutexa",
        "$Codex_Ent_Fumerolas_Genus_Name;" => "Fumerola",
        "$Codex_Ent_Fungoids_Genus_Name;" => "Fungoida",
        "$Codex_Ent_Osseus_Genus_Name;" => "Osseus",
        "$Codex_Ent_Recepta_Genus_Name;" => "Recepta",
        "$Codex_Ent_Stratum_Genus_Name;" => "Stratum",
        "$Codex_Ent_Tubus_Genus_Name;" => "Tubus",
        "$Codex_Ent_Tussocks_Genus_Name;" => "Tussock",
        "$Codex_Ent_Ground_Struct_Ice_Name;" => "Crystalline Shards",
        "$Codex_Ent_Brancae_Name;" => "Brain Trees",
        "$Codex_Ent_Seed_Name;" => "Brain Tree",
        "$Codex_Ent_Sphere_Name;" => "Anemone",
        "$Codex_Ent_Tube_Name;" => "Sinuous Tubers",
        "$Codex_Ent_Vents_Name;" => "Amphora Plant",
        "$Codex_Ent_Cone_Name;" => "Bark Mounds",
        _ => {
            if localized.is_empty() {
                identifier
            } else {
                localized
            }
        }
    }
    .to_owned()
}

pub fn colony_distance(genus: &str) -> u32 {
    match genus {
        "Aleoida" | "Clypeus" | "Concha" | "Frutexa" | "Recepta" => 150,
        "Bacterium" | "Fonticulua" | "Stratum" => 500,
        "Cactoida" | "Fungoida" => 300,
        "Electricae" => 1000,
        "Osseus" | "Tubus" => 800,
        "Tussock" => 200,
        _ => 100,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installed_botanist_state_and_notification_oracle() {
        let cases: Vec<Value> =
            serde_json::from_str(include_str!("../../fixtures/botanist-oracle.json")).unwrap();
        for case in cases {
            let mut worker = Botanist::new(default_settings());
            for (index, step) in case["Steps"].as_array().unwrap().iter().enumerate() {
                match step["event"].as_str().unwrap() {
                    "Status" => {
                        worker.on_status(step);
                    }
                    "Settings" => worker.set_settings(step.clone()),
                    "Batch" => worker.set_batch(step["Enabled"].as_bool().unwrap()),
                    _ => {
                        worker.on_event(&event(step.clone()));
                    }
                }
                let rows:Vec<Value>=worker.rows().iter().map(|row|{let v=row.grid_values();json!({"Body":v[0],"BioTotal":v[1],"Species":v[2],"Analysed":v[3],"ColonyDistance":v[4]})}).collect();
                let calls:Vec<Value>=worker.take_notification_events().iter().map(|v|if v["action"]=="cancel" {json!({"Action":"cancel"})}else{json!({"Action":v["action"],"Title":v["title"],"Detail":v["detail"],"Timeout":v["timeout"],"Rendering":v["rendering"]})}).collect();
                assert_eq!(
                    json!({"Rows":rows,"Calls":calls}),
                    case["Expected"][index],
                    "{} step{index}",
                    case["Label"]
                );
            }
        }
    }
    fn event(data: Value) -> JournalEvent {
        JournalEvent {
            raw_json: None,
            event: data["event"].as_str().unwrap().into(),
            timestamp: "2026-01-01T00:00:00Z".into(),
            data,
        }
    }

    #[test]
    fn tracks_signals_samples_and_analysis_without_duplicate_rows() {
        let mut botanist = Botanist::new(default_settings());
        botanist.on_event(&event(json!({"event":"LoadGame","Odyssey":true})));
        botanist.on_event(&event(
            json!({"event":"SAASignalsFound","BodyName":"Test 1","SystemAddress":42,"BodyID":3,
            "Signals":[{"Type":"$SAA_SignalType_Biological;","Count":2}]}),
        ));
        assert_eq!(botanist.rows()[0].bio_total, 2);
        let scan = json!({"event":"ScanOrganic","SystemAddress":42,"Body":3,"Species_Localised":"Stratum Tectonicas",
            "Genus":"$Codex_Ent_Stratum_Genus_Name;","ScanType":"Log"});
        botanist.on_event(&event(scan.clone()));
        botanist.on_event(&event(scan));
        assert_eq!(botanist.rows().len(), 1);
        assert_eq!(botanist.rows()[0].colony_distance, 500);
        botanist.on_event(&event(
            json!({"event":"ScanOrganic","SystemAddress":42,"Body":3,
            "Species_Localised":"Stratum Tectonicas","ScanType":"Analyse"}),
        ));
        assert!(botanist.rows()[0].analysed);
        assert!(botanist.sampler().is_none());
    }

    #[test]
    fn sampler_updates_two_distances_and_closes_on_jump() {
        let mut botanist = Botanist::new(default_settings());
        botanist.on_status(&json!({"Latitude":0.0,"Longitude":0.0,"PlanetRadius":1_000_000.0}));
        let mut scan = json!({"event":"ScanOrganic","Body":1,"Species_Localised":"Tussock","Genus":"$Codex_Ent_Tussocks_Genus_Name;","ScanType":"Log"});
        assert!(botanist.on_event(&event(scan.clone())).is_empty());
        assert_eq!(botanist.on_event(&event(scan.clone())).len(), 1);
        assert_eq!(botanist.sampler().unwrap().first_distance, Some(0.0));
        botanist.on_status(&json!({"Latitude":0.0,"Longitude":0.02,"PlanetRadius":1_000_000.0}));
        assert!(botanist.sampler().unwrap().ready());
        scan["ScanType"] = json!("Sample");
        botanist.on_event(&event(scan));
        assert!(!botanist.sampler().unwrap().ready());
        botanist.on_status(&json!({"Latitude":0.0,"Longitude":0.04,"PlanetRadius":1_000_000.0}));
        assert!(botanist.sampler().unwrap().ready());
        botanist.on_event(&event(json!({"event":"FSDJump","StarSystem":"Next"})));
        assert!(botanist.sampler().is_none());
    }

    #[test]
    fn batch_updates_rows_without_sampler_notifications() {
        let mut botanist = Botanist::new(default_settings());
        botanist.set_batch(true);
        assert!(botanist
            .on_event(&event(
                json!({"event":"ScanOrganic","Species_Localised":"Unknown","ScanType":"Analyse","Body":2})
            ))
            .is_empty());
        assert!(botanist.rows().is_empty());
        botanist.set_batch(false);
        assert!(!botanist.rows()[0].analysed);
        assert!(botanist.sampler().is_none());
    }

    #[test]
    fn haversine_handles_dateline_poles_and_antipodes() {
        let radius = 1_000_000.0;
        assert!(botanist_distance((0.0, 179.9), (0.0, -179.9), radius) < 3500.0);
        // Preserve the original approximation and its repeated first-latitude
        // cosine; the independent source oracle checks the full rounded text.
        assert!((botanist_distance((0.0, 0.0), (45.0, -45.0), radius) - 1_143_718.0).abs() < 1.0);
        assert_eq!(botanist_distance((15.0, 30.0), (15.0, 30.0), radius), 0.0);
    }

    #[test]
    fn absent_coordinates_stay_unknown() {
        let mut botanist = Botanist::new(default_settings());
        botanist.on_event(&event(
            json!({"event":"ScanOrganic","Species_Localised":"Unknown","ScanType":"Log"}),
        ));
        botanist.on_event(&event(
            json!({"event":"ScanOrganic","Species_Localised":"Unknown","ScanType":"Log"}),
        ));
        assert_eq!(botanist.sampler().unwrap().first_distance, None);
        assert!(!botanist.sampler().unwrap().ready());
    }

    #[test]
    fn normalizes_frontier_genus_typo_and_preserves_unknown_names() {
        assert_eq!(
            english_genus("$Codex_Ent_Clepeus_Genus_Name;;", "Localized"),
            "Clypeus"
        );
        assert_eq!(
            english_genus("future-genus", "Future genus"),
            "Future genus"
        );
        assert_eq!(colony_distance("Future genus"), 100);
    }

    #[test]
    fn overlay_settings_do_not_disable_sample_tracking() {
        let mut botanist = Botanist::new(json!({"OverlayEnabled":false,"OverlayIsSticky":false}));
        let scan =
            json!({"event":"ScanOrganic","Species_Localised":"Unknown","ScanType":"Log","Body":2});
        assert!(botanist.on_event(&event(scan.clone())).is_empty());
        assert_eq!(botanist.rows().len(), 1);
        assert!(botanist.sampler().is_none());
        botanist.set_settings(json!({"OverlayEnabled":true,"OverlayIsSticky":false}));
        botanist.on_event(&event(scan));
        assert!(!botanist.sampler().unwrap().sticky);
        botanist.set_settings(json!({"OverlayEnabled":false}));
        assert!(botanist.sampler().is_some());
    }
}
