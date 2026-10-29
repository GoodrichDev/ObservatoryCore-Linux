//! Core's original notification settings and routing test controls.
use eframe::egui;
use observatory_io::Settings;
use serde_json::{json, Value};
pub enum Action {
    Popup,
    Audio,
    Routing(u8),
}
fn get<'a>(s: &'a Settings, key: &str) -> Option<&'a Value> {
    s.extra
        .get(key)
        .or_else(|| s.extra.get("legacy_settings").and_then(|v| v.get(key)))
}
#[allow(deprecated)]
fn at<R>(ui: &mut egui::Ui, xy: [f32; 2], size: [f32; 2], f: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let bounds = egui::Rect::from_min_size(
        ui.max_rect().min + egui::vec2(xy[0], xy[1]),
        egui::vec2(size[0], size[1]),
    );
    ui.allocate_ui_at_rect(bounds, f).inner
}
fn spin(
    ui: &mut egui::Ui,
    s: &mut Settings,
    key: &str,
    default: i64,
    xy: [f32; 2],
    bounds: std::ops::RangeInclusive<i64>,
) {
    let mut n = get(s, key)
        .and_then(Value::as_i64)
        .unwrap_or(default)
        .clamp(*bounds.start(), *bounds.end());
    at(ui, xy, [67.0, 23.0], |ui| {
        if ui
            .add_sized([67.0, 23.0], egui::DragValue::new(&mut n).range(bounds))
            .changed()
        {
            s.extra.insert(key.into(), json!(n));
        }
    });
}
pub fn popup(ui: &mut egui::Ui, s: &mut Settings, monitor_count: usize) -> Vec<Action> {
    let mut actions = Vec::new();
    let style = crate::native_popup::Style::from_settings(s);
    for (label, x, y) in [
        ("Display:", 55.0, 25.0),
        ("Corner:", 57.0, 54.0),
        ("Font:", 69.0, 83.0),
        ("Scale (%):", 45.0, 111.0),
        ("Font Scale (%):", 201.0, 111.0),
        ("Duration (ms):", 20.0, 140.0),
        ("Colour:", 57.0, 171.0),
    ] {
        at(ui, [x, y], [100.0, 15.0], |ui| {
            ui.label(label);
        });
    }
    let mut screen = style.screen;
    at(ui, [117.0, 22.0], [121.0, 23.0], |ui| {
        egui::ComboBox::from_id_salt("Core notification display")
            .width(121.0)
            .selected_text(if screen < 0 {
                "Primary".into()
            } else {
                (screen + 1).to_string()
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut screen, -1, "Primary");
                if monitor_count > 1 {
                    for i in 0..monitor_count {
                        ui.selectable_value(&mut screen, i as i64, (i + 1).to_string());
                    }
                }
            });
    });
    if screen != style.screen {
        s.extra.insert("NativeNotifyScreen".into(), json!(screen));
    }
    let corners = [
        "Bottom-Right",
        "Bottom-Left",
        "Top-Right",
        "Top-Left",
        "Center-Top",
        "Center-Bottom",
        "Center-Left",
        "Center-Right",
    ];
    let mut corner = style.corner;
    at(ui, [117.0, 51.0], [121.0, 23.0], |ui| {
        egui::ComboBox::from_id_salt("Core notification corner")
            .width(121.0)
            .selected_text(corners[corner as usize])
            .show_ui(ui, |ui| {
                for (i, label) in corners.iter().enumerate() {
                    ui.selectable_value(&mut corner, i as u8, *label);
                }
            });
    });
    if corner != style.corner {
        s.extra.insert("NativeNotifyCorner".into(), json!(corner));
    }
    let mut font = style.font.clone();
    at(ui, [117.0, 80.0], [242.0, 23.0], |ui| {
        egui::ComboBox::from_id_salt("Core notification font")
            .width(242.0)
            .selected_text(&font)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut font, style.font.clone(), &style.font);
                for family in crate::font_choices::choices(ui.ctx()) {
                    ui.selectable_value(&mut font, family.clone(), family);
                }
            });
    });
    if font != style.font {
        s.extra.insert("NativeNotifyFont".into(), json!(font));
    }
    spin(
        ui,
        s,
        "NativeNotifyScale",
        (style.scale * 100.0).round() as i64,
        [118.0, 109.0],
        1..=500,
    );
    spin(
        ui,
        s,
        "NativeNotifyFontScale",
        (style.font_scale * 100.0).round() as i64,
        [292.0, 109.0],
        1..=500,
    );
    spin(
        ui,
        s,
        "NativeNotifyTimeout",
        style.timeout,
        [118.0, 138.0],
        100..=60000,
    );
    let mut transparent = style.transparent;
    at(ui, [201.0, 140.0], [125.0, 19.0], |ui| {
        if ui.checkbox(&mut transparent, "Transparency").changed() {
            s.extra
                .insert("NativeNotifyTransparent".into(), json!(transparent));
        }
    });
    let mut rgba = [
        (style.colour >> 16) as u8,
        (style.colour >> 8) as u8,
        style.colour as u8,
        (style.colour >> 24) as u8,
    ];
    at(ui, [118.0, 167.0], [51.0, 23.0], |ui| {
        if ui.color_edit_button_srgba_unmultiplied(&mut rgba).changed() {
            s.extra.insert(
                "NativeNotifyColour".into(),
                json!(u32::from_be_bytes([rgba[3], rgba[0], rgba[1], rgba[2]])),
            );
        }
    });
    at(ui, [187.0, 167.0], [51.0, 23.0], |ui| {
        if ui
            .add_sized([51.0, 23.0], egui::Button::new("Test"))
            .clicked()
        {
            actions.push(Action::Popup);
        }
    });
    at(ui, [117.0, 196.0], [85.0, 19.0], |ui| {
        ui.checkbox(&mut s.popup, "Enabled");
    });
    actions
}
pub fn audio(
    ui: &mut egui::Ui,
    s: &mut Settings,
    voices: &[(String, String)],
    _devices: &[(String, String)],
) -> Vec<Action> {
    let mut actions = Vec::new();
    let old = get(s, "ChimeEnabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut chime = old;
    for (label, x, y) in [
        ("Type:", 69.0, 27.0),
        (if chime { "Chime:" } else { "Voice:" }, 65.0, 56.0),
        ("Speed:", 62.0, 98.0),
    ] {
        at(ui, [x, y], [45.0, 15.0], |ui| {
            ui.label(label);
        });
    }
    at(ui, [118.0, 24.0], [213.0, 23.0], |ui| {
        egui::ComboBox::from_id_salt("Core audio type")
            .width(213.0)
            .selected_text(if chime { "Chime" } else { "Voice" })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut chime, false, "Voice");
                ui.selectable_value(&mut chime, true, "Chime");
            });
    });
    if old != chime {
        s.extra.insert("ChimeEnabled".into(), json!(chime));
    }
    at(ui, [117.0, 53.0], [214.0, 23.0], |ui| {
        if chime {
            let old = get(s, "ChimeSelected")
                .and_then(Value::as_u64)
                .unwrap_or(0)
                .min(5);
            let mut selected = old;
            egui::ComboBox::from_id_salt("Core chime")
                .width(214.0)
                .selected_text((selected + 1).to_string())
                .show_ui(ui, |ui| {
                    for i in 0..6 {
                        ui.selectable_value(&mut selected, i, (i + 1).to_string());
                    }
                });
            if old != selected {
                s.extra.insert("ChimeSelected".into(), json!(selected));
            }
        } else {
            select(ui, s, "", "speech_voice", voices);
        }
    });
    at(ui, [117.0, 95.0], [214.0, 23.0], |ui| {
        ui.spacing_mut().slider_width = 214.0;
        let mut rate = get(s, "VoiceRate")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .clamp(-10, 10);
        ui.add_enabled_ui(!chime, |ui| {
            if ui
                .add(egui::Slider::new(&mut rate, -10..=10).show_value(false))
                .changed()
            {
                s.extra.insert("VoiceRate".into(), json!(rate));
                s.extra.insert(
                    "speech_rate".into(),
                    json!((175.0 * 2_f64.powf(rate as f64 / 10.0)).round() as u64),
                );
            }
        });
    });
    at(ui, [118.0, 133.0], [80.0, 19.0], |ui| {
        ui.checkbox(&mut s.speech, "Enabled");
    });
    at(ui, [208.0, 130.0], [51.0, 23.0], |ui| {
        if ui
            .add_sized([51.0, 23.0], egui::Button::new("Test"))
            .clicked()
        {
            actions.push(Action::Audio);
        }
    });
    actions
}
pub fn select(
    ui: &mut egui::Ui,
    s: &mut Settings,
    label: &str,
    key: &str,
    choices: &[(String, String)],
) {
    let old = get(s, key).and_then(Value::as_str).unwrap_or("").to_owned();
    let mut value = old.clone();
    ui.horizontal(|ui| {
        if !label.is_empty() {
            ui.label(label);
        }
        egui::ComboBox::from_id_salt(key)
            .width(ui.available_width())
            .selected_text(
                choices
                    .iter()
                    .find(|(id, _)| *id == value)
                    .map(|(_, label)| label.as_str())
                    .unwrap_or(if value.is_empty() { "Default" } else { &value }),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut value, String::new(), "Default");
                for (id, label) in choices {
                    ui.selectable_value(&mut value, id.clone(), label);
                }
            });
    });
    if value != old {
        s.extra.insert(key.into(), json!(value));
        if key == "speech_voice" {
            s.extra.insert("VoiceSelected".into(), json!(value));
        }
        if key == "audio_device" {
            s.extra.insert("AudioDevice".into(), json!(value));
        }
    }
}
pub fn routing(ui: &mut egui::Ui) -> Option<Action> {
    let id = ui.id().with("Core routing switches");
    let mut flags = ui.data_mut(|d| d.get_temp::<[bool; 3]>(id).unwrap_or_default());
    let mut result = None;
    for (i, label) in ["Popup", "Audio", "Plugin"].iter().enumerate() {
        at(ui, [i as f32 * 80.0, 0.0], [75.0, 19.0], |ui| {
            ui.checkbox(&mut flags[i], *label);
        });
    }
    at(ui, [0.0, 19.0], [90.0, 23.0], |ui| {
        if ui
            .add_sized([90.0, 23.0], egui::Button::new("Test Routing"))
            .clicked()
        {
            result = Some(Action::Routing(
                flags
                    .iter()
                    .enumerate()
                    .fold(0, |bits, (i, on)| bits | if *on { 1 << i } else { 0 }),
            ));
        }
    });
    ui.data_mut(|d| d.insert_temp(id, flags));
    result
}
