//! Installed NotificationOptions / SendScanNotification / DistanceNotifier routing.
//! Rendering bits match Observatory.Framework.NotificationRendering (1|2|4).
use super::*;

pub(super) const CATEGORIES: [&str; 11] = [
    "PersonalCodex",
    "Marked",
    "HighValue",
    "Distance",
    "OnScanDetails",
    "NewGalactic",
    "Samples",
    "AtmosphericBody",
    "AtmosphericBodyWithBios",
    "NonAtmosphericBodyWithBios",
    "RadiantStarUpdates",
];

pub(super) fn defaults() -> Value {
    Value::Object(
        CATEGORIES
            .into_iter()
            .map(|name| {
                (
                    name.to_owned(),
                    json!({"Popup":true,"Voice":true,"Plugin":false}),
                )
            })
            .collect(),
    )
}

impl BioInsights {
    pub(super) fn meets_value_threshold(&self, key: &str, value: u64) -> bool {
        let default = if key == "MinBodyValueMillions" {
            20.0
        } else {
            12.0
        };
        let threshold = self.settings[key].as_f64().unwrap_or(default);
        threshold > 0.0 && value as f64 >= threshold * 1_000_000.0
    }
    fn notification_body_name(&self, system: u64, scan: &Value) -> String {
        body_name(
            text(scan, "BodyName"),
            self.systems
                .get(&system)
                .map(|s| s.name.as_str())
                .unwrap_or(""),
        )
    }
    fn rendering(&self, category: &str) -> u8 {
        let options = &self.settings["Notifications"][category];
        u8::from(options["Popup"].as_bool().unwrap_or(true))
            | (u8::from(options["Voice"].as_bool().unwrap_or(true)) << 1)
            | (u8::from(options["Plugin"].as_bool().unwrap_or(false)) << 2)
    }
    fn emit_notification(
        &mut self,
        system: u64,
        body: u64,
        title: String,
        detail: String,
        categories: &[&str],
    ) -> Option<Finding> {
        let rendering = categories
            .iter()
            .fold(0, |mask, category| mask | self.rendering(category));
        if self.batch || !self.notifications_allowed || rendering == 0 {
            return None;
        }
        let finding = self.finding(system, body, title, detail);
        self.notification_renderings
            .push((finding.clone(), rendering));
        Some(finding)
    }
    /// A finding's channel selection travels with it until the native host
    /// persists/displays it; it is never inferred from the finding's prose.
    pub fn take_notification_rendering(&mut self, finding: &Finding) -> Option<u8> {
        self.notification_renderings
            .iter()
            .position(|(value, _)| value == finding)
            .map(|index| self.notification_renderings.remove(index).1)
    }
    pub(super) fn prediction_notifications(&mut self, system: u64, body: u64) -> Vec<Finding> {
        if self.batch || !self.notifications_allowed {
            return Vec::new();
        }
        let Some(state) = self.systems.get(&system).and_then(|s| s.bodies.get(&body)) else {
            return Vec::new();
        };
        if state.scan["Landable"].as_bool() != Some(true) {
            return Vec::new();
        }
        let scan = state.scan.clone();
        let bios = state.bio_count.unwrap_or(0);
        let predictions = self.predictions(system, body);
        let summary = self.summary(system, body);
        let high_body = self.meets_value_threshold(
            "MinBodyValueMillions",
            summary["BaseMinimumValue"].as_u64().unwrap_or(0),
        );
        let high_species = predictions
            .iter()
            .any(|p| self.meets_value_threshold("MinIndividualBiologicalValueMillions", p.value));
        let marked = predictions.iter().any(|p| {
            self.is_marked(&p.species, "")
                || p.variants
                    .iter()
                    .any(|(variant, _, _, _)| self.is_marked(&p.species, variant))
        });
        let personal = self.has_personal_codex_prediction(system, &predictions);
        let (regional, never) = self.galactic_flags(system, &predictions);
        let mut clauses = Vec::new();
        let mut categories = Vec::new();
        for (category, active, clause) in [
            (
                "PersonalCodex",
                personal,
                "may host a new personal codex entry",
            ),
            ("Marked", marked, "may host a marked biological"),
            ("HighValue", high_body, "has high biological value"),
            (
                "HighValue",
                !high_body && high_species,
                "may host a high value biological",
            ),
            (
                "NewGalactic",
                regional || never,
                "may contain a new galactic codex entry",
            ),
        ] {
            if active && self.rendering(category) != 0 {
                categories.push(category);
                clauses.push(clause);
            }
        }
        let mut findings = Vec::new();
        let key = (system, body, "body-interest".to_owned());
        if !categories.is_empty() && !self.announced.contains(&key) {
            let detail = join_clauses(&clauses);
            if let Some(finding) = self.emit_notification(
                system,
                body,
                self.notification_body_name(system, &scan),
                detail,
                &categories,
            ) {
                self.announced.insert(key);
                findings.push(finding);
            }
        }
        let atmospheric = number(&scan, "SurfacePressure") > 0.0;
        let category = match (atmospheric, bios > 0) {
            (true, true) => Some("AtmosphericBodyWithBios"),
            (true, false) => Some("AtmosphericBody"),
            (false, true) => Some("NonAtmosphericBodyWithBios"),
            _ => None,
        };
        if let Some(category) = category {
            let key = (system, body, category.to_owned());
            if !self.announced.contains(&key) {
                let detail = if atmospheric {
                    let atmosphere = atmospheric_name(text(&scan, "AtmosphereType"));
                    format!(
                        "{} atmospheric landable{}",
                        atmosphere,
                        if bios > 0 { " with life" } else { "" }
                    )
                    .trim()
                    .to_owned()
                } else {
                    "Non-atmospheric landable with life".to_owned()
                };
                let detail = if state_unknown_autoscan(&scan, bios) {
                    format!("Autoscanned {detail}")
                } else {
                    detail
                };
                if let Some(finding) = self.emit_notification(
                    system,
                    body,
                    self.notification_body_name(system, &scan),
                    detail,
                    &[category],
                ) {
                    self.announced.insert(key);
                    findings.push(finding);
                }
            }
        }
        findings
    }
    pub(super) fn confirmed_species_notification(
        &mut self,
        system: u64,
        body: u64,
        species: &str,
    ) -> Option<Finding> {
        let (genus, species) = rules::species_key(species)?;
        let key = (system, body, format!("confirmed:{species}"));
        if self.announced.contains(&key) {
            return None;
        }
        let data = &rules::codex().genus[genus];
        let base = data.species[species].base_value;
        let value = format!(
            "{:.2}",
            ((base as f64 / 1_000_000.0) * 100.0).round_ties_even() / 100.0
        )
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned();
        let finding = self.emit_notification(
            system,
            body,
            format!("{} confirmed present", companion::full_name(genus, species)),
            format!(
                "Base Value {value}M, Minimum sample distance {}m",
                data.minimum_sample_distance
            ),
            &["OnScanDetails"],
        )?;
        self.announced.insert(key);
        Some(finding)
    }
    pub(super) fn sample_notification(
        &mut self,
        system: u64,
        body: u64,
        kind: &str,
        species: &str,
    ) -> Option<Finding> {
        let (genus, species) = rules::species_key(species)?;
        let distance = rules::codex().genus[genus].minimum_sample_distance;
        let (title, detail) = match kind {
            "Log" => (
                "First partial sample collected",
                format!("Travel {distance}m to collect next sample"),
            ),
            "Sample" => (
                "Second partial sample collected",
                format!("Travel {distance}m to collect last sample"),
            ),
            "Analyse" => (
                "Sample collection complete",
                format!(
                    "Three partial samples of {} collected",
                    companion::full_name(genus, species)
                ),
            ),
            _ => return None,
        };
        self.emit_notification(system, body, title.to_owned(), detail, &["Samples"])
    }
    pub(super) fn distance_notification(&mut self) -> Option<Finding> {
        if self.distance_notified || self.batch {
            return None;
        }
        if self.active_sample.is_none()
            && self.distance_cache["SystemID64"].as_u64() == Some(self.current_system)
        {
            let distance = self.distance_cache["MinimumDistance"].as_u64()?;
            let radius = self.distance_cache["RadiusKM"].as_f64()?;
            let samples = self.distance_cache["ScanPosition"].as_array()?;
            let position = self.position?;
            if distance == 0 || radius <= 0.0 || !(1..=2).contains(&samples.len()) {
                return None;
            }
            for sample in samples {
                let point = [sample["Latitude"].as_f64()?, sample["Longitude"].as_f64()?];
                if great_circle(position, point, radius) < distance as f64 {
                    return None;
                }
            }
            self.distance_notified = true;
            return self.emit_notification(
                self.current_system,
                self.current_body.unwrap_or(0),
                "Minimum Distance Reached".to_owned(),
                format!("You've travelled over {distance} meters from previous sample"),
                &["Distance"],
            );
        }
        let (system, body, species) = self.active_sample.as_ref()?;
        let (system, body) = (*system, *body);
        if self.current_system != system || self.current_body != Some(body) || self.radius <= 0.0 {
            return None;
        }
        let position = self.position?;
        let (genus, _) = rules::species_key(species)?;
        let distance = rules::codex().genus[genus].minimum_sample_distance;
        let samples = &self
            .systems
            .get(&system)?
            .bodies
            .get(&body)?
            .observed
            .get(species)?
            .samples;
        if samples.is_empty()
            || samples
                .iter()
                .any(|sample| great_circle(position, *sample, self.radius) < distance as f64)
        {
            return None;
        }
        self.distance_notified = true;
        self.emit_notification(
            system,
            body,
            "Minimum Distance Reached".to_owned(),
            format!("You've travelled over {distance} meters from previous sample"),
            &["Distance"],
        )
    }
    pub(super) fn radiant_stars(&self, system: u64) -> BTreeMap<u64, String> {
        if self.rendering("RadiantStarUpdates") == 0 {
            return BTreeMap::new();
        }
        let Some(sys) = self.systems.get(&system) else {
            return BTreeMap::new();
        };
        let bodies = sys.rule_bodies();
        let spheres = sys.position.map(rules::spheres_at).unwrap_or_default();
        sys.bodies
            .iter()
            .filter(|(_, b)| b.scan.get("PlanetClass").is_some())
            .map(|(id, b)| {
                let context = rules::Context {
                    scan: &b.scan,
                    bodies: &bodies,
                    regions: sys.position.map(rules::regions_at).unwrap_or(0),
                    spheres: &spheres,
                    complete: sys.complete,
                };
                (
                    *id,
                    context
                        .radiant()
                        .map(|s| text(s, "StarType").to_owned())
                        .unwrap_or_else(|| "Unknown".to_owned()),
                )
            })
            .collect()
    }
    pub(super) fn radiant_notifications(
        &mut self,
        system: u64,
        previous: BTreeMap<u64, String>,
    ) -> Vec<Finding> {
        if self.batch || previous.is_empty() {
            return Vec::new();
        }
        let current = self.radiant_stars(system);
        let changes: Vec<_> = previous
            .into_iter()
            .filter_map(|(body, old)| {
                let new = current.get(&body)?;
                (old != *new).then(|| (body, old, new.clone()))
            })
            .collect();
        if changes.len() == 1 {
            let (body, old, new) = &changes[0];
            let title = self
                .systems
                .get(&system)
                .and_then(|s| s.bodies.get(body))
                .map(|b| self.notification_body_name(system, &b.scan))
                .unwrap_or_default();
            self.emit_notification(
                system,
                *body,
                title,
                format!("Radiant Star updated from {old} to {new}"),
                &["RadiantStarUpdates"],
            )
            .into_iter()
            .collect()
        } else if !changes.is_empty() {
            let names = changes
                .iter()
                .filter_map(|(body, _, _)| {
                    self.systems
                        .get(&system)?
                        .bodies
                        .get(body)
                        .map(|b| self.notification_body_name(system, &b.scan))
                })
                .collect::<Vec<_>>()
                .join(", ");
            self.emit_notification(
                system,
                changes[0].0,
                "Radiant Star updated for multiple bodies".to_owned(),
                names,
                &["RadiantStarUpdates"],
            )
            .into_iter()
            .collect()
        } else {
            Vec::new()
        }
    }
}
fn body_name(body: &str, system: &str) -> String {
    let body = body.trim();
    let system = system.trim();
    if !system.is_empty() && body.eq_ignore_ascii_case(system) {
        return "Main Star".to_owned();
    }
    if system.is_empty()
        || !body
            .to_ascii_lowercase()
            .starts_with(&system.to_ascii_lowercase())
    {
        return body.to_owned();
    }
    let mut label = "Body";
    let mut parts = Vec::new();
    for word in body[system.len()..].split_whitespace() {
        if word == "Belt" || word == "Ring" {
            label = word;
        } else {
            parts.push(word);
        }
    }
    format!("{label} {}", parts.join(" ")).trim().to_owned()
}
fn atmospheric_name(name: &str) -> String {
    let mut value = String::new();
    for character in name.chars() {
        if character.is_ascii_uppercase() && !value.is_empty() {
            value.push(' ');
        }
        value.push(character);
    }
    value
}
fn state_unknown_autoscan(scan: &Value, bios: u64) -> bool {
    bios == 0 && text(scan, "ScanType") == "AutoScan"
}
fn join_clauses(clauses: &[&str]) -> String {
    match clauses.split_last() {
        Some((last, [])) => (*last).to_owned(),
        Some((last, rest)) => format!("{} and {}", rest.join(", "), last),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn notification_body_titles_match_original_pure_method_captures() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../../../fixtures/bioinsights-bodyname-oracle.json"
        ))
        .unwrap();
        for case in oracle["cases"].as_array().unwrap() {
            assert_eq!(
                body_name(
                    case["body"].as_str().unwrap(),
                    case["system"].as_str().unwrap()
                ),
                case["result"].as_str().unwrap()
            );
        }
    }
    #[test]
    fn all_source_categories_route_independent_channel_masks() {
        for category in CATEGORIES {
            for mask in 0..=7 {
                let mut worker = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
                worker.settings["Notifications"][category] =
                    json!({"Popup":mask&1!=0,"Voice":mask&2!=0,"Plugin":mask&4!=0});
                let result =
                    worker.emit_notification(1, 2, "Test".into(), "Detail".into(), &[category]);
                assert_eq!(result.is_some(), mask != 0, "{category}: {mask}");
                if let Some(finding) = result {
                    assert_eq!(worker.take_notification_rendering(&finding), Some(mask));
                }
            }
        }
    }
    fn event(data: Value) -> JournalEvent {
        JournalEvent::parse(&data.to_string()).unwrap()
    }
    #[test]
    fn source_samples_confirmation_and_distance_rearm_for_each_partial() {
        let mut worker = BioInsights::new(json!({"GalacticCodexRemote":false,"Notifications":{
            "Samples":{"Popup":false,"Voice":false,"Plugin":true},
            "OnScanDetails":{"Popup":false,"Voice":true,"Plugin":false},
            "Distance":{"Popup":true,"Voice":false,"Plugin":false}
        }}))
        .unwrap();
        worker.on_event(&event(
            json!({"event":"Location","SystemAddress":1,"StarSystem":"Test"}),
        ));
        worker.on_event(&event(json!({"event":"Scan","SystemAddress":1,"BodyID":1,"BodyName":"Test 1","PlanetClass":"Rocky body","Landable":true,"Radius":1000000})));
        worker.on_status(
            &json!({"BodyName":"Test 1","PlanetRadius":1000000,"Latitude":0.0,"Longitude":0.0}),
        );
        let sample = |kind| {
            event(
                json!({"event":"ScanOrganic","SystemAddress":1,"BodyID":1,"ScanType":kind,"Species":"$Codex_Ent_Bacterial_06_Name;"}),
            )
        };
        let first = worker.on_event(&sample("Log"));
        assert_eq!(
            first.iter().map(|f| f.title.as_str()).collect::<Vec<_>>(),
            [
                "Bacterium Alcyoneum confirmed present",
                "First partial sample collected"
            ]
        );
        assert_eq!(worker.take_notification_rendering(&first[0]), Some(2));
        assert_eq!(worker.take_notification_rendering(&first[1]), Some(4));
        assert_eq!(first[1].detail, "Travel 500m to collect next sample");
        assert!(worker
            .on_status(&json!({"BodyName":"Test 1","Latitude":0.01,"Longitude":0.0}))
            .is_empty());
        let reached =
            worker.on_status(&json!({"BodyName":"Test 1","Latitude":0.03,"Longitude":0.0}));
        assert_eq!(reached[0].title, "Minimum Distance Reached");
        assert_eq!(
            reached[0].detail,
            "You've travelled over 500 meters from previous sample"
        );
        assert_eq!(worker.take_notification_rendering(&reached[0]), Some(1));
        assert!(worker
            .on_status(&json!({"BodyName":"Test 1","Latitude":0.04,"Longitude":0.0}))
            .is_empty());
        let second = worker.on_event(&sample("Sample"));
        assert_eq!(second.len(), 1); // confirmation is once per body/species
        assert_eq!(second[0].title, "Second partial sample collected");
        assert!(worker
            .on_status(&json!({"BodyName":"Test 1","Latitude":0.06,"Longitude":0.0}))
            .is_empty());
        assert_eq!(
            worker.on_status(&json!({"BodyName":"Test 1","Latitude":0.08,"Longitude":0.0}))[0]
                .title,
            "Minimum Distance Reached"
        );
        assert_eq!(
            worker.on_event(&sample("Analyse"))[0].title,
            "Sample collection complete"
        );
    }
    #[test]
    fn virtual_support_scans_update_data_without_notifications() {
        let mut worker = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        worker.on_event(&event(
            json!({"event":"Location","SystemAddress":1,"StarSystem":"Test"}),
        ));
        let scan = json!({"event":"Scan","SystemAddress":1,"BodyID":1,"BodyName":"Test 1","PlanetClass":"Rocky body","Landable":true,"SurfacePressure":1000,"AtmosphereType":"CarbonDioxide","BioSignals":1});
        assert!(worker.on_message("VirtualScan", &scan).is_empty());
        assert!(worker.notification_renderings.is_empty());
        assert_eq!(worker.main_view()["Groups"][0]["BodyName"], "Test 1");
        // A later real journal event remains eligible for source notifications.
        assert!(!worker.on_event(&event(scan)).is_empty());
    }
    #[test]
    fn zero_thresholds_disable_evaluator_overlay_and_main_high_value_flags() {
        let mut worker=BioInsights::new(json!({"GalacticCodexRemote":false,"MinBodyValueMillions":0,"MinIndividualBiologicalValueMillions":0,"PersonalCodexPossibleAny":false,"PersonalCodexPossibleCurrent":false})).unwrap();
        worker.on_event(&event(
            json!({"event":"Location","StarSystem":"Test","SystemAddress":42,"StarPos":[0,0,0]}),
        ));
        worker.on_event(&event(json!({"event":"Scan","SystemAddress":42,"BodyID":0,"BodyName":"Test A","StarType":"F","SurfaceTemperature":6500,"Radius":695700000,"DistanceFromArrivalLS":0})));
        worker.on_event(&event(json!({"event":"Scan","SystemAddress":42,"BodyID":1,"BodyName":"Test A 1","Landable":true,"PlanetClass":"High metal content body","Atmosphere":"thin carbon dioxide atmosphere","AtmosphereType":"CarbonDioxide","AtmosphereComposition":[{"Name":"CarbonDioxide","Percent":100}],"SurfaceTemperature":180,"SurfaceGravity":1,"SurfacePressure":100,"MassEM":0.1,"Radius":1000000,"SemiMajorAxis":149597870700_u64,"DistanceFromArrivalLS":499,"Parents":[{"Star":0}],"Volcanism":""})));
        worker.on_event(&event(json!({"event":"FSSBodySignals","SystemAddress":42,"BodyID":1,"Signals":[{"Type":"$SAA_SignalType_Biological;","Count":2}]})));
        let view = worker.main_view();
        assert_eq!(view["Groups"][0]["HighValue"], false);
        let rows = view["Groups"][0]["Rows"].as_array().unwrap();
        assert!(!rows.is_empty());
        assert!(rows.iter().all(|row| row["HighValue"] == false));
        assert!(worker
            .take_messages()
            .iter()
            .any(|message| message["command"] == "SetBioInsightsFlags"
                && message["payload"]["IsHighValue"] == false));
        worker.on_status(&json!({"GuiFocus":9}));
        assert!(worker
            .take_messages()
            .iter()
            .all(|message| message["command"] != "ShowBitmapNotification"));
        worker.settings["MinBodyValueMillions"] = json!(0.001);
        worker.settings["MinIndividualBiologicalValueMillions"] = json!(0.001);
        worker.publish_body(42, 1);
        assert!(worker
            .take_messages()
            .iter()
            .any(|message| message["command"] == "SetBioInsightsFlags"
                && message["payload"]["IsHighValue"] == true));
        assert_eq!(worker.main_view()["Groups"][0]["HighValue"], true);
        worker.on_status(&json!({"GuiFocus":9}));
        assert!(worker
            .take_messages()
            .iter()
            .any(|message| message["command"] == "ShowBitmapNotification"));
    }
    #[test]
    fn overlay_palette_applies_custom_colours_only_in_source_theme_mode() {
        let mut worker=BioInsights::new(json!({"GalacticCodexRemote":false,"Theme":"Original","ThemeSettings":{"ThemeIndex":1,"ThemeMode":0,"Colors":{"DefaultText":{"R":1,"G":2,"B":3}}}})).unwrap();
        let mut payload = json!({});
        worker.notification_style(&mut payload, false);
        assert_eq!(payload["ColorHex"], "FFFF6F00"); // selected source orange, cached custom ignored
        worker.settings["ThemeSettings"]["ThemeMode"] = json!(1);
        worker.notification_style(&mut payload, false);
        assert_eq!(payload["ColorHex"], "FF010203");
        worker.settings["ThemeSettings"]["ThemeMode"] = json!(2);
        worker.notification_style(&mut payload, false);
        assert_eq!(payload["ColorHex"], "FFFF6F00");
        worker.settings["UseCustomNotificationColours"] = json!(true);
        worker.settings["ColorHex"] = json!("FFAABBCC");
        worker.notification_style(&mut payload, false);
        assert_eq!(payload["ColorHex"], "FFAABBCC");
    }
    #[test]
    fn zero_value_thresholds_disable_high_value_and_atmosphere_uses_source_words() {
        let mut worker=BioInsights::new(json!({"GalacticCodexRemote":false,"MinBodyValueMillions":0,"MinIndividualBiologicalValueMillions":0})).unwrap();
        worker.on_event(&event(
            json!({"event":"Location","SystemAddress":1,"StarSystem":"Test"}),
        ));
        let findings=worker.on_event(&event(json!({"event":"Scan","SystemAddress":1,"BodyID":1,"BodyName":"Test 1","PlanetClass":"Rocky body","Landable":true,"SurfacePressure":1000,"AtmosphereType":"CarbonDioxide"})));
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].detail, "Carbon Dioxide atmospheric landable");
        assert_eq!(worker.take_notification_rendering(&findings[0]), Some(3));
    }
    #[test]
    fn categories_or_channels_and_replay_stays_quiet() {
        let mut worker = BioInsights::new(json!({"GalacticCodexRemote":false,"Notifications":{
            "HighValue":{"Popup":true,"Voice":false,"Plugin":false},
            "Marked":{"Popup":false,"Voice":false,"Plugin":true}}}))
        .unwrap();
        let finding = worker
            .emit_notification(
                1,
                2,
                "Body".into(),
                "Detail".into(),
                &["HighValue", "Marked"],
            )
            .unwrap();
        assert_eq!(worker.take_notification_rendering(&finding), Some(5));
        worker.set_batch(true);
        assert!(worker
            .sample_notification(1, 2, "Log", "Codex_Ent_Bacterial_06_Name")
            .is_none());
        assert!(worker.notification_renderings.is_empty());
    }
}
