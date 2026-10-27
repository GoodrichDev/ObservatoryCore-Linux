//! Settings and explicit forms of Evaluator 1.3.444.637.
use eframe::egui::{self, Color32, Id};
use serde_json::{json, Value};
#[derive(Clone, Copy)]
enum Form {
    Always,
    Notifications,
    Theme,
    Stellar,
    Popout,
}
#[derive(Clone, Default)]
struct State {
    form: Option<Form>,
    draft: Value,
    colours: bool,
}
fn check(ui: &mut egui::Ui, value: &mut Value, key: &str, label: &str, default: bool) {
    let mut enabled = value[key].as_bool().unwrap_or(default);
    if ui.checkbox(&mut enabled, label).changed() {
        value[key] = json!(enabled);
    }
}
fn integer(
    ui: &mut egui::Ui,
    value: &mut Value,
    key: &str,
    label: &str,
    default: i64,
    range: std::ops::RangeInclusive<i64>,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut n = value[key].as_i64().unwrap_or(default);
        if ui.add(egui::DragValue::new(&mut n).range(range)).changed() {
            value[key] = json!(n);
        }
    });
}
fn save(actions: &mut Vec<Value>) {
    actions.push(json!({"command":"SaveSettings","payload":{}}));
}
fn help(ui: &mut egui::Ui, title: &str, message: &str) {
    let id = Id::new(("Evaluator source help", title));
    let mut open = ui.ctx().data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
    if ui.button("?").clicked() {
        open = true;
    }
    egui::Window::new(title)
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .show(ui.ctx(), |ui| {
            ui.label(message);
        });
    ui.ctx().data_mut(|d| d.insert_temp(id, open));
}
pub fn show(
    ui: &mut egui::Ui,
    settings: &mut Value,
    _summary: &Value,
    overlay_defaults: &Value,
) -> Vec<Value> {
    let id = Id::new("Evaluator source settings forms");
    let mut state = ui
        .ctx()
        .data(|d| d.get_temp::<State>(id))
        .unwrap_or_default();
    let mut actions = Vec::new();
    ui.strong("♦ Mapping Options ♦");
    for (key, label, default) in [
        ("MinimumBodyValue", "Hide Bodies Below", 10000),
        (
            "MinimumValueForWorthMapping",
            "Minimum Mapping Value",
            1000000,
        ),
        ("MaxDistanceELW", "Maximum Distance - Earth-Like Worlds", 0),
        ("MaxDistanceWW", "Maximum Distance - Water Worlds", 0),
        ("MaxDistanceAW", "Maximum Distance - Ammonia Worlds", 0),
        (
            "MaxDistanceAtmospheric",
            "Maximum Distance - Atmospheric",
            0,
        ),
        ("MaxDistanceOther", "Maximum Distance - Other", 0),
    ] {
        integer(ui, settings, key, label, default, 0..=i32::MAX as i64);
    }
    check(
        ui,
        settings,
        "IncludeAtmosphericBio",
        "Atmospherics with Bios worth mapping",
        false,
    );
    check(
        ui,
        settings,
        "SetTargetedInFSSWorthMapping",
        "Set bodies targeted in FSS as Worth Mapping",
        true,
    );
    ui.separator();
    ui.strong("♦ BioInsights Mapping Options ♦");
    for (key, label) in [
        (
            "BIHighValueWorthMapping",
            "Valuable Biologicals/Bodies Worth Mapping?",
        ),
        (
            "BIPersonalCodexWorthMapping",
            "Possible Personal Codex Entries Worth Mapping?",
        ),
        (
            "BIGalacticCodexWorthMapping",
            "Possible Galactic Codex Entries Worth Mapping?",
        ),
        (
            "BIMarkedWorthMapping",
            "Possible Marked Biologicals Worth Mapping?",
        ),
    ] {
        check(ui, settings, key, label, false);
    }
    for (form, label) in [
        (Form::Always, "Always Show"),
        (Form::Stellar, "Configure Stellar Overlay"),
        (Form::Notifications, "Configure Notifications"),
        (Form::Theme, "Configure Theming"),
        (Form::Popout, "Configure Popout"),
    ] {
        if label == "Configure Stellar Overlay" {
            ui.separator();
            ui.strong("♦ Stellar Overlay (requires StellarOverlay plugin) ♦");
        }
        if ui.button(label).clicked() {
            state.form = Some(form);
            state.draft = settings.clone();
            state.colours = false;
        }
    }
    ui.separator();
    ui.strong("♦ Interface ♦");
    choice(
        ui,
        settings,
        "NativeOutput",
        "Native Output (requires restart)",
        " Enabled",
        &[" Enabled", "Disabled"],
    );
    for (key, label, default) in [
        ("ExpandOutput", "Expand output to window size", true),
        ("ShowHeaders", "Show Headers", true),
        (
            "ShowTotal",
            "Show Unsold Data Total (requires Hecate)",
            false,
        ),
    ] {
        check(ui, settings, key, label, default);
    }
    integer(
        ui,
        settings,
        "MaxSystems",
        "Maximum Systems to show after Read All",
        1000,
        1..=i32::MAX as i64,
    );
    choice(
        ui,
        settings,
        "ButtonSize",
        "Button Size",
        "  Normal",
        &[
            "    Smaller",
            "   Small",
            "  Normal",
            " Large",
            "Extra Large",
        ],
    );
    check(ui, settings, "VerboseLogging", "Verbose Logging", false);
    ui.separator();
    ui.strong("♦ Updates ♦");
    check(ui, settings, "InstallUpdates", "Install Updates", true);
    check(ui, settings, "InstallBetas", "Install Betas", false);
    if let Some(form) = state.form {
        let (title, size) = match form {
            Form::Always => ("Evaluator - Always Show Settings", [701.0, 269.0]),
            Form::Notifications => ("Evaluator Notification Settings", [549.0, 282.0]),
            Form::Theme => ("Evaluator - Theming", [414.0, 131.0]),
            Form::Stellar => ("Evaluator - Stellar Overlay Settings", [445.0, 255.0]),
            Form::Popout => ("Evaluator - Popout Settings", [701.0, 210.0]),
        };
        let mut open = true;
        ui.ctx().show_viewport_immediate(
            egui::ViewportId::from_hash_of(title),
            egui::ViewportBuilder::default()
                .with_title(title)
                .with_inner_size(size)
                .with_resizable(false),
            |ctx, _| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().inner_margin(10.0))
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            match form {
                                Form::Always => always(ui, &mut state.draft),
                                Form::Notifications => notifications(ui, &mut state.draft),
                                Form::Stellar => stellar(ui, &mut state.draft, overlay_defaults),
                                Form::Popout => popout(ui, &mut state.draft),
                                Form::Theme => {
                                    let theme = &mut state.draft["ThemeSettings"];
                                    let mut selected = theme["ThemeIndex"].as_u64().unwrap_or(1);
                                    egui::ComboBox::from_id_salt("Evaluator source theme")
                                        .selected_text(if selected == 0 {
                                            "Default"
                                        } else {
                                            "Elite Orange"
                                        })
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut selected, 0, "Default");
                                            ui.selectable_value(&mut selected, 1, "Elite Orange");
                                        });
                                    theme["ThemeIndex"] = json!(selected);
                                    let mut custom = theme["ThemeMode"] == 1;
                                    if ui.checkbox(&mut custom, "Custom Colours").changed() {
                                        theme["ThemeMode"] = json!(u8::from(custom));
                                    }
                                    if custom && ui.button("Edit Colours").clicked() {
                                        state.colours = true;
                                    }
                                }
                            }
                            ui.add_space(10.0);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .button(if matches!(form, Form::Notifications) {
                                            "Apply and Save"
                                        } else {
                                            "Save"
                                        })
                                        .clicked()
                                    {
                                        *settings = state.draft.clone();
                                        save(&mut actions);
                                        open = false;
                                    }
                                    if matches!(form, Form::Theme) && ui.button("Apply").clicked() {
                                        *settings = state.draft.clone();
                                        save(&mut actions);
                                    }
                                },
                            );
                        });
                    });
                if ctx.input(|i| i.viewport().close_requested()) {
                    open = false;
                }
            },
        );
        if !open {
            state.form = None;
            state.colours = false;
        }
    }
    if state.colours {
        let mut open = true;
        ui.ctx().show_viewport_immediate(
            egui::ViewportId::from_hash_of("Evaluator - Custom Colours"),
            egui::ViewportBuilder::default()
                .with_title("Evaluator - Custom Colours")
                .with_inner_size([480.0, 600.0]),
            |ctx, _| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (key, fallback) in super::Theme::new(&state.draft).data["Colors"]
                            .as_object()
                            .unwrap()
                        {
                            let original = state.draft["ThemeSettings"]["Colors"]
                                .get(key)
                                .unwrap_or(fallback);
                            let mut color = [
                                original["R"].as_u64().unwrap_or(0) as u8,
                                original["G"].as_u64().unwrap_or(0) as u8,
                                original["B"].as_u64().unwrap_or(0) as u8,
                            ];
                            ui.horizontal(|ui| {
                                ui.label(key);
                                if ui.color_edit_button_srgb(&mut color).changed() {
                                    state.draft["ThemeSettings"]["Colors"][key] =
                                        json!({"R":color[0],"G":color[1],"B":color[2]});
                                }
                            });
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Reset").clicked() {
                            state.draft["ThemeSettings"]["Colors"] =
                                super::Theme::new(&state.draft).data["Colors"].clone();
                        }
                        if ui.button("Apply").clicked() {
                            *settings = state.draft.clone();
                            save(&mut actions);
                        }
                        if ui.button("Save").clicked() {
                            *settings = state.draft.clone();
                            save(&mut actions);
                            open = false;
                        }
                    });
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    open = false;
                }
            },
        );
        state.colours = open;
    }
    ui.ctx().data_mut(|d| d.insert_temp(id, state));
    actions
}
fn choice(
    ui: &mut egui::Ui,
    settings: &mut Value,
    key: &str,
    label: &str,
    default: &str,
    choices: &[&str],
) {
    ui.horizontal(|ui| {
        ui.label(label);
        let current = settings[key].as_str().unwrap_or(default).to_owned();
        egui::ComboBox::from_id_salt(("Evaluator source choice", key))
            .selected_text(current.trim())
            .show_ui(ui, |ui| {
                for option in choices {
                    if ui
                        .selectable_label(current == *option, option.trim())
                        .clicked()
                    {
                        settings[key] = json!(option);
                    }
                }
            });
    });
}
fn always(ui: &mut egui::Ui, settings: &mut Value) {
    ui.strong("Always Show");
    ui.columns(2,|columns|{
 for(key,label)in[("AlwaysShowMain","Main star"),("AlwaysShowELW","Earth-like worlds"),("AlwaysShowWW","Water worlds"),("AlwaysShowTerraformables","Terraformables"),("AlwaysShowAW","Ammonia worlds"),("AlwaysShowAtmospherics","Atmospheric landables"),("AlwaysShowAtmosphericsBios","Atmospherics with bios"),("AlwaysShowNonAtmosphericsBios","Non-atmospherics with bios")]{check(&mut columns[0],settings,key,label,false);}
 for(key,label)in[("AlwaysShowBIHighValue","BioInsights high value"),("AlwaysShowBIPersonalCodex","BioInsights personal codex"),("AlwaysShowBIGalacticCodex","BioInsights galactic codex"),("AlwaysShowBIMarked","BioInsights marked")]{check(&mut columns[1],settings,key,label,false);}help(&mut columns[1],"Help - Always Show","When selected, these options allow you to always include bodies in the output window even if they do not otherwise meet the 'Worth Mapping' criteria");});
}
fn notifications(ui: &mut egui::Ui, settings: &mut Value) {
    let settings = &mut settings["Notifications"];
    egui::Grid::new("Evaluator notification channels").show(ui, |ui| {
        for title in ["Notification", "Popup", "Voice", "Plugin"] {
            ui.strong(title);
        }
        ui.end_row();
        for (key, label) in [
            ("BodiesWorthMapping", "Bodies worth mapping"),
            ("EfficientRoute", "Efficient route"),
        ] {
            ui.label(label);
            for (channel, default) in [("Popup", true), ("Voice", true), ("Plugin", false)] {
                check(ui, &mut settings[key], channel, "", default);
            }
            ui.end_row();
        }
        for (key, label, default) in [
            ("DSSPersistent", "Bodies left to map [DSS]", true),
            ("AlwaysPersistent", "Bodies left to map [Always]", false),
        ] {
            ui.label(label);
            check(ui, &mut settings[key], "Popup", "", default);
            ui.label("");
            ui.label("");
            ui.end_row();
        }
    });
    ui.separator();
    ui.strong("Persistent Notification Position");
    integer(
        ui,
        settings,
        "PersistentPositionX",
        "X Pos [%] (0-100):",
        80,
        0..=100,
    );
    integer(
        ui,
        settings,
        "PersistentPositionY",
        "Y Pos [%] (0-100):",
        25,
        0..=100,
    );
}
fn popout(ui: &mut egui::Ui, settings: &mut Value) {
    integer(
        ui,
        settings,
        "PopoutOpacity",
        "Opacity [%] (10-100)",
        100,
        10..=100,
    );
    ui.columns(2, |columns| {
        check(
            &mut columns[0],
            settings,
            "PopoutAlwaysShow",
            "Always Show Popout",
            true,
        );
        let enabled = settings["PopoutAlwaysShow"].as_bool() != Some(true);
        columns[0].add_enabled_ui(enabled, |ui| {
            for (key, label) in [
                ("PopoutShowLeftPanel", "Show Popout on Left Panel"),
                ("PopoutShowRolePanel", "Show Popout on Role Panel (Bottom)"),
                ("PopoutShowSysMap", "Show Popout on System Map"),
            ] {
                check(ui, settings, key, label, true);
            }
        });
        columns[1].add_enabled_ui(enabled, |ui| {
            for (key, label) in [
                ("PopoutShowNoFocus", "Show Popout on Main Window"),
                ("PopoutShowRightPanel", "Show Popout on Right Panel"),
                ("PopoutShowDSSMode", "Show Popout in DSS mode"),
                ("PopoutShowFSSMode", "Show Popout in FSS Mode"),
            ] {
                check(ui, settings, key, label, true);
            }
        });
    });
    help(ui,"Help - Popout Options","When using the popped out display, these options allow you to show or hide the popout based on the current GUI focus");
}
fn stellar(ui: &mut egui::Ui, settings: &mut Value, defaults: &Value) {
    check(
        ui,
        settings,
        "UseCustomNotificationColours",
        "Use Custom Notification Colours",
        false,
    );
    ui.add_enabled_ui(
        settings["UseCustomNotificationColours"]
            .as_bool()
            .unwrap_or(false),
        |ui| {
            for (key, label) in [
                ("TitleColorHex", "Title"),
                ("ColorHex", "Text"),
                ("BorderColorHex", "Border"),
                ("BackgroundColorHex", "Background"),
            ] {
                let raw = settings[key]
                    .as_str()
                    .unwrap_or("FFFFFFFF")
                    .trim_start_matches('#');
                let value = u32::from_str_radix(raw, 16).unwrap_or(u32::MAX);
                let mut color = Color32::from_rgba_unmultiplied(
                    (value >> 16) as u8,
                    (value >> 8) as u8,
                    value as u8,
                    (value >> 24) as u8,
                );
                ui.horizontal(|ui| {
                    ui.label(label);
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        let [r, g, b, a] = color.to_array();
                        settings[key] = json!(format!("{a:02X}{r:02X}{g:02X}{b:02X}"));
                    }
                });
            }
            if ui
                .add_enabled(
                    [
                        "TitleColorHex",
                        "ColorHex",
                        "BorderColorHex",
                        "BackgroundColorHex",
                    ]
                    .iter()
                    .all(|key| defaults[key].is_string()),
                    egui::Button::new("Reset"),
                )
                .clicked()
            {
                for key in [
                    "TitleColorHex",
                    "ColorHex",
                    "BorderColorHex",
                    "BackgroundColorHex",
                ] {
                    settings[key] = defaults[key].clone();
                }
            }
        },
    );
}
