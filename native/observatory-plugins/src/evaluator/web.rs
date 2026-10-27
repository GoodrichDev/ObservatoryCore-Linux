//! Hecate's optional HTTP/WebSocket interface shares the native worker state.
use super::*;

impl Evaluator {
    pub(super) fn web_request(&mut self, command: &str, payload: &Value) {
        let uri = payload["URI"].as_str().unwrap_or("");
        if self.server_address.is_empty() {
            if let Some((_, rest)) = uri.split_once("://") {
                let authority = rest.split('/').next().unwrap_or("");
                self.server_address = authority.to_owned();
            }
        }

        let path = uri
            .split_once("://")
            .map_or(uri, |(_, rest)| rest.find('/').map_or("/", |i| &rest[i..]));
        let mut parts: Vec<_> = path
            .split('?')
            .next()
            .unwrap_or("")
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        if parts
            .first()
            .is_some_and(|s| s.eq_ignore_ascii_case("evaluator"))
        {
            parts.remove(0);
        }
        let page = parts.first().copied().unwrap_or("");
        if page == "unsold" && payload["_VaultReady"] != true {
            self.pending_web.push((command.into(), payload.clone()));
            self.queue(
                "Hecate",
                "EvaluatorGetData",
                json!({"FID":self.fid_number()}),
            );
            return;
        }
        let mut status = 200;
        let mut content_type = "text/html; charset=utf-8";
        let mut location = None;
        let content = match page {
            "" => {
                status = 301;
                location = Some("/evaluator/main");
                "Moved".into()
            }
            "alert" => {
                if let Some(id) = parts.get(1).and_then(|id| id.parse::<u8>().ok()) {
                    self.dismiss_ui_alert(id);
                }
                self.web_main_html()
            }
            "main" => self.web_page(&self.web_table()),
            "unsold" => self.web_page(&self.web_unsold()),
            "filter" => {
                let filter = match parts.get(1).copied().unwrap_or("0") {
                    "1" => "MappingOnly",
                    "2" => "All",
                    _ => "Default",
                };
                self.on_message("SetFilter", &json!({"Filter":filter}));
                self.web_page(&self.web_table())
            }
            "toggle" if parts.len() == 4 => {
                if let (Ok(id), Ok(body)) = (parts[1].parse::<u64>(), parts[2].parse::<u64>()) {
                    self.on_message(
                        "SetWorthMapping",
                        &json!({"ID64":id,"BodyID":body,"IsWorthMapping":parts[3]!="0"}),
                    );
                }
                self.web_page(&self.web_table())
            }
            "zoom" => self.web_page(&self.web_table()),
            "about" => self.source_about(),
            "settings" => {
                content_type = "application/json";
                self.settings.to_string()
            }
            "static" if parts.get(1) == Some(&"style.css") => {
                content_type = "text/css";
                self.stylesheet()
            }
            "static" if parts.get(1) == Some(&"app.js") => {
                content_type = "text/javascript";
                include_str!("app.js")
                    .replace("$$USE_WS$$", "1")
                    .replace("$$WS_SERVER$$", &self.server_address)
                    .replace("$$PLUGIN$$", "evaluator")
            }
            "api" if parts.get(1) == Some(&"rows") => {
                content_type = "application/json";
                json!(self.rows()).to_string()
            }
            "api" if parts.get(1) == Some(&"summary") => {
                content_type = "application/json";
                self.summary().to_string()
            }
            _ => {
                status = 404;
                content_type = "text/plain";
                "not found".into()
            }
        };
        let content = if command != "WebSocketRequest"
            && status == 200
            && content_type.starts_with("text/html")
        {
            self.source_document(&content, false)
        } else {
            content
        };
        if command == "WebSocketRequest" {
            let data = if page == "alert" {
                json!({"event":"hide","id":format!("alert_{}",parts.get(1).unwrap_or(&"0"))})
            } else if matches!(page, "toggle" | "filter") {
                json!({"event":"reload"})
            } else {
                json!({"event":"content","id":"maincontent","content":content})
            };
            self.queue("Hecate","WebSocketResponse",json!({"ClientID":payload.get("ClientID").cloned().unwrap_or(Value::Null),"Json":data.to_string()}));
        } else {
            let mut response =
                json!({"StatusCode":status,"ContentType":content_type,"Content":content});
            if let Some(location) = location {
                response["Location"] = json!(location);
            }
            self.queue("Hecate","WebResponse",json!({"RequestID":payload.get("RequestID").cloned().unwrap_or(Value::Null),"Response":response}));
        }
    }
    fn stylesheet(&self) -> String {
        self.source_stylesheet()
    }
    fn web_page(&self, content: &str) -> String {
        content.to_owned()
    }
    fn web_table(&self) -> String {
        self.web_main_html()
    }
    fn web_unsold(&self) -> String {
        self.source_unsold()
    }
}
pub(super) fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn web_routes_match_native_filters_and_keep_request_correlation() {
        let mut worker = Evaluator::new(json!({}));
        worker.on_message(
            "WebRequest",
            &json!({"URI":"http://localhost:58080/evaluator/main","RequestID":"a"}),
        );
        let messages = worker.take_messages();
        assert_eq!(messages[0]["command"], "WebResponse");
        assert_eq!(messages[0]["payload"]["RequestID"], "a");
        assert!(messages[0]["payload"]["Response"]["Content"]
            .as_str()
            .unwrap()
            .contains("Visit / Map"));
        worker.on_message(
            "WebSocketRequest",
            &json!({"URI":"/evaluator/filter/2","ClientID":"c"}),
        );
        assert_eq!(worker.summary()["Filter"], "All");
        assert_eq!(worker.take_messages()[0]["payload"]["ClientID"], "c");
        worker.on_message(
            "WebRequest",
            &json!({"URI":"/evaluator/static/../../file","RequestID":"b"}),
        );
        assert_eq!(
            worker.take_messages()[0]["payload"]["Response"]["StatusCode"],
            404
        );
    }
    #[test]
    fn first_unsold_web_response_awaits_vault_query() {
        let mut worker = Evaluator::new(json!({}));
        worker.on_message(
            "WebRequest",
            &json!({"URI":"/evaluator/unsold","RequestID":"fresh"}),
        );
        assert_eq!(worker.take_messages()[0]["command"], "EvaluatorGetData");
        worker.on_message(
            "EvaluatorGetDataResponse",
            &json!({"Data":[["Fresh system","2026-10-26T00:00:00Z",12345]]}),
        );
        let response = worker.take_messages().pop().unwrap();
        assert_eq!(response["payload"]["RequestID"], "fresh");
        assert!(response["payload"]["Response"]["Content"]
            .as_str()
            .unwrap()
            .contains("Fresh system"));
    }
}
