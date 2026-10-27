//! Hecate/Spansh support scans follow Handle*SystemResponse and LoadData:
//! source data may enrich the current system, never impersonate gameplay.
use super::*;
impl BioInsights {
    /// Preserve the sender supplied by the native plugin bus. Source Ready
    /// records peer identity/version; it does not open UI or emit extra work.
    pub fn on_peer_message(
        &mut self,
        sender: &str,
        command: &str,
        payload: &Value,
    ) -> Vec<Finding> {
        if command == "Ready" && !sender.is_empty() {
            if sender == "Hecate" && self.browser_route.first().is_some_and(|p| p == "hecatenag") {
                self.browser_route = vec!["main".into()];
                *self.browser_cache.get_mut().expect("browser cache") = None;
            }
            self.seen_plugins
                .entry(sender.to_owned())
                .or_insert_with(|| {
                    payload
                        .get("Version")
                        .or_else(|| payload.get("version"))
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_owned()
                });
            return Vec::new();
        }
        if command == "LegacyPluginMessage" {
            let message = &payload["message"];
            let legacy_command = message
                .get("Command")
                .and_then(Value::as_str)
                .or_else(|| message.get(0).and_then(Value::as_str))
                .unwrap_or("UNDEFINED");
            let version = self
                .seen_plugins
                .get(sender)
                .map(String::as_str)
                .unwrap_or("");
            if self.diagnostics.len() == 256 {
                self.diagnostics.remove(0);
            }
            self.diagnostics.push(json!({"Level":"Info","Operation":"LegacyPluginMessage","Message":format!("{sender} v{version} called legacy HandlePluginMessage [CMD: {legacy_command}]")}));
            return Vec::new();
        }
        if command == "SetServerAddress" && sender != "Hecate" {
            return Vec::new();
        }
        self.on_message(command, payload)
    }
    pub(super) fn register_overlay_colours(&mut self) {
        if self.setting("UseCustomNotificationColours") {
            self.message("StellarOverlay","RegisterPluginCustomColours",json!({"TitleColor":self.settings["TitleColorHex"],"TextColor":self.settings["ColorHex"],"BackgroundColor":self.settings["BackgroundColorHex"],"BorderColor":self.settings["BorderColorHex"],"GetDefaultColors":true}));
        } else {
            self.message("StellarOverlay", "UnregisterPluginCustomColours", json!({}));
        }
    }
    pub(super) fn matches_fid(&self, value: &Value) -> bool {
        let source = value
            .as_str()
            .map(str::to_owned)
            .or_else(|| value.as_u64().map(|n| n.to_string()));
        source.is_some_and(|fid| {
            !self.commander.is_empty()
                && fid.trim_start_matches('F') == self.commander.trim_start_matches('F')
        })
    }
    pub(super) fn request_unsold_total(&mut self) {
        if self.batch {
            return;
        }
        if !self.commander.is_empty() {
            self.message(
                "Hecate",
                "BioInsightsGetTotal",
                json!({"FID":self.commander}),
            );
        }
    }
    pub(super) fn request_system_knowledge(&mut self, timestamp: &str) {
        if self.current_system == 0 || timestamp.is_empty() {
            return;
        }
        let payload = json!({"ID64":self.current_system,"DateTime":timestamp});
        if self.setting("HecateUseDB") {
            self.message("Hecate", "LoadSystem", payload.clone());
        }
        if self.setting("HecateUseSpansh") {
            self.message("Hecate", "SpanshSystem", payload);
        }
    }
    pub(super) fn load_system_knowledge(&mut self, command: &str, payload: &Value) {
        if payload["ID64"].as_u64() != Some(self.current_system) {
            return;
        }
        let Some(events) = payload["JournalEvents"].as_array() else {
            return;
        };
        let fids = payload["FIDs"].as_array();
        if command == "LoadSystemResponse" && fids.is_none() {
            return;
        }
        let old_batch = self.batch;
        let old_allowed = self.notifications_allowed;
        let timestamp = self.timestamp.clone();
        let alert = self.alert_body;
        self.batch = true;
        self.notifications_allowed = false;
        for (index, item) in events.iter().enumerate() {
            // Native Hecate replies with journal objects; accept the original
            // framework's JournalEventArgs envelope when importing a fixture.
            let mut event = item
                .get("journalEvent")
                .or_else(|| item.get("JournalEvent"))
                .unwrap_or(item)
                .clone();
            let kind = text(&event, "event").to_owned();
            if !matches!(
                kind.as_str(),
                "Scan"
                    | "ScanBaryCentre"
                    | "FSSBodySignals"
                    | "SAASignalsFound"
                    | "SAAScanComplete"
            ) {
                continue;
            }
            if event["SystemAddress"]
                .as_u64()
                .is_some_and(|id| id != self.current_system)
            {
                continue;
            }
            event["SystemAddress"] = json!(self.current_system);
            // A Spansh map is public knowledge, not this commander's completed
            // DSS. Hecate marks that require a commander use its parallel FIDs.
            let current = fids
                .filter(|ids| ids.len() == events.len())
                .and_then(|ids| ids.get(index))
                .is_some_and(|fid| self.matches_fid(fid));
            if kind == "SAAScanComplete" && !current {
                continue;
            }
            if kind == "Scan" {
                if let Some(existing) = event["BodyID"]
                    .as_u64()
                    .and_then(|body| self.systems.get(&self.current_system)?.bodies.get(&body))
                {
                    let prior = text(&existing.scan, "_KnowledgeSource");
                    if existing.scan["PlanetClass"].is_string()
                        && (prior.is_empty()
                            || prior == "Hecate" && command == "SpanshSystemResponse")
                    {
                        continue;
                    }
                }
                event["_KnowledgeSource"] = json!(if command == "LoadSystemResponse" {
                    "Hecate"
                } else {
                    "Spansh"
                });
            }
            if let Ok(parsed) = JournalEvent::parse(&event.to_string()) {
                let _ = self.on_event(&parsed);
            }
        }
        self.batch = old_batch;
        self.notifications_allowed = old_allowed;
        self.timestamp = timestamp;
        self.alert_body = alert;
        self.sync_overlay();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(v: Value) -> JournalEvent {
        JournalEvent::parse(&v.to_string()).unwrap()
    }
    #[test]
    fn ready_sender_and_custom_colour_protocol_preserve_source_contract() {
        let mut bio = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        bio.on_peer_message("StellarOverlay", "Ready", &json!({"Version":"1.0.5"}));
        bio.on_peer_message("StellarOverlay", "Ready", &json!({"Version":"changed"}));
        assert_eq!(bio.tools_summary()["ReadyPeers"]["StellarOverlay"], "1.0.5");
        bio.on_peer_message(
            "Other",
            "SetServerAddress",
            &json!({"ServerAddress":"127.0.0.1","ServerPort":58080}),
        );
        assert!(bio.web_server.is_empty());
        bio.on_peer_message(
            "Hecate",
            "SetServerAddress",
            &json!({"ServerAddress":"127.0.0.1","ServerPort":58080}),
        );
        assert_eq!(bio.web_server, "127.0.0.1:58080");
        let defaults = bio.overlay_defaults.clone();
        bio.on_peer_message(
            "StellarOverlay",
            "RegisterPluginCustomColoursResponse",
            &json!({"TitleColor":"FFFFFFFF"}),
        );
        assert_eq!(bio.overlay_defaults, defaults);
        let reply = json!({"TitleColor":"FF112233","TextColor":"FF445566","BackgroundColor":"80112233","BorderColor":"FF778899"});
        bio.on_peer_message(
            "StellarOverlay",
            "RegisterPluginCustomColoursResponse",
            &reply,
        );
        assert_eq!(bio.overlay_defaults["ColorHex"], "FF445566");
        let mut settings = bio.settings.clone();
        settings["UseCustomNotificationColours"] = json!(true);
        bio.set_settings(settings).unwrap();
        let messages = bio.take_messages();
        let register = messages
            .iter()
            .find(|m| m["command"] == "RegisterPluginCustomColours")
            .unwrap();
        assert_eq!(register["payload"]["GetDefaultColors"], true);
        assert_eq!(register["payload"]["TextColor"], bio.settings["ColorHex"]);
        let mut settings = bio.settings.clone();
        settings["UseCustomNotificationColours"] = json!(false);
        bio.set_settings(settings).unwrap();
        assert!(bio
            .take_messages()
            .iter()
            .any(|m| m["command"] == "UnregisterPluginCustomColours"));
    }
    #[test]
    fn controls_request_source_protocol_and_support_scan_cannot_take_over_commander_or_samples() {
        let mut bio = BioInsights::new(
            json!({"GalacticCodexRemote":false,"HecateUseDB":true,"HecateUseSpansh":false}),
        )
        .unwrap();
        bio.on_event(&event(
            json!({"event":"LoadGame","FID":"F123","Commander":"Source"}),
        ));
        bio.on_event(&event(json!({"timestamp":"2026-10-25T00:00:00Z","event":"Location","SystemAddress":42,"StarSystem":"Test"})));
        let messages = bio.take_messages();
        assert!(messages.iter().any(|m| m["command"] == "LoadSystem"
            && m["payload"]["ID64"] == 42
            && m["payload"]["DateTime"] == "2026-10-25T00:00:00Z"));
        assert!(!messages.iter().any(|m| m["command"] == "SpanshSystem"));
        let state = bio.state();
        let payload = json!({"ID64":42,"JournalEvents":[{"event":"Scan","BodyID":3,"BodyName":"Test 3","PlanetClass":"Rocky body","Radius":1000000},{"event":"FSSBodySignals","BodyID":3,"Signals":[{"Type":"$SAA_SignalType_Biological;","Count":2}]},{"event":"LoadGame","FID":"F999"},{"event":"ScanOrganic","BodyID":3,"Species":"$Codex_Ent_Stratum_07_Name;","ScanType":"Analyse"},{"event":"SAAScanComplete","BodyID":3}],"FIDs":[456,456,456,456,456]});
        bio.on_message("LoadSystemResponse", &payload);
        assert_eq!(bio.state(), state);
        assert_eq!(bio.systems[&42].bodies[&3].bio_count, Some(2));
        assert!(!bio.systems[&42].bodies[&3].mapped);
        assert!(bio.systems[&42].bodies[&3].observed.is_empty());
        assert!(bio
            .take_messages()
            .iter()
            .all(|m| m["target"] == "Evaluator"));
        bio.on_event(&event(json!({"event":"Scan","SystemAddress":42,"BodyID":3,"BodyName":"Latest","PlanetClass":"Icy body"})));
        bio.on_message("SpanshSystemResponse",&json!({"ID64":42,"JournalEvents":[{"event":"Scan","BodyID":3,"PlanetClass":"Rocky body"}]}));
        assert_eq!(bio.systems[&42].bodies[&3].scan["PlanetClass"], "Icy body");
        bio.on_message("LoadSystemResponse",&json!({"ID64":43,"JournalEvents":[{"event":"Scan","BodyID":5,"PlanetClass":"Rocky body"}],"FIDs":[123]}));
        assert!(!bio.systems[&42].bodies.contains_key(&5));
    }
    #[test]
    fn database_total_is_authoritative_and_commander_scoped() {
        let mut bio = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        bio.on_event(&event(json!({"event":"LoadGame","FID":"F123"})));
        bio.on_message(
            "BioInsightsGetTotalResponse",
            &json!({"FID":456,"Total":999}),
        );
        assert_eq!(bio.overlay()["UnsoldTotal"], 0);
        bio.on_message(
            "BioInsightsGetTotalResponse",
            &json!({"FID":123,"Total":19010800}),
        );
        assert_eq!(bio.overlay()["UnsoldTotal"], 19010800);
        bio.on_message(
            "BioInsightsGetDataResponse",
            &json!({"Data":[[1,"date","body",1,1000,true],[2,"date","body",2,500,false]]}),
        );
        assert_eq!(bio.overlay()["UnsoldTotal"], 5500);
        bio.on_event(&event(json!({"event":"LoadGame","FID":"F456"})));
        assert_eq!(bio.overlay()["UnsoldTotal"], 0);
    }
    #[test]
    fn history_cap_uses_visit_order_and_zero_means_unlimited() {
        let mut bio =
            BioInsights::new(json!({"GalacticCodexRemote":false,"MaximumSystemsReadAll":2}))
                .unwrap();
        for id in [999, 3, 888] {
            bio.on_event(&event(json!({"event":"Location","SystemAddress":id})));
        }
        assert!(!bio.systems.contains_key(&999));
        assert!(bio.systems.contains_key(&3));
        assert!(bio.systems.contains_key(&888));
        let mut settings = bio.settings.clone();
        settings["MaximumSystemsReadAll"] = json!(0);
        bio.set_settings(settings).unwrap();
        for id in 1..=10 {
            bio.on_event(&event(json!({"event":"Location","SystemAddress":id})));
        }
        assert_eq!(bio.systems.len(), 11);
    }
}
