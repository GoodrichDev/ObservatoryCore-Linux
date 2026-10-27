//! Original Theme.generateSymbolCSS contract; source captures cover both
//! installed themes, row-height changes and optional symbol font/padding fields.
use base64::Engine as _;
use serde_json::Value;

fn color(value: &Value) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        value["R"].as_u64().unwrap_or(0),
        value["G"].as_u64().unwrap_or(0),
        value["B"].as_u64().unwrap_or(0)
    )
}
fn brush(theme: &Value, overrides: Option<&Value>, symbol: &Value, field: &str) -> Option<String> {
    if symbol[field].is_object() {
        return Some(color(&symbol[field]));
    }
    let name = symbol[format!("{field}Key")].as_str()?;
    Some(color(
        overrides
            .and_then(|o| o.get(name))
            .unwrap_or(&theme["Colors"][name]),
    ))
}
pub(super) fn stylesheet(theme: &Value, overrides: Option<&Value>) -> String {
    let mut labels = vec![".TEMPLATE_LABEL".to_owned()];
    let mut hovers = vec![".TEMPLATE_HOVER_AFTER:hover:after".to_owned()];
    let mut rules = String::new();
    for (name, symbol) in theme["Symbols"].as_object().into_iter().flatten() {
        if symbol["IsLabel"] == true {
            labels.push(format!(".{name}"));
        }
        let hover = symbol["SupressMouseOver"] != true;
        if hover {
            hovers.push(format!(".{name}:hover:after"));
        }
        rules.push_str(&format!(".{name} {{\r\n"));
        for (field, property) in [
            ("ContentColor", "color"),
            ("ContentBackColor", "background-color"),
        ] {
            if let Some(value) = brush(theme, overrides, symbol, field) {
                rules.push_str(&format!("{property}: {value};\r\n"));
            }
        }
        for (field, property) in [
            ("Font", "font-family"),
            ("FontWeight", "font-weight"),
            ("VerticalAlign", "vertical-align"),
        ] {
            if let Some(value) = symbol[field].as_str() {
                let value = if field == "Font"
                    && !((value.starts_with('\'') && value.ends_with('\''))
                        || (value.starts_with('"') && value.ends_with('"')))
                {
                    format!("'{value}'")
                } else {
                    value.to_owned()
                };
                rules.push_str(&format!("{property}: {value};\r\n"));
            }
        }
        for (field, property) in [
            ("FontSize", "font-size"),
            ("PaddingLeft", "padding-left"),
            ("PaddingRight", "padding-right"),
            ("PaddingTop", "padding-top"),
            ("PaddingBottom", "padding-bottom"),
            ("Width", "width"),
        ] {
            if let Some(value) = symbol[field].as_i64() {
                rules.push_str(&format!("{property}: {value}px;\r\n"));
            }
        }
        rules.push_str("}\r\n\r\n");
        let content = symbol["Content"].as_str().unwrap_or("");
        let kind = symbol["ContentType"].as_str().unwrap_or("None");
        if kind != "None" && kind != "Unknown" {
            rules.push_str(&format!(".{name}:before {{\r\n"));
            if kind == "Text" {
                rules.push_str(&format!("content: \"{content}\";\r\n"));
            } else {
                let url = match kind {
                    "Base64SVG" => {
                        let bytes = base64::engine::general_purpose::STANDARD
                            .decode(content)
                            .unwrap_or_default();
                        let svg = String::from_utf8_lossy(&bytes).replace(
                            "$$COLOR$$",
                            brush(theme, overrides, symbol, "ContentColor")
                                .as_deref()
                                .unwrap_or("#000000")
                                .trim_start_matches('#'),
                        );
                        format!(
                            "data:image/svg+xml;base64,{}",
                            base64::engine::general_purpose::STANDARD.encode(svg)
                        )
                    }
                    "Base64PNG" => format!("data:image/png;base64,{content}"),
                    _ => content
                        .replace('\\', "/")
                        .replace(' ', "%20")
                        .replace('\'', "%27"),
                };
                let size = theme["Settings"]["RowHeight"].as_i64().unwrap_or(20) - 1;
                rules.push_str(&format!("background-image: url(\"{url}\");\r\nheight: {size}px;\r\nwidth: {size}px;\r\ndisplay: inline-block;\r\nbackground-position: center;\r\nbackground-repeat: no-repeat;\r\n"));
                if kind == "Base64PNG" {
                    rules.push_str("padding-left: 1px;\r\npadding-right: 1px;\r\n");
                }
                rules.push_str("vertical-align: bottom;\r\ncontent: \" \";\r\n");
            }
            rules.push_str("}\r\n\r\n");
        }
        rules.push_str(&format!(".{name}:hover:after {{\r\n"));
        if hover {
            rules.push_str(&format!(
                "content: \"{}\";\r\n",
                symbol["Description"].as_str().unwrap_or("")
            ));
        }
        rules.push_str("}\r\n\r\n");
    }
    format!(
        "{}\r\n/* Native source symbol rules */\r\n{rules}",
        include_str!("symbols-template.css")
            .replace("$$TEMPLATE_LABEL$$", &labels.join(",\r\n"))
            .replace("$$TEMPLATE_HOVER_AFTER$$", &hovers.join(",\r\n"))
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;
    fn rules(css: &str) -> BTreeMap<String, BTreeMap<String, String>> {
        // Generated rules contain no nested selectors. Split CRLF declarations,
        // preserving semicolons inside base64 data URLs and quoted descriptions.
        let mut result = BTreeMap::new();
        let tail = css
            .split_once("/* Native source symbol rules */")
            .map(|(_, tail)| tail)
            .unwrap_or_else(|| {
                css.find(".PersonalCodexEntry {")
                    .map(|i| &css[i..])
                    .unwrap_or(css)
            });
        for rule in tail.split("}\r\n") {
            let Some((selector, body)) = rule.split_once('{') else {
                continue;
            };
            let properties = body
                .trim()
                .split(";\r\n")
                .filter_map(|line| line.trim().trim_end_matches(';').split_once(':'))
                .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
                .collect();
            result.insert(selector.trim().to_owned(), properties);
        }
        result
    }
    #[test]
    fn generated_symbol_properties_match_twelve_original_csharp_theme_captures() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../../../fixtures/bioinsights-symbol-css-oracle.json"
        ))
        .unwrap();
        for case in oracle["cases"].as_array().unwrap() {
            let mut theme: Value = serde_json::from_str(if case["theme"] == "theme.json" {
                include_str!("theme.json")
            } else {
                include_str!("theme-orange.json")
            })
            .unwrap();
            theme["Settings"]["RowHeight"] = case["height"].clone();
            if case["custom"] == true {
                let symbol = &mut theme["Symbols"]["RB"];
                for(key,value)in json!({"Font":"Arial","FontSize":19,"FontWeight":"normal","PaddingLeft":3,"PaddingRight":7,"PaddingTop":2,"PaddingBottom":4,"Width":27,"VerticalAlign":"middle","SupressMouseOver":true}).as_object().unwrap(){symbol[key]=value.clone();}
            }
            let actual = stylesheet(&theme, None);
            for marker in [".TEMPLATE_LABEL", ".TEMPLATE_HOVER_AFTER:hover:after"] {
                let selectors = |css: &str| {
                    let start = css.find(marker).unwrap();
                    css[start..]
                        .split_once('{')
                        .unwrap()
                        .0
                        .split(',')
                        .map(str::trim)
                        .map(str::to_owned)
                        .collect::<std::collections::BTreeSet<_>>()
                };
                assert_eq!(
                    selectors(&actual),
                    selectors(case["css"].as_str().unwrap()),
                    "{marker} {case}"
                );
            }
            let expected = rules(case["css"].as_str().unwrap());
            let generated = rules(&actual);
            for (key, value) in &expected {
                assert_eq!(
                    generated.get(key),
                    Some(value),
                    "{} height{} custom{} {key}",
                    case["theme"],
                    case["height"],
                    case["custom"]
                );
            }
            assert!(actual.contains("font-family: Calibri;"));
            assert!(actual.contains("padding-left: 10px;"));
        }
    }
}
