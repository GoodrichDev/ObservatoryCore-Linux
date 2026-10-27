//! BaseUIHandler UIAlert enum, original alerts.txt and dismiss behavior.
use super::*;
const TYPES: &[(&str, &str, &str, bool)] = &[
    (
        "UpdateAvailable",
        "alert-info",
        "An update is available",
        true,
    ),
    (
        "UpdateDownloaded",
        "alert-info",
        "An update has been downloaded - restart Observatory to install",
        true,
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
        "ReadAllRequired",
        "alert-danger",
        "<strong>Read All required</strong> - BioInsights needs to rebuild it's database",
        true,
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
        "BioInsights plugin installed",
        false,
    ),
];
impl BioInsights {
    pub(super) fn set_ui_alert(&mut self, payload: &Value) {
        let Some(index) = TYPES.iter().position(|row| row.0 == text(payload, "Type")) else {
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
                self.ui_alerts.remove(&5);
                self.set_ui_alert(&json!({"Type":"ReadAllCompleted"}));
            }
            "Cancelled" => self.set_ui_alert(&json!({"Type":"ReadAllCancelled"})),
            "Required" => self.set_ui_alert(&json!({"Type":"ReadAllRequired"})),
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
        self.ui_alerts.iter().map(|(id,description)|format!("<div id=\"alert_{id}\"><div class=\"alert {}\"><a class=\"alertbtn\" href=\"http://localhost/bioinsights/alert/{id}\">&times;</a> {description}</div><br /></div>",TYPES[*id as usize-1].1)).collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_read_all_banners_and_dismiss_are_retained_across_page_navigation() {
        let mut bio = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        bio.on_message("SourceReadAllState", &json!({"State":"Required"}));
        assert!(bio
            .browser_html()
            .contains("<strong>Read All required</strong>"));
        bio.on_message("SourceReadAllState", &json!({"State":"Started"}));
        bio.on_message("SourceReadAllState", &json!({"State":"Cancelled"}));
        assert_eq!(bio.ui_alerts().as_array().unwrap().len(), 2);
        bio.on_message("SourceReadAllState", &json!({"State":"Started"}));
        bio.on_message("SourceReadAllState", &json!({"State":"Completed"}));
        assert_eq!(bio.ui_alerts()[0]["ID"], 3);
        bio.on_message(
            "BrowserNavigate",
            &json!({"Url":"observatory://bioinsights/about"}),
        );
        bio.on_message(
            "BrowserNavigate",
            &json!({"Url":"observatory://bioinsights/alert/3"}),
        );
        assert_eq!(bio.browser_route(), "about");
        assert!(!bio.browser_html().contains("alert_3"));
    }
}
