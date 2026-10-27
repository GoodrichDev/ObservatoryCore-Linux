//! The installed Utils body-name/SSML formatting and notification metadata.
use super::*;

pub(super) fn ssml(text: &str) -> String {
    format!("<speak version=\"1.0\" xmlns=\"http://www.w3.org/2001/10/synthesis\" xml:lang=\"en-US\"><voice name=\"\">{text}</voice></speak>")
}
pub(super) fn body_name(body: &str, system: &str, tts: bool, tag: bool) -> String {
    let text = if body.trim().eq_ignore_ascii_case(system.trim()) {
        "Main Star".to_owned()
    } else if body.to_lowercase().starts_with(&system.to_lowercase()) {
        let mut label = "Body";
        let mut output = String::new();
        for token in body[system.len()..].trim_matches(' ').split(' ') {
            if token.eq_ignore_ascii_case("belt") || token.eq_ignore_ascii_case("ring") {
                label = token;
            } else if tts {
                output.push_str(&format!(
                    " <say-as interpret-as=\"{}\">{token}</say-as>",
                    if token.parse::<i32>().is_ok() {
                        "cardinal"
                    } else {
                        "spell-out"
                    }
                ));
            } else {
                output.push_str(token);
                output.push(' ');
            }
        }
        format!("{label} {output}").trim().to_owned()
    } else {
        body.to_owned()
    };
    if tts && tag {
        ssml(&text)
    } else {
        text
    }
}
impl Evaluator {
    pub fn take_notification_payload(&mut self, finding: &Finding) -> Option<Value> {
        self.notification_payloads
            .iter()
            .position(|(value, _)| value == finding)
            .map(|index| self.notification_payloads.remove(index).1)
    }
    pub(super) fn route_notification_detail(&self, first: bool, tts: bool) -> Option<String> {
        let system = self.systems.get(&self.key())?;
        let Some(body) = system.route.first().and_then(|id| system.bodies.get(id)) else {
            return (!first).then(|| "No more bodies to map".into());
        };
        let name = body_name(&body.name, &system.name, tts, false);
        let short = name.get(5..).unwrap_or("");
        Some(format!(
            "{} body to map is {short}",
            if first { "First" } else { "Next" }
        ))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn body_names_and_ssml_match_all_32_original_formatter_captures() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../../../fixtures/evaluator-notification-oracle.json"
        ))
        .unwrap();
        for case in oracle["cases"].as_array().unwrap() {
            assert_eq!(
                body_name(
                    case["body"].as_str().unwrap(),
                    case["system"].as_str().unwrap(),
                    case["tts"].as_bool().unwrap(),
                    case["tag"].as_bool().unwrap()
                ),
                case["text"].as_str().unwrap(),
                "{case}"
            );
        }
    }
}
