//! Explicit, non-destructive import of original Observatory settings formats.
use anyhow::{bail, ensure, Context, Result};
use observatory_io::Settings;
use serde_json::{json, Map, Value};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

const MAX_SETTINGS_BYTES: u64 = 4 * 1024 * 1024;
const BUILTINS: [(&str, &str); 7] = [
    ("Explorer", "b11e8725-3f71-4445-9b09-682f9eab8b59"),
    ("Botanist", "116d68f6-1c21-49ad-ae33-f0fef619af24"),
    ("Communicator", "581b2f03-1c16-4f02-9e20-e3bef5e9518b"),
    ("BioInsights", "4d617474-4701-4249-4f49-4e5349474854"),
    ("Evaluator", "4d617474-4702-0045-5641-4c5541544f52"),
    ("Hecate", "4d617474-4707-0000-0000-484543415445"),
    ("StellarOverlay", "4d617474-4703-5354-454c-4c41524f5645"),
];

/// Import only preferences that exist in the source. Existing native settings,
/// unrelated plugin configuration and unknown fields remain intact. This does
/// not save either file; the caller commits the returned settings atomically.
pub fn import(path: &Path, mut current: Settings) -> Result<Settings> {
    let imported = source_properties(path)?;
    ensure!(
        current.plugins.is_object(),
        "Current native plugin settings must be an object"
    );
    let mut notes = Vec::new();
    if let Some(folder) = string(&imported, "JournalFolder")? {
        if folder.is_empty() {
            current.journal_dir = None;
        } else if let Some(folder) = native_path(folder, path)? {
            current.journal_dir = Some(folder);
        } else {
            notes.push("The original Windows journal path was retained in legacy_settings. Select its Linux path in Settings.");
        }
    }
    for (legacy, field) in [
        ("NativeNotify", &mut current.popup),
        ("VoiceNotify", &mut current.speech),
        ("StartMonitor", &mut current.auto_start),
        ("StartReadAll", &mut current.start_read_all),
    ] {
        if let Some(value) = boolean(&imported, legacy)? {
            *field = value;
        }
    }
    if let Some(volume) = integer(&imported, "VoiceVolume")? {
        current
            .extra
            .insert("speech_volume".into(), json!(volume.clamp(0, 100)));
    }
    if let Some(rate) = integer(&imported, "VoiceRate")? {
        // SAPI exposes -10..10, whereas eSpeak expects words per minute. Preserve
        // the original rate below, and explain this explicit approximation.
        let rate = rate.clamp(-10, 10) as f64;
        let words_per_minute = (175.0 * 2_f64.powf(rate / 10.0)).round() as u64;
        current
            .extra
            .insert("speech_rate".into(), json!(words_per_minute));
        notes.push("SAPI VoiceRate was translated approximately to eSpeak words per minute; its original value is retained in legacy_settings.");
    }
    if let Some(voice) = string(&imported, "VoiceSelected")? {
        if voice.is_empty() || native_voice_identifier(voice) {
            current.extra.insert("speech_voice".into(), json!(voice));
        } else {
            notes.push("The original Windows voice name was retained in legacy_settings. Choose an installed eSpeak voice in Settings.");
        }
    }
    if let Some(theme) = string(&imported, "Theme")? {
        match theme.to_ascii_lowercase().as_str() {
            "dark" => current.theme = "dark".into(),
            "light" => current.theme = "light".into(),
            "" => {}
            _ => {}
        }
    }
    if let Some(settings) = nested_object(&imported, "PluginSettings")? {
        for &(name, guid) in &BUILTINS {
            // Original PluginManager.LoadPluginSettings prefers GUID over full
            // name. Do not let an obsolete name-keyed copy override that value.
            let selected = settings
                .iter()
                .filter_map(|(key, value)| builtin_rank(key, name, guid).map(|rank| (rank, value)))
                .min_by_key(|(rank, _)| *rank);
            if let Some((_, value)) = selected {
                let object = value
                    .as_object()
                    .with_context(|| format!("Original {name} settings must be an object"))?;
                let mut object = object.clone();
                if name == "Explorer" {
                    if let Some(original_path) = object
                        .get("CustomCriteriaFile")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                    {
                        match native_path(&original_path, path)? {
                            Some(native) => {
                                object.insert("CustomCriteriaFile".into(), json!(native));
                            }
                            None => {
                                object.remove("CustomCriteriaFile");
                            }
                        }
                        let criteria = object
                            .get("CustomCriteriaFile")
                            .or_else(|| current.plugins[name].get("CustomCriteriaFile"))
                            .and_then(Value::as_str);
                        let enabled = object
                            .get("EnableCustomCriteria")
                            .or_else(|| current.plugins[name].get("EnableCustomCriteria"))
                            .and_then(Value::as_bool)
                            .unwrap_or(false);
                        if enabled
                            && !criteria.is_some_and(|file| {
                                Path::new(file).is_file() && File::open(file).is_ok()
                            })
                        {
                            object.insert("EnableCustomCriteria".into(), json!(false));
                            notes.push("Original Explorer custom criteria are retained in legacy_settings but disabled until their Lua file is available at a readable Linux path.");
                        }
                    }
                }
                let destination = current
                    .plugins
                    .as_object_mut()
                    .unwrap()
                    .entry(name)
                    .or_insert_with(|| json!({}));
                ensure!(
                    destination.is_object(),
                    "Current {name} settings must be an object"
                );
                merge(destination.as_object_mut().unwrap(), &object);
            }
        }
    }
    if let Some(enabled) = nested_object(&imported, "PluginsEnabled")? {
        let mut disabled = match current.extra.get("disabled_plugins") {
            Some(Value::Array(values)) => values.clone(),
            None => Vec::new(),
            Some(_) => bail!("Current disabled_plugins setting must be an array"),
        };
        for (key, enabled) in enabled {
            if let Some((name, _)) = BUILTINS
                .iter()
                .find(|(name, guid)| builtin_rank(&key, name, guid).is_some())
            {
                let enabled = value_bool(&enabled)
                    .with_context(|| format!("Invalid original enabled flag for {name}"))?;
                disabled.retain(|value| value.as_str() != Some(name));
                if !enabled {
                    disabled.push(json!(name));
                }
            }
        }
        current
            .extra
            .insert("disabled_plugins".into(), Value::Array(disabled));
    }
    apply_core_properties(&imported, &mut current, path)?;
    if let Some(theme) = string(&imported, "Theme")? {
        if !theme.is_empty()
            && !matches!(theme.to_ascii_lowercase().as_str(), "dark" | "light")
            && current.theme != theme
        {
            notes.push("The original custom theme was retained in legacy_settings; the current native theme remains selected.");
        }
    }

    // Keep every source property, including serialized plugin dictionaries and
    // WinForms-specific preferences. An import can be inspected/revisited later.
    let legacy = current
        .extra
        .entry("legacy_settings")
        .or_insert_with(|| json!({}));
    if !legacy.is_object() {
        bail!("Current legacy_settings value must be an object");
    }
    merge(legacy.as_object_mut().unwrap(), &imported);
    if !notes.is_empty() {
        let existing = current
            .extra
            .entry("legacy_import_notes")
            .or_insert_with(|| json!([]));
        ensure!(
            existing.is_array(),
            "Current legacy_import_notes value must be an array"
        );
        for note in notes {
            let note = json!(note);
            if !existing.as_array().unwrap().contains(&note) {
                existing.as_array_mut().unwrap().push(note);
            }
        }
    }
    Ok(current)
}

fn json_setting(source: &Map<String, Value>, key: &str) -> Result<Option<Value>> {
    let Some(value) = source.get(key).filter(|value| !value.is_null()) else {
        return Ok(None);
    };
    if let Some(text) = value.as_str() {
        if text.trim().is_empty() {
            return Ok(None);
        }
        return Ok(Some(
            crate::core_theme::parse(text)
                .with_context(|| format!("Invalid original {key} JSON"))?,
        ));
    }
    Ok(Some(value.clone()))
}
fn pair(value: &Value, first: &str, second: &str) -> Result<[i64; 2]> {
    if let Some(array) = value.as_array() {
        ensure!(array.len() == 2, "Original geometry needs two coordinates");
        return Ok([
            array[0].as_i64().context("Invalid first coordinate")?,
            array[1].as_i64().context("Invalid second coordinate")?,
        ]);
    }
    if value.is_object() {
        return Ok([
            value[first]
                .as_i64()
                .context("Invalid first geometry field")?,
            value[second]
                .as_i64()
                .context("Invalid second geometry field")?,
        ]);
    }
    let text = value
        .as_str()
        .context("Original geometry must be text, object or pair")?;
    if let Some((first, second)) = text.split_once(',') {
        return Ok([first.trim().parse()?, second.trim().parse()?]);
    }
    let wrapped = format!("<geometry>{text}</geometry>");
    let xml = roxmltree::Document::parse(&wrapped).context("Invalid original geometry XML")?;
    let get = |name: &str| -> Result<i64> {
        xml.descendants()
            .find(|node| node.has_tag_name(name))
            .and_then(|node| node.text())
            .with_context(|| format!("Missing geometry {name}"))?
            .parse()
            .with_context(|| format!("Invalid geometry {name}"))
    };
    Ok([get(first)?, get(second)?])
}
fn apply_core_properties(
    source: &Map<String, Value>,
    current: &mut Settings,
    path: &Path,
) -> Result<()> {
    for key in [
        "NativeNotify",
        "VoiceNotify",
        "StartMonitor",
        "StartReadAll",
        "TryPrimeSystemContextOnStartMonitor",
        "NativeNotifyTransparent",
        "ChimeEnabled",
        "AltMonitor",
    ] {
        if let Some(value) = boolean(source, key)? {
            current.extra.insert(key.into(), json!(value));
        }
    }
    for key in [
        "NativeNotifyColour",
        "NativeNotifyCorner",
        "NativeNotifyScreen",
        "VoiceVolume",
        "VoiceRate",
        "NativeNotifyScale",
        "NativeNotifyTimeout",
        "LastTabIndex",
        "ExportFormat",
        "NativeNotifyFontScale",
        "ChimeSelected",
    ] {
        if let Some(value) = integer(source, key)? {
            current.extra.insert(key.into(), json!(value));
        }
    }
    for key in [
        "NativeNotifyFont",
        "CoreVersion",
        "VoiceSelected",
        "AudioDevice",
    ] {
        if let Some(value) = string(source, key)? {
            current.extra.insert(key.into(), json!(value));
        }
    }
    if let Some(value) = source.get("AudioVolume").filter(|value| !value.is_null()) {
        let value = if let Some(text) = value.as_str() {
            text.trim().parse::<f64>()?
        } else {
            value
                .as_f64()
                .context("Original AudioVolume must be a number")?
        };
        ensure!(value.is_finite(), "Original AudioVolume is not finite");
        current.extra.insert("AudioVolume".into(), json!(value));
    }
    for (key, fields, native) in [
        ("MainWindowSize", ["Width", "Height"], "window_size"),
        ("MainWindowPosition", ["X", "Y"], "window_position"),
    ] {
        if let Some(value) = source.get(key).filter(|value| !value.is_null()) {
            let value = pair(value, fields[0], fields[1])?;
            current
                .extra
                .insert(key.into(), json!({fields[0]:value[0],fields[1]:value[1]}));
            current.extra.insert(native.into(), json!(value));
        }
    }
    if let Some(folder) = string(source, "ExportFolder")? {
        if let Some(native) = native_path(folder, path)? {
            current.extra.insert("ExportFolder".into(), json!(native));
        }
    }
    if let Some(themes) = json_setting(source, "SavedThemes")? {
        for theme in themes.as_array().context("SavedThemes must be an array")? {
            crate::core_theme::add(theme, current)?;
        }
        current.extra.insert("SavedThemes".into(), themes);
    }
    if let Some(theme) = string(source, "Theme")? {
        if crate::core_theme::names(current).contains(&theme.to_owned()) {
            current.theme = theme.into();
        }
    }
    if let Some(order) = json_setting(source, "TabOrder")? {
        let mut tabs = Vec::new();
        for name in order.as_array().context("TabOrder must be an array")? {
            let name = name.as_str().context("TabOrder entry must be a string")?;
            tabs.push(json!(plugin_name(name)));
        }
        current.extra.insert("TabOrder".into(), order);
        current.extra.insert("tab_order".into(), json!(tabs));
    }
    if let Some(sizing) = json_setting(source, "ColumnSizing")? {
        let mut columns = Map::new();
        let mut versions = Map::new();
        for row in sizing.as_array().context("ColumnSizing must be an array")? {
            let name = row["PluginName"]
                .as_str()
                .context("ColumnSizing missing PluginName")?;
            ensure!(
                row["ColumnWidth"].is_object(),
                "ColumnSizing widths must be an object"
            );
            columns.insert(plugin_name(name), row["ColumnWidth"].clone());
            versions.insert(
                plugin_name(name),
                json!(row["PluginVersion"].as_str().unwrap_or("")),
            );
        }
        current.extra.insert("ColumnSizing".into(), sizing);
        current.extra.insert("column_sizing".into(), json!(columns));
        current
            .extra
            .insert("column_sizing_versions".into(), json!(versions));
    }
    let mut windows = current
        .extra
        .get("plugin_windows")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    if let Some(locations) = json_setting(source, "PopoutLocation").unwrap_or(None) {
        if let Some(locations) = locations.as_object() {
            for (name, value) in locations {
                let name = plugin_name(name);
                let entry = windows.entry(name).or_insert(json!({}));
                if let (Some(x), Some(y)) = (value["X"].as_i64(), value["Y"].as_i64()) {
                    entry["position"] = json!([x, y]);
                }
                if let (Some(width), Some(height)) =
                    (value["Width"].as_i64(), value["Height"].as_i64())
                {
                    entry["size"] = json!([width, height]);
                }
            }
        }
        current.extra.insert("PopoutLocation".into(), locations);
    }
    if let Some(open) = json_setting(source, "OpenPopouts")? {
        for (name, window) in windows.iter_mut() {
            if BUILTINS
                .iter()
                .any(|(short, guid)| builtin_rank(name, short, guid).is_some())
            {
                window["open"] = json!(false);
            }
        }
        for name in open.as_array().context("OpenPopouts must be an array")? {
            let name = plugin_name(
                name.as_str()
                    .context("OpenPopouts entry must be a string")?,
            );
            windows.entry(name).or_insert(json!({}))["open"] = json!(true);
        }
        current.extra.insert("OpenPopouts".into(), open);
    }
    if !windows.is_empty() {
        current
            .extra
            .insert("plugin_windows".into(), json!(windows));
    }
    Ok(())
}
fn plugin_name(name: &str) -> String {
    let name = name.trim();
    if matches!(name, "Core" | "Settings") {
        return "Core".into();
    }
    BUILTINS
        .iter()
        .find(|(short, guid)| builtin_rank(name, short, guid).is_some())
        .map(|(short, _)| (*short).to_owned())
        .unwrap_or_else(|| name.to_owned())
}

pub(super) fn source_properties(path: &Path) -> Result<Map<String, Value>> {
    let source = read_source(path)?;
    let imported = if source.trim_start().starts_with('<') {
        parse_xml(&source)?
    } else {
        let value: Value = serde_json::from_str(source.trim_start_matches('\u{feff}'))
            .context("Invalid original Observatory JSON settings")?;
        value
            .as_object()
            .context("Original Observatory settings must be a JSON object")?
            .clone()
    };
    Ok(imported)
}

fn read_source(path: &Path) -> Result<String> {
    let mut bytes = Vec::new();
    File::open(path)
        .with_context(|| format!("Cannot open original settings {}", path.display()))?
        .take(MAX_SETTINGS_BYTES + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= MAX_SETTINGS_BYTES,
        "Original settings exceed 4 MiB"
    );
    if bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff]) {
        ensure!(
            bytes.len().is_multiple_of(2),
            "Original UTF-16 settings have an incomplete code unit"
        );
        let little = bytes.starts_with(&[0xff, 0xfe]);
        let units: Vec<u16> = bytes[2..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|unit| {
                if little {
                    u16::from_le_bytes(*unit)
                } else {
                    u16::from_be_bytes(*unit)
                }
            })
            .collect();
        String::from_utf16(&units).context("Original settings are not valid UTF-16")
    } else {
        String::from_utf8(bytes)
            .map(|text| text.trim_start_matches('\u{feff}').to_owned())
            .context("Original settings are not valid UTF-8")
    }
}

fn parse_xml(source: &str) -> Result<Map<String, Value>> {
    let document =
        roxmltree::Document::parse(source).context("Invalid original user.config XML")?;
    let section = document
        .descendants()
        .find(|node| {
            node.is_element()
                && matches!(
                    node.tag_name().name(),
                    "Observatory.Properties.Core" | "ObservatoryCore.Properties.Core"
                )
                && node
                    .parent()
                    .is_some_and(|parent| parent.has_tag_name("userSettings"))
        })
        .context("user.config has no Observatory.Properties.Core userSettings section")?;
    let mut result = Map::new();
    for setting in section
        .children()
        .filter(|node| node.has_tag_name("setting"))
    {
        let name = setting
            .attribute("name")
            .context("Original XML setting has no name")?;
        let value = setting
            .children()
            .find(|node| node.has_tag_name("value"))
            .with_context(|| format!("Original XML setting {name} has no value"))?;
        // Preserve non-scalar WinForms serialized values as their original XML.
        let content = if value.children().any(|node| node.is_element()) {
            value
                .children()
                .map(|node| &source[node.range()])
                .collect::<String>()
        } else {
            value
                .children()
                .filter_map(|node| node.text())
                .collect::<String>()
        };
        ensure!(
            !result.contains_key(name),
            "Duplicate original XML setting {name}"
        );
        result.insert(name.to_owned(), Value::String(content));
    }
    Ok(result)
}

fn string<'a>(source: &'a Map<String, Value>, key: &str) -> Result<Option<&'a str>> {
    match source.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        Some(_) => bail!("Original {key} setting must be a string"),
    }
}

fn value_bool(value: &Value) -> Result<bool> {
    match value {
        Value::Bool(value) => Ok(*value),
        Value::String(value) if value.trim().eq_ignore_ascii_case("true") => Ok(true),
        Value::String(value) if value.trim().eq_ignore_ascii_case("false") => Ok(false),
        _ => bail!("Expected a boolean"),
    }
}

fn boolean(source: &Map<String, Value>, key: &str) -> Result<Option<bool>> {
    source
        .get(key)
        .filter(|value| !value.is_null())
        .map(|value| value_bool(value).with_context(|| format!("Invalid original {key} setting")))
        .transpose()
}

fn integer(source: &Map<String, Value>, key: &str) -> Result<Option<i64>> {
    match source.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => value
            .trim()
            .parse::<i64>()
            .map(Some)
            .with_context(|| format!("Original {key} setting must be an integer")),
        Some(value) => value
            .as_i64()
            .map(Some)
            .with_context(|| format!("Original {key} setting must be an integer")),
    }
}

fn nested_object(source: &Map<String, Value>, key: &str) -> Result<Option<Map<String, Value>>> {
    let Some(value) = source.get(key).filter(|value| !value.is_null()) else {
        return Ok(None);
    };
    if value.as_str().is_some_and(|value| value.trim().is_empty()) {
        return Ok(None);
    }
    let value = if let Value::String(text) = value {
        serde_json::from_str::<Value>(text)
            .with_context(|| format!("Invalid original {key} JSON"))?
    } else {
        value.clone()
    };
    value
        .as_object()
        .cloned()
        .map(Some)
        .with_context(|| format!("Original {key} must contain a JSON object"))
}

fn builtin_rank(key: &str, name: &str, guid: &str) -> Option<u8> {
    let key = key.trim().trim_matches(['{', '}']);
    if key.eq_ignore_ascii_case(guid) {
        Some(0)
    } else if key.eq_ignore_ascii_case(&format!("Observatory {name}")) {
        Some(1)
    } else if key.eq_ignore_ascii_case(name)
        || (name == "Hecate" && key.eq_ignore_ascii_case("Hecate - Data Vault"))
        || (name == "StellarOverlay" && key.eq_ignore_ascii_case("Stellar Overlay"))
    {
        Some(2)
    } else {
        None
    }
}

fn merge(destination: &mut Map<String, Value>, imported: &Map<String, Value>) {
    for (key, value) in imported {
        if let (Some(Value::Object(existing)), Value::Object(imported)) =
            (destination.get_mut(key), value)
        {
            merge(existing, imported);
        } else {
            destination.insert(key.clone(), value.clone());
        }
    }
}

fn native_voice_identifier(value: &str) -> bool {
    let mut variant = value.split('+');
    let language = variant.next().unwrap_or("");
    if let Some(suffix) = variant.next() {
        if suffix.is_empty() || !suffix.chars().all(|ch| ch.is_ascii_alphanumeric()) {
            return false;
        }
    }
    if variant.next().is_some() {
        return false;
    }
    let mut parts = language.split('-');
    let base = parts.next().unwrap_or("");
    (2..=3).contains(&base.len())
        && base.chars().all(|ch| ch.is_ascii_lowercase())
        && parts.all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_alphanumeric()))
}

pub(super) fn native_path(value: &str, source: &Path) -> Result<Option<PathBuf>> {
    let bytes = value.as_bytes();
    if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'\\' | b'/')
    {
        let rest = value[3..].replace('\\', "/");
        if bytes[0].eq_ignore_ascii_case(&b'z') {
            return Ok(Some(Path::new("/").join(rest)));
        }
        // If the settings still live inside their Wine prefix, drive mappings
        // remain unambiguous. A copied C:\\ path alone cannot identify a prefix.
        let source = if source.is_absolute() {
            source.to_owned()
        } else {
            std::env::current_dir()?.join(source)
        };
        if let Some(drive_c) = source
            .ancestors()
            .find(|path| path.file_name().is_some_and(|name| name == "drive_c"))
        {
            let drive = format!("drive_{}", (bytes[0] as char).to_ascii_lowercase());
            return Ok(Some(
                drive_c
                    .parent()
                    .context("Invalid Wine prefix")?
                    .join(drive)
                    .join(rest),
            ));
        }
        if cfg!(unix) && source.starts_with("/mnt") {
            let drive = format!("/mnt/{}", (bytes[0] as char).to_ascii_lowercase());
            return Ok(Some(Path::new(&drive).join(rest)));
        }
        return Ok(None);
    }
    if value.starts_with("\\\\") {
        return Ok(None);
    }
    let value = PathBuf::from(value);
    if value.is_absolute() {
        return Ok(Some(value));
    }
    let source = if source.is_absolute() {
        source.to_owned()
    } else {
        std::env::current_dir()?.join(source)
    };
    Ok(Some(source.parent().unwrap_or(Path::new(".")).join(value)))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn import_json(value: Value, current: Settings) -> Settings {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("Observatory.config");
        std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        import(&path, current).unwrap()
    }

    #[test]
    fn explicit_empty_original_popout_list_closes_known_native_popouts() {
        let folder = tempfile::tempdir().unwrap();
        let source = folder.path().join("settings.json");
        std::fs::write(&source, json!({"OpenPopouts":"[]"}).to_string()).unwrap();
        let mut current = Settings::default();
        current.extra.insert(
            "plugin_windows".into(),
            json!({"Evaluator":{"open":true,"size":[700,500]},"Other":{"open":true}}),
        );
        let imported = import(&source, current).unwrap();
        assert_eq!(imported.extra["plugin_windows"]["Evaluator"]["open"], false);
        assert_eq!(
            imported.extra["plugin_windows"]["Evaluator"]["size"],
            json!([700, 500])
        );
        assert_eq!(imported.extra["plugin_windows"]["Other"]["open"], true);
    }
    #[test]
    fn complete_original_preferences_are_typed_and_normalized_for_native_consumers() {
        let imported = import_json(
            json!({"NativeNotifyFont":"Segoe UI","NativeNotifyColour":"4294944000","NativeNotifyCorner":"7","NativeNotifyScreen":"1","NativeNotifyScale":"125","NativeNotifyFontScale":"110","NativeNotifyTimeout":"9000","NativeNotifyTransparent":"False","ChimeEnabled":"True","ChimeSelected":"3","AudioDevice":"Default Output","AudioVolume":"0.4","MainWindowSize":"1024, 768","MainWindowPosition":{"X":-1280,"Y":10},"LastTabIndex":"2","TryPrimeSystemContextOnStartMonitor":"False","AltMonitor":"True","ExportFormat":"1","ExportFolder":"exports","Theme":"Survey","SavedThemes":json!([{"Name":"Survey","Theme":{"Default.BackColor":{"R":1,"G":2,"B":3}}}]).to_string(),"TabOrder":json!(["Core","Observatory Explorer","Stellar Overlay"]).to_string(),"ColumnSizing":json!([{"PluginName":"Observatory Explorer","PluginVersion":"1.5.2","ColumnWidth":{"Body Name":321}}]).to_string(),"PopoutLocation":json!({"Observatory Explorer":{"X":-100,"Y":12,"Width":800,"Height":600}}).to_string(),"OpenPopouts":json!(["Observatory Explorer"]).to_string(),"FutureField":{"all":"retained"}}),
            Settings::default(),
        );
        assert_eq!(imported.extra["window_size"], json!([1024, 768]));
        assert_eq!(imported.extra["window_position"], json!([-1280, 10]));
        assert_eq!(
            imported.extra["MainWindowSize"],
            json!({"Width":1024,"Height":768})
        );
        assert_eq!(
            imported.extra["tab_order"],
            json!(["Core", "Explorer", "StellarOverlay"])
        );
        assert_eq!(
            imported.extra["column_sizing"]["Explorer"]["Body Name"],
            321
        );
        assert_eq!(
            imported.extra["plugin_windows"]["Explorer"],
            json!({"position":[-100,12],"size":[800,600],"open":true})
        );
        for (key, expected) in [
            ("NativeNotifyColour", 4294944000i64),
            ("NativeNotifyCorner", 7),
            ("NativeNotifyScale", 125),
            ("NativeNotifyFontScale", 110),
            ("NativeNotifyTimeout", 9000),
            ("ChimeSelected", 3),
            ("LastTabIndex", 2),
            ("ExportFormat", 1),
        ] {
            assert_eq!(imported.extra[key], expected, "{key}");
        }
        assert_eq!(imported.extra["NativeNotifyTransparent"], false);
        assert_eq!(imported.extra["ChimeEnabled"], true);
        assert_eq!(imported.extra["TryPrimeSystemContextOnStartMonitor"], false);
        assert_eq!(imported.extra["AltMonitor"], true);
        assert_eq!(imported.extra["AudioVolume"], 0.4);
        assert_eq!(imported.theme, "Survey");
        assert_eq!(
            imported.extra["source_themes"]["Survey"]["Default.BackColor"]["B"],
            3
        );
        assert_eq!(
            imported.extra["legacy_settings"]["FutureField"]["all"],
            "retained"
        );
        assert!(!imported.extra.contains_key("legacy_import_notes"));
    }
    #[test]
    fn portable_source_shape_maps_guid_plugins_flags_and_keeps_unmapped_preferences() {
        let mut current = Settings {
            poll_interval_ms: 777,
            ..Settings::default()
        };
        current.plugins = json!({"Explorer":{"KeepNative":true,"SmallObject":true},"MyNativePlugin":{"untouched":7}});
        current.extra.insert(
            "disabled_plugins".into(),
            json!(["Botanist", "MyNativePlugin"]),
        );
        current.extra.insert("native_extra".into(), json!(42));
        let imported = import_json(
            json!({
                "$id":"1", "JournalFolder":"Z:\\home\\pilot\\journals", "NativeNotify":false, "VoiceNotify":true,
                "VoiceVolume":65,"VoiceRate":0,"VoiceSelected":"Microsoft Zira Desktop", "StartMonitor":false,"StartReadAll":true,"Theme":"Light",
                "PluginSettings":serde_json::to_string(&json!({
                    "B11E8725-3F71-4445-9B09-682F9EAB8B59":{"SmallObject":false,"NewPreference":9},
                    "Observatory Explorer":{"SmallObject":true},
                    "116d68f6-1c21-49ad-ae33-f0fef619af24":{"OverlayIsSticky":false},
                    "Observatory Communicator":{"ShowNPC":true}, "Unknown CLR Plugin":{"old":true}
                })).unwrap(),
                "PluginsEnabled":"{\"Observatory Explorer\":false,\"Observatory Botanist\":true}",
                "PopoutLocation":"unmapped original location", "FuturePreference":{"keep":"all"}
            }),
            current,
        );
        assert_eq!(
            imported.journal_dir,
            Some(PathBuf::from("/home/pilot/journals"))
        );
        assert!(!imported.popup);
        assert!(imported.speech);
        assert!(!imported.auto_start);
        assert!(imported.start_read_all);
        assert_eq!(imported.theme, "light");
        assert_eq!(imported.poll_interval_ms, 777);
        assert_eq!(imported.extra["speech_rate"], 175);
        assert_eq!(imported.extra["speech_volume"], 65);
        assert!(!imported.extra.contains_key("speech_voice"));
        assert_eq!(imported.plugins["Explorer"]["SmallObject"], false);
        assert_eq!(imported.plugins["Explorer"]["KeepNative"], true);
        assert_eq!(imported.plugins["Botanist"]["OverlayIsSticky"], false);
        assert_eq!(imported.plugins["Communicator"]["ShowNPC"], true);
        assert_eq!(imported.plugins["MyNativePlugin"]["untouched"], 7);
        assert_eq!(
            imported.extra["disabled_plugins"],
            json!(["MyNativePlugin", "Explorer"])
        );
        assert_eq!(
            imported.extra["legacy_settings"]["FuturePreference"]["keep"],
            "all"
        );
        assert_eq!(
            imported.extra["legacy_settings"]["PopoutLocation"],
            "unmapped original location"
        );
        assert_eq!(imported.extra["native_extra"], 42);
        assert_eq!(
            imported.extra["legacy_import_notes"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn installed_xml_decodes_escaped_json_and_preserves_other_preferences() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("user.config");
        std::fs::write(&path, r#"<?xml version="1.0" encoding="utf-8"?><configuration><userSettings><Observatory.Properties.Core>
          <setting name="NativeNotify" serializeAs="String"><value>True</value></setting>
          <setting name="VoiceVolume" serializeAs="String"><value>40</value></setting>
          <setting name="PluginSettings" serializeAs="String"><value>{&quot;Observatory Botanist&quot;:{&quot;OverlayEnabled&quot;:false}}</value></setting>
          <setting name="PluginsEnabled" serializeAs="String"><value>{"Observatory Communicator":false}</value></setting>
          <setting name="NativeNotifyFont" serializeAs="String"><value>Example &amp; Font</value></setting>
        </Observatory.Properties.Core></userSettings></configuration>"#).unwrap();
        let imported = import(&path, Settings::default()).unwrap();
        assert!(imported.popup);
        assert_eq!(imported.extra["speech_volume"], 40);
        assert_eq!(imported.plugins["Botanist"]["OverlayEnabled"], false);
        assert_eq!(imported.extra["disabled_plugins"], json!(["Communicator"]));
        assert_eq!(
            imported.extra["legacy_settings"]["NativeNotifyFont"],
            "Example & Font"
        );
    }

    #[test]
    fn sparse_import_does_not_reset_current_settings_or_overwrite_native_voice() {
        let mut current = Settings {
            speech: true,
            auto_start: false,
            theme: "light".into(),
            ..Settings::default()
        };
        current.extra.insert("speech_voice".into(), json!("en-gb"));
        current
            .extra
            .insert("legacy_settings".into(), json!({"previous":true}));
        let imported = import_json(
            json!({"VoiceSelected":"Microsoft Voice", "Theme":"My Custom Theme","Unknown":5}),
            current,
        );
        assert!(imported.speech);
        assert!(!imported.auto_start);
        assert_eq!(imported.theme, "light");
        assert_eq!(imported.extra["speech_voice"], "en-gb");
        assert_eq!(imported.extra["legacy_settings"]["previous"], true);
        assert_eq!(imported.extra["legacy_settings"]["Unknown"], 5);
    }

    #[test]
    fn native_voice_codes_and_sapi_rate_are_explicitly_mapped() {
        let imported = import_json(
            json!({"VoiceSelected":"en-us+f3","VoiceRate":10,"VoiceVolume":1000}),
            Settings::default(),
        );
        assert_eq!(imported.extra["speech_voice"], "en-us+f3");
        assert_eq!(imported.extra["speech_rate"], 350);
        assert_eq!(imported.extra["speech_volume"], 100);
        assert!(!native_voice_identifier("Microsoft Zira Desktop"));
        assert!(!native_voice_identifier("Zira"));
    }

    #[test]
    fn invalid_json_and_nested_json_fail_without_modifying_source() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("Observatory.config");
        for text in [
            "{ damaged",
            "[]",
            r#"{"PluginSettings":"{damaged"}"#,
            r#"{"PluginsEnabled":"[]"}"#,
            r#"{"NativeNotify":12}"#,
        ] {
            std::fs::write(&path, text).unwrap();
            assert!(
                import(&path, Settings::default()).is_err(),
                "should reject {text}"
            );
            assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
        }
    }

    #[test]
    fn resolves_wine_drive_path_only_with_prefix_context() {
        let source = Path::new("/home/pilot/prefix/drive_c/Observatory/Observatory.config");
        assert_eq!(
            native_path("C:\\users\\steamuser\\Saved Games", source).unwrap(),
            Some(PathBuf::from(
                "/home/pilot/prefix/drive_c/users/steamuser/Saved Games"
            ))
        );
        assert!(native_path(
            "C:\\users\\pilot\\Saved Games",
            Path::new("/tmp/Observatory.config")
        )
        .unwrap()
        .is_none());
    }

    #[test]
    fn unavailable_legacy_custom_criteria_do_not_prevent_native_startup() {
        let imported = import_json(
            json!({"PluginSettings":{
                "Observatory Explorer":{"CustomCriteriaFile":"C:\\Missing\\ObservatoryCriteria.lua","EnableCustomCriteria":true}
            }}),
            Settings::default(),
        );
        assert_eq!(imported.plugins["Explorer"]["EnableCustomCriteria"], false);
        assert_eq!(
            imported.extra["legacy_settings"]["PluginSettings"]["Observatory Explorer"]
                ["EnableCustomCriteria"],
            true
        );
        assert!(imported.extra["legacy_import_notes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|note| note.as_str().unwrap().contains("custom criteria")));
    }

    #[test]
    fn xml_dtd_is_rejected_and_utf16_bom_is_supported() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("user.config");
        std::fs::write(
            &path,
            r#"<!DOCTYPE configuration [<!ENTITY x "expanded">]><configuration/>"#,
        )
        .unwrap();
        assert!(import(&path, Settings::default()).is_err());
        let source = r#"<?xml version="1.0" encoding="utf-16"?><configuration><userSettings><Observatory.Properties.Core><setting name="Theme"><value>Light</value></setting></Observatory.Properties.Core></userSettings></configuration>"#;
        let mut bytes = vec![0xff, 0xfe];
        for unit in source.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        std::fs::write(&path, bytes).unwrap();
        assert_eq!(import(&path, Settings::default()).unwrap().theme, "light");
    }
    #[test]
    fn installed_edjp_guids_preserve_settings_enable_flags_and_source_enums() {
        let mut imported = import_json(
            json!({"PluginSettings":{
            "4d617474-4701-4249-4f49-4e5349474854":{"OutputGridFull":true,"MinimumValue":12000000},
            "4d617474-4702-0045-5641-4c5541544f52":{"MinimumBodyValue":25000,"Notifications":{"DSSPersistent":{"Popup":false}}},
            "4d617474-4707-0000-0000-484543415445":{"ServerSettings":{"EnableWebServer":true}},
            "4d617474-4703-5354-454c-4c41524f5645":{"NotificationShape":0,"TextAlignment":2,"RadarSettings":{"Enabled":false}}
        },"PluginsEnabled":{"4d617474-4707-0000-0000-484543415445":false}}),
            Settings::default(),
        );
        crate::runtime::default_plugin_settings(&mut imported);
        assert_eq!(imported.plugins["BioInsights"]["MinimumValue"], 12000000);
        assert_eq!(imported.plugins["Evaluator"]["MinimumBodyValue"], 25000);
        assert_eq!(
            imported.plugins["Evaluator"]["Notifications"]["DSSPersistent"]["Popup"],
            false
        );
        assert_eq!(
            imported.plugins["Hecate"]["ServerSettings"]["EnableWebServer"],
            true
        );
        assert_eq!(imported.extra["disabled_plugins"], json!(["Hecate"]));
        assert_eq!(
            imported.plugins["StellarOverlay"]["NotificationShape"],
            "Closed"
        );
        assert_eq!(imported.plugins["StellarOverlay"]["TextAlignment"], "Right");
        assert_eq!(
            imported.plugins["StellarOverlay"]["RadarSettings"]["Enabled"],
            false
        );
        assert_eq!(
            imported.plugins["StellarOverlay"]["RadarSettings"]["PositionYPct"],
            50.0
        );
    }
}
