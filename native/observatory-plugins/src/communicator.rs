//! Sent/received journal communications, channel filters and alert routing.
use observatory_domain::{Finding, JournalEvent};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub fn default_settings() -> Value {
    json!({
        "NotifyPrivate":true,"NotifyWing":true,"NotifySquadron":true,
        "NotifySystem":true,"NotifyLocal":true,"NotifyNPC":false,
        "SuppressVoice":false,"ShowPrivate":true,"ShowWing":true,
        "ShowSquadron":true,"ShowSystem":true,"ShowLocal":true,"ShowNPC":false
    })
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MessageRow {
    pub timestamp: String,
    pub channel: String,
    pub from: String,
    pub message: String,
    pub sent: bool,
}

pub struct Communicator {
    settings: Value,
    rows: Vec<MessageRow>,
    commander: Option<String>,
    system: String,
    hidden_rows: std::collections::HashSet<usize>,
}

impl Communicator {
    pub fn new(settings: Value) -> Self {
        Self {
            settings,
            rows: Vec::new(),
            commander: None,
            system: String::new(),
            hidden_rows: Default::default(),
        }
    }

    pub fn reset(&mut self) {
        self.rows.clear();
        self.commander = None;
        self.system.clear();
        self.hidden_rows.clear();
    }

    pub fn settings(&self) -> &Value {
        &self.settings
    }

    pub fn set_settings(&mut self, settings: Value) {
        if ["Private", "Local", "System", "Wing", "Squadron", "NPC"]
            .iter()
            .any(|channel| {
                let key = format!("Show{channel}");
                self.settings[&key] != settings[&key]
            })
        {
            self.hidden_rows.clear();
        }
        self.settings = settings;
    }
    pub fn clear_visible_rows(&mut self) {
        self.hidden_rows.extend(0..self.rows.len());
    }

    // The source worker still raises plugin notifications during a batch read.
    // Native popup/audio suppression belongs to Core, after plugin dispatch.
    pub fn set_batch(&mut self, _batch: bool) {}

    pub fn suppress_voice(&self) -> bool {
        self.setting("SuppressVoice", false)
    }

    pub fn notification_rendering(&self) -> u8 {
        if self.suppress_voice() {
            5
        } else {
            7
        }
    }

    /// All messages remain available so changing a channel filter is lossless.
    pub fn all_rows(&self) -> &[MessageRow] {
        &self.rows
    }

    pub fn rows(&self) -> Vec<&MessageRow> {
        self.rows
            .iter()
            .enumerate()
            .filter(|(index, row)| {
                !self.hidden_rows.contains(index) && self.channel_shown(&row.channel)
            })
            .map(|(_, row)| row)
            .collect()
    }

    pub fn set_channel_shown(&mut self, channel: &str, shown: bool) {
        self.hidden_rows.clear();
        let key = format!("Show{}", channel_key(channel));
        if !self.settings.is_object() {
            self.settings = default_settings();
        }
        self.settings[key] = json!(shown);
    }

    pub fn channel_shown(&self, channel: &str) -> bool {
        let key = channel_key(channel);
        self.setting(&format!("Show{key}"), key != "NPC")
    }

    pub fn channel_notified(&self, channel: &str) -> bool {
        let key = channel_key(channel);
        self.setting(&format!("Notify{key}"), key != "NPC")
    }

    pub fn on_event(&mut self, event: &JournalEvent) -> Vec<Finding> {
        match event.event.as_str() {
            "LoadGame" => {
                self.commander = event
                    .data
                    .get("Commander")
                    .and_then(Value::as_str)
                    .map(str::to_owned);
            }
            "FSDJump" | "Location" | "CarrierJump" => {
                self.system = event.str("StarSystem").to_owned();
            }
            "SendText" | "ReceiveText" => {
                let sent = event.event == "SendText";
                let channel = if sent {
                    localized(event, "To")
                } else {
                    event.str("Channel")
                };
                let from = if sent {
                    self.commander.as_deref().unwrap_or("You")
                } else {
                    localized(event, "From")
                };
                let message = if sent {
                    event.str("Message")
                } else {
                    localized(event, "Message")
                };
                let row = MessageRow {
                    timestamp: display_timestamp(&event.timestamp),
                    channel: display_channel(channel).to_owned(),
                    from: from.to_owned(),
                    message: message.to_owned(),
                    sent,
                };
                let findings = if !sent && self.channel_notified(&row.channel) {
                    vec![Finding {
                        plugin: "Communicator".into(),
                        timestamp: event.timestamp.clone(),
                        system: self.system.clone(),
                        body: String::new(),
                        title: row.from.clone(),
                        detail: row.message.clone(),
                    }]
                } else {
                    Vec::new()
                };
                self.rows.push(row);
                return findings;
            }
            _ => {}
        }
        Vec::new()
    }

    fn setting(&self, key: &str, default: bool) -> bool {
        self.settings
            .get(key)
            .and_then(Value::as_bool)
            .unwrap_or(default)
    }
}

/// Framework JournalBase parses only whole-second UTC journal timestamps and
/// displays the resulting local DateTime; invalid values become DateTime.MinValue.
pub fn display_timestamp(timestamp: &str) -> String {
    if timestamp.len() != 20 {
        return "0001-01-01 00:00:00".into();
    }
    chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%SZ")
        .map(|time| {
            time.and_utc()
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|_| "0001-01-01 00:00:00".into())
}

fn localized<'a>(event: &'a JournalEvent, key: &str) -> &'a str {
    let localized_key = format!("{key}_Localised");
    let value = event.data.get(&localized_key).and_then(Value::as_str);
    value.unwrap_or_else(|| event.str(key))
}

fn channel_key(channel: &str) -> &'static str {
    match channel.to_ascii_lowercase().as_str() {
        "local" => "Local",
        "system" | "starsystem" => "System",
        "wing" => "Wing",
        "squadron" => "Squadron",
        "npc" => "NPC",
        _ => "Private",
    }
}

fn display_channel(channel: &str) -> &str {
    match channel.to_ascii_lowercase().as_str() {
        "local" => "Local",
        "starsystem" => "System",
        "wing" => "Wing",
        "squadron" => "Squadron",
        "npc" => "NPC",
        _ => channel,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub(crate) fn localize_oracle_time(timestamp: &Value) -> Value {
        let text = timestamp.as_str().unwrap_or("");
        if text.is_empty() || text.starts_with("0001-") {
            return timestamp.clone();
        }
        json!(
            chrono::NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S")
                .unwrap()
                .and_utc()
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        )
    }
    #[test]
    fn installed_communicator_event_and_filter_oracle() {
        let cases: Vec<Value> =
            serde_json::from_str(include_str!("../../fixtures/communicator-oracle.json")).unwrap();
        for mut case in cases {
            for row in case["Expected"]["Rows"].as_array_mut().unwrap() {
                row["Timestamp"] = localize_oracle_time(&row["Timestamp"]);
            }
            let mut worker = Communicator::new(case["Settings"].clone());
            let mut calls = Vec::new();
            for step in case["Events"].as_array().unwrap() {
                if step["event"] == "Batch" {
                    worker.set_batch(step["Enabled"].as_bool().unwrap());
                    continue;
                }
                let input = JournalEvent {
                    raw_json: None,
                    event: step["event"].as_str().unwrap().into(),
                    timestamp: step["timestamp"].as_str().unwrap_or("").into(),
                    data: step.clone(),
                };
                for finding in worker.on_event(&input) {
                    calls.push(json!({"Title":finding.title,"Detail":finding.detail,"Rendering":worker.notification_rendering()}));
                }
            }
            let rows:Vec<Value>=worker.rows().iter().map(|r|json!({"Timestamp":r.timestamp,"Channel":r.channel,"From":r.from,"Message":r.message})).collect();
            assert_eq!(
                json!({"Rows":rows,"Calls":calls}),
                case["Expected"],
                "{}",
                case["Label"]
            );
        }
    }
    fn event(value: Value) -> JournalEvent {
        JournalEvent {
            raw_json: None,
            event: value["event"].as_str().unwrap().into(),
            timestamp: "2026-01-01T00:00:00Z".into(),
            data: value,
        }
    }

    #[test]
    fn outgoing_messages_use_commander_and_never_notify() {
        let mut comm = Communicator::new(default_settings());
        comm.on_event(&event(
            json!({"event":"LoadGame","Commander":"Test Commander"}),
        ));
        let findings = comm.on_event(&event(
            json!({"event":"SendText","To":"wing","Message":"Hello"}),
        ));
        assert!(findings.is_empty());
        assert_eq!(comm.rows()[0].channel, "Wing");
        assert_eq!(comm.rows()[0].from, "Test Commander");
        assert!(comm.rows()[0].sent);
    }

    #[test]
    fn channel_filters_can_restore_hidden_messages() {
        let mut comm = Communicator::new(default_settings());
        assert!(comm
            .on_event(&event(
                json!({"event":"ReceiveText","Channel":"npc","From":"Station","Message":"Welcome"})
            ))
            .is_empty());
        assert!(comm.rows().is_empty());
        assert_eq!(comm.all_rows().len(), 1);
        comm.set_channel_shown("NPC", true);
        assert_eq!(comm.rows().len(), 1);
        assert!(!comm.channel_notified("npc"));
    }

    #[test]
    fn received_localized_text_generates_plugin_notifications_even_during_batch() {
        let mut comm = Communicator::new(default_settings());
        let message = json!({"event":"ReceiveText","Channel":"starsystem","From":"$npc;", "From_Localised":"Pilot", "Message":"$message;","Message_Localised":"Hello"});
        let findings = comm.on_event(&event(message.clone()));
        assert_eq!(findings[0].title, "Pilot");
        assert_eq!(findings[0].detail, "Hello");
        assert_eq!(comm.rows()[0].channel, "System");
        comm.set_batch(true);
        assert_eq!(comm.on_event(&event(message)).len(), 1);
        assert_eq!(comm.all_rows().len(), 2);
    }

    #[test]
    fn unknown_recipients_are_private_and_settings_apply_independently() {
        let comm = Communicator::new(
            json!({"ShowPrivate":false,"NotifyPrivate":true,"SuppressVoice":true}),
        );
        assert!(!comm.channel_shown("Another Commander"));
        assert!(comm.channel_notified("Another Commander"));
        assert!(comm.suppress_voice());
    }

    #[test]
    fn every_channel_honors_its_own_settings() {
        let mut settings = default_settings();
        for (channel, key) in [
            ("private", "Private"),
            ("wing", "Wing"),
            ("squadron", "Squadron"),
            ("starsystem", "System"),
            ("local", "Local"),
            ("npc", "NPC"),
        ] {
            settings[format!("Show{key}")] = json!(false);
            settings[format!("Notify{key}")] = json!(false);
            let mut comm = Communicator::new(settings.clone());
            assert!(!comm.channel_shown(channel));
            assert!(!comm.channel_notified(channel));
            assert!(comm
                .on_event(&event(
                    json!({"event":"ReceiveText","Channel":channel,"From":"Test","Message":"Hello"})
                ))
                .is_empty());
            assert!(comm.rows().is_empty());
            comm.set_channel_shown(channel, true);
            assert_eq!(comm.rows().len(), 1);
        }
    }
}
