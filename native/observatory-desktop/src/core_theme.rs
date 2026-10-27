//! Observatory .eot files retain their complete control/property colour map.
use anyhow::{ensure, Context, Result};
use eframe::egui::{self, Color32};
use observatory_io::Settings;
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::Path, sync::OnceLock};
pub fn parse(source: &str) -> Result<Value> {
    // System.Text.Json's original AllowTrailingCommas accepts commas immediately
    // preceding closing containers, including mixed indentation and strings.
    let bytes = source.trim_start_matches('\u{feff}').as_bytes();
    let mut result = Vec::with_capacity(bytes.len());
    let (mut quoted, mut escaped) = (false, false);
    for (i, &b) in bytes.iter().enumerate() {
        if quoted {
            result.push(b);
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                quoted = false;
            }
            continue;
        }
        if b == b'"' {
            quoted = true;
        }
        if b == b','
            && bytes[i + 1..]
                .iter()
                .find(|c| !c.is_ascii_whitespace())
                .is_some_and(|b| *b == b'}' || *b == b']')
        {
            continue;
        }
        result.push(b);
    }
    Ok(serde_json::from_slice(&result)?)
}
fn decode(value: &Value) -> Result<Color32> {
    if let Some(name) = value["Name"].as_str() {
        static COLORS: OnceLock<BTreeMap<String, [u8; 4]>> = OnceLock::new();
        let colors = COLORS.get_or_init(|| {
            serde_json::from_str(include_str!("../../fixtures/core-known-colours.json")).unwrap()
        });
        let rgba = colors
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, v)| *v)
            .unwrap_or([0; 4]);
        return Ok(Color32::from_rgba_unmultiplied(
            rgba[0], rgba[1], rgba[2], rgba[3],
        ));
    }
    let part = |key: &str| -> Result<u8> {
        let n = value.get(key).and_then(Value::as_i64).unwrap_or(0);
        ensure!((0..=255).contains(&n), "Colour {key} must be in 0..255");
        Ok(n as u8)
    };
    Ok(Color32::from_rgb(part("R")?, part("G")?, part("B")?))
}
pub fn import(path: &Path, s: &mut Settings) -> Result<()> {
    ensure!(
        std::fs::metadata(path)?.len() <= 4 * 1024 * 1024,
        "Theme exceeds 4 MiB"
    );
    let theme = parse(&std::fs::read_to_string(path)?)?;
    add(&theme, s)
}
pub fn add(theme: &Value, s: &mut Settings) -> Result<()> {
    let name = theme["Name"].as_str().context("Missing theme name")?;
    let colours = theme["Theme"]
        .as_object()
        .context("Missing theme content")?;
    for value in colours.values() {
        decode(value)?;
    }
    let themes = s.extra.entry("source_themes").or_insert(json!({}));
    ensure!(themes.is_object(), "source_themes must be an object");
    themes[name] = theme["Theme"].clone();
    Ok(())
}
pub fn names(s: &Settings) -> Vec<String> {
    s.extra
        .get("source_themes")
        .and_then(Value::as_object)
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}
/// IObservatoryCore.GetCurrentThemeDetails, including every source control key.
pub fn details(s: &Settings) -> Value {
    let defaults: Value =
        serde_json::from_str(include_str!("../../fixtures/core-default-themes.json"))
            .expect("source default themes");
    let source = s
        .extra
        .get("source_themes")
        .and_then(|v| v.get(&s.theme))
        .or_else(|| {
            defaults.get(if s.theme.eq_ignore_ascii_case("light") {
                "Light"
            } else {
                "Dark"
            })
        });
    let known: BTreeMap<String, [u8; 4]> =
        serde_json::from_str(include_str!("../../fixtures/core-known-colours.json"))
            .expect("source named colors");
    let mut result = serde_json::Map::new();
    for (key, value) in source.and_then(Value::as_object).into_iter().flatten() {
        let (rgba, name) = if let Some(name) = value["Name"].as_str() {
            let found = known.iter().find(|(key, _)| key.eq_ignore_ascii_case(name));
            (
                found.map(|(_, rgba)| *rgba).unwrap_or([0; 4]),
                found
                    .map(|(name, _)| name.clone())
                    .unwrap_or_else(|| name.to_owned()),
            )
        } else {
            let rgba = [
                value["R"].as_u64().unwrap_or(0).min(255) as u8,
                value["G"].as_u64().unwrap_or(0).min(255) as u8,
                value["B"].as_u64().unwrap_or(0).min(255) as u8,
                255,
            ];
            (
                rgba,
                format!("ff{:02x}{:02x}{:02x}", rgba[0], rgba[1], rgba[2]),
            )
        };
        result.insert(
            key.clone(),
            json!({"R":rgba[0],"G":rgba[1],"B":rgba[2],"A":rgba[3],"Name":name}),
        );
    }
    Value::Object(result)
}
pub fn colour(s: &Settings, control: &str, property: &str) -> Option<Color32> {
    let theme = s.extra.get("source_themes")?.get(&s.theme)?;
    let value = theme
        .get(format!("{control}.{property}"))
        .or_else(|| theme.get(format!("Default.{property}")))?;
    decode(value).ok()
}
pub fn apply(ctx: &egui::Context, s: &Settings) {
    if !names(s).contains(&s.theme) {
        return;
    }
    let mut v = ctx.style().visuals.clone();
    if let Some(c) = colour(s, "Form", "ForeColor") {
        v.override_text_color = Some(c);
        for w in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            w.fg_stroke.color = c;
        }
    }
    if let Some(c) = colour(s, "Form", "BackColor") {
        v.panel_fill = c;
        v.window_fill = c;
        v.widgets.noninteractive.bg_fill = c;
    }
    if let Some(c) = colour(s, "Button", "BackColor") {
        v.widgets.inactive.bg_fill = c;
        v.widgets.inactive.weak_bg_fill = c;
        v.widgets.hovered.bg_fill = c;
        v.widgets.active.bg_fill = c;
    }
    if let Some(c) = colour(s, "Button", "ForeColor") {
        v.widgets.inactive.fg_stroke.color = c;
        v.widgets.hovered.fg_stroke.color = c;
        v.widgets.active.fg_stroke.color = c;
    }
    if let Some(c) = colour(s, "TextBox", "BackColor") {
        v.extreme_bg_color = c;
    }
    if let Some(c) = colour(s, "ListView", "BackColor") {
        v.extreme_bg_color = c;
        v.faint_bg_color = c;
    }
    if let Some(c) = colour(s, "DataGridView", "GridColor") {
        v.widgets.noninteractive.bg_stroke.color = c;
    }
    if let Some(c) = colour(s, "LinkLabel", "LinkColor") {
        v.hyperlink_color = c;
    }
    ctx.set_visuals(v);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_theme_accepts_trailing_commas_rgb_named_and_type_fallback() {
        let theme=parse(r#"{"Name":"Survey","Theme":{"Default.ForeColor":{"Name":"Orange"},"Default.BackColor":{"R":1,"G":2,"B":3,},"Button.ForeColor":{"Name":"Black"},"FutureControl.UnknownColor":{"Name":"AliceBlue"},},}"#).unwrap();
        let mut s = Settings::default();
        add(&theme, &mut s).unwrap();
        s.theme = "Survey".into();
        assert_eq!(colour(&s, "Button", "ForeColor"), Some(Color32::BLACK));
        assert_eq!(
            colour(&s, "Label", "ForeColor"),
            Some(Color32::from_rgb(255, 165, 0))
        );
        assert_eq!(
            colour(&s, "Panel", "BackColor"),
            Some(Color32::from_rgb(1, 2, 3))
        );
        assert_eq!(
            s.extra["source_themes"]["Survey"]["FutureControl.UnknownColor"]["Name"],
            "AliceBlue"
        );
    }
}
