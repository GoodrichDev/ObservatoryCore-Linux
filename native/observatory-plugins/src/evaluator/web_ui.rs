//! Exact source-generated Evaluator HTML row layout and embedded symbol theme.
use super::*;
use base64::Engine as _;
use std::sync::OnceLock;
fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
fn escape(s: &str) -> String {
    super::web::escape(s)
}
fn credits(n: i64) -> String {
    let source = n.to_string();
    let mut result = String::new();
    for (i, c) in source.chars().enumerate() {
        if i > 0 && (source.len() - i).is_multiple_of(3) && c != '-' && !result.ends_with('-') {
            result.push(' ');
        }
        result.push(c);
    }
    result
}
fn theme(settings: &Value) -> &'static Value {
    static ORIGINAL: OnceLock<Value> = OnceLock::new();
    static ORANGE: OnceLock<Value> = OnceLock::new();
    if settings["ThemeSettings"]["ThemeIndex"]
        .as_u64()
        .map(|i| i == 0)
        .unwrap_or(settings["Theme"] == "Original")
    {
        ORIGINAL.get_or_init(|| serde_json::from_str(include_str!("theme.json")).unwrap())
    } else {
        ORANGE.get_or_init(|| serde_json::from_str(include_str!("theme-orange.json")).unwrap())
    }
}

impl Evaluator {
    /// Native display reference built from the original embedded document shell.
    pub fn browser_html(&self, page: &str) -> String {
        let content = match page {
            "about" => self.source_about(),
            "unsold" => self.source_unsold(),
            _ => self.web_main_html(),
        };
        self.source_document(&content, true)
    }
    pub(super) fn source_document(&self, content: &str, embedded: bool) -> String {
        let stylesheet = self.source_stylesheet();
        let mut buttons = String::from("<div class=\"buttonheader\">");
        for (id, title) in [
            ("main", "Main Window"),
            ("unsold", "Show Current Data"),
            ("about", "About"),
            ("popout", "Popout Window"),
            ("zoom", "Zoom"),
            ("settings", "Settings"),
        ] {
            let hidden = if !embedded && matches!(id, "popout" | "zoom" | "settings") {
                " style=\"display:none\""
            } else {
                ""
            };
            buttons.push_str(&format!("<a href=\"/evaluator/{id}\"><button id=\"{id}\" title=\"{title}\"{hidden}></button></a>"));
        }
        buttons.push_str("</div>");
        let web_server = if self.server_address.is_empty() {
            String::new()
        } else if self.server_port.is_empty() {
            self.server_address.clone()
        } else {
            format!("{}:{}", self.server_address, self.server_port)
        };
        let script = if embedded {
            String::new()
        } else {
            format!(
                "<script>{}</script>",
                include_str!("app.js")
                    .replace("$$USE_WS$$", if web_server.is_empty() { "0" } else { "1" })
                    .replace("$$WS_SERVER$$", &web_server)
                    .replace("$$PLUGIN$$", "evaluator")
            )
        };
        format!("<!doctype html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Evaluator</title><style>{stylesheet}</style></head><body style=\"zoom:{}%\">{buttons}<div id=\"webapp\"></div><div id=\"disconnected\"></div><div class=\"content\" id=\"maincontent\">{content}</div>{script}</body></html>",self.settings["ZoomLevel"].as_u64().unwrap_or(100))
    }
    pub(super) fn source_stylesheet(&self) -> String {
        let theme = theme(&self.settings);
        let mut style = format!(
            "{}\n{}",
            include_str!("style.css"),
            include_str!("buttons.css")
        );
        for (key, value) in theme["Colors"].as_object().unwrap() {
            let value = self.settings["ThemeSettings"]["Colors"]
                .get(key)
                .filter(|_| self.settings["ThemeSettings"]["ThemeMode"] == 1)
                .unwrap_or(value);
            style = style.replace(
                &format!("$${}$$", key.to_uppercase()),
                &format!(
                    "{:02X}{:02X}{:02X}",
                    value["R"].as_u64().unwrap_or(0),
                    value["G"].as_u64().unwrap_or(0),
                    value["B"].as_u64().unwrap_or(0)
                ),
            );
        }
        for key in ["RowHeight", "RowSpacing"] {
            style = style.replace(
                &format!("$${}$$", key.to_uppercase()),
                &theme["Settings"][key].as_u64().unwrap_or(0).to_string(),
            );
        }
        let button_size = match text(&self.settings, "ButtonSize").trim() {
            "Smaller" => 16,
            "Small" => 24,
            "Large" => 48,
            "Extra Large" => 64,
            _ => 32,
        };
        style = style
            .replace("$$BUTTONSIZE$$", &button_size.to_string())
            .replace("$$BUTTONVISIBLE$$", "1");
        for (key, symbol) in theme["Symbols"].as_object().unwrap() {
            if symbol["IsLabel"] == true {
                let rendered = self.symbol(key, Some(""));
                if let Some((_, rest)) = rendered.split_once("style=\"") {
                    if let Some((rules, _)) = rest.split_once('"') {
                        style.push_str(&format!("\n.{key}{{{rules}}}"));
                    }
                }
                if text(symbol, "ContentType") == "Text" {
                    style.push_str(&format!(
                        "\n.{key}:empty::before{{content:{};}}",
                        serde_json::to_string(text(symbol, "Content")).unwrap()
                    ));
                }
            }
        }
        style
    }
    fn symbol(&self, key: &str, override_text: Option<&str>) -> String {
        let theme = theme(&self.settings);
        let s = &theme["Symbols"][key];
        if !s.is_object() {
            return String::new();
        }
        let description = escape(text(s, "Description"));
        let color = |v: &Value| {
            format!(
                "#{:02X}{:02X}{:02X}",
                v["R"].as_u64().unwrap_or(0),
                v["G"].as_u64().unwrap_or(0),
                v["B"].as_u64().unwrap_or(0)
            )
        };
        let color_value = |field: &str| -> &Value {
            if s[field].is_object() {
                &s[field]
            } else {
                let named = text(s, &format!("{field}Key"));
                let name = if named.is_empty() {
                    if field == "ContentColor" {
                        "DefaultText"
                    } else {
                        "DefaultBackground"
                    }
                } else {
                    named
                };
                self.settings["ThemeSettings"]["Colors"]
                    .get(name)
                    .filter(|_| self.settings["ThemeSettings"]["ThemeMode"] == 1)
                    .unwrap_or(&theme["Colors"][name])
            }
        };
        let foreground = color(color_value("ContentColor"));
        let background = color(color_value("ContentBackColor"));
        let inner = if let Some(text) = override_text {
            escape(text)
        } else {
            match text(s, "ContentType") {
                "Base64SVG" => {
                    let data = base64::engine::general_purpose::STANDARD
                        .decode(text(s, "Content"))
                        .unwrap_or_default();
                    let svg = String::from_utf8_lossy(&data)
                        .replace("$$COLOR$$", foreground.trim_start_matches('#'));
                    format!(
                        "<img height=16 src=\"data:image/svg+xml;base64,{}\" />",
                        base64::engine::general_purpose::STANDARD.encode(svg)
                    )
                }
                "Base64PNG" => format!(
                    "<img height=16 src=\"data:image/png;base64,{}\" />",
                    text(s, "Content")
                ),
                "Text" => {
                    let content = text(s, "Content");
                    if content.starts_with('\\') {
                        content
                            .split('\\')
                            .filter_map(|hex| {
                                u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
                            })
                            .collect()
                    } else {
                        escape(content)
                    }
                }
                _ => String::new(),
            }
        };
        let rounded = if s["IsLabel"] == true {
            format!(
                "background-color:{background};border-radius:5px;padding:0 {}px 0 {}px;",
                s["PaddingRight"].as_u64().unwrap_or(4),
                s["PaddingLeft"].as_u64().unwrap_or(4)
            )
        } else {
            String::new()
        };
        format!(
            "<span title=\"{description}\" style=\"color:{foreground};{rounded}font-weight:{};\">{inner}</span>",s["FontWeight"].as_str().unwrap_or("normal")
        )
    }
}

fn templates() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| serde_json::from_str(include_str!("html-templates.json")).unwrap())
}
fn stars(kind: &str) -> String {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| serde_json::from_str(include_str!("star-types.json")).unwrap())[kind]
        .as_str()
        .unwrap_or(kind)
        .to_owned()
}
fn planet(kind: &str) -> &str {
    match kind {
        "High metal content body" => "HMC",
        "Metal rich body" => "MR",
        "Rocky body" => "ROCKY",
        "Icy body" => "ICY",
        "Rocky ice body" => "RIB",
        "Sudarsky class I gas giant" => "GG1",
        "Sudarsky class II gas giant" => "GG2",
        "Sudarsky class III gas giant" => "GG3",
        "Sudarsky class IV gas giant" => "GG4",
        "Sudarsky class V gas giant" => "GG5",
        "Water world" => "WW",
        "Gas giant with water based life" => "GGWL",
        "Gas giant with ammonia based life" => "GGAL",
        "Helium rich gas giant" => "HRGG",
        "Helium gas giant" => "HGG",
        "Earthlike body" => "ELW",
        "Water giant" => "WG",
        "Ammonia world" => "AW",
        _ => kind,
    }
}
fn marker(key: &str) -> String {
    format!("[{key}]")
}
fn body_template(q: &Value, row: &str, checkbox: bool, highlight: bool, base: &str) -> String {
    let row = if q["IsNextToVisit"] == true && highlight {
        format!("{row} highlight")
    } else {
        row.into()
    };
    let boolv = |key: &str| q[key] == true;
    let mut html = format!(
        "<tr><td style='width:32px;'>{}</td><td class=\"{row}\">{}</td>",
        if boolv("IsNextToVisit") {
            marker("NextBody")
        } else {
            "&nbsp;&nbsp;&nbsp;&nbsp;".into()
        },
        escape(text(q, "Body"))
    );
    if boolv("IsStar") {
        let star = stars(text(q, "Type"));
        html.push_str(&format!("<td class=\"{row}\" colspan=2><span class=\"STAR\" title=\"{star}\">{star}</span></td>"));
    } else {
        let kind = planet(text(q, "Type"));
        html.push_str(&format!(
            "<td class=\"{row}\"><span class=\"{kind}\" ></span></td><td class=\"{row} w100\">"
        ));
        for (field, key) in [
            ("IsLandable", "IsLandable"),
            ("IsHighValue", "IsHighValue"),
            ("Terraformable", "Terraformable"),
        ] {
            if boolv(field) {
                html.push_str(&marker(key));
            }
        }
        if boolv("WasMapped") {
            html.push_str(&marker("WasMapped"));
        } else if boolv("WasDiscovered") {
            html.push_str(&marker("WasDiscovered"));
        }
        if boolv("WasFootfalled") {
            html.push_str(&marker("WasFootfalled"));
        }
        if q["BioCount"].as_i64().unwrap_or(-1) > 0 {
            html.push_str(&marker("HasBiologicals"));
        }
        if !text(q, "Volcanism").is_empty() {
            html.push_str(&marker("HasVolcanism"));
        }
        html.push_str("&nbsp;");
        for field in [
            "BioInsightsPersonalCodex",
            "BioInsightsMarked",
            "BioInsightsGalacticCodex",
            "BioInsightsHighValue",
            "PluginWorthVisiting",
        ] {
            if boolv(field) {
                html.push_str(&marker(field));
            }
        }
        html.push_str("</td>");
    }
    html.push_str(&format!(
        "<td class=\"{row} right w50\">{} LS</td>",
        credits(q["Distance"].as_f64().unwrap_or(0.0).round_ties_even() as i64)
    ));
    let gravity = q["Gravity"].as_f64().unwrap_or(0.0);
    if gravity > 0.0 {
        let symbol = if gravity >= 2.0 {
            "Alert"
        } else if gravity >= 1.0 {
            "Warn1"
        } else if gravity >= 0.5 {
            "Warn0"
        } else {
            "OK"
        };
        let number = format!("{gravity:.2}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_owned();
        html.push_str(&format!(
            "<td class=\"{row} right w50\">[{symbol}:{number} G]</td>"
        ));
    } else {
        html.push_str(&format!("<td class=\"{row} w50\">&nbsp;</td>"));
    }
    if boolv("IsStar") {
        html.push_str(&format!("<td class=\"{row}\" colspan=2>&nbsp;</td>"));
    } else if boolv("Done") {
        html.push_str(&format!(
            "<td class=\"{row} left w50\" colspan=2\">[Mapped]</td>"
        ));
    } else if checkbox {
        html.push_str(&format!("<td class=\"{row}\"><a href=\"{base}/toggle/{}/{}/{}\">[{}]</a></td><td class=\"{row} left w30\">",q["ID64"],q["BodyID"],u8::from(!boolv("Map")),if boolv("Map"){"WorthMapping"}else{"NotWorthMapping"}));
        let order = q["Order"].as_u64().unwrap_or(0);
        if order > 0 {
            html.push_str(&format!(
                "<span title=\"Visit Order\" class=\"visitorder\">[{order}]</span>"
            ));
        } else {
            html.push_str("&nbsp;");
        }
        html.push_str("</td>");
    } else {
        html.push_str(&format!(
            "<td class=\"{row} left w50\" colspan=2\">&nbsp;</td>"
        ));
    }
    html.push_str(&format!(
        "<td class=\"{row} right\">{}</td><td class=\"{row} right padleft\">{}</td></tr>",
        credits(q["CurrentValue"].as_i64().unwrap_or(0)),
        credits(q["PotentialMaxValue"].as_i64().unwrap_or(0))
    ));
    html
}
impl Evaluator {
    fn expand_symbols(&self, source: &str) -> String {
        let mut out = String::new();
        let mut rest = source;
        while let Some(start) = rest.find('[') {
            out.push_str(&rest[..start]);
            let Some(end) = rest[start..].find(']') else {
                out.push_str(&rest[start..]);
                return out;
            };
            let token = &rest[start + 1..start + end];
            let (key, text) = token
                .split_once(':')
                .map_or((token, None), |(key, text)| (key, Some(text)));
            if theme(&self.settings)["Symbols"][key].is_object() {
                out.push_str(&self.symbol(key, text));
            } else {
                out.push_str(&rest[start..=start + end]);
            }
            rest = &rest[start + end + 1..];
        }
        out.push_str(rest);
        out
    }
    pub fn web_main_html(&self) -> String {
        let headers = self.settings["ShowHeaders"].as_bool().unwrap_or(true);
        let total = flag(&self.settings, "ShowTotal");
        let t = templates()["headers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["head"] == headers && v["total"] == total)
            .unwrap();
        let mut html = text(t, "html")
            .replace("Oracle", &escape(&self.commander))
            .replace(
                "123 456",
                &credits(self.summary()["UnsoldTotal"].as_i64().unwrap_or(0)),
            );
        if !self.settings["ExpandOutput"].as_bool().unwrap_or(true) {
            html = html.replace("width=\"100%\"", "");
        }
        let rows = self.rows();
        let mut by_visit = BTreeMap::<(u64, u64), Vec<&Value>>::new();
        for row in &rows {
            by_visit
                .entry((
                    row["ID64"].as_u64().unwrap_or(0),
                    row["VisitID"].as_u64().unwrap_or(0),
                ))
                .or_default()
                .push(row);
        }
        let summary = self.summary();
        let systems: Vec<_> = summary["Systems"]
            .as_array()
            .map(|values| values.iter().collect())
            .unwrap_or_default();
        for sys in systems {
            let current = sys["Current"] == true;
            let filter = match self.filter.as_str() {
                "MappingOnly" => 1,
                "All" => 2,
                _ => 0,
            };
            let template = templates()["systems"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["current"] == current && v["filter"] == filter)
                .unwrap();
            let mut header = text(template, "html")
                .replace("Oracle", &escape(text(sys, "System")))
                .replace(
                    "123 456",
                    &credits(sys["SystemValue"].as_i64().unwrap_or(0)),
                )
                .replace(
                    "654 321",
                    &credits(sys["PotentialValue"].as_i64().unwrap_or(0)),
                )
                .replace(
                    "7 / 12",
                    &format!("{} / {}", sys["ScannedBodies"], sys["TotalBodies"]),
                );
            if sys["ScannedBodies"] == sys["TotalBodies"] {
                header = header.replace("BodiesNotAllScanned", "BodiesAllScanned");
            }
            html.push_str(&header);
            for (index, row) in by_visit
                .get(&(
                    sys["ID64"].as_u64().unwrap_or(0),
                    sys["VisitID"].as_u64().unwrap_or(0),
                ))
                .into_iter()
                .flatten()
                .enumerate()
            {
                html.push_str(&body_template(
                    row,
                    if index % 2 == 0 { "row1" } else { "row2" },
                    current,
                    theme(&self.settings)["Settings"]["UseHighlightRow"]
                        .as_bool()
                        .unwrap_or(false),
                    "/evaluator",
                ));
            }
            if headers {
                html.push_str("<tr><th class=\"banner border\" colspan=10>&nbsp;</th>");
            }
            html.push_str("<tr><td colspan=10>&nbsp;</td></tr>");
        }
        html.push_str("<tr><td>&nbsp;</td></tr></table></div><span id=\"theend\"></span><br />");
        self.source_alerts()
            + &self.expand_symbols(&html.replace("http://localhost/evaluator", "/evaluator"))
    }
    pub(super) fn source_about(&self) -> String {
        let mut html = templates()["about"].as_str().unwrap().to_owned();
        for (key, symbol) in theme(&self.settings)["Symbols"].as_object().unwrap() {
            html = html.replace(&format!("{{{{desc:{key}}}}}"), text(symbol, "Description"));
        }
        self.source_alerts() + &self.expand_symbols(&html)
    }
    pub(super) fn source_unsold(&self) -> String {
        let rows = self
            .unsold_data
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let total: i64 = rows.iter().map(|row| row[2].as_i64().unwrap_or(0)).sum();
        let mut html = templates()["unsold"][0]["html"].as_str().unwrap().replace(
            "class=\"right\">0",
            &format!("class=\"right\">{}", credits(total)),
        );
        let mut index = 0;
        for row in rows {
            let value = row[2].as_i64().unwrap_or(0);
            if value <= 0 {
                continue;
            }
            let date = row[1].as_str().unwrap_or("");
            let date = chrono::DateTime::parse_from_rfc3339(date).map_or_else(
                |_| date.to_owned(),
                |d| d.format("%d-%b-%y %H:%M").to_string(),
            );
            let class = if index % 2 == 0 { "row2" } else { "row1" };
            index += 1;
            html.push_str(&format!("<tr><td class=\"{class}\">{}</td><td class=\"{class}\">{}</td><td class=\"{class} right\">{}</td></tr>",escape(&date),escape(row[0].as_str().unwrap_or("")),credits(value)));
        }
        self.source_alerts() + &html
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn about_and_unsold_pages_match_original_generators_and_keep_duplicate_system_visits() {
        let oracle: Value =
            serde_json::from_str(include_str!("../../../fixtures/evaluator-html-oracle.json"))
                .unwrap();
        let mut worker = Evaluator::new(json!({}));
        let mut expected = oracle["about"].as_str().unwrap().to_owned();
        for (key, symbol) in theme(&worker.settings)["Symbols"].as_object().unwrap() {
            expected =
                expected.replace(&format!("{{{{desc:{key}}}}}"), text(symbol, "Description"));
        }
        expected = worker.expand_symbols(&expected);
        assert_eq!(worker.source_about(), expected);
        for case in oracle["unsold"].as_array().unwrap() {
            worker.unsold_data = case["lines"].clone();
            assert_eq!(worker.source_unsold(), case["html"], "{case}");
        }
    }
    #[test]
    fn all_512_html_body_rows_match_original_source_generators() {
        let oracle: Value =
            serde_json::from_str(include_str!("../../../fixtures/evaluator-html-oracle.json"))
                .unwrap();
        for case in oracle["rows"].as_array().unwrap() {
            let mut q = case["q"].clone();
            for (field, value) in case["q"]["Extra"].as_object().unwrap() {
                q[field] = value.clone();
            }
            assert_eq!(
                body_template(
                    &q,
                    "row1",
                    case["checkbox"] == true,
                    true,
                    "http://localhost/evaluator"
                ),
                case["html"].as_str().unwrap(),
                "mask{} checkbox{}",
                case["mask"],
                case["checkbox"]
            );
        }
    }
}
