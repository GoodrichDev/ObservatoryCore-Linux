//! Installed HecateSettings and FormServerSettings actions.
use eframe::egui::{self, Id};
use serde_json::{json, Value};

#[derive(Clone, Default)]
struct State {
    server: bool,
    reset: bool,
    draft: Value,
}
fn check(ui: &mut egui::Ui, value: &mut Value, key: &str, label: &str, default: bool) {
    let mut current = value[key].as_bool().unwrap_or(default);
    if ui.checkbox(&mut current, label).changed() {
        value[key] = json!(current);
    }
}
fn field(ui: &mut egui::Ui, value: &mut Value, key: &str, label: &str) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut text = value[key].as_str().map(str::to_owned).unwrap_or_else(|| {
            if value[key].is_null() {
                String::new()
            } else {
                value[key].to_string()
            }
        });
        if ui.text_edit_singleline(&mut text).changed() {
            value[key] = json!(text);
        }
    });
}
fn validated(draft: &Value) -> Value {
    let mut result = draft.clone();
    let port = draft["ServerSettings"]["ServerPort"]
        .as_u64()
        .or_else(|| {
            draft["ServerSettings"]["ServerPort"]
                .as_str()
                .and_then(|s| s.trim().parse().ok())
        })
        .filter(|p| (8000..59000).contains(p))
        .unwrap_or(58080);
    result["ServerSettings"]["ServerPort"] = json!(port);
    let ip = draft["ServerSettings"]["ServerLANIPAddress"]
        .as_str()
        .unwrap_or("");
    if !ip.is_empty() && ip.parse::<std::net::IpAddr>().is_err() {
        result["ServerSettings"]["ServerLANIPAddress"] = json!("");
    }
    result
}
pub fn show(ui: &mut egui::Ui, value: &mut Value) -> Vec<Value> {
    let id = Id::new("Hecate original settings forms");
    let mut state = ui
        .ctx()
        .data(|d| d.get_temp::<State>(id))
        .unwrap_or_default();
    let mut actions = Vec::new();
    if ui.button("Reset Journal Database").clicked() {
        state.reset = true;
    }
    ui.separator();
    ui.strong("♦ Web Server ♦");
    if ui.button("Open Web Server Settings").clicked() {
        state.server = true;
        state.draft = value.clone();
    }
    ui.separator();
    ui.strong("♦ Automatic Updates ♦");
    check(
        ui,
        value,
        "InstallUpdates",
        "Automatically install updates",
        true,
    );
    check(
        ui,
        value,
        "InstallBetas",
        "Install beta releases *NOT RECOMMENDED*",
        false,
    );
    ui.separator();
    ui.strong("♦ Advanced ♦");
    check(ui, value, "VerboseLogging", "Verbose Logging", false);
    if state.server {
        let mut close = false;
        ui.ctx().show_viewport_immediate(
            egui::ViewportId::from_hash_of("Hecate - Web Server Settings"),
            egui::ViewportBuilder::default()
                .with_title("Hecate - Web Server Settings")
                .with_inner_size([468.0, 263.0])
                .with_resizable(false),
            |ctx, _| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().inner_margin(12))
                    .show(ctx, |ui| {
                        ui.strong("Web Server");
                        check(
                            ui,
                            &mut state.draft["ServerSettings"],
                            "EnableWebServer",
                            "Enable Web Server",
                            false,
                        );
                        field(
                            ui,
                            &mut state.draft["ServerSettings"],
                            "ServerLANIPAddress",
                            "LAN IP Address",
                        );
                        field(
                            ui,
                            &mut state.draft["ServerSettings"],
                            "ServerPort",
                            "Port [8000–59000]",
                        );
                        if ui.button("Save").clicked() {
                            *value = validated(&state.draft);
                            actions.push(json!({"command":"SaveSettings","payload":{}}));
                            close = true;
                        }
                    });
                if ctx.input(|i| i.viewport().close_requested()) {
                    close = true;
                }
            },
        );
        if close {
            state.server = false;
        }
    }
    if state.reset {
        let mut close = false;
        ui.ctx().show_viewport_immediate(egui::ViewportId::from_hash_of("Hecate reset journal confirmation"),egui::ViewportBuilder::default().with_title("Are you sure?").with_inner_size([350.0,100.0]).with_resizable(false),|ctx,_|{
            egui::CentralPanel::default().show(ctx,|ui|{
                ui.label("Reset journal database?");
                ui.horizontal(|ui|{
                    if ui.button("OK").clicked(){actions.push(json!({"command":"ResetJournalDatabase","payload":{"Confirmed":true}}));close=true;}
                    if ui.button("Cancel").clicked(){close=true;}
                });
            });
            if ctx.input(|i|i.viewport().close_requested()){close=true;}
        });
        if close {
            state.reset = false;
        }
    }
    ui.ctx().data_mut(|d| d.insert_temp(id, state));
    actions
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn server_setter_and_form_validation_match_installed_bounds() {
        for (input, expected) in [(7999, 58080), (8000, 8000), (58999, 58999), (59000, 58080)] {
            assert_eq!(
                validated(
                    &json!({"ServerSettings":{"ServerPort":input,"ServerLANIPAddress":"invalid"}})
                )["ServerSettings"],
                json!({"ServerPort":expected,"ServerLANIPAddress":""})
            );
        }
        assert_eq!(
            validated(
                &json!({"ServerSettings":{"ServerPort":"12345","ServerLANIPAddress":"127.0.0.1"}})
            )["ServerSettings"],
            json!({"ServerPort":12345,"ServerLANIPAddress":"127.0.0.1"})
        );
    }
}
