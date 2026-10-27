//! Source popout policy from UIHandler.HandleGuiStatusChange and
//! BaseUIHandler.doPopout. Native viewport integration lives in the host UI.
use eframe::egui;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;

/// The three presentation flags serialized by source PopoutSettings.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Chrome {
    #[serde(rename = "IsOnTop")]
    pub on_top: bool,
    #[serde(rename = "ShowTitleBar")]
    pub title_bar: bool,
    #[serde(rename = "HideButtons")]
    pub hide_buttons: bool,
}
impl Default for Chrome {
    fn default() -> Self {
        Self {
            on_top: false,
            title_bar: true,
            hide_buttons: false,
        }
    }
}

fn apply_menu(command: &str, chrome: &mut Chrome) -> Value {
    match command {
        "Toggle OnTop" => chrome.on_top = !chrome.on_top,
        // Source Lock/Unlock Position dispatches ToggleTitleBar.
        "Lock/Unlock Position" => chrome.title_bar = !chrome.title_bar,
        "Toggle Buttons" => chrome.hide_buttons = !chrome.hide_buttons,
        "Force Save Position" => return json!({"command":"SavePopoutPosition","payload":{}}),
        "Close Popout" => return json!({"command":"Popout","payload":{"Restore":true}}),
        _ => return Value::Null,
    }
    json!({"command":"SavePopoutPosition","payload":{}})
}

/// Attach to the existing popout's context menu. These are the source
/// POMenuStrip options; toggling chrome requests the original settings save.
pub fn menu(ui: &mut egui::Ui, chrome: &mut Chrome) -> Vec<Value> {
    let mut actions = Vec::new();
    ui.menu_button("Navigate", |ui| {
        for (label, route) in [
            ("Main Window", "Main"),
            ("Show Current Data", "Unsold"),
            ("About", "About"),
            ("Codex Companion", "CodexCompanion"),
        ] {
            if ui.button(label).clicked() {
                actions.extend(super::navigate(ui.ctx(), route, &json!({})));
                ui.close_menu();
            }
        }
    });
    for label in [
        "Toggle OnTop",
        "Lock/Unlock Position",
        "Toggle Buttons",
        "Force Save Position",
        "Close Popout",
    ] {
        if ui.button(label).clicked() {
            actions.push(apply_menu(label, chrome));
            ui.close_menu();
        }
    }
    actions
}

/// Exact source GUI focus gates. AlwaysShow bypasses the other conditions.
pub fn visible(settings: &Value, gui_focus: u64, flags: u64, population: u64) -> bool {
    let enabled = |key: &str, default: bool| settings[key].as_bool().unwrap_or(default);
    if enabled("PopoutAlwaysShow", true) {
        return true;
    }
    let focus = match gui_focus {
        0 => enabled("PopoutShowNoFocus", true),
        2 => enabled("PopoutShowLeftPanel", true),
        1 => enabled("PopoutShowRightPanel", true),
        4 => enabled("PopoutShowRolePanel", true),
        7 => enabled("PopoutShowSysMap", true),
        9 => enabled("PopoutShowFSSMode", true),
        10 => enabled("PopoutShowDSSMode", true),
        _ => false,
    };
    focus
        && (!enabled("PopoutHideUnlessSupercruiseOrSrv", false) || flags & (16 | 67_108_864) != 0)
        && (!enabled("PopoutHidePopulated", false) || population == 0)
}

/// FormPopout receives PopoutOpacity/100; the source editor allows 10..100.
pub fn opacity(settings: &Value) -> f32 {
    settings["PopoutOpacity"]
        .as_f64()
        .unwrap_or(100.0)
        .clamp(10.0, 100.0) as f32
        / 100.0
}

/// Ctrl+Popout bypasses restored location and size, using the original panel's
/// dimensions and allowing the native window manager to place the window.
pub fn geometry(
    panel_size: [f32; 2],
    restore: bool,
    saved_position: Option<[f32; 2]>,
    saved_size: [f32; 2],
) -> (Option<[f32; 2]>, [f32; 2]) {
    if restore {
        (saved_position, saved_size)
    } else {
        (None, panel_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn visibility_matches_all_73728_independent_source_cases() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../../fixtures/bioinsights-popout-oracle.json"
        ))
        .unwrap();
        let keys = oracle["setting_bit_order"].as_array().unwrap();
        let contexts = oracle["contexts"].as_array().unwrap();
        let mut checked = 0;
        for case in oracle["cases"].as_array().unwrap() {
            let mask = case["mask"].as_u64().unwrap();
            let mut settings = json!({});
            for (bit, key) in keys.iter().enumerate() {
                settings[key.as_str().unwrap()] = json!(mask & (1 << bit) != 0);
            }
            for (index, context) in contexts.iter().enumerate() {
                for (gui_focus, expected) in
                    case["visible"][index].as_str().unwrap().bytes().enumerate()
                {
                    assert_eq!(
                        visible(
                            &settings,
                            gui_focus as u64,
                            context["flags"].as_u64().unwrap(),
                            context["population"].as_u64().unwrap()
                        ),
                        expected == b'1',
                        "mask {mask}, focus {gui_focus}, context {index}"
                    );
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, 73_728);
    }

    #[test]
    fn source_always_show_bypasses_population_and_travel_gates() {
        let settings = json!({"PopoutAlwaysShow":true,"PopoutHidePopulated":true,"PopoutHideUnlessSupercruiseOrSrv":true});
        assert!(visible(&settings, 5, 0, 8_000_000));
        assert_eq!(opacity(&json!({})), 1.0);
        assert_eq!(opacity(&json!({"PopoutOpacity":25})), 0.25);
    }

    #[test]
    fn control_popout_uses_panel_dimensions_and_ignores_saved_location() {
        assert_eq!(
            geometry([1500.0, 800.0], false, Some([20.0, 30.0]), [600.0, 400.0]),
            (None, [1500.0, 800.0])
        );
        assert_eq!(
            geometry([1500.0, 800.0], true, Some([20.0, 30.0]), [600.0, 400.0]),
            (Some([20.0, 30.0]), [600.0, 400.0])
        );
    }

    #[test]
    fn source_context_menu_toggles_and_serializes_original_chrome_flags() {
        let mut chrome = Chrome::default();
        for menu in ["Toggle OnTop", "Lock/Unlock Position", "Toggle Buttons"] {
            assert_eq!(
                apply_menu(menu, &mut chrome)["command"],
                "SavePopoutPosition"
            );
        }
        assert_eq!(
            serde_json::to_value(&chrome).unwrap(),
            json!({"IsOnTop":true,"ShowTitleBar":false,"HideButtons":true})
        );
        assert_eq!(apply_menu("Close Popout", &mut chrome)["command"], "Popout");
    }
}
