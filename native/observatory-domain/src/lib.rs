//! Lossless journal interchange shared by native workers and extensions.
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JournalEvent {
    pub event: String,
    pub timestamp: String,
    pub data: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_json: Option<std::sync::Arc<str>>,
}

impl JournalEvent {
    pub fn parse(line: &str) -> Result<Self> {
        let line = line.trim_start_matches('\u{feff}');
        let raw_json = Some(std::sync::Arc::from(line));
        // Frontier emitted a non-JSON infinity for some 2017 scan rotations.
        // Keep the same narrow legacy repair as the original JournalReader.
        let repaired;
        let line = if line.contains("\"RotationPeriod\":inf,") {
            repaired = line.replace("\"RotationPeriod\":inf,", "");
            repaired.as_str()
        } else {
            line
        };
        let data: Value = serde_json::from_str(line).context("invalid journal JSON")?;
        let event = data.get("event").and_then(Value::as_str).unwrap_or("");
        if event.is_empty() || !data.is_object() {
            bail!("journal entry has no event name");
        }
        Ok(Self {
            event: event.to_owned(),
            timestamp: data
                .get("timestamp")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
            data,
            raw_json,
        })
    }
    pub fn json(&self) -> std::borrow::Cow<'_, str> {
        self.raw_json
            .as_deref()
            .map(std::borrow::Cow::Borrowed)
            .unwrap_or_else(|| std::borrow::Cow::Owned(self.data.to_string()))
    }

    pub fn str(&self, key: &str) -> &str {
        self.data.get(key).and_then(Value::as_str).unwrap_or("")
    }
    pub fn number(&self, key: &str) -> f64 {
        self.data
            .get(key)
            .and_then(|value| {
                value.as_f64().or_else(|| match value.as_str()? {
                    "NaN" => Some(f64::NAN),
                    "Infinity" => Some(f64::INFINITY),
                    "-Infinity" => Some(f64::NEG_INFINITY),
                    _ => None,
                })
            })
            .unwrap_or(0.0)
    }
    pub fn boolean(&self, key: &str) -> bool {
        self.data.get(key).and_then(Value::as_bool).unwrap_or(false)
    }
    pub fn id(&self, key: &str) -> Option<u64> {
        self.data.get(key).and_then(Value::as_u64)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Finding {
    pub plugin: String,
    pub timestamp: String,
    pub system: String,
    pub body: String,
    pub title: String,
    pub detail: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_journal_json_survives_clone_serialization_and_legacy_envelopes() {
        let raw =
            " { \"Custom\": 2, \"event\" : \"Probe\", \"timestamp\":\"2026-10-25T00:00:00Z\" } ";
        let event = JournalEvent::parse(raw).unwrap();
        assert_eq!(event.json(), raw);
        assert_ne!(event.data.to_string(), raw);
        assert!(std::sync::Arc::ptr_eq(
            event.raw_json.as_ref().unwrap(),
            event.clone().raw_json.as_ref().unwrap()
        ));
        let roundtrip: JournalEvent =
            serde_json::from_value(serde_json::to_value(&event).unwrap()).unwrap();
        assert_eq!(roundtrip.json(), raw);
        let legacy: JournalEvent = serde_json::from_value(
            serde_json::json!({"event":"Probe","timestamp":"","data":{"event":"Probe"}}),
        )
        .unwrap();
        assert!(legacy.raw_json.is_none());
        assert_eq!(legacy.json(), "{\"event\":\"Probe\"}");
    }
    #[test]
    fn unknown_events_and_large_identifiers_round_trip() {
        let raw =
            r#"{"event":"FutureEvent","SystemAddress":18446744073709551615,"nested":{"new":true}}"#;
        let event = JournalEvent::parse(raw).unwrap();
        assert_eq!(event.id("SystemAddress"), Some(u64::MAX));
        assert_eq!(event.data["nested"]["new"], true);
        assert!(JournalEvent::parse("[]").is_err());
        assert!(JournalEvent::parse("{\"event\":\"\"}").is_err());
    }
    #[test]
    fn historical_frontier_infinite_rotation_is_readable() {
        let event =
            JournalEvent::parse(r#"{"event":"Scan","RotationPeriod":inf,"BodyName":"Old body"}"#)
                .unwrap();
        assert_eq!(event.str("BodyName"), "Old body");
        assert!(event.data.get("RotationPeriod").is_none());
        assert!(JournalEvent::parse(r#"{"event":"Scan","Other":inf}"#).is_err());
    }
}
