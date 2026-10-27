//! Journal radar transitions from the installed processJournalEvent dispatcher.
use super::*;
impl BioInsights {
    pub(super) fn radar_event(&mut self, event: &JournalEvent, prior_system: u64) {
        let vehicle = |kind: u64, current: bool| {
            if current {
                json!({"ContextID":kind,"UseCurrentPosition":true})
            } else {
                // Framework Touchdown coordinates are nonnullable doubles;
                // absent JSON fields deserialize to zero in the original.
                json!({"ContextID":kind,"X":event.data["Latitude"].as_f64().unwrap_or(0.0),"Y":event.data["Longitude"].as_f64().unwrap_or(0.0)})
            }
        };
        match event.event.as_str() {
            "Location"
                if prior_system != self.current_system
                    && !event.boolean("Docked")
                    && (event.boolean("InSRV") || event.boolean("OnFoot")) =>
            {
                if let Some(body) = self.body_id(event, self.current_system) {
                    self.radar_planet(self.current_system, body);
                }
            }
            "SupercruiseExit" if event.str("BodyType") == "Planet" => {
                if let Some(body) = self.body_id(event, self.current_system) {
                    self.radar_planet(self.current_system, body);
                }
            }
            "Touchdown" => {
                if let Some(body) = self.body_id(event, self.current_system) {
                    self.radar_planet(self.current_system, body);
                }
                self.message("StellarOverlay", "RadarAddVehicle", vehicle(0, false));
            }
            "Liftoff" => self.message(
                "StellarOverlay",
                "RadarRemoveVehicle",
                json!({"ContextID":0}),
            ),
            // Context zero is the source ship position when launching an SRV.
            "LaunchSRV" => self.message("StellarOverlay", "RadarAddVehicle", vehicle(0, true)),
            "Disembark"
                if event.boolean("OnPlanet")
                    && !event.boolean("OnStation")
                    && event.boolean("SRV") =>
            {
                self.message("StellarOverlay", "RadarAddVehicle", vehicle(1, true))
            }
            "Embark"
                if event.boolean("OnPlanet")
                    && !event.boolean("OnStation")
                    && event.boolean("SRV") =>
            {
                self.message(
                    "StellarOverlay",
                    "RadarRemoveVehicle",
                    json!({"ContextID":1}),
                )
            }
            "DockingRequested" | "Docked" => {
                self.message("StellarOverlay", "RadarForceHidden", json!({"Value":true}))
            }
            "Died" | "Shutdown" => self.message("StellarOverlay", "RadarHide", json!({})),
            "ReceiveText"
                if !self.commander_name.is_empty()
                    && event
                        .str("Message_Localised")
                        .starts_with(&format!("CMDR {} ", self.commander_name.to_uppercase()))
                    && event
                        .str("Message")
                        .ends_with("$HumanoidEmote_point_Action;;") =>
            {
                self.message("StellarOverlay", "RadarAddWaypoint", json!({}))
            }
            _ => {}
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn event(v: Value) -> JournalEvent {
        JournalEvent::parse(&v.to_string()).unwrap()
    }
    #[test]
    fn radar_dispatch_matches_original_journal_types_and_all_location_flags() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../../../fixtures/bioinsights-journal-oracle.json"
        ))
        .unwrap();
        let selected = [
            "Location",
            "FSDJump",
            "CarrierJump",
            "StartJump",
            "ApproachBody",
            "SupercruiseExit",
            "SupercruiseEntry",
            "Touchdown",
            "Liftoff",
            "LaunchSRV",
            "DockSRV",
            "Disembark",
            "Embark",
            "ReceiveText",
            "Died",
            "DockingRequested",
            "Docked",
            "Shutdown",
        ];
        for case in oracle
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| selected.contains(&text(c, "Event")))
        {
            let kind = text(case, "Event");
            let f = case["Flags"].as_u64().unwrap();
            let mut bio = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
            bio.commander = "F123".into();
            bio.commander_name = "Source".into();
            bio.current_system = if f & 64 != 0 { 0 } else { 42 };
            bio.current_body = Some(1);
            let state = BodyState {
                scan: json!({"BodyID":1,"BodyName":"Source 1","PlanetClass":"Rocky body","Landable":true,"Radius":1_000_000}),
                bio_count: Some(1),
                ..Default::default()
            };
            bio.systems.insert(
                42,
                SystemState {
                    bodies: BTreeMap::from([(1, state)]),
                    ..Default::default()
                },
            );
            bio.on_event(&event(json!({"event":kind,"SystemAddress":42,"BodyID":1,"BodyType":"Planet","OnPlanet":f&1!=0,"OnStation":f&2!=0,"SRV":f&4!=0,"OnFoot":f&8!=0,"InSRV":f&16!=0,"Docked":f&32!=0,"Latitude":1,"Longitude":2,"Message_Localised":"CMDR SOURCE points","Message":"test$HumanoidEmote_point_Action;;"})));
            let actual: Vec<String> = bio
                .take_messages()
                .iter()
                .filter_map(|m| {
                    let command = text(m, "command");
                    let p = &m["payload"];
                    match command {
                        "RadarSetPlanet" => Some("RadarShow:42:1".into()),
                        "RadarSetColonyRanges" => None,
                        "RadarAddVehicle" => Some(format!(
                            "RadarAddVehicle:{}:{}:{}",
                            p["ContextID"],
                            if p["UseCurrentPosition"] == true {
                                String::new()
                            } else {
                                p["X"].as_f64().unwrap().to_string()
                            },
                            if p["UseCurrentPosition"] == true {
                                String::new()
                            } else {
                                p["Y"].as_f64().unwrap().to_string()
                            }
                        )),
                        "RadarRemoveVehicle" => {
                            Some(format!("RadarRemoveVehicle:{}", p["ContextID"]))
                        }
                        "RadarForceHidden" => Some("RadarForceHidden:True".into()),
                        c if c.starts_with("Radar") => Some(c.into()),
                        _ => None,
                    }
                })
                .collect();
            let expected: Vec<String> = case["Actions"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Value::as_str)
                .filter(|v| v.starts_with("Radar"))
                .map(str::to_owned)
                .collect();
            assert_eq!(actual, expected, "{kind} flags {f}");
        }
    }
    #[test]
    fn radar_preference_gates_controls_but_hide_remains_available() {
        let mut bio =
            BioInsights::new(json!({"GalacticCodexRemote":false,"StellarOverlayUseRadar":false}))
                .unwrap();
        for kind in ["LaunchSRV", "Liftoff", "Docked", "Shutdown"] {
            bio.on_event(&event(json!({"event":kind})));
        }
        let messages = bio.take_messages();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["command"], "RadarHide");
    }
    #[test]
    fn touchdown_coordinates_use_framework_nonnullable_double_defaults() {
        let mut bio = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        bio.on_event(&event(json!({"event":"Touchdown"})));
        let message = bio
            .take_messages()
            .into_iter()
            .find(|m| m["command"] == "RadarAddVehicle")
            .unwrap();
        assert_eq!(message["payload"]["X"], 0.0);
        assert_eq!(message["payload"]["Y"], 0.0);
    }
}
