//! Source-named BioInsights settings and the plugin's explicit settings forms.
//! Labels, groups, bounds and form dimensions come from the installed 1.3.444.1052.
use eframe::egui::{self, Color32, Id};
use serde_json::{json, Value};

#[derive(Clone, Copy, PartialEq)]
enum Form {
    Notifications,
    Theming,
    StellarOverlay,
    Popout,
}
#[derive(Clone, Default)]
struct Forms {
    form: Option<Form>,
    colours: bool,
    wipe: bool,
    missing_overlay: bool,
    draft: Value,
}

fn open_form(forms: &mut Forms, form: Form, settings: &Value) {
    forms.form = Some(form);
    forms.draft = settings.clone();
    forms.colours = false;
}

fn check(ui: &mut egui::Ui, value: &mut Value, key: &str, label: &str, default: bool) {
    let mut checked = value[key].as_bool().unwrap_or(default);
    if ui.checkbox(&mut checked, label).changed() {
        value[key] = json!(checked);
    }
}
// Keep each source control's default, two bounds and increment together.
#[allow(clippy::too_many_arguments)]
fn numeric(
    ui: &mut egui::Ui,
    value: &mut Value,
    key: &str,
    label: &str,
    default: f64,
    min: f64,
    max: f64,
    step: f64,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut number = value[key].as_f64().unwrap_or(default);
        if ui
            .add(
                egui::DragValue::new(&mut number)
                    .range(min..=max)
                    .speed(step),
            )
            .changed()
        {
            value[key] = if step.fract() == 0.0 {
                json!(number.round() as i64)
            } else {
                json!(number)
            };
        }
    });
}
fn group(ui: &mut egui::Ui, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    ui.add_space(8.0);
    ui.separator();
    ui.strong(title);
    body(ui);
}
fn save(actions: &mut Vec<Value>) {
    actions.push(json!({"command":"SaveSettings","payload":{}}));
}

pub fn show(
    ui: &mut egui::Ui,
    settings: &mut Value,
    summary: &Value,
    overlay_defaults: &Value,
) -> Vec<Value> {
    let id = Id::new("bioinsights-source-settings-forms");
    let mut forms = ui
        .ctx()
        .data_mut(|d| d.get_temp::<Forms>(id))
        .unwrap_or_default();
    let mut actions = Vec::new();
    group(ui, "♦ Personal Codex Entries ♦", |ui| {
        check(
            ui,
            settings,
            "PersonalCodexPossibleAny",
            "Show entries I've not seen in galaxy",
            true,
        );
        check(
            ui,
            settings,
            "PersonalCodexPossibleCurrent",
            "Show entries I've not seen in current region",
            false,
        );
    });
    group(ui, "♦ Notifications and Theming ♦", |ui| {
        if ui.button("Configure Notifications").clicked() {
            open_form(&mut forms, Form::Notifications, settings);
        }
        if ui.button("Configure Theming").clicked() {
            open_form(&mut forms, Form::Theming, settings);
        }
    });
    group(
        ui,
        "♦ Stellar Overlay (requires StellarOverlay plugin) ♦",
        |ui| {
            if ui.button("Configure Stellar Overlay").clicked() {
                if summary["ReadyPeers"].get("StellarOverlay").is_some()
                    || summary["StellarOverlayAvailable"] == true
                {
                    open_form(&mut forms, Form::StellarOverlay, settings);
                } else {
                    forms.missing_overlay = true;
                }
            }
        },
    );
    group(
        ui,
        "♦ Hecate [Data Vault] (requires Hecate plugin) ♦",
        |ui| {
            check(ui, settings, "HecateUseDB", "Use Hecate system data", true);
            check(ui, settings, "HecateUseSpansh", "Use spansh data", true);
        },
    );
    group(ui, "♦ Popout ♦", |ui| {
        if ui.button("Configure Popout Settings").clicked() {
            open_form(&mut forms, Form::Popout, settings);
        }
    });
    group(ui, "♦ UI ♦", |ui| {
        ui.horizontal(|ui| {
            ui.label("Native Output (requires restart)");
            let current = settings["NativeOutput"]
                .as_str()
                .unwrap_or(" Enabled")
                .to_owned();
            egui::ComboBox::from_id_salt("bio-native-output")
                .selected_text(current.trim())
                .show_ui(ui, |ui| {
                    for mode in [" Enabled", "Disabled"] {
                        if ui.selectable_label(current == mode, mode.trim()).clicked() {
                            settings["NativeOutput"] = json!(mode);
                        }
                    }
                });
        });
        for (key, label, default) in [
            (
                "HideNotPresent",
                "Hide biologicals confirmed as not present",
                false,
            ),
            (
                "ShowBodyFields",
                "Extend output to include body details",
                false,
            ),
            ("ExpandOutput", "Expand output to window size", true),
            ("HideAutoscans", "Hide Autoscanned bodies", false),
            (
                "HideAutoscansNoPredictions",
                "Hide Autoscanned bodies with no predictions",
                true,
            ),
            ("ShowHeaders", "Show Headers", true),
            ("ShowTotal", "Show Unsold Data Total", false),
            ("ShowSplitters", "Show Splitters", false),
        ] {
            check(ui, settings, key, label, default);
        }
        numeric(
            ui,
            settings,
            "MaximumSystemsReadAll",
            "Maximum number of systems to display after Read All",
            100.0,
            1.0,
            1_000_000.0,
            1.0,
        );
        ui.horizontal(|ui| {
            ui.label("Button Size (requires restart)");
            let current = settings["ButtonSize"]
                .as_str()
                .unwrap_or("  Normal")
                .to_owned();
            egui::ComboBox::from_id_salt("bio-button-size")
                .selected_text(current.trim())
                .show_ui(ui, |ui| {
                    for size in [
                        "    Smaller",
                        "   Small",
                        "  Normal",
                        " Large",
                        "Extra Large",
                    ] {
                        if ui.selectable_label(current == size, size.trim()).clicked() {
                            settings["ButtonSize"] = json!(size);
                        }
                    }
                });
        });
    });
    group(ui, "♦ Valuable Bodies ♦", |ui| {
        numeric(
            ui,
            settings,
            "MinBodyValueMillions",
            "Minimum Base Value Over (millions)",
            20.0,
            0.0,
            100.0,
            0.5,
        );
        numeric(
            ui,
            settings,
            "MinIndividualBiologicalValueMillions",
            "Individual Biological Value Over (millions)",
            12.0,
            0.0,
            20.0,
            0.5,
        );
    });
    group(ui, "♦ Advanced ♦", |ui| {
        ui.label("Body to produce extra debug for");
        let mut debug_body = settings["DebugBody"].as_str().unwrap_or("").to_owned();
        if ui.text_edit_singleline(&mut debug_body).changed() {
            settings["DebugBody"] = json!(debug_body);
        }
        if ui.button("Wipe current commander unsold data").clicked() {
            forms.wipe = true;
        }
        check(
            ui,
            settings,
            "VerboseLogging",
            "Enable Verbose Logging",
            false,
        );
        check(
            ui,
            settings,
            "SendDebugData",
            "Send debug data to improve BioInsights",
            true,
        );
    });
    let ctx = ui.ctx().clone();
    if forms.missing_overlay {
        let mut open = true;
        egui::Window::new("Error")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(&ctx, |ui| {
                ui.label("Stellar Overlay not detected");
                if ui.button("OK").clicked() {
                    forms.missing_overlay = false;
                }
            });
        if !open {
            forms.missing_overlay = false;
        }
    }
    if let Some(form) = forms.form {
        let (title, size) = match form {
            Form::Notifications => ("BioInsights Notification Settings", [652.0, 365.0]),
            Form::Theming => ("BioInsights - Theming", [414.0, 131.0]),
            Form::StellarOverlay => ("BioInsights - Stellar Overlay Settings", [874.0, 415.0]),
            Form::Popout => ("BioInsights - Popout Settings", [701.0, 259.0]),
        };
        let mut open = true;
        ctx.show_viewport_immediate(
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
                                Form::Notifications => notifications(ui, &mut forms.draft),
                                Form::Theming => theming(ui, &mut forms.draft, &mut forms.colours),
                                Form::StellarOverlay => {
                                    stellar(ui, &mut forms.draft, overlay_defaults, &mut actions)
                                }
                                Form::Popout => popout(ui, &mut forms.draft),
                            }
                            ui.add_space(8.0);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .button(if form == Form::Notifications {
                                            "Apply and Save"
                                        } else {
                                            "Save"
                                        })
                                        .clicked()
                                    {
                                        *settings = forms.draft.clone();
                                        save(&mut actions);
                                        open = false;
                                    }
                                    if form == Form::Theming && ui.button("Apply").clicked() {
                                        *settings = forms.draft.clone();
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
            forms.form = None;
            forms.colours = false;
        }
    }
    if forms.colours {
        let mut open = true;
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("BioInsights - Custom Colours"),
            egui::ViewportBuilder::default()
                .with_title("BioInsights - Custom Colours")
                .with_inner_size([480.0, 600.0]),
            |ctx, _| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .show(ui, |ui| custom_colours(ui, &mut forms.draft));
                    if ui.button("Save").clicked() {
                        open = false;
                    }
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    open = false;
                }
            },
        );
        forms.colours = open;
    }
    if forms.wipe {
        let mut open = true;
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("bio-wipe-confirmation"),
            egui::ViewportBuilder::default()
                .with_title("Are you sure?")
                .with_inner_size([390.0, 110.0])
                .with_resizable(false),
            |ctx, _| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("Delete all unsold data for current commander?");
                    if let Some(commander) = summary["Commander"].as_str() {
                        ui.label(commander);
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            actions.push(
                                json!({"command":"WipeUnsoldData","payload":{"Confirmed":true}}),
                            );
                            open = false;
                        }
                        if ui.button("No").clicked() {
                            open = false;
                        }
                    });
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    open = false;
                }
            },
        );
        forms.wipe = open;
    }
    ui.ctx().data_mut(|d| d.insert_temp(id, forms));
    actions
}

fn notifications(ui: &mut egui::Ui, settings: &mut Value) {
    if !settings["Notifications"].is_object() {
        settings["Notifications"] = json!({});
    }
    egui::Grid::new("bio-source-notification-matrix")
        .num_columns(4)
        .striped(true)
        .show(ui, |ui| {
            for label in ["Notification", "Popup", "Voice", "Plugin"] {
                ui.strong(label);
            }
            ui.end_row();
            for (key, label) in [
                ("PersonalCodex", "Possible personal Codex entries"),
                ("NewGalactic", "Possible new Galactic Codex entries"),
                ("Marked", "Possible marked species"),
                ("HighValue", "High Value bodies or individual species"),
                (
                    "OnScanDetails",
                    "Value and colony distance details on species confirmation",
                ),
                (
                    "NonAtmosphericBodyWithBios",
                    "Non-atmospheric landable bodies with biologicals",
                ),
                (
                    "AtmosphericBodyWithBios",
                    "Atmospheric landable bodies with biologicals",
                ),
                ("AtmosphericBody", "Atmospheric landable bodies"),
                ("Distance", "Distance travelled notifications"),
                ("Samples", "Samples taken"),
                ("RadiantStarUpdates", "Radiant Star changes"),
            ] {
                ui.label(label);
                if !settings["Notifications"][key].is_object() {
                    settings["Notifications"][key] =
                        json!({"Popup":true,"Voice":true,"Plugin":false});
                }
                for channel in ["Popup", "Voice", "Plugin"] {
                    ui.push_id((key, channel), |ui| {
                        check(
                            ui,
                            &mut settings["Notifications"][key],
                            channel,
                            "",
                            channel != "Plugin",
                        )
                    });
                }
                ui.end_row();
            }
        });
}

fn theming(ui: &mut egui::Ui, settings: &mut Value, colours: &mut bool) {
    if !settings["ThemeSettings"].is_object() {
        settings["ThemeSettings"] = json!({"ThemeIndex":1,"ThemeMode":0});
    }
    ui.horizontal(|ui| {
        ui.label("Theme:");
        let index = settings["ThemeSettings"]["ThemeIndex"]
            .as_u64()
            .unwrap_or(1);
        egui::ComboBox::from_id_salt("bio-source-theme")
            .selected_text(if index == 0 {
                "Original"
            } else {
                "Elite Orange"
            })
            .show_ui(ui, |ui| {
                for (i, label) in ["Original", "Elite Orange"].into_iter().enumerate() {
                    if ui.selectable_label(index == i as u64, label).clicked() {
                        settings["ThemeSettings"]["ThemeIndex"] = json!(i);
                        settings["Theme"] = json!(label);
                    }
                }
            });
    });
    ui.horizontal(|ui| {
        let mut custom = settings["ThemeSettings"]["ThemeMode"].as_u64() == Some(1);
        if ui.checkbox(&mut custom, "Use Custom Colours").changed() {
            settings["ThemeSettings"]["ThemeMode"] = json!(u8::from(custom));
        }
        if ui.button("Edit").clicked() {
            *colours = true;
        }
    });
}

fn custom_colours(ui: &mut egui::Ui, settings: &mut Value) {
    let theme: Value = serde_json::from_str(if settings["ThemeSettings"]["ThemeIndex"] == 0 {
        include_str!("../../observatory-plugins/src/bioinsights/theme.json")
    } else {
        include_str!("../../observatory-plugins/src/bioinsights/theme-orange.json")
    })
    .expect("installed BioInsights theme");
    for (key, default) in theme["Colors"].as_object().unwrap() {
        let current = settings["ThemeSettings"]["Colors"]
            .get(key)
            .unwrap_or(default);
        let mut colour = [
            current["R"].as_u64().unwrap_or(0) as u8,
            current["G"].as_u64().unwrap_or(0) as u8,
            current["B"].as_u64().unwrap_or(0) as u8,
        ];
        ui.horizontal(|ui| {
            if ui.color_edit_button_srgb(&mut colour).changed() {
                if !settings["ThemeSettings"]["Colors"].is_object() {
                    settings["ThemeSettings"]["Colors"] = json!({});
                }
                settings["ThemeSettings"]["Colors"][key] =
                    json!({"R":colour[0],"G":colour[1],"B":colour[2]});
            }
            ui.label(key);
        });
    }
    if ui.button("Reset").clicked() {
        settings["ThemeSettings"]["Colors"] = json!({});
    }
}

fn popout(ui: &mut egui::Ui, settings: &mut Value) {
    ui.group(|ui| {
        ui.strong("Popout Display Options");
        check(
            ui,
            settings,
            "PopoutAlwaysShow",
            "Always Show Popout when Popped Out",
            true,
        );
        ui.columns(2, |columns| {
            for (index, key, label, default) in [
                (0, "PopoutShowNoFocus", "Show Popout on Main Window", true),
                (0, "PopoutShowLeftPanel", "Show Popout on Left Panel", true),
                (
                    0,
                    "PopoutShowRightPanel",
                    "Show Popout on Right Panel",
                    true,
                ),
                (
                    0,
                    "PopoutShowRolePanel",
                    "Show Popout on Role Panel (Bottom)",
                    true,
                ),
                (1, "PopoutShowSysMap", "Show Popout on System Map", true),
                (1, "PopoutShowFSSMode", "Show Popout in FSS Mode", true),
                (1, "PopoutShowDSSMode", "Show Popout in DSS mode", true),
            ] {
                check(&mut columns[index], settings, key, label, default);
            }
        });
        check(
            ui,
            settings,
            "PopoutHideUnlessSupercruiseOrSrv",
            "Hide Popout when not in Supercruise or SRV",
            false,
        );
        check(
            ui,
            settings,
            "PopoutHidePopulated",
            "Hide Popout in populated systems",
            false,
        );
        numeric(
            ui,
            settings,
            "PopoutOpacity",
            "Opacity [%] (10-100)",
            100.0,
            10.0,
            100.0,
            1.0,
        );
    });
}

fn stellar(
    ui: &mut egui::Ui,
    settings: &mut Value,
    overlay_defaults: &Value,
    actions: &mut Vec<Value>,
) {
    ui.strong("Insight Overlay");
    ui.columns(3, |columns| {
        for (index, dss, title, enable, x, y) in [
            (
                0,
                false,
                "FSS Insight Overlay Settings",
                "StellarOverlaySendFSSNotification",
                "FSSNotificationXPos",
                "FSSNotificationYPos",
            ),
            (
                1,
                true,
                "DSS Insight Overlay Settings",
                "StellarOverlaySendDSSNotification",
                "DSSNotificationXPos",
                "DSSNotificationYPos",
            ),
        ] {
            let ui = &mut columns[index];
            ui.strong(title);
            check(ui, settings, enable, "Enabled", true);
            numeric(
                ui,
                settings,
                x,
                "Horizontal Position [%] (1-100):",
                2.0,
                0.0,
                100.0,
                1.0,
            );
            numeric(
                ui,
                settings,
                y,
                "Vertical Position [%] (1-100):",
                if dss { 98.0 } else { 2.0 },
                0.0,
                100.0,
                1.0,
            );
            if ui.button("Test").clicked() {
                actions.push(
                    json!({"command":"TestOverlay","payload":{"DSS":dss,"Settings":settings}}),
                );
            }
        }
        let ui = &mut columns[2];
        ui.strong("General Settings");
        numeric(
            ui,
            settings,
            "FSSNotificationFontSize",
            "Font Size (6-72):",
            12.0,
            6.0,
            72.0,
            1.0,
        );
        numeric(
            ui,
            settings,
            "FSSNotificationOpacity",
            "Opacity [%] (10-100):",
            80.0,
            10.0,
            100.0,
            1.0,
        );
        numeric(
            ui,
            settings,
            "FSSNotificationBorderWidth",
            "Border Width (0-20):",
            2.0,
            0.0,
            20.0,
            1.0,
        );
        numeric(
            ui,
            settings,
            "FSSNotificationTTL",
            "TTL [ms] (1000-15000)",
            6000.0,
            1000.0,
            15000.0,
            1.0,
        );
    });
    ui.separator();
    ui.columns(2, |columns| {
        let ui = &mut columns[0];
        ui.strong("Custom Notification Colours");
        check(
            ui,
            settings,
            "UseCustomNotificationColours",
            "Enabled",
            false,
        );
        for (key, label) in [
            ("TitleColorHex", "Title Colour"),
            ("ColorHex", "Text Colour"),
            ("BorderColorHex", "Border Colour"),
            ("BackgroundColorHex", "Background Colour"),
        ] {
            let raw = settings[key]
                .as_str()
                .or_else(|| overlay_defaults[key].as_str())
                .unwrap_or("FF000000");
            let argb = u32::from_str_radix(raw.trim_start_matches('#'), 16).unwrap_or(0xff000000);
            let mut colour = Color32::from_rgba_unmultiplied(
                (argb >> 16) as u8,
                (argb >> 8) as u8,
                argb as u8,
                (argb >> 24) as u8,
            );
            ui.horizontal(|ui| {
                if ui.color_edit_button_srgba(&mut colour).changed() {
                    let [red, green, blue, alpha] = colour.to_srgba_unmultiplied();
                    settings[key] =
                        json!(format!("{:02X}{:02X}{:02X}{:02X}", alpha, red, green, blue));
                }
                ui.label(label);
            });
        }
        ui.horizontal(|ui| {
            if ui.button("Test").clicked() {
                actions.push(
                    json!({"command":"TestOverlay","payload":{"DSS":false,"Settings":settings}}),
                );
            }
            let keys = [
                "TitleColorHex",
                "ColorHex",
                "BorderColorHex",
                "BackgroundColorHex",
            ];
            if ui
                .add_enabled(
                    keys.iter().all(|key| overlay_defaults[key].is_string()),
                    egui::Button::new("Reset"),
                )
                .clicked()
            {
                for key in keys {
                    settings[key] = overlay_defaults[key].clone();
                }
            }
        });
        let ui = &mut columns[1];
        ui.strong("Radar");
        check(ui, settings, "StellarOverlayUseRadar", "Enabled", true);
        ui.label(
            "These settings require the Stellar Overlay plugin to be installed to have any effect",
        );
    });
}
