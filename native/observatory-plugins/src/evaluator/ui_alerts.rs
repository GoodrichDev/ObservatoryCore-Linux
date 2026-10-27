//! BaseUIHandler UIAlert enum, original alerts.txt and dismiss behavior.
use super::*;
const TYPES: &[(&str, &str, &str, bool)] = &[
    (
        "UpdateAvailable",
        "alert-info",
        "An update is available",
        false,
    ),
    (
        "UpdateDownloaded",
        "alert-info",
        "An update has been downloaded - restart Observatory to install",
        false,
    ),
    (
        "ReadAllCompleted",
        "alert-success",
        "Read All completed",
        false,
    ),
    (
        "ReadAllCancelled",
        "alert-danger",
        "Read All cancelled",
        false,
    ),
    (
        "UpdateInstalled",
        "alert-success",
        "Update installed",
        false,
    ),
    (
        "PluginInstalled",
        "alert-success",
        "Evaluator plugin installed",
        false,
    ),
];
impl Evaluator {
    pub(super) fn set_ui_alert(&mut self, payload: &Value) {
        let Some(index) = TYPES
            .iter()
            .position(|row| row.0 == payload["Type"].as_str().unwrap_or(""))
        else {
            return;
        };
        let id = index as u8 + 1;
        if payload["Active"] == false {
            self.ui_alerts.remove(&id);
        } else {
            self.ui_alerts.insert(
                id,
                payload
                    .get("Description")
                    .and_then(Value::as_str)
                    .unwrap_or(TYPES[index].2)
                    .to_owned(),
            );
        }
    }
    pub(super) fn read_all_state(&mut self, state: &str) {
        match state {
            "Started" => {
                self.ui_alerts.remove(&3);
                self.ui_alerts.remove(&4);
            }
            "Completed" => {
                self.set_ui_alert(&json!({"Type":"ReadAllCompleted"}));
            }
            "Cancelled" => self.set_ui_alert(&json!({"Type":"ReadAllCancelled"})),
            _ => {}
        }
    }
    pub(super) fn expire_ui_alerts(&mut self) {
        self.ui_alerts.retain(|id, _| TYPES[*id as usize - 1].3);
    }
    pub(super) fn dismiss_ui_alert(&mut self, id: u8) {
        self.ui_alerts.remove(&id);
    }
    pub(super) fn ui_alerts(&self) -> Value {
        json!(self.ui_alerts.iter().map(|(id,description)|{
            let source=TYPES[*id as usize-1];
            json!({"ID":id,"Type":source.0,"Class":source.1,"Description":description,"Persistent":source.3})
        }).collect::<Vec<_>>())
    }
    pub(super) fn source_alerts(&self) -> String {
        self.ui_alerts.iter().map(|(id,description)|format!("<div id=\"alert_{id}\"><div class=\"alert {}\"><a class=\"alertbtn\" href=\"/evaluator/alert/{id}\">&times;</a> {description}</div><br /></div>",TYPES[*id as usize-1].1)).collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn event(value: Value) -> JournalEvent {
        JournalEvent::parse(&value.to_string()).unwrap()
    }
    #[test]
    fn installed_evaluator_alert_ids_templates_and_expiry_differ_from_bio() {
        let source = include_str!("../../../fixtures/evaluator-alerts.txt");
        for (line, row) in source.lines().zip(TYPES.iter()) {
            let fields: Vec<_> = line.split('|').collect();
            assert_eq!(fields[0], row.0);
            assert_eq!(fields[1], row.1);
            assert_eq!(fields[3], row.2);
            assert_eq!(fields[2] == "true", row.3);
        }
        let mut worker = Evaluator::new(json!({}));
        for (index, (kind, class, description, persistent)) in TYPES.iter().enumerate() {
            assert!(!persistent);
            worker.on_message("SourceUIAlert", &json!({"Type":kind,"Active":true}));
            assert!(worker.web_main_html().contains(&format!(
                "<div id=\"alert_{}\"><div class=\"alert {class}\">",
                index + 1
            )));
            assert!(worker.web_main_html().contains(description));
        }
        assert_eq!(worker.summary()["UIAlerts"].as_array().unwrap().len(), 6);
        worker.on_message("SourceReadAllState", &json!({"State":"Started"}));
        assert_eq!(worker.ui_alerts.len(), 4);
        worker.on_message("SourceReadAllState", &json!({"State":"Completed"}));
        assert!(worker.ui_alerts.contains_key(&3));
        worker.on_message("DismissUIAlert", &json!({"ID":3}));
        assert!(!worker.ui_alerts.contains_key(&3));
        worker.on_message("SourceReadAllState", &json!({"State":"Required"}));
        assert_eq!(worker.ui_alerts.len(), 4);
        worker.on_event(&event(
            json!({"event":"Location","SystemAddress":42,"StarSystem":"First"}),
        ));
        assert!(worker.ui_alerts.is_empty());
        worker.on_message("SourceReadAllState", &json!({"State":"Cancelled"}));
        assert!(worker.ui_alerts.contains_key(&4));
        worker.on_event(&event(
            json!({"event":"Location","SystemAddress":42,"StarSystem":"First"}),
        ));
        assert!(worker.ui_alerts.contains_key(&4));
        worker.on_event(&event(
            json!({"event":"FSDJump","SystemAddress":42,"StarSystem":"First"}),
        ));
        assert!(worker.ui_alerts.is_empty());
    }
    #[test]
    fn web_dismiss_hides_only_matching_alert_and_preserves_request_client() {
        let mut worker = Evaluator::new(json!({}));
        worker.on_message("SourceUIAlert", &json!({"Type":"UpdateInstalled"}));
        worker.on_message(
            "WebSocketRequest",
            &json!({"URI":"/evaluator/alert/5","ClientID":"client"}),
        );
        assert!(worker.ui_alerts.is_empty());
        let messages = worker.take_messages();
        let reply = messages.last().unwrap();
        assert_eq!(reply["payload"]["ClientID"], "client");
        let data: Value = serde_json::from_str(reply["payload"]["Json"].as_str().unwrap()).unwrap();
        assert_eq!(data, json!({"event":"hide","id":"alert_5"}));
    }
}
