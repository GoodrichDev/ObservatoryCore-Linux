//! Installed OverlaySettings, FormNotificationSettings and FormRadarSettings.
//! Explicit child forms keep edits private until the source Apply action.
use crate::font_choices;
use eframe::egui::{self, Id};
use serde_json::{json, Value};

#[derive(Clone, Copy, PartialEq)]
enum Form {
    Notifications,
    Radar,
}
#[derive(Clone, Default)]
struct State {
    form: Option<Form>,
    draft: Value,
    color_picker: Option<(String, [u8; 4])>,
}

const NOTIFICATION_NUMBERS: [(&str, &str, i64, i64, i64); 13] = [
    ("FontScaling", "Font Scaling (%)", 50, 200, 100),
    ("BorderWidth", "Border Width", 0, 20, 3),
    ("CornerRounding", "Corner Rounding", 0, 50, 20),
    ("XPadding", "Horizontal Padding", 0, 50, 5),
    ("YPadding", "Vertical Padding", 0, 50, 5),
    ("EdgeIndentation", "Edge Indentation", 0, 100, 20),
    ("VerticalSpacing", "Vertical Spacing", 0, 100, 10),
    ("MaxNotifications", "Number of Notifications", 1, 5, 3),
    ("TTL", "Duration (ms)", 1000, 15000, 5000),
    (
        "DelayBetweenNotificationsMS",
        "Notification Staggering (ms)",
        0,
        1000,
        200,
    ),
    ("MaxWidth", "Max Width (%)", 20, 50, 25),
    ("FPS", "FPS", 10, 300, 120),
    (
        "VerticalPositionPercent",
        "Vertical Position (%)",
        0,
        100,
        97,
    ),
];
const RADAR_NUMBERS: [(&str, &str, i64, i64, i64); 4] = [
    ("PositionXPct", "Horizontal Position (%)", 0, 100, 5),
    ("PositionYPct", "Vertical Position (%)", 0, 100, 55),
    ("SizePct", "Size (%)", 20, 100, 30),
    ("FontScaling", "Font Scaling (%)", 50, 200, 100),
];
fn checkbox(ui: &mut egui::Ui, value: &mut Value, key: &str, label: &str, default: bool) {
    let mut selected = value[key].as_bool().unwrap_or(default);
    if ui.checkbox(&mut selected, label).changed() {
        value[key] = json!(selected);
    }
}
fn text(ui: &mut egui::Ui, value: &mut Value, key: &str) {
    let mut current = value[key].as_str().map(str::to_owned).unwrap_or_else(|| {
        if value[key].is_null() {
            String::new()
        } else {
            value[key].to_string()
        }
    });
    if ui.text_edit_singleline(&mut current).changed() {
        value[key] = json!(current);
    }
}
fn selection(ui: &mut egui::Ui, value: &mut Value, key: &str, choices: &[&str]) {
    let mut current = value[key].as_str().unwrap_or(choices[0]).to_owned();
    egui::ComboBox::from_id_salt(key)
        .selected_text(&current)
        .show_ui(ui, |ui| {
            for choice in choices {
                ui.selectable_value(&mut current, (*choice).into(), *choice);
            }
        });
    value[key] = json!(current);
}
fn number(value: &Value, min: i64, max: i64, fallback: i64) -> i64 {
    let parsed = value
        .as_i64()
        .or_else(|| {
            value
                .as_f64()
                .filter(|n| n.fract() == 0.0)
                .map(|n| n as i64)
        })
        .or_else(|| value.as_str().and_then(|s| s.trim().parse().ok()));
    parsed
        .filter(|n| (min..=max).contains(n))
        .unwrap_or(fallback)
}
fn validated(draft: &Value, radar: bool) -> Value {
    let mut result = draft.clone();
    if radar {
        for (key, _, min, max, fallback) in RADAR_NUMBERS {
            result["RadarSettings"][key] =
                json!(number(&draft["RadarSettings"][key], min, max, fallback));
        }
    } else {
        for (key, _, min, max, fallback) in NOTIFICATION_NUMBERS {
            result[key] = json!(number(&draft[key], min, max, fallback));
        }
        for key in [
            "TitleColorHex",
            "ColorHex",
            "BorderColorHex",
            "BackgroundColorHex",
        ] {
            // Utils.SanitizeStringToARGBString uppercases, trims leading '#'
            // and retains at most eight characters.
            let value = draft[key]
                .as_str()
                .unwrap_or("")
                .to_uppercase()
                .trim_start_matches('#')
                .chars()
                .take(8)
                .collect::<String>();
            result[key] = json!(value);
        }
        if result["SelectedFont"]
            .as_str()
            .is_none_or(|s| s.trim().is_empty())
        {
            result["SelectedFont"] = json!("Arial");
        }
    }
    result
}
pub fn show(ui: &mut egui::Ui, value: &mut Value) -> Vec<Value> {
    let id = Id::new("StellarOverlay original settings forms");
    let mut state = ui
        .ctx()
        .data(|d| d.get_temp::<State>(id))
        .unwrap_or_default();
    let mut actions = Vec::new();
    ui.strong("♦ Overlay Settings ♦");
    checkbox(
        ui,
        value,
        "HandleObservatoryNotifications",
        "Handle Observatory Notifications",
        true,
    );
    checkbox(
        ui,
        value,
        "DisplayOverride",
        "Show popups when Elite is not active window",
        false,
    );
    ui.horizontal(|ui| {
        ui.label("Display Monitor");
        selection(
            ui,
            value,
            "SelectedMonitor",
            &["0 (Default)", "1", "2", "3"],
        );
    });
    ui.separator();
    ui.strong("♦ Notification Options ♦");
    if ui.button("Change Notification Settings").clicked() {
        state.form = Some(Form::Notifications);
        state.draft = value.clone();
    }
    ui.separator();
    ui.strong("♦ Radar ♦");
    if ui.button("Change Radar Settings").clicked() {
        state.form = Some(Form::Radar);
        state.draft = value.clone();
    }
    ui.separator();
    ui.strong("♦ Automatic Updates ♦");
    checkbox(
        ui,
        value,
        "InstallUpdates",
        "Automatically install updates",
        true,
    );
    checkbox(
        ui,
        value,
        "InstallBetas",
        "Install beta releases *NOT RECOMMENDED*",
        false,
    );
    ui.separator();
    ui.strong("♦ Debug ♦");
    checkbox(ui, value, "VerboseLogging", "Enable Debug Logging", false);
    for (label, command) in [
        ("Write Overlay State to Log", "DebugState"),
        ("Spawn Debug Notifications", "DebugNotifications"),
    ] {
        if ui.button(label).clicked() {
            actions.push(json!({"command":command,"payload":{}}));
        }
    }
    if let Some(form) = state.form {
        let radar = form == Form::Radar;
        let title = if radar {
            "StellarOverlay - Radar Settings"
        } else {
            "StellarOverlay - Notification Settings"
        };
        let size = if radar {
            [380.0, 178.0]
        } else {
            [538.0, 614.0]
        };
        let mut close = false;
        ui.ctx().show_viewport_immediate(egui::ViewportId::from_hash_of(title),egui::ViewportBuilder::default().with_title(title).with_inner_size(size).with_resizable(false),|ctx,_|{
            egui::CentralPanel::default().frame(egui::Frame::new().inner_margin(12)).show(ctx,|ui|{
                egui::ScrollArea::vertical().show(ui,|ui|{
                    egui::Grid::new(title).num_columns(2).show(ui,|ui|{
                        if radar {
                            for (key,label,_,_,_) in RADAR_NUMBERS {ui.label(label);text(ui,&mut state.draft["RadarSettings"],key);ui.end_row();}
                        }else{
                            ui.label("Shape");selection(ui,&mut state.draft,"NotificationShape",&["Open","Closed"]);ui.end_row();
                            ui.label("Font");let mut font=state.draft["SelectedFont"].as_str().unwrap_or("Arial").to_owned();font_choices::select(ui,&mut font);state.draft["SelectedFont"]=json!(font);ui.end_row();
                            ui.label("Alignment");selection(ui,&mut state.draft,"TextAlignment",&["Left","Center","Right"]);ui.end_row();
                            for (key,label,_,_,_) in NOTIFICATION_NUMBERS {ui.label(label);text(ui,&mut state.draft,key);ui.end_row();}
                            for (key,label) in [("TitleColorHex","Title Colour (ARGB)"),("ColorHex","Text Colour (ARGB)"),("BorderColorHex","Border Colour (ARGB)"),("BackgroundColorHex","Background Colour (ARGB)")] {
                                let color=super::color(&state.draft,key,egui::Color32::WHITE);
                                if ui.add(egui::Button::new(label).fill(color)).on_hover_text("Double-click to select a colour").double_clicked(){state.color_picker=Some((key.into(),color.to_srgba_unmultiplied()));}
                                text(ui,&mut state.draft,key);ui.end_row();
                            }
                            ui.label("Display From Right");checkbox(ui,&mut state.draft,"FromRight","",true);ui.end_row();
                        }
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui|{
                        if ui.button(if radar {"Apply"}else{"Apply and Test"}).clicked(){
                            *value=validated(&state.draft,radar);state.draft=value.clone();
                            actions.push(json!({"command":"SaveSettings","payload":{}}));
                            actions.push(json!({"command":if radar{"TestRadar"}else{"TestNotifications"},"payload":{"Settings":value}}));
                        }
                        if ui.button("Close").clicked(){close=true;}
                    });
                });
            });
            if ctx.input(|i|i.viewport().close_requested()){close=true;}
        });
        if close {
            state.form = None;
        }
    }
    if let Some((key, mut channels)) = state.color_picker.clone() {
        let mut close = false;
        ui.ctx().show_viewport_immediate(
            egui::ViewportId::from_hash_of("StellarOverlay colour picker"),
            egui::ViewportBuilder::default()
                .with_title("Colour Picker")
                .with_inner_size([349.0, 212.0])
                .with_resizable(false),
            |ctx, _| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let color = egui::Color32::from_rgba_unmultiplied(
                        channels[0],
                        channels[1],
                        channels[2],
                        channels[3],
                    );
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(201.0, 45.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 0.0, color);
                    for (index, label) in [(0, "R"), (1, "G"), (2, "B"), (3, "A")] {
                        ui.add(egui::Slider::new(&mut channels[index], 0..=255).text(label));
                    }
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() {
                            state.draft[&key] = json!(format!(
                                "{:02X}{:02X}{:02X}{:02X}",
                                channels[3], channels[0], channels[1], channels[2]
                            ));
                            close = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    close = true;
                }
            },
        );
        state.color_picker = if close { None } else { Some((key, channels)) };
    }
    ui.ctx().data_mut(|d| d.insert_temp(id, state));
    actions
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installed_form_sanitize_bounds_reset_invalid_text_instead_of_clamping() {
        let draft = json!({"TTL":"0","MaxNotifications":50,"MaxWidth":19,"FPS":301,"FontScaling":"125","XPadding":51,"CornerRounding":"invalid","TitleColorHex":"#1234567890","SelectedFont":""});
        let result = validated(&draft, false);
        assert_eq!(result["TTL"], 5000);
        assert_eq!(result["MaxNotifications"], 3);
        assert_eq!(result["MaxWidth"], 25);
        assert_eq!(result["FPS"], 120);
        assert_eq!(result["FontScaling"], 125);
        assert_eq!(result["XPadding"], 5);
        assert_eq!(result["CornerRounding"], 20);
        assert_eq!(result["TitleColorHex"], "12345678");
        assert_eq!(result["SelectedFont"], "Arial");
        let result = validated(
            &json!({"RadarSettings":{"PositionXPct":"-1","PositionYPct":"101","SizePct":19,"FontScaling":201}}),
            true,
        );
        assert_eq!(
            result["RadarSettings"],
            json!({"PositionXPct":5,"PositionYPct":55,"SizePct":30,"FontScaling":100})
        );
    }
}
