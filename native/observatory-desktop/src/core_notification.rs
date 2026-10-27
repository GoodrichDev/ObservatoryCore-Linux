//! NotificationArgs aliases and identity shared by every Core delivery channel.
use ring::rand::{SecureRandom, SystemRandom};
use serde_json::{json, Value};
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Delivery {
    pub notification: bool,
    pub visual: bool,
    pub audio: bool,
    pub update: bool,
    pub cancel: bool,
}
pub fn delivery(
    action: &str,
    rendering: u8,
    batch: bool,
    visual: bool,
    audio: bool,
    popup_override: bool,
    audio_override: bool,
) -> Delivery {
    match action {
        "cancel" => Delivery {
            visual: true,
            cancel: true,
            ..Default::default()
        },
        "update" if !batch => Delivery {
            notification: rendering & 4 != 0,
            visual: rendering & 1 != 0,
            audio: audio && rendering & 2 != 0,
            update: true,
            cancel: false,
        },
        "update" => Delivery::default(),
        _ => Delivery {
            notification: rendering & 4 != 0 || !batch && (popup_override || audio_override),
            visual: !batch && rendering & 1 != 0 && visual && !popup_override,
            audio: !batch && rendering & 2 != 0 && audio && !audio_override,
            ..Default::default()
        },
    }
}
pub fn normalize(mut value: Value) -> Value {
    if !value.is_object() {
        return value;
    }
    for (native, original) in [
        ("id", "Guid"),
        ("title", "Title"),
        ("detail", "Detail"),
        ("extended_details", "ExtendedDetails"),
        ("rendering", "Rendering"),
        ("timeout", "Timeout"),
        ("x", "XPos"),
        ("y", "YPos"),
        ("title_ssml", "TitleSsml"),
        ("detail_ssml", "DetailSsml"),
        ("coalescing_id", "CoalescingId"),
        ("plugin", "Sender"),
        ("plugin", "sender"),
        ("action", "Action"),
    ] {
        if value.get(native).is_none() {
            if let Some(original) = value.get(original).cloned() {
                value[native] = original;
            }
        }
    }
    if value["action"].as_str().unwrap_or("show") == "show"
        && value["id"].as_str().is_none_or(str::is_empty)
    {
        let mut bytes = [0u8; 16];
        if SystemRandom::new().fill(&mut bytes).is_err() {
            use std::sync::atomic::{AtomicU64, Ordering};
            static SEQUENCE: AtomicU64 = AtomicU64::new(1);
            let epoch = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                ^ u128::from(SEQUENCE.fetch_add(1, Ordering::Relaxed));
            bytes = epoch.to_be_bytes();
        }
        bytes[6] = (bytes[6] & 15) | 64;
        bytes[8] = (bytes[8] & 63) | 128;
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
        value["id"] = json!(format!(
            "{}-{}-{}-{}-{}",
            &hex[..8],
            &hex[8..12],
            &hex[12..16],
            &hex[16..20],
            &hex[20..]
        ));
    }
    value
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all768_original_core_send_update_cancel_channel_cases_match() {
        let cases: Vec<Value> = serde_json::from_str(include_str!(
            "../../fixtures/core-notification-routing-oracle.json"
        ))
        .unwrap();
        for c in cases {
            let action = c["Action"].as_str().unwrap();
            let d = delivery(
                action,
                c["Rendering"].as_u64().unwrap() as u8,
                c["Batch"].as_bool().unwrap(),
                c["Visual"].as_bool().unwrap(),
                c["Audio"].as_bool().unwrap(),
                c["PopupOverride"].as_bool().unwrap(),
                c["AudioOverride"].as_bool().unwrap(),
            );
            let mut actual = Vec::new();
            if d.notification {
                actual.push("notification-event")
            }
            if d.visual {
                actual.push(match action {
                    "update" => "visual-update",
                    "cancel" => "visual-cancel",
                    _ => "visual",
                })
            }
            if d.audio {
                actual.push("audio")
            }
            if d.update {
                actual.push("update-event")
            }
            if d.cancel {
                actual.push("cancel-event")
            }
            assert_eq!(json!(actual), c["Calls"], "{c}");
        }
    }
    #[test]
    fn channels_share_original_identity_and_ssml_without_replacing_explicit_empty_guid() {
        let value = normalize(
            json!({"Title":"Title","Detail":"Body","TitleSsml":"<speak>Title</speak>","DetailSsml":"<speak>Body</speak>","Rendering":3,"Timeout":0,"XPos":25,"YPos":50}),
        );
        assert_eq!(value["title"], "Title");
        assert_eq!(value["detail_ssml"], "<speak>Body</speak>");
        assert_eq!(value["rendering"], 3);
        assert_eq!(value["id"].as_str().unwrap().len(), 36);
        assert_eq!(normalize(value.clone())["id"], value["id"]);
        assert_eq!(
            normalize(json!({"Guid":"00000000-0000-0000-0000-000000000000"}))["id"],
            "00000000-0000-0000-0000-000000000000"
        );
        assert!(normalize(json!({"action":"update"}))["id"].is_null());
    }
}
