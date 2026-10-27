//! Source HTML pages shared by the embedded browser and Hecate HTTP output.
//! Main row/header templates are captures of the original pure generators.
use super::*;
use base64::Engine as _;
use std::sync::OnceLock;
fn arr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v[key].as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn escape(s: &str) -> String {
    super::web::escape(s)
}
fn credits(n: u64) -> String {
    let s = n.to_string();
    s.chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if i > 0 && (s.len() - i).is_multiple_of(3) {
                vec![' ', c]
            } else {
                vec![c]
            }
        })
        .collect()
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
impl BioInsights {
    pub fn browser_route(&self) -> String {
        // ccshow updates two elements in the current document. Its selected
        // colour is internal view state, not a document navigation/reload.
        if self
            .browser_route
            .first()
            .is_some_and(|p| p == "codexcompanion")
        {
            self.browser_route
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join("/")
        } else {
            self.browser_route.join("/")
        }
    }
    pub fn browser_html(&self) -> String {
        if let Some(cached) = self.browser_cache.lock().expect("browser cache").as_ref() {
            return cached.clone();
        }
        let content = self.browser_content(&self.browser_route);
        let html = self
            .source_document(&content, true)
            .replace(
                "http://localhost/bioinsights/",
                "observatory://bioinsights/",
            )
            .replace("href=\"/bioinsights/", "href=\"observatory://bioinsights/");
        *self.browser_cache.lock().expect("browser cache") = Some(html.clone());
        html
    }
    pub(super) fn browser_navigate(&mut self, payload: &Value) {
        let url = text(payload, "Url");
        let parts = super::web::route_parts(url);
        let page = parts.first().map(String::as_str).unwrap_or("main");
        match page {
            "gethecate" => self.message(
                "Core",
                "InstallNativePlugin",
                json!({"Plugin":"Hecate","Dependency":true}),
            ),
            "skiphecate" => {
                self.hecate_nag_skipped = true;
                self.browser_route = vec!["main".into()];
            }
            "hecatenag" => self.browser_route = parts,
            "popout" | "settings" | "zoom" => self.message(
                "Core",
                "BioInsightsUiAction",
                json!({"Action":match page{"popout"=>"Popout","settings"=>"Settings",_=>"Zoom"}}),
            ),
            "alert" => {
                if let Some(id) = parts.get(1).and_then(|id| id.parse::<u8>().ok()) {
                    self.dismiss_ui_alert(id);
                }
            }
            "ccmark" if parts.len() >= 4 => {
                if let Some(species) = parts[2]
                    .parse::<u64>()
                    .ok()
                    .and_then(super::web::species_by_number)
                {
                    let variant = parts[3]
                        .parse::<usize>()
                        .ok()
                        .and_then(|i| rules::COLOURS.get(i))
                        .copied()
                        .unwrap_or("");
                    self.on_message(
                        "MarkVariant",
                        &json!({"SpeciesID":species,"Variant":variant}),
                    );
                }
            }
            "ccshow" if parts.len() >= 4 => {
                self.browser_route = vec![
                    "codexcompanion".into(),
                    parts[1].clone(),
                    parts[2].clone(),
                    parts[3].clone(),
                ];
            }
            "toggleunsold" if parts.len() == 3 => {
                if let Ok(id) = parts[1].parse::<u64>() {
                    self.on_message("SetBonus", &json!({"ID":id,"Value":parts[2]=="1"}));
                }
            }
            "unsold" => {
                self.browser_route = parts;
                self.on_message("RefreshUnsold", &json!({}));
            }
            "main" | "about" | "codexcompanion" => self.browser_route = parts,
            _ => {}
        }
        *self.browser_cache.get_mut().expect("browser cache") = None;
    }
    fn browser_content(&self, parts: &[String]) -> String {
        let content = match parts.first().map(String::as_str).unwrap_or("main") {
            "hecatenag" => self.source_hecate_nag(),
            "unsold" => self.source_unsold(),
            "about" => self.source_about(),
            "codexcompanion" | "ccshow" => self.source_codex(parts),
            _ => self.source_main(),
        };
        self.source_alerts() + &content
    }
    pub(super) fn source_hecate_nag(&self) -> String {
        include_str!("hecate.html").trim_start_matches('\u{feff}').replace("<body class=\"hecatebody\">","<div class=\"hecatebody\">").replace("</body>","</div>").replace("http://127.0.0.1/bioinsights/","http://localhost/bioinsights/").replace(">Download</button>",">Enable Hecate</button><br /><small>Hecate is included with this Linux build.</small>")
    }
    pub(super) fn source_document(&self, content: &str, embedded: bool) -> String {
        let stylesheet = self.source_stylesheet();
        let mut buttons = String::from("<div class=\"buttonheader\">");
        for (id, title) in [
            ("main", "Main Window"),
            ("unsold", "Show Current Data"),
            ("about", "About"),
            ("codexcompanion", "Codex Companion"),
            ("popout", "Popout Window"),
            ("zoom", "Zoom"),
            ("settings", "Settings"),
        ] {
            let hidden = if !embedded && matches!(id, "popout" | "zoom" | "settings") {
                " style=\"display:none\""
            } else {
                ""
            };
            buttons.push_str(&format!("<a href=\"/bioinsights/{id}\"><button id=\"{id}\" title=\"{title}\"{hidden}></button></a>"));
        }
        buttons.push_str("</div>");
        let script = if embedded {
            "<script>document.addEventListener('keydown',function(e){if(e.ctrlKey&&['+','-','='].includes(e.key))e.preventDefault();});</script>".to_owned()
        } else {
            format!(
                "<script>{}</script>",
                include_str!("app.js")
                    .replace(
                        "$$USE_WS$$",
                        if self.web_server.is_empty() { "0" } else { "1" }
                    )
                    .replace("$$WS_SERVER$$", &self.web_server)
                    .replace("$$PLUGIN$$", "bioinsights")
            )
        };
        format!("<!doctype html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>BioInsights</title><style>{stylesheet}</style></head><body style=\"zoom:{}%\">{buttons}<div id=\"webapp\"></div><div id=\"disconnected\"></div><div class=\"content\" id=\"maincontent\">{content}</div>{script}</body></html>",self.settings["ZoomLevel"].as_u64().unwrap_or(100))
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
        // Original ThemeSettings.UseHighlightRow defaults false. Its theme
        // has no HighlightBackground entry; the inactive CSS token uses the
        // same zero RGB default as a missing source ThemeColor.
        if theme["Colors"].get("HighlightBackground").is_none() {
            style = style.replace("$$HIGHLIGHTBACKGROUND$$", "000000");
        }
        for key in ["BorderWidth", "RowSpacing"] {
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
        style.push_str(&super::symbol_css::stylesheet(
            theme,
            (self.settings["ThemeSettings"]["ThemeMode"] == 1)
                .then_some(&self.settings["ThemeSettings"]["Colors"]),
        ));
        style
    }
    fn symbol(&self, key: &str, override_text: Option<&str>) -> String {
        if !theme(&self.settings)["Symbols"][key].is_object() {
            return String::new();
        }
        // Original BaseUIHandler.GetSymbol emits a classed span; content,
        // image dimensions, labels and hover text come from symbols.css.
        format!(
            "<span class=\"{}\">{}</span>",
            escape(key),
            override_text.map(escape).unwrap_or_default()
        )
    }
    fn expand_symbols(&self, source: &str) -> String {
        let mut out = String::new();
        let mut rest = source;
        while let Some(start) = rest.find('[') {
            out.push_str(&rest[..start]);
            let Some(end) = rest[start..].find(']') else {
                out.push_str(&rest[start..]);
                return out;
            };
            let key = &rest[start + 1..start + end];
            let (key, value) = key
                .split_once(':')
                .map(|(k, v)| (k, Some(v)))
                .unwrap_or((key, None));
            if theme(&self.settings)["Symbols"][key].is_object() {
                out.push_str(&self.symbol(key, value));
            } else {
                out.push_str(&rest[start..=start + end]);
            }
            rest = &rest[start + end + 1..];
        }
        out.push_str(rest);
        out
    }
    fn row_flags(&self, row: &Value, body: bool) -> String {
        let mut out = String::new();
        for (field, key) in [
            ("Current", "CurrentBody"),
            ("HighValue", "HighValue"),
            ("PersonalCodex", "PersonalCodexEntry"),
            ("Marked", "Mark"),
            ("GalacticNeverSeen", "PossibleCodexEntryNeverSeen"),
            ("GalacticCodex", "PossibleCodexEntry"),
        ] {
            if !body && matches!(field, "Current" | "HighValue") {
                continue;
            }
            if row[field] == true && !(field == "GalacticCodex" && row["GalacticNeverSeen"] == true)
            {
                out.push_str(&self.symbol(key, None));
            }
        }
        out
    }
    pub(super) fn source_main(&self) -> String {
        let view = self.main_view();
        let mut html = format!(
            "<div class=\"scroll\"><table {}>",
            if self.settings["ExpandOutput"] != false {
                "width=\"100%\""
            } else {
                ""
            }
        );
        static HEADERS: OnceLock<Value> = OnceLock::new();
        let headers = HEADERS
            .get_or_init(|| serde_json::from_str(include_str!("header-templates.json")).unwrap());
        let header = headers
            .as_array()
            .unwrap()
            .iter()
            .find(|h| {
                h["show_headers"].as_bool() == Some(self.settings["ShowHeaders"] != false)
                    && h["show_body_fields"].as_bool()
                        == Some(self.settings["ShowBodyFields"] == true)
                    && h["show_total"].as_bool() == Some(self.settings["ShowTotal"] == true)
            })
            .unwrap();
        html.push_str(
            &text(header, "html")
                .replace("Reference Commander", &escape(text(&view, "Commander")))
                .replace(
                    "123 456 789",
                    &credits(view["UnsoldTotal"].as_u64().unwrap_or(0)),
                ),
        );
        for group in arr(&view, "Groups") {
            let highlight = if group["Current"] == true
                && theme(&self.settings)["Settings"]["UseHighlightRow"] == true
            {
                "highlight"
            } else {
                ""
            };
            let body = escape(text(group, "BodyName"));
            let id = group["ID64"].as_u64().unwrap_or(0);
            let bodyid = group["BodyID"].as_u64().unwrap_or(0);
            let count = group["Signals"].as_u64().unwrap_or(0);
            let signals = format!("{count} bio{}", if count == 1 { "" } else { "s" });
            html.push_str(&format!("<tr><td class=\"body roundleft {highlight}\" id=\"{id}-{bodyid}\" colspan=4>{}{body}</td><td class=\"planet {highlight}\">{}</td><td colspan=1 class=\"signals {highlight}\">{}</td><td colspan=3 class=\"radiant {highlight}\"><span title=\"Radiant Star\">Star: {}</span></td><td colspan=2 class=\"value {highlight}\"><span title=\"Base value\">{}</span></td></tr>",self.row_flags(group,true),self.symbol(text(group,"PlanetSymbol"),None),self.symbol(text(group,"SignalSymbol"),Some(&signals)),escape(text(group,"RadiantStar")),escape(text(group,"ValueText"))));
            if self.settings["ShowBodyFields"] == true {
                let e = &group["ExtraFields"];
                html.push_str(&format!("<tr><td>&nbsp;</td><td class=\"body extra bl {highlight}\" colspan=3>{}</td><td class=\"body extra {highlight}\" colspan=2>{}</td><td class=\"body extra {highlight}\" colspan=2>{}</td><td class=\"body extra {highlight}\">{}</td><td colspan=2 class=\"body extra br {highlight}\">{}</td></tr>",escape(text(e,"Atmosphere")),escape(text(e,"Pressure")),escape(text(e,"Volcanism")),escape(text(e,"Gravity")),escape(text(e,"Temperature"))));
            }
            for (index, row) in arr(group, "Rows").iter().enumerate() {
                html.push_str(&self.source_row(row, index));
            }
            if arr(group, "Rows").is_empty() {
                html.push_str("<tr><td>&nbsp;&nbsp;</td><td colspan=10 width=\"100%\" class=\"row1 NotPresentItalic center\">No predictions</td></tr>");
            }
            if self.settings["ShowSplitters"] == true {
                html.push_str("<tr><td colspan=11><hr /?></td></tr>");
            }
        }
        if self.settings["ShowHeaders"] != false {
            html.push_str("<tr><th class=\"banner \" colspan=11>&nbsp;</th></tr>");
        }
        html.push_str("<tr><th id=\"footer\" class=\"body\" colspan=11>&nbsp;</th></table></div><span id=\"theend\" />");
        html
    }
    fn source_row(&self, row: &Value, index: usize) -> String {
        static ROWS: OnceLock<Value> = OnceLock::new();
        let templates =
            ROWS.get_or_init(|| serde_json::from_str(include_str!("row-templates.json")).unwrap());
        let template = templates
            .as_array()
            .unwrap()
            .iter()
            .find(|t| {
                t["type"] == row["PredictionType"]
                    && t["seen"] == row["Seen"]
                    && t["samples"] == row["SampleState"]
            })
            .unwrap_or(&templates[0]);
        let mut html = text(template, "html").to_owned();
        let mut variants = Vec::new();
        for variant in arr(row, "Variants") {
            if variant["ShowInUI"] == false
                || variant["NotPresent"] == true && self.settings["HideNotPresent"] == true
            {
                continue;
            }
            let body = format!(
                "{}{}<span title=\"{}\">{}</span>",
                self.row_flags(variant, false),
                self.symbol(text(variant, "Likelihood"), None),
                escape(text(variant, "Reason")),
                escape(text(variant, "Colour"))
            );
            variants.push(if variant["NotPresent"] == true {
                format!("<span class=\"NotPresentItalic\">{body}</span>")
            } else {
                body
            });
        }
        if let Some(start) = html.find("<td colspan=2") {
            if let Some(inner) = html[start..].find('>') {
                let inner = start + inner + 1;
                if let Some(end) = html[inner..].find("</td>") {
                    html.replace_range(
                        inner..inner + end,
                        &variants.join(if row["PredictionType"] == "SurfaceMaterial" {
                            " or "
                        } else {
                            ", "
                        }),
                    );
                }
            }
        }
        html = html
            .replace(
                "[PersonalCodexEntry]Bacterium",
                &format!(
                    "{}{}",
                    self.row_flags(row, false),
                    escape(text(row, "Genus"))
                ),
            )
            .replace("Alcyoneum", &escape(text(row, "Species")))
            .replace(
                "http://localhost/bioinsights/codexcompanion/0/0",
                &format!(
                    "/bioinsights/codexcompanion/{}/{}",
                    row["GenusNumber"], row["SpeciesNumber"]
                ),
            )
            .replace("1.66M", &escape(text(row, "ValueText")))
            .replace("500m", &format!("{}m", row["Distance"]));
        // GenerateSystemTable adds this class before passing rowclass to
        // GenerateHTMLBodyDetails for every NotPresent species.
        if row["Seen"] == "NotPresent" {
            html = html.replace("row1", "row1 NotPresentItalic");
        }
        if !row["Stripe"]
            .as_u64()
            .unwrap_or(index as u64)
            .is_multiple_of(2)
        {
            html = html.replace("row1", "row2");
        }
        self.expand_symbols(&html)
    }
    pub(super) fn source_unsold(&self) -> String {
        let mut body=String::from("<tr><th class=\"unsold\">Date</th><th class=\"unsold\">Body</th><th class=\"unsold\">Species</th><th class=\"unsold\">Base Value</th><th class=\"unsold\">First<br/>Logged</th><th class=\"unsold\">Expected Value</th></tr>");
        let mut total = 0u64;
        let mut previous = "unicorns".to_owned();
        let mut stripe = 1;
        for row in self.unsold_data() {
            let name = text(&row, "BodyName");
            if previous != name {
                stripe = 3 - stripe;
            }
            previous = name.to_owned();
            let base = row["BaseValue"].as_u64().unwrap_or(0);
            let first = row["FirstFootfall"] == true;
            let expected = base * if first { 5 } else { 1 };
            total += expected;
            let id = row["ID"].as_u64().unwrap_or(0);
            body.push_str(&format!("<tr class=\"spacing\"><td class=\"unsold row{stripe}\">{}</td><td class=\"unsold row{stripe}\">{}</td><td class=\"unsold row{stripe}\">{}</td><td id=\"base-{id}\" class=\"unsold row{stripe} right\">{}</td><td class=\"unsold row{stripe}\"><a id=\"link-{id}\" href=\"/bioinsights/toggleunsold/{id}/{}\">{}</a><span class=\"hidden\" id=\"bonus-{id}\">{}</span></td><td class=\"unsold row{stripe} right\" id=\"value-{id}\">{}</td></tr>",escape(&source_date(text(&row,"DateTime"))),escape(name),escape(text(&row,"FullName")),credits(base),if first{0}else{1},self.symbol(if first{"OptionOn"}else{"OptionOff"},None),if first{1}else{0},credits(expected)));
        }
        format!("<table><tr><td colspan=5 class=\"unsold extra\"><b>Total</b></td><td id=\"total\" class=\"unsold right\"><b>{}</b></td></tr><tr><td colspan=6>&nbsp;</td></tr>{body}</table>",credits(total))
    }
    pub(super) fn source_about(&self) -> String {
        // Complete original GenerateAboutPage output, captured without UI or
        // filesystem effects. Theme descriptions and symbols are live inputs.
        let mut html = include_str!("about-source.html").to_owned();
        for (key, symbol) in theme(&self.settings)["Symbols"].as_object().unwrap() {
            html = html.replace(
                &format!("$$Description:{key}$$"),
                text(symbol, "Description"),
            );
        }
        self.expand_symbols(&html)
    }
    pub(super) fn source_codex(&self, parts: &[String]) -> String {
        let g = parts
            .get(1)
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let s = parts
            .get(2)
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let colour = parts.get(3).and_then(|s| s.parse::<u64>().ok());
        let row = self
            .codex_companion()
            .into_iter()
            .find(|r| r["GenusNumber"] == g && r["SpeciesNumber"] == s);
        let gallery = row
            .as_ref()
            .map(|r| self.source_gallery(text(r, "FullName")))
            .unwrap_or_default();
        let region = row
            .as_ref()
            .map(|r| self.source_region_map(r, colour))
            .unwrap_or_default();
        let map_name = row
            .as_ref()
            .map(|r| {
                let full = text(r, "FullName");
                if r["Horizons"] != true {
                    if let Some(v) =
                        colour.and_then(|c| arr(r, "Variants").iter().find(|v| v["ColourID"] == c))
                    {
                        return format!("{full} - {}", text(v, "Colour"));
                    }
                }
                full.to_owned()
            })
            .unwrap_or_default();
        self.codex_markup(g, s, &gallery, &region, &escape(&map_name))
    }
    fn codex_markup(
        &self,
        genus: u64,
        species: u64,
        gallery: &str,
        region: &str,
        map_name: &str,
    ) -> String {
        // These are the original installed pure page builders evaluated for the
        // complete 117-species catalogue. Only their actual runtime inputs are
        // substituted: commander, personal marks/regions and image content.
        // capture-bio-codex-page.py records provenance and four state oracles.
        static PAGES: OnceLock<BTreeMap<String, String>> = OnceLock::new();
        let pages = PAGES.get_or_init(|| {
            serde_json::from_str(include_str!("companion-pages.json")).expect("source Codex pages")
        });
        let key = format!("{genus}/{species}");
        let Some(template) = pages.get(&key) else {
            return String::new();
        };
        let mut html = template
            .replace("$$Commander$$", &escape(&self.commander_name))
            .replace("$$Gallery$$", gallery)
            .replace("$$Map$$", region)
            .replace("$$MapName$$", map_name);
        if species != 0 {
            if let Some(row) = self
                .codex_companion()
                .iter()
                .find(|r| r["GenusNumber"] == genus && r["SpeciesNumber"] == species)
            {
                for variant in arr(row, "Variants") {
                    let colour = variant["ColourID"].as_u64().unwrap_or(0);
                    let mark = self.symbol(
                        if variant["Marked"] == true {
                            "OptionOn"
                        } else {
                            "OptionOff"
                        },
                        None,
                    );
                    let count = variant["SeenRegionCount"].as_u64().unwrap_or(0);
                    let seen = match count {
                        0 => "No".to_owned(),
                        1 => "1 region".to_owned(),
                        n => format!("{n} regions"),
                    };
                    html = html
                        .replace(&format!("$$Mark:{colour}$$"), &mark)
                        .replace(&format!("$$Seen:{colour}$$"), &seen);
                }
            }
        }
        html
    }
    fn source_region_map(&self, species: &Value, colour: Option<u64>) -> String {
        let mask = colour
            .and_then(|c| arr(species, "Variants").iter().find(|v| v["ColourID"] == c))
            .and_then(|v| v["SeenRegionMask"].as_u64())
            .unwrap_or(0);
        let possible = species["RegionMask"].as_u64().unwrap_or(0);
        let source_theme = theme(&self.settings);
        let color = self.settings["ThemeSettings"]["Colors"]
            .get("DefaultText")
            .filter(|_| self.settings["ThemeSettings"]["ThemeMode"] == 1)
            .unwrap_or(&source_theme["Colors"]["DefaultText"]);
        let legend_color = format!(
            "#{:02X}{:02X}{:02X}",
            color["R"].as_u64().unwrap_or(255),
            color["G"].as_u64().unwrap_or(255),
            color["B"].as_u64().unwrap_or(255)
        );
        let coords = include_str!("region-coordinates.json");
        let image =
            base64::engine::general_purpose::STANDARD.encode(include_bytes!("regionmap.png"));
        let possible = if possible == 0 {
            (1u64 << 43) - 2
        } else {
            possible
        };
        format!(
            r#"<canvas id="regionmapcanvas" width="400" height="400"></canvas><script>(function(){{const c=document.getElementById('regionmapcanvas'),g=c.getContext('2d'),img=new Image();img.onload=function(){{g.drawImage(img,0,0);let a=g.getImageData(0,0,400,400),p=a.data;function fill(x,y,color){{let at=(y*400+x)*4,target=[p[at],p[at+1],p[at+2],p[at+3]];if(target.every((v,i)=>v===color[i]))return;let todo=[[x,y]];while(todo.length){{let [x,y]=todo.pop();if(x<0||x>=400||y<0||y>=400)continue;let n=(y*400+x)*4;if(!target.every((v,i)=>p[n+i]===v))continue;for(let i=0;i<4;i++)p[n+i]=color[i];todo.push([x-1,y],[x+1,y],[x,y-1],[x,y+1]);}}}}let brown=[136,78,8,255],mint=[164,242,203,255],coords={coords};for(let [r,x,y]of coords)if(BigInt('{possible}')&(1n<<BigInt(r)))fill(Math.floor(x*400/720),Math.floor(y*400/720),brown);for(let [r,x,y]of coords)if(BigInt('{mask}')&(1n<<BigInt(r)))fill(Math.floor(x*400/720),Math.floor(y*400/720),mint);fill(345,375,brown);fill(345,390,mint);g.putImageData(a,0,0);g.fillStyle='{legend_color}';g.font='bold 16px "Century Gothic"';g.textBaseline='top';g.fillText('Possible',352,368);g.fillText('Seen',352,383);}};img.src='data:image/png;base64,{image}';}})();</script>"#
        )
    }
    fn source_gallery(&self, species: &str) -> String {
        let unknown = base64::engine::general_purpose::STANDARD.encode(include_bytes!(
            "../../../observatory-desktop/assets/bioinsights/species.Unknown.jpg"
        ));
        let species = serde_json::to_string(species).unwrap();
        format!(
            r#"<img id="galleryimage" width="480" height="270" src="data:image/jpeg;base64,{unknown}"/><span id="gallerycredit" style="font-family:Verdana"></span><script>(function(){{let image=document.getElementById('galleryimage');if(image.dataset.loaded)return;image.dataset.loaded='1';fetch('https://elitetools.github.io/codexcompanion/manifest.json').then(r=>r.json()).then(m=>{{let a=m[{species}];if(!Array.isArray(a)||!a.length)return;let v=a[new Date().getSeconds()%a.length];let get=k=>v[Object.keys(v).find(x=>x.toLowerCase()===k.toLowerCase())]||'';document.getElementById('galleryimage').src='https://elitetools.github.io/codexcompanion/'+get('FileName');let name=get('Name');if(name){{let host=document.getElementById('gallerycredit');host.appendChild(document.createElement('br'));let span=document.createElement('span');span.className='h3cc';span.textContent=name;host.appendChild(span);host.appendChild(document.createElement('br'));let credit=document.createElement('i');credit.textContent='CMDR '+get('Cmdr');host.appendChild(credit);}}}}).catch(()=>{{}});}})();</script>"#
        )
    }
}
fn source_date(value: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(value) {
        return dt.format("%d-%b-%Y %H:%M").to_string();
    }
    for format in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%d %H:%M:%S%.f"] {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(value, format) {
            return dt.format("%d-%b-%Y %H:%M").to_string();
        }
    }
    value.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn about_and_unsold_match_complete_original_page_builders() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../fixtures/bioinsights-codex-page-oracle.json"
        ))
        .unwrap();
        let mut bio = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        for index in [0, 1] {
            bio.settings["ThemeSettings"]["ThemeIndex"] = json!(index);
            let mut expected = fixture["About"].as_str().unwrap().to_owned();
            for (key, symbol) in theme(&bio.settings)["Symbols"]
                .as_object()
                .unwrap()
                .iter()
                .rev()
            {
                expected =
                    expected.replace(&format!("description://{key}"), text(symbol, "Description"));
            }
            for (key, value) in [
                ("BioCount", Some("5 bios")),
                ("BioCountMismatch", Some("2 bios")),
            ] {
                expected = expected.replace(
                    &format!("symbol://{key}[{}]", value.unwrap()),
                    &bio.symbol(key, value),
                );
            }
            for key in theme(&bio.settings)["Symbols"]
                .as_object()
                .unwrap()
                .keys()
                .rev()
            {
                expected = expected.replace(&format!("symbol://{key}"), &bio.symbol(key, None));
            }
            assert_eq!(bio.source_about(), expected, "theme {index}");
        }
        for case in fixture["Unsold"].as_array().unwrap() {
            let count = case["Count"].as_u64().unwrap();
            let bonus = case["Bonus"].as_bool().unwrap();
            bio.vault_unsold = (0..count)
                .map(|i| {
                    json!([
                        i + 1,
                        format!("2026-10-{}T09:05:47Z", 26 + i),
                        if i < 2 { "Source A 1" } else { "Source A 2" },
                        if i == 3 { 91 } else { 1 + i },
                        1234567 + i,
                        bonus
                    ])
                })
                .collect();
            let actual = bio
                .source_unsold()
                .replace(&bio.symbol("OptionOn", None), "symbol://OptionOn")
                .replace(&bio.symbol("OptionOff", None), "symbol://OptionOff");
            assert_eq!(
                actual,
                case["Html"].as_str().unwrap(),
                "count={count},bonus={bonus}"
            );
        }
    }
    #[test]
    fn complete_codex_page_markup_matches_original_builders_and_personal_states() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../fixtures/bioinsights-codex-page-oracle.json"
        ))
        .unwrap();
        let mut bio = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        bio.commander_name = "Synthetic".into();
        let catalogue = bio.codex_companion();
        for case in fixture["Cases"].as_array().unwrap() {
            bio.personal_codex.clear();
            bio.marked_variants.clear();
            let genus = case["Genus"].as_u64().unwrap();
            let species = case["Species"].as_u64().unwrap();
            let mask = case["Seen"].as_u64().unwrap();
            let row = catalogue
                .iter()
                .find(|r| r["GenusNumber"] == genus && r["SpeciesNumber"] == species);
            if let Some(row) = row {
                let key = text(row, "SpeciesID");
                for variant in arr(row, "Variants") {
                    let colour = text(variant, "Colour");
                    if case["Marked"] == true {
                        bio.marked_variants.insert((key.into(), colour.into()));
                    }
                    for region in 1..=42 {
                        if mask & (1u64 << region) != 0 {
                            bio.personal_codex
                                .insert((region, key.into(), colour.into()));
                        }
                    }
                }
            }
            let region = format!(
                "map://{}/0",
                row.and_then(|r| r["RegionMask"].as_u64()).unwrap_or(0)
            );
            let name = row.map(|r| text(r, "FullName")).unwrap_or("");
            let mut actual = bio.codex_markup(
                genus,
                species,
                "<img src=\"gallery://synthetic\" width=480 height=270 />",
                &region,
                name,
            );
            for id in ["OptionOff", "OptionOn"] {
                actual = actual.replace(&bio.symbol(id, None), &format!("symbol://{id}"));
            }
            assert_eq!(
                actual,
                case["Html"].as_str().unwrap(),
                "genus={genus},species={species},seen={mask}"
            );
        }
    }
    fn event(v: Value) -> JournalEvent {
        JournalEvent::parse(&v.to_string()).unwrap()
    }
    #[test]
    fn missing_hecate_uses_real_native_install_action_and_skip_retains_current_host() {
        let mut bio = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        bio.set_batch(true);
        bio.set_batch(false);
        bio.take_messages();
        assert_eq!(bio.browser_route(), "hecatenag");
        assert!(bio.main_view()["DependencyNag"] == true);
        assert!(bio.browser_html().contains("Plugin Missing or Disabled"));
        assert!(bio.browser_html().contains("Enable Hecate"));
        bio.on_message(
            "BrowserNavigate",
            &json!({"Url":"observatory://bioinsights/gethecate"}),
        );
        let messages = bio.take_messages();
        assert!(messages.iter().any(|m| m["target"] == "Core"
            && m["command"] == "InstallNativePlugin"
            && m["payload"]["Plugin"] == "Hecate"));
        assert_eq!(bio.browser_route(), "hecatenag");
        bio.on_peer_message("Hecate", "Ready", &json!({"Version":"1.0"}));
        assert_eq!(bio.browser_route(), "main");
        let mut skipped = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        skipped.set_batch(true);
        skipped.set_batch(false);
        skipped.on_message(
            "BrowserNavigate",
            &json!({"Url":"observatory://bioinsights/skiphecate"}),
        );
        skipped.set_batch(true);
        skipped.set_batch(false);
        assert_eq!(skipped.browser_route(), "main");
    }
    #[test]
    fn embedded_pages_use_real_source_layout_and_in_place_routes() {
        let mut bio = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        bio.on_event(&event(
            json!({"event":"LoadGame","FID":"F123","Commander":"CascadianX"}),
        ));
        bio.on_event(&event(
            json!({"event":"Location","SystemAddress":42,"StarSystem":"Source"}),
        ));
        bio.on_event(&event(json!({"event":"Scan","SystemAddress":42,"BodyID":1,"BodyName":"Source A 1","PlanetClass":"Rocky body","Landable":true,"Radius":1000000,"Atmosphere":"thin carbon dioxide atmosphere","AtmosphereType":"CarbonDioxide","SurfaceTemperature":180,"SurfacePressure":1000,"SurfaceGravity":1})));
        bio.on_event(&event(json!({"event":"FSSBodySignals","SystemAddress":42,"BodyID":1,"Signals":[{"Type":"$SAA_SignalType_Biological;","Count":4}]})));
        bio.take_messages();
        let main = bio.browser_html();
        assert!(main.contains("colspan=11>CMDR CascadianX"));
        assert!(main.contains("Source A 1"));
        assert!(main.contains("<th colspan=2 class=\"border\">Variants</th>"));
        assert!(main.contains("observatory://bioinsights/about"));
        assert!(!main.contains("$$"));
        assert!(!main.contains("PossibleVariants</th>"));
        bio.on_message(
            "BrowserNavigate",
            &json!({"Url":"observatory://bioinsights/about"}),
        );
        let about = bio.browser_html();
        assert!(about.contains("Symbol Legend"));
        assert!(about.contains("CMDR Marxanthius"));
        assert!(
            bio.take_messages().is_empty(),
            "ordinary navigation cannot create a host window"
        );
        bio.on_message(
            "BrowserNavigate",
            &json!({"Url":"observatory://bioinsights/codexcompanion/2/11"}),
        );
        let codex = bio.browser_html();
        assert!(codex.contains("<th class=\"cc left\" colspan=4>"));
        assert!(!codex.contains("Odyssey Genuses"));
        assert!(codex.contains("Requirements"));
        assert!(codex.contains("Bacterium Alcyoneum"));
        assert!(codex.contains("<canvas id=\"regionmapcanvas\""));
        assert!(codex.contains("codexcompanion/manifest.json"));
        bio.on_message(
            "BrowserNavigate",
            &json!({"Url":"observatory://bioinsights/ccmark/2/11/8"}),
        );
        assert!(bio.is_marked("Codex_Ent_Bacterial_06_Name", "Green"));
        assert!(bio.browser_html().contains("Bacterium Alcyoneum"));
        let route = bio.browser_route();
        bio.on_message(
            "BrowserNavigate",
            &json!({"Url":"observatory://bioinsights/ccshow/2/11/8"}),
        );
        assert_eq!(
            bio.browser_route(),
            route,
            "source map selection updates elements without document navigation"
        );
        assert!(bio
            .browser_html()
            .contains("Bacterium Alcyoneum - Green</span>"));
        bio.on_message(
            "BrowserNavigate",
            &json!({"Url":"observatory://bioinsights/settings"}),
        );
        assert!(bio.take_messages().iter().any(|m| m["target"] == "Core"
            && m["command"] == "BioInsightsUiAction"
            && m["payload"]["Action"] == "Settings"));
    }
    #[test]
    fn absent_species_inherit_callers_original_italic_class_in_every_cell() {
        let bio = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        let row = json!({"PredictionType":"ParentStar","Seen":"NotPresent","SampleState":"No","Genus":"Frutexa","Species":"Metallicum","ValueText":"1.63M","Stripe":1,"Distance":150,"Variants":[]});
        let html = bio.source_row(&row, 0);
        assert!(html.contains("class=\"row2 NotPresentItalic\""));
        assert!(html.contains("class=\"species row2 NotPresentItalic\""));
        assert!(html.contains("<span class=\"Cross\"></span>"));
        assert!(!html.contains("class=\"row2\""));
    }
    #[test]
    fn web_mode_preserves_source_theme_override_and_uses_hecate_socket() {
        let mut bio=BioInsights::new(json!({"GalacticCodexRemote":false,"ThemeSettings":{"ThemeIndex":1,"ThemeMode":1,"Colors":{"DefaultText":{"R":1,"G":2,"B":3}}}})).unwrap();
        bio.on_message(
            "SetServerAddress",
            &json!({"ServerAddress":"127.0.0.1","ServerPort":"58080"}),
        );
        let document = bio.source_document(&bio.source_main(), false);
        assert!(document.contains("color: #010203"));
        assert!(document.contains("const usews = 1"));
        assert!(document.contains("ws://127.0.0.1:58080/bioinsights"));
        assert!(document.contains("id=\"settings\" title=\"Settings\" style=\"display:none\""));
        assert!(!document.contains("$$"));
        assert_eq!(
            bio.symbol("BiologicalLogged", None),
            "<span class=\"BiologicalLogged\"></span>"
        );
        assert_eq!(bio.symbol("RB", None), "<span class=\"RB\"></span>");
        assert!(document.contains(".BiologicalLogged {"));
        assert!(document.contains("background-color: #A8A8A8;"));
    }
}
