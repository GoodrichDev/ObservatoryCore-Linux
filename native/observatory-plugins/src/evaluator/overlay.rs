use super::*;

impl Evaluator {
    pub(super) fn request_overlay(&mut self, only_update: bool) {
        if only_update && !self.persistent_open {
            return;
        }
        self.persistent_open = true;
        self.update_overlay();
    }
    pub(super) fn close_overlay(&mut self) {
        self.persistent_open = false;
        self.update_overlay();
    }
    pub(super) fn update_overlay(&mut self) {
        let options = &self.settings["Notifications"];
        let wanted = self.persistent_open;
        if !wanted {
            if !self.overlay_payload.is_null() {
                self.queue(
                    "Core",
                    "Notification",
                    json!({"id":"native.evaluator.remaining","action":"cancel","plugin":"Evaluator"}),
                );
                self.overlay_payload = Value::Null;
            }
            return;
        }
        let names = self
            .systems
            .get(&self.key())
            .map(|sys| {
                sys.route
                    .iter()
                    .filter_map(|id| sys.bodies.get(id))
                    .map(|b| {
                        b.name
                            .strip_prefix(&sys.name)
                            .unwrap_or(&b.name)
                            .trim()
                            .to_owned()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let detail = if names.is_empty() {
            " - none - ".to_owned()
        } else {
            let mut text = String::from("⋙ ");
            for (index, name) in names.iter().enumerate() {
                if index > 0 {
                    text.push_str(", ");
                    if index % 10 == 0 {
                        text.push_str("\n     ");
                    }
                }
                text.push_str(name);
            }
            text
        };
        let payload = json!({"id":"native.evaluator.remaining","plugin":"Evaluator","rendering":1,"title":"Bodies still to map","detail":detail,
            "timeout":0,"x":options["PersistentPositionX"].as_f64().unwrap_or(80.0),
            "y":options["PersistentPositionY"].as_f64().unwrap_or(25.0),"proportional":true});
        if self.overlay_payload != payload {
            let action = if self.overlay_payload.is_null() {
                "show"
            } else {
                "update"
            };
            self.overlay_payload = payload.clone();
            let mut request = payload;
            request["action"] = json!(action);
            self.queue("Core", "Notification", request);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persistent_dss_route_uses_source_position_and_hides_on_focus_change() {
        let mut worker = Evaluator::new(json!({"MinimumBodyValue":0}));
        for value in [
            json!({"event":"Location","StarSystem":"A","SystemAddress":42}),
            json!({"event":"Scan","BodyName":"A 1","BodyID":1,"PlanetClass":"Earthlike body","MassEM":1}),
        ] {
            worker.on_event(&JournalEvent::parse(&value.to_string()).unwrap());
        }
        worker.take_messages();
        worker.on_status(&json!({"GuiFocus":10}));
        let messages = worker.take_messages();
        let p = &messages.last().unwrap()["payload"];
        assert_eq!(p["timeout"], 0);
        assert_eq!(p["x"], 80.0);
        assert_eq!(p["y"], 25.0);
        assert_eq!(p["title"], "Bodies still to map");
        assert_eq!(p["rendering"], 1);
        assert_eq!(messages.last().unwrap()["target"], "Core");
        worker.on_status(&json!({"GuiFocus":0}));
        assert_eq!(
            worker.take_messages().last().unwrap()["payload"]["action"],
            "cancel"
        );
        // ReadAll itself does not cancel an already issued source notification.
        worker.on_status(&json!({"GuiFocus":10}));
        worker.take_messages();
        worker.set_batch(true);
        assert!(worker.take_messages().is_empty());
    }
}
