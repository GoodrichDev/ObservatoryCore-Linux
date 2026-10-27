//! The optional Hecate HTTP/WebSocket interface uses the same native state as
//! the desktop worker. It does not start a server or access external services.
use super::*;

impl BioInsights {
    pub(super) fn web_request(&mut self, command: &str, payload: &Value) {
        let websocket = command == "WebSocketRequest";
        let uri = text(payload, "URI");
        let parts = route_parts(uri);
        let page = parts.first().map(String::as_str).unwrap_or("");
        if page == "unsold" && !self.commander.is_empty() && payload["_VaultReady"] != true {
            // The Hecate bus is asynchronous: reply only after the vault query
            // returns, so the first page includes freshly analysed biology.
            self.pending_web.push((command.to_owned(), payload.clone()));
            self.message(
                "Hecate",
                "BioInsightsGetData",
                json!({"FID":self.commander}),
            );
            return;
        }
        let mut status = 200u16;
        let mut content_type = "text/html; charset=utf-8";
        let mut location = None;
        let content = match page {
            "" => {
                status = 301;
                location = Some("/bioinsights/main");
                "Moved".to_owned()
            }
            "hecatenag" => self.source_hecate_nag(),
            "gethecate" => {
                self.message(
                    "Core",
                    "InstallNativePlugin",
                    json!({"Plugin":"Hecate","Dependency":true}),
                );
                self.source_hecate_nag()
            }
            "skiphecate" => {
                self.hecate_nag_skipped = true;
                self.web_table()
            }
            "main" => self.web_page("BioInsights", &self.web_table()),
            "unsold" => self.web_page("Unsold biological data", &self.web_unsold()),
            "toggleunsold" if parts.len() == 3 => {
                if let Ok(id) = parts[1].parse::<u64>() {
                    self.message(
                        "Hecate",
                        "BioInsightsSetBonus",
                        json!({"ID":id,"Value":parts[2]=="1"}),
                    );
                    if !self.commander.is_empty() {
                        self.message(
                            "Hecate",
                            "BioInsightsGetData",
                            json!({"FID":self.commander}),
                        );
                    }
                    for row in &mut self.vault_unsold {
                        if row.get(0).and_then(Value::as_u64) == Some(id) {
                            row[5] = json!(parts[2] == "1");
                        }
                    }
                }
                self.web_page("Unsold biological data", &self.web_unsold())
            }
            "ccmark" if parts.len() == 4 => {
                if let Ok(id) = parts[2].parse::<u64>() {
                    if let Some(species) = species_by_number(id) {
                        let colour = parts[3]
                            .parse::<usize>()
                            .ok()
                            .and_then(|c| rules::COLOURS.get(c))
                            .copied()
                            .unwrap_or("");
                        self.on_message(
                            "MarkVariant",
                            &json!({"SpeciesID":species,"Variant":colour}),
                        );
                    }
                }
                self.web_page("Codex Companion", &self.source_codex(&parts))
            }
            "ccshow" | "codexcompanion" => {
                self.web_page("Codex Companion", &self.source_codex(&parts))
            }
            "settings" => {
                content_type = "application/json";
                serde_json::to_string_pretty(&self.settings).unwrap_or_default()
            }
            "about" => self.source_about(),
            "static" if parts.get(1).is_some_and(|name| name == "style.css") => {
                content_type = "text/css";
                self.original_stylesheet()
            }
            "api" if parts.get(1).is_some_and(|p| p == "rows") => {
                content_type = "application/json";
                json!(self.rows()).to_string()
            }
            "api" if parts.get(1).is_some_and(|p| p == "overlay") => {
                content_type = "application/json";
                self.overlay().to_string()
            }
            "alert" => {
                if let Some(id) = parts.get(1).and_then(|id| id.parse::<u8>().ok()) {
                    self.dismiss_ui_alert(id);
                }
                self.web_table()
            }
            "zoom" => self.web_page("BioInsights", &self.web_table()),
            _ => {
                status = 404;
                content_type = "text/plain";
                "not found".to_owned()
            }
        };
        let content = if content_type.starts_with("text/html") && status == 200 {
            self.source_alerts() + &content
        } else {
            content
        };
        let content = if !websocket && content_type.starts_with("text/html") && status == 200 {
            self.source_document(&content, false)
        } else {
            content
        };
        if websocket {
            let event = if page == "alert" {
                json!({"event":"hide","id":format!("alert_{}",parts.get(1).map(String::as_str).unwrap_or("0"))})
            } else if matches!(page, "toggleunsold" | "ccmark") {
                json!({"event":"reload"})
            } else {
                json!({"event":"content","id":"maincontent","content":content})
            };
            self.message("Hecate","WebSocketResponse",json!({"ClientID":payload.get("ClientID").cloned().unwrap_or(Value::Null),"Json":event.to_string()}));
        } else {
            let mut response =
                json!({"StatusCode":status,"ContentType":content_type,"Content":content});
            if let Some(location) = location {
                response["Location"] = json!(location);
            }
            self.message("Hecate","WebResponse",json!({"RequestID":payload.get("RequestID").cloned().unwrap_or(Value::Null),"Response":response}));
        }
    }
    fn web_page(&self, _title: &str, content: &str) -> String {
        content.to_owned()
    }
    fn web_table(&self) -> String {
        self.source_main()
    }
    fn web_unsold(&self) -> String {
        self.source_unsold()
    }
    fn original_stylesheet(&self) -> String {
        self.source_stylesheet()
    }
}

pub(super) fn species_by_number(id: u64) -> Option<&'static str> {
    rules::codex().genus.values().find_map(|g| {
        g.species
            .keys()
            .find(|s| species_number(s) == id)
            .map(String::as_str)
    })
}
pub(super) fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn unescape(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Ok(a), Ok(b)) = (
                char::from(bytes[i + 1]).to_digit(16).ok_or(()),
                char::from(bytes[i + 2]).to_digit(16).ok_or(()),
            ) {
                decoded.push((a * 16 + b) as u8);
                i += 3;
                continue;
            }
        }
        decoded.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

pub(super) fn route_parts(uri: &str) -> Vec<String> {
    let path = uri
        .split_once("://")
        .map(|(scheme, rest)| {
            if scheme == "observatory" {
                rest
            } else {
                rest.find('/').map(|i| &rest[i..]).unwrap_or("/")
            }
        })
        .unwrap_or(uri);
    let mut parts: Vec<String> = path
        .split('?')
        .next()
        .unwrap_or("")
        .split('/')
        .filter(|s| !s.is_empty())
        .map(unescape)
        .collect();
    if parts
        .first()
        .is_some_and(|p| p.eq_ignore_ascii_case("bioinsights"))
    {
        parts.remove(0);
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hecate_http_and_websocket_requests_preserve_correlation() {
        let mut worker = BioInsights::new(json!({})).unwrap();
        worker.on_message("WebRequest",&json!({"URI":"http://localhost:58080/bioinsights/main","RequestID":"request-1","Response":{}}));
        let messages = worker.take_messages();
        let response = &messages[0];
        assert_eq!(response["command"], "WebResponse");
        assert_eq!(response["payload"]["RequestID"], "request-1");
        assert_eq!(response["payload"]["Response"]["StatusCode"], 200);
        assert!(response["payload"]["Response"]["Content"]
            .as_str()
            .unwrap()
            .contains("Radiant Star"));
        worker.on_message(
            "WebSocketRequest",
            &json!({"URI":"/bioinsights/main","ClientID":"client-1"}),
        );
        let response = worker.take_messages().pop().unwrap();
        assert_eq!(response["command"], "WebSocketResponse");
        assert_eq!(response["payload"]["ClientID"], "client-1");
        worker.on_message(
            "WebRequest",
            &json!({"URI":"/bioinsights/static/../../secret","RequestID":"request-2"}),
        );
        assert_eq!(
            worker.take_messages()[0]["payload"]["Response"]["StatusCode"],
            404
        );
    }
    #[test]
    fn first_unsold_response_waits_for_fresh_vault_rows() {
        let mut worker = BioInsights::new(json!({})).unwrap();
        worker.commander = "F123".to_owned();
        worker.on_message(
            "WebRequest",
            &json!({"URI":"/bioinsights/unsold","RequestID":"fresh-unsold"}),
        );
        let messages = worker.take_messages();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["command"], "BioInsightsGetData");
        worker.on_message("BioInsightsGetDataResponse",&json!({"Processed":true,"Data":[[1,"2026-10-26T00:00:00Z","Freshly analysed body",73,19010800,true]]}));
        let response = worker.take_messages().pop().unwrap();
        assert_eq!(response["payload"]["RequestID"], "fresh-unsold");
        assert!(response["payload"]["Response"]["Content"]
            .as_str()
            .unwrap()
            .contains("Freshly analysed body"));
    }
}
