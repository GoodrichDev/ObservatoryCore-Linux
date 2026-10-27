use crate::runtime::{Command, Controller, Snapshot};
use anyhow::Result;
use eframe::egui::{self, Color32, RichText};
use egui_extras::{Column, TableBuilder};
use observatory_io::{ConfigPaths, Settings};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, HashSet},
    io::Read,
    path::PathBuf,
    process::{Command as ProcessCommand, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

#[cfg(test)]
#[path = "ui_bench.rs"]
mod ui_bench;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
struct PluginWindow {
    open: bool,
    position: Option<[f32; 2]>,
    size: [f32; 2],
    filter: String,
    sort: usize,
    descending: bool,
    on_top: bool,
    title_bar: bool,
    hide_buttons: bool,
}
impl Default for PluginWindow {
    fn default() -> Self {
        Self {
            open: false,
            position: None,
            size: [800.0, 450.0],
            filter: String::new(),
            sort: usize::MAX,
            descending: false,
            on_top: false,
            title_bar: true,
            hide_buttons: false,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct CustomTheme {
    base: String,
    background: [u8; 4],
    panel: [u8; 4],
    text: [u8; 4],
    accent: [u8; 4],
    selection: [u8; 4],
    widgets: [u8; 4],
    link: [u8; 4],
}
impl CustomTheme {
    #[cfg(test)]
    fn from_base(base: &str) -> Self {
        let visuals = original_visuals(base);
        Self {
            base: if visuals.dark_mode { "dark" } else { "light" }.into(),
            background: visuals.extreme_bg_color.to_array(),
            panel: visuals.panel_fill.to_array(),
            text: visuals.text_color().to_array(),
            accent: original_tab_colors(base).1.to_array(),
            selection: visuals.selection.bg_fill.to_array(),
            widgets: visuals.widgets.inactive.bg_fill.to_array(),
            link: visuals.hyperlink_color.to_array(),
        }
    }
    fn visuals(&self) -> egui::Visuals {
        let color =
            |rgba: [u8; 4]| Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3]);
        let mut visuals = original_visuals(&self.base);
        visuals.extreme_bg_color = color(self.background);
        visuals.panel_fill = color(self.panel);
        visuals.window_fill = color(self.panel);
        visuals.faint_bg_color = color(self.panel).gamma_multiply(0.8);
        visuals.override_text_color = Some(color(self.text));
        visuals.hyperlink_color = color(self.link);
        visuals.widgets.noninteractive.bg_fill = color(self.panel);
        visuals.widgets.inactive.bg_fill = color(self.widgets);
        visuals.widgets.hovered.bg_fill = color(self.widgets).gamma_multiply(1.25);
        visuals.widgets.active.bg_fill = color(self.accent);
        visuals.selection.bg_fill = color(self.selection);
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, color(self.text));
        visuals
    }
}
#[derive(Clone, Default)]
struct AudioChoices {
    loaded: bool,
    voices: Vec<(String, String)>,
    devices: Vec<(String, String)>,
    errors: Vec<String>,
}

pub struct ObservatoryApp {
    game_focus: crate::game_focus::GameFocus,
    bio_browser_mode: bool,
    bio_output_enabled: bool,
    browser_parent: Option<u32>,
    bio_browser: Option<Result<crate::embedded_browser::Browser, String>>,
    pending_bio_actions: Vec<Value>,
    browser_chrome_actions: Vec<String>,
    root_size: Option<[f32; 2]>,
    root_position: Option<[f32; 2]>,
    file_dialog: Option<crate::file_dialog::FileDialog>,
    grid_columns: BTreeMap<String, Vec<String>>,
    grid_exports: BTreeMap<String, Arc<crate::runtime::GridExport>>,
    pending_export: Option<Arc<crate::runtime::GridExport>>,
    controller: Controller,
    settings: Settings,
    paths: ConfigPaths,
    tab: String,
    journal_path: String,
    export_path: String,
    filter: String,
    sort: usize,
    descending: bool,
    plugin_windows: BTreeMap<String, PluginWindow>,
    close_plugin_popout: Option<String>,
    tab_order: Vec<String>,
    audio_choices: Arc<Mutex<AudioChoices>>,
    frames: usize,
    smoke_frames: Option<usize>,
    settings_error: Option<String>,
    screenshot: Option<PathBuf>,
    screenshot_viewport: Option<String>,
    capture_ready_at: Instant,
    settings_open: bool,
    settings_plugin: Option<String>,
    about_open: bool,
    about_plugin: Option<String>,
    filter_open: bool,
    monitor_active: bool,
    message_boxes: std::collections::VecDeque<Value>,
}
impl ObservatoryApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        settings: Settings,
        paths: ConfigPaths,
        smoke_frames: Option<usize>,
        screenshot: Option<PathBuf>,
    ) -> Result<Self> {
        crate::passive_windows::initialize(cc);
        apply_theme(&cc.egui_ctx, &settings);
        cc.egui_ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("Core source column sizing"),
                Arc::new(Mutex::new(initial_column_sizing(&settings))),
            )
        });
        crate::stellar_ui::prepare_font(&cc.egui_ctx, &settings.plugins["StellarOverlay"]);
        let journal_path = settings
            .journal_dir
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let export_path = paths
            .data_dir
            .join("observatory-export.csv")
            .display()
            .to_string();
        let controller = Controller::spawn(settings.clone(), paths.clone())?;
        controller.snapshot.lock().unwrap().repaint = Some(cc.egui_ctx.clone());
        let tab_order: Vec<String> = settings
            .extra
            .get("tab_order")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .or_else(|| {
                cc.storage
                    .and_then(|s| s.get_string("tab_order"))
                    .and_then(|s| serde_json::from_str(&s).ok())
            })
            .unwrap_or_default();
        let tab = settings
            .extra
            .get("last_tab")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| {
                settings
                    .extra
                    .get("LastTabIndex")
                    .and_then(Value::as_u64)
                    .and_then(|i| tab_order.get(i as usize).cloned())
            })
            .or_else(|| cc.storage.and_then(|s| s.get_string("tab")))
            .unwrap_or_else(|| "Core".into());
        let mut plugin_windows: BTreeMap<String, PluginWindow> = settings
            .extra
            .get("plugin_windows")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .or_else(|| {
                cc.storage
                    .and_then(|s| s.get_string("plugin_windows"))
                    .and_then(|s| serde_json::from_str(&s).ok())
            })
            .unwrap_or_default();
        for (key, window) in &mut plugin_windows {
            if key.starts_with("@custom/") {
                window.open = false;
            }
        }
        let audio_choices = Arc::new(Mutex::new(AudioChoices::default()));
        enumerate_audio(audio_choices.clone());
        Ok(Self {
            bio_output_enabled: !settings.plugins["BioInsights"]["NativeOutput"]
                .as_str()
                .unwrap_or(" Enabled")
                .starts_with("Disabled"),
            bio_browser_mode: settings
                .extra
                .get("bioinsights_renderer")
                .and_then(Value::as_str)
                == Some("webkit"),
            browser_parent: crate::embedded_browser::Browser::root_window(cc),
            bio_browser: None,
            pending_bio_actions: Vec::new(),
            browser_chrome_actions: Vec::new(),
            file_dialog: None,
            grid_columns: Default::default(),
            grid_exports: Default::default(),
            pending_export: None,
            root_size: None,
            root_position: None,
            game_focus: crate::game_focus::GameFocus::start(),
            controller,
            settings,
            paths,
            tab,
            journal_path,
            export_path,
            filter: String::new(),
            sort: usize::MAX,
            descending: false,
            plugin_windows,
            close_plugin_popout: None,
            tab_order,
            audio_choices,
            frames: 0,
            smoke_frames,
            settings_error: crate::passive_windows::capability_error(),
            screenshot,
            screenshot_viewport: None,
            capture_ready_at: Instant::now(),
            settings_open: false,
            settings_plugin: None,
            about_open: false,
            about_plugin: None,
            filter_open: false,
            monitor_active: false,
            message_boxes: Default::default(),
        })
    }

    pub fn set_initial_view(
        &mut self,
        view: Option<String>,
        detach: Option<String>,
        screenshot_viewport: Option<String>,
    ) {
        if let Some(view) = view {
            if view == "Settings" {
                self.settings_open = true;
                self.settings_plugin = None;
                self.tab = "Core".into();
            } else {
                self.tab = view;
            }
        }
        if let Some(detach) = detach {
            self.plugin_windows.entry(detach).or_default().open = true;
        }
        self.screenshot_viewport = screenshot_viewport;
        if self.settings_open && self.screenshot_viewport.is_none() && self.screenshot.is_some() {
            self.screenshot_viewport = Some("Core Settings".into());
        }
    }
    pub fn defer_screenshot(&mut self, delay_ms: u64) {
        self.capture_ready_at = Instant::now() + Duration::from_millis(delay_ms.min(60_000));
    }
    fn export_plugin(&mut self, name: &str) {
        let format = self
            .settings
            .extra
            .get("ExportFormat")
            .or_else(|| {
                self.settings
                    .extra
                    .get("legacy_settings")
                    .and_then(|v| v.get("ExportFormat"))
            })
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let extension = if format == 1 { "xlsx" } else { "csv" };
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| self.paths.data_dir.clone());
        let documents = home.join("Documents");
        let home = if documents.is_dir() { documents } else { home };
        let path = home.join(format!(
            "Export-{name}-{}.{}",
            chrono::Local::now().format("%Y-%m-%dT%H%M"),
            extension
        ));
        self.pending_export = self.grid_exports.get(name).cloned();
        self.choose_file(crate::file_dialog::Purpose::Export(name.into()), &path);
    }
    fn choose_file(&mut self, purpose: crate::file_dialog::Purpose, initial: &std::path::Path) {
        if self.file_dialog.is_some() {
            return;
        }
        match crate::file_dialog::FileDialog::new(purpose, initial) {
            Ok(dialog) => self.file_dialog = Some(dialog),
            Err(error) => self.settings_error = Some(format!("File chooser: {error:#}")),
        }
    }
    fn open_settings(&mut self, name: Option<String>) {
        self.settings_open = true;
        self.settings_plugin = name;
    }
    fn plugin_context_menu(&mut self, ui: &mut egui::Ui, name: &str, show_popout: bool) {
        if name == "Core" {
            if ui.button("Core Settings").clicked() {
                self.open_settings(None);
                ui.close_menu();
            }
            if ui.button("About Core").clicked() {
                self.about_open = true;
                self.about_plugin = None;
                ui.close_menu();
            }
            if ui.button("Open Plugins Folder").clicked() {
                self.open_plugin_folder();
                ui.close_menu();
            }
        } else {
            if show_popout && ui.button(format!("Popout {name}")).clicked() {
                self.plugin_windows.entry(name.into()).or_default().open = true;
                ui.close_menu();
            }
            if ui.button(format!("{name} Settings")).clicked() {
                self.open_settings(Some(name.into()));
                ui.close_menu();
            }
            if ui.button(format!("About {name}")).clicked() {
                self.about_open = true;
                self.about_plugin = Some(name.into());
                ui.close_menu();
            }
            if ui.button(format!("Open {name} Data Folder")).clicked() {
                let folder = if matches!(
                    name,
                    "Explorer"
                        | "Botanist"
                        | "Communicator"
                        | "BioInsights"
                        | "Evaluator"
                        | "Hecate"
                        | "StellarOverlay"
                ) {
                    self.paths.data_dir.join(name)
                } else {
                    let id = self
                        .controller
                        .snapshot
                        .lock()
                        .unwrap()
                        .plugin_details
                        .get(name)
                        .and_then(|details| details["id"].as_str())
                        .unwrap_or(name)
                        .to_owned();
                    self.paths.data_dir.join("storage").join(id)
                };
                if let Err(error) = std::fs::create_dir_all(&folder) {
                    self.settings_error = Some(error.to_string());
                } else {
                    self.open_folder(&folder);
                }
                ui.close_menu();
            }
        }
    }
    fn open_plugin_folder(&mut self) {
        let folder = self.paths.plugin_dir.clone();
        self.open_folder(&folder);
    }
    fn open_folder(&mut self, folder: &std::path::Path) {
        let executable = if cfg!(windows) {
            "explorer"
        } else {
            "xdg-open"
        };
        match ProcessCommand::new(executable)
            .arg(folder)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(mut child) => {
                thread::spawn(move || {
                    let _ = child.wait();
                });
            }
            Err(error) => self.settings_error = Some(format!("Cannot open plugin folder: {error}")),
        }
    }
    fn core_page(&mut self, ui: &mut egui::Ui, snapshot: &Snapshot) {
        let available = ui.available_size();
        let list_height = (available.y - 62.0).max(50.0);
        egui::Frame::new()
            .stroke(egui::Stroke::new(1.0_f32, Color32::from_gray(105)))
            .inner_margin(4)
            .show(ui, |ui| {
                ui.set_min_size(egui::vec2((available.x - 10.0).max(100.0), list_height));
                ui.set_max_height(list_height);
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .max_height(list_height)
                    .show(ui, |ui| {
                        ui.label(RichText::new("Plugins").strong().size(15.6));
                        egui::Grid::new("original-plugin-list")
                            .spacing([16.0, 8.0])
                            .min_row_height(25.0)
                            .show(ui, |ui| {
                                for label in
                                    ["Name", "Author", "Type", "Version", "Status", "Enabled", ""]
                                {
                                    ui.label(RichText::new(label).strong());
                                }
                                ui.end_row();
                                for name in plugin_names(snapshot) {
                                    let details = snapshot.plugin_details.get(&name);
                                    let builtin = matches!(
                                        name.as_str(),
                                        "Explorer" | "Botanist" | "Communicator"
                                    );
                                    let full_name = details
                                        .and_then(|v| v["full_name"].as_str())
                                        .map(str::to_owned)
                                        .unwrap_or_else(|| {
                                            if builtin {
                                                format!("Observatory {name}")
                                            } else {
                                                name.clone()
                                            }
                                        });
                                    let author = details
                                        .and_then(|v| v["author"].as_str())
                                        .filter(|s| !s.is_empty())
                                        .unwrap_or(if builtin {
                                            "Vithigar"
                                        } else {
                                            "(not provided)"
                                        });
                                    let role = details.and_then(|v| v["role"].as_str()).unwrap_or(
                                        if name == "StellarOverlay" {
                                            "Notifier"
                                        } else {
                                            "Worker"
                                        },
                                    );
                                    let version = details
                                        .and_then(|v| v["version"].as_str())
                                        .or_else(|| {
                                            snapshot
                                                .extensions
                                                .iter()
                                                .find(|e| e.0 == name)
                                                .map(|e| e.1.as_str())
                                        })
                                        .unwrap_or(env!("CARGO_PKG_VERSION"));
                                    let status =
                                        details.and_then(|v| v["status"].as_str()).unwrap_or("OK");
                                    let failed = status == "Error"
                                        || details.is_some_and(|v| v["worker_healthy"] == false);
                                    ui.label(full_name);
                                    ui.label(author);
                                    ui.label(role);
                                    ui.label(version);
                                    ui.label(status);
                                    let mut disabled = self
                                        .settings
                                        .extra
                                        .get("disabled_plugins")
                                        .and_then(Value::as_array)
                                        .cloned()
                                        .unwrap_or_default();
                                    let mut enabled = !failed
                                        && !disabled.iter().any(|v| v.as_str() == Some(&name));
                                    if ui
                                        .add_enabled(!failed, egui::Checkbox::new(&mut enabled, ""))
                                        .changed()
                                    {
                                        disabled.retain(|v| v.as_str() != Some(&name));
                                        if !enabled {
                                            disabled.push(json!(name));
                                        }
                                        self.settings.extra.insert(
                                            "disabled_plugins".into(),
                                            Value::Array(disabled),
                                        );
                                        self.controller
                                            .send(Command::Configure(self.settings.clone()));
                                    }
                                    ui.add_enabled_ui(!failed, |ui| {
                                        ui.menu_button("▾", |ui| {
                                            self.plugin_context_menu(ui, &name, false)
                                        });
                                    });
                                    ui.end_row();
                                }
                            });
                    });
            });
        let controls = ui.available_rect_before_wrap();
        ui.scope_builder(egui::UiBuilder::new().max_rect(controls), |ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            let monitor = if self.monitor_active {
                "Active"
            } else {
                "Stopped"
            };
            ui.label(format!("Current Monitor Status: {monitor}"));
            ui.label(format!(
                "Last Journal Event Processed: {}",
                if snapshot.last_event.is_empty() {
                    "None"
                } else if crate::extensions::journal_type(&snapshot.last_event).0 == "JournalBase" {
                    "JournalBase"
                } else {
                    &snapshot.last_event
                }
            ));
            ui.label(format!(
                "Total Journal Lines Processed: {}",
                format_line_total(snapshot.total)
            ));
            if let Some(error) = crate::passive_windows::capability_error() {
                ui.colored_label(Color32::LIGHT_RED, error);
            }
        });
        let button_rect = egui::Rect::from_min_size(
            egui::pos2(
                (controls.max.x - 206.0).max(controls.min.x),
                controls.max.y - 23.0,
            ),
            egui::vec2(206.0, 23.0),
        );
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(button_rect)
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
            |ui| {
                if ui
                    .add_sized([100.0, 23.0], egui::Button::new("Core Settings"))
                    .clicked()
                {
                    self.open_settings(None);
                }
                if ui
                    .add_sized([100.0, 23.0], egui::Button::new("Plugin Folder"))
                    .clicked()
                {
                    self.open_plugin_folder();
                }
            },
        );
    }
    fn original_shell(&mut self, ctx: &egui::Context, snapshot: &Snapshot) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                if snapshot.read_progress.is_some() {
                    ui.disable();
                }
                let layout = original_layout(ui.max_rect());
                ui.scope_builder(egui::UiBuilder::new().max_rect(layout.tabs), |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);
                    let mut tab_order =
                        normalize_tab_order(&self.tab_order, &available_tabs(snapshot));
                    let mut swap = None;
                    let tabs = ui.horizontal_wrapped(|ui| {
                        for (index, name) in tab_order.iter().enumerate() {
                            let selected = self.tab == *name;
                            let colors = tab_colors(&self.settings);
                            let response = ui.add(
                                egui::Button::new(name)
                                    .fill(if selected { colors.1 } else { colors.0 })
                                    .corner_radius(0.0)
                                    .min_size(egui::vec2(0.0, 24.0))
                                    .sense(egui::Sense::click_and_drag()),
                            );
                            if response.clicked() {
                                self.tab = name.clone();
                                self.filter.clear();
                                self.sort = usize::MAX;
                                self.descending = false;
                                self.filter_open = false;
                            }
                            if response.drag_started() {
                                ui.ctx().data_mut(|data| {
                                    data.insert_temp(egui::Id::new("dragged-original-tab"), index)
                                });
                            }
                            if response.contains_pointer()
                                && ui.input(|input| input.pointer.primary_down())
                            {
                                if let Some(from) = ui.ctx().data(|data| {
                                    data.get_temp::<usize>(egui::Id::new("dragged-original-tab"))
                                }) {
                                    if from != index {
                                        swap = Some((from, index));
                                    }
                                }
                            }
                            response.context_menu(|ui| self.plugin_context_menu(ui, name, true));
                        }
                    });
                    if let Some((from, to)) = swap {
                        tab_order.swap(from, to);
                        self.tab_order = tab_order.clone();
                        self.settings
                            .extra
                            .insert("tab_order".into(), json!(tab_order));
                        ui.ctx().data_mut(|data| {
                            data.insert_temp(egui::Id::new("dragged-original-tab"), to)
                        });
                    }
                    if ui.input(|input| input.pointer.any_released()) {
                        ui.ctx().data_mut(|data| {
                            data.remove::<usize>(egui::Id::new("dragged-original-tab"))
                        });
                    }
                    let content = egui::Rect::from_min_max(
                        egui::pos2(layout.tabs.min.x, tabs.response.rect.max.y),
                        layout.tabs.max,
                    );
                    ui.scope_builder(egui::UiBuilder::new().max_rect(content), |ui| {
                        ui.set_clip_rect(ui.clip_rect().intersect(content));
                        let tab = self.tab.clone();
                        egui::Frame::new()
                            .stroke(egui::Stroke::new(1.0_f32, Color32::from_gray(105)))
                            .inner_margin(if tab == "Core" { 6 } else { 3 })
                            .show(ui, |ui| {
                                ui.set_min_size(ui.available_size());
                                self.render_tab(ui, ctx, snapshot, &tab);
                            });
                    });
                });
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(layout.bottom)
                        .layout(egui::Layout::right_to_left(egui::Align::Center)),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        if ui
                            .add_sized([75.0, 23.0], egui::Button::new("Read All"))
                            .clicked()
                        {
                            self.controller.send(Command::Replay);
                        }
                        let label = if self.monitor_active {
                            "Stop Monitor"
                        } else {
                            "Start Monitor"
                        };
                        if ui
                            .add_sized([97.0, 23.0], egui::Button::new(label))
                            .clicked()
                        {
                            if self.monitor_active {
                                self.controller.send(Command::Stop);
                            } else {
                                self.controller.send(Command::Start);
                            }
                            self.monitor_active = !self.monitor_active;
                        }
                        let has_grid = is_basic_ui(&self.tab);
                        if ui
                            .add_enabled(
                                has_grid,
                                egui::Button::new("Clear").min_size(egui::vec2(75.0, 23.0)),
                            )
                            .clicked()
                        {
                            self.controller.send(Command::Clear(self.tab.clone()));
                        }
                        if ui
                            .add_enabled(
                                has_grid,
                                egui::Button::new("Export").min_size(egui::vec2(75.0, 23.0)),
                            )
                            .on_hover_text(&self.export_path)
                            .clicked()
                        {
                            self.export_plugin(&self.tab.clone());
                        }
                    },
                );
                let about = egui::Rect::from_min_size(
                    layout.bottom.min + egui::vec2(4.0, -5.0),
                    egui::vec2(42.0, 28.0),
                );
                if ui
                    .put(about, egui::Button::new(RichText::new("ℹ").size(18.0)))
                    .on_hover_text("About Elite Observatory Core")
                    .clicked()
                {
                    self.about_open = true;
                    self.about_plugin = None;
                }
            });
    }
    fn screenshot_child(&mut self, ctx: &egui::Context, name: &str) {
        if self.frames < 3
            || Instant::now() < self.capture_ready_at
            || self.screenshot_viewport.as_deref() != Some(name)
        {
            return;
        }
        if let Some(path) = self.screenshot.take() {
            ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("viewport-capture"),
            ))
            .add(egui::PaintCallback {
                rect: ctx.screen_rect(),
                callback: Arc::new(eframe::egui_glow::CallbackFn::new(move |info, painter| {
                    let image = painter.read_screen_rgba(info.screen_size_px);
                    let pixels = image
                        .pixels
                        .iter()
                        .flat_map(|p| p.to_array())
                        .collect::<Vec<_>>();
                    if let Err(error) = image::save_buffer(
                        &path,
                        &pixels,
                        image.width() as u32,
                        image.height() as u32,
                        image::ColorType::Rgba8,
                    ) {
                        eprintln!("save viewport screenshot: {error}");
                    }
                })),
            });
        }
    }
    fn settings_window(&mut self, ctx: &egui::Context, snapshot: &Snapshot) {
        if !self.settings_open {
            return;
        }
        let plugin = self.settings_plugin.clone();
        let title = plugin
            .as_ref()
            .map(|name| format!("{name} Settings"))
            .unwrap_or_else(|| "Core Settings".into());
        if plugin.is_none() {
            let mut close = false;
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of(("settings-window", &title)),
                egui::ViewportBuilder::default()
                    .with_title(&title)
                    .with_inner_size([492.0, 735.0])
                    .with_resizable(false),
                |ctx, _| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::NONE)
                        .show(ctx, |ui| self.settings(ui, ctx, snapshot));
                    self.screenshot_child(ctx, &title);
                    close = ctx.input(|i| i.viewport().close_requested());
                },
            );
            if close {
                self.settings_open = false;
            }
            return;
        }
        let mut open = true;
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of(("settings-window", &title)),
            egui::ViewportBuilder::default()
                .with_title(&title)
                .with_inner_size(if plugin.is_some() {
                    [600.0, 650.0]
                } else {
                    [492.0, 735.0]
                })
                .with_min_inner_size([420.0, 250.0]),
            |ctx, _| {
                egui::TopBottomPanel::bottom("settings-close")
                    .frame(egui::Frame::new().inner_margin(12))
                    .show(ctx, |ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add_sized([75.0, 23.0], egui::Button::new("Close"))
                                .clicked()
                            {
                                open = false;
                            }
                        });
                    });
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().inner_margin(12))
                    .show(ctx, |ui| {
                        if let Some(name) = &plugin {
                            let overlay_defaults = self.settings.plugins["StellarOverlay"].clone();
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                if let Some(value) = self.settings.plugins.get_mut(name) {
                                    let actions = if name == "StellarOverlay" {
                                        crate::stellar_ui::settings(ui, value)
                                    } else if name == "Evaluator" {
                                        crate::evaluator_ui::settings(
                                            ui,
                                            value,
                                            snapshot.eval_summary.as_ref(),
                                            &overlay_defaults,
                                        )
                                    } else if name == "Hecate" {
                                        crate::hecate_settings::show(ui, value)
                                    } else if name == "BioInsights" {
                                        crate::bioinsights_settings::show(
                                            ui,
                                            value,
                                            snapshot.bio_summary.as_ref(),
                                            &overlay_defaults,
                                        )
                                    } else {
                                        value_editor(ui, "Settings", value);
                                        Vec::new()
                                    };
                                    for action in actions {
                                        if action["command"] == "SaveSettings" {
                                            self.controller
                                                .send(Command::Configure(self.settings.clone()));
                                        } else {
                                            self.controller.send(Command::PluginMessage(
                                                name.clone(),
                                                action["command"].as_str().unwrap_or("").into(),
                                                action["payload"].clone(),
                                            ));
                                        }
                                    }
                                }
                                if ui.button("Save and apply settings").clicked() {
                                    self.controller
                                        .send(Command::Configure(self.settings.clone()));
                                }
                            });
                        } else {
                            self.settings(ui, ctx, snapshot);
                        }
                    });
                self.screenshot_child(ctx, &title);
                if ctx.input(|input| input.viewport().close_requested()) {
                    open = false;
                }
                ctx.request_repaint_after(Duration::from_millis(100));
            },
        );
        self.settings_open = open;
    }
    fn about_window(&mut self, ctx: &egui::Context, snapshot: &Snapshot) {
        if !self.about_open {
            return;
        }
        let name = self.about_plugin.as_deref().unwrap_or("Core");
        let metadata = snapshot
            .plugin_details
            .get(name)
            .map(|d| d["about"].clone())
            .filter(|v| v.is_object())
            .unwrap_or_else(|| crate::about::metadata(name));
        let title = format!("About {}", metadata["FullName"].as_str().unwrap_or(name));
        let mut close = false;
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of(("Core AboutForm", name)),
            egui::ViewportBuilder::default()
                .with_title(title)
                .with_inner_size([514.0, 263.0])
                .with_resizable(false),
            |ctx, _| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show(ctx, |ui| crate::about::show(ui, &metadata));
                close = ctx.input(|i| i.viewport().close_requested());
            },
        );
        if close {
            self.about_open = false;
        }
    }
    fn results(&mut self, ui: &mut egui::Ui, snapshot: &Snapshot, plugin: Option<&str>) {
        let search = self.filter.to_lowercase();
        let key = format!(
            "{}|{}|{}|{}|{}|{}",
            Arc::as_ptr(&snapshot.findings) as usize,
            search,
            self.sort,
            self.descending,
            snapshot.current_system,
            self.settings.plugins["Explorer"]["OnlyShowCurrentSystem"]
        );
        let rows = cached_rows(
            ui.ctx(),
            egui::Id::new(("Core grid", plugin, ui.ctx().viewport_id())),
            key,
            || {
                let mut rows = snapshot
                    .findings
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| plugin.is_none_or(|p| f.plugin == p))
                    .filter(|(_, f)| {
                        plugin != Some("Explorer")
                            || !self.settings.plugins["Explorer"]["OnlyShowCurrentSystem"]
                                .as_bool()
                                .unwrap_or(false)
                            || f.system == snapshot.current_system
                    })
                    .map(|(index, f)| {
                        vec![
                            snapshot
                                .finding_grid_times
                                .get(index)
                                .and_then(|time| time.clone())
                                .unwrap_or_else(|| {
                                    observatory_plugins::communicator::display_timestamp(
                                        &f.timestamp,
                                    )
                                }),
                            f.body.clone(),
                            f.title.clone(),
                            f.detail.clone(),
                        ]
                    })
                    .filter(|row| {
                        search.is_empty() || row.iter().any(|v| v.to_lowercase().contains(&search))
                    })
                    .collect::<Vec<_>>();
                sort_rows(&mut rows, self.sort, self.descending);
                rows
            },
        );
        if let Some(plugin) = plugin {
            self.grid_exports.insert(
                plugin.into(),
                Arc::new(crate::runtime::GridExport {
                    headers: ["Time", "Body Name", "Description", "Details"]
                        .into_iter()
                        .map(str::to_owned)
                        .collect(),
                    rows: rows.clone(),
                }),
            );
        }
        data_table(
            ui,
            ["Time", "Body Name", "Description", "Details"].as_slice(),
            &rows,
            &mut self.sort,
            &mut self.descending,
        );
    }

    fn settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, _snapshot: &Snapshot) {
        let before = serde_json::to_value(&self.settings).unwrap_or_default();
        let origin = ui.max_rect().min;
        let rect = |x: f32, y: f32, w: f32, h: f32| {
            egui::Rect::from_min_size(origin + egui::vec2(x, y), egui::vec2(w, h))
        };
        let frame = |ui: &mut egui::Ui, bounds: egui::Rect, title: &str| {
            ui.painter().rect_stroke(
                bounds,
                0,
                egui::Stroke::new(1.0_f32, Color32::from_gray(105)),
                egui::StrokeKind::Inside,
            );
            ui.painter().text(
                bounds.min,
                egui::Align2::LEFT_TOP,
                title,
                egui::FontId::proportional(13.0),
                ui.visuals().text_color(),
            );
        };
        let core = rect(12.0, 12.0, 468.0, 276.0);
        frame(ui, core, "Core Settings");
        let mut actions = Vec::new();
        #[allow(deprecated)]
        ui.allocate_ui_at_rect(core, |ui| {
            let base = ui.max_rect().min;
            let r = |x: f32, y: f32, w: f32, h: f32| {
                egui::Rect::from_min_size(base + egui::vec2(x, y), egui::vec2(w, h))
            };
            for (label, x, y) in [
                ("Journal Folder:", 19.0, 25.0),
                ("Export Format:", 18.0, 50.0),
                ("Theme:", 57.0, 79.0),
                ("Volume:", 53.0, 118.0),
                ("Audio Device:", 23.0, 153.0),
                ("Notification\nRouting Test:", 32.0, 232.0),
            ] {
                ui.put(r(x, y, 90.0, 30.0), egui::Label::new(label));
            }
            if ui
                .put(
                    r(117.0, 25.0, 327.0, 20.0),
                    egui::Label::new(&self.journal_path).sense(egui::Sense::click()),
                )
                .clicked()
            {
                let path = PathBuf::from(&self.journal_path);
                self.choose_file(crate::file_dialog::Purpose::Journal, &path);
            }
            let mut format = self
                .settings
                .extra
                .get("ExportFormat")
                .and_then(Value::as_u64)
                .unwrap_or(0)
                .min(1);
            #[allow(deprecated)]
            ui.allocate_ui_at_rect(r(117.0, 47.0, 214.0, 23.0), |ui| {
                egui::ComboBox::from_id_salt("Core ExportFormat")
                    .width(214.0)
                    .selected_text(if format == 0 {
                        "Tab-Separated Values (csv)"
                    } else {
                        "Office Open XML (xlsx)"
                    })
                    .show_ui(ui, |ui| {
                        for (i, label) in ["Tab-Separated Values (csv)", "Office Open XML (xlsx)"]
                            .iter()
                            .enumerate()
                        {
                            if ui.selectable_value(&mut format, i as u64, *label).changed() {
                                self.settings
                                    .extra
                                    .insert("ExportFormat".into(), json!(format));
                            }
                        }
                    });
            });
            #[allow(deprecated)]
            ui.allocate_ui_at_rect(r(117.0, 76.0, 214.0, 23.0), |ui| {
                let old = self.settings.theme.clone();
                egui::ComboBox::from_id_salt("Core Theme")
                    .width(214.0)
                    .selected_text(if old.eq_ignore_ascii_case("dark") {
                        "Dark"
                    } else if old.eq_ignore_ascii_case("light") {
                        "Light"
                    } else {
                        &old
                    })
                    .show_ui(ui, |ui| {
                        for name in ["Dark".to_owned(), "Light".to_owned()]
                            .into_iter()
                            .chain(crate::core_theme::names(&self.settings))
                            .chain(custom_themes(&self.settings).keys().cloned())
                        {
                            ui.selectable_value(&mut self.settings.theme, name.clone(), name);
                        }
                    });
                if old != self.settings.theme {
                    apply_theme(ctx, &self.settings);
                }
            });
            if ui
                .put(r(344.0, 74.0, 88.0, 23.0), egui::Button::new("Add Theme"))
                .clicked()
            {
                let path = self.paths.data_dir.clone();
                self.choose_file(crate::file_dialog::Purpose::Theme, &path);
            }
            let mut volume = self
                .settings
                .extra
                .get("VoiceVolume")
                .and_then(Value::as_u64)
                .unwrap_or(75)
                .min(100);
            if ui
                .put(
                    r(117.0, 112.0, 214.0, 23.0),
                    egui::Slider::new(&mut volume, 0..=100).show_value(false),
                )
                .changed()
            {
                self.settings
                    .extra
                    .insert("VoiceVolume".into(), json!(volume));
                self.settings
                    .extra
                    .insert("AudioVolume".into(), json!(volume as f64 / 100.0));
            }
            #[allow(deprecated)]
            ui.allocate_ui_at_rect(r(117.0, 150.0, 214.0, 23.0), |ui| {
                crate::core_controls::select(
                    ui,
                    &mut self.settings,
                    "",
                    "audio_device",
                    &self.audio_choices.lock().unwrap().devices,
                );
            });
            ui.put(
                r(117.0, 179.0, 180.0, 19.0),
                egui::Checkbox::new(&mut self.settings.auto_start, "Start Monitor On Launch"),
            );
            ui.put(
                r(117.0, 204.0, 135.0, 19.0),
                egui::Checkbox::new(&mut self.settings.start_read_all, "Read All On Launch"),
            );
            let mut polling = self
                .settings
                .extra
                .get("AltMonitor")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if ui
                .put(
                    r(253.0, 203.0, 100.0, 19.0),
                    egui::Checkbox::new(&mut polling, "Use Polling"),
                )
                .changed()
            {
                self.settings
                    .extra
                    .insert("AltMonitor".into(), json!(polling));
                if polling {
                    self.settings.poll_interval_ms = 250;
                }
            }
            #[allow(deprecated)]
            ui.allocate_ui_at_rect(r(117.0, 229.0, 230.0, 43.0), |ui| {
                if let Some(action) = crate::core_controls::routing(ui) {
                    actions.push(action);
                }
            });
        });
        #[allow(deprecated)]
        ui.allocate_ui_at_rect(rect(12.0, 293.0, 468.0, 226.0), |ui| {
            frame(ui, ui.max_rect(), "Popup Notifications");
            actions.extend(crate::core_controls::popup(
                ui,
                &mut self.settings,
                self.game_focus.monitor_count(),
            ));
        });
        let choices = self.audio_choices.lock().unwrap().clone();
        #[allow(deprecated)]
        ui.allocate_ui_at_rect(rect(12.0, 525.0, 468.0, 173.0), |ui| {
            frame(ui, ui.max_rect(), "Audio Notifications");
            actions.extend(crate::core_controls::audio(
                ui,
                &mut self.settings,
                &choices.voices,
                &choices.devices,
            ));
        });
        if ui
            .put(rect(405.0, 704.0, 75.0, 23.0), egui::Button::new("Close"))
            .clicked()
        {
            self.settings_open = false;
        }
        if let Some(error) = &self.settings_error {
            ui.put(
                rect(12.0, 735.0, 468.0, 50.0),
                egui::Label::new(RichText::new(error).color(Color32::LIGHT_RED)),
            );
        }
        if before != serde_json::to_value(&self.settings).unwrap_or_default() {
            self.apply_settings();
        }
        for action in actions {
            self.core_action(action);
        }
    }
    fn apply_settings(&mut self) {
        self.settings.journal_dir = if self.journal_path.trim().is_empty() {
            None
        } else {
            Some(PathBuf::from(self.journal_path.trim()))
        };
        if self
            .settings
            .journal_dir
            .as_ref()
            .is_some_and(|path| !path.is_dir())
        {
            self.settings_error = Some("Journal directory does not exist.".into());
            return;
        }
        self.controller
            .send(Command::Configure(self.settings.clone()));
        self.settings_error = None;
    }
    fn core_action(&self, action: crate::core_controls::Action) {
        let (mask, direct) = match action {
            crate::core_controls::Action::Popup => (1, true),
            crate::core_controls::Action::Audio => (2, true),
            crate::core_controls::Action::Routing(mask) => (mask, false),
        };
        self.controller.send(Command::TestNotification(
            self.settings.clone(),
            mask,
            direct,
        ));
    }
    fn render_plugin_windows(
        &mut self,
        ctx: &egui::Context,
        snapshot: &Snapshot,
        frame: &mut eframe::Frame,
    ) {
        let available = available_tabs(snapshot);
        let names = self
            .plugin_windows
            .iter()
            .filter(|(key, window)| {
                let name = key.strip_prefix("@custom/").unwrap_or(key);
                window.open
                    && name != "Settings"
                    && (available.iter().any(|n| n == name)
                        || matches!(name, "All findings" | "Status" | "Errors"))
            })
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        for key in names {
            let name = key.strip_prefix("@custom/").unwrap_or(&key).to_owned();
            let mut save_position = false;
            let mut window = self.plugin_windows.remove(&key).unwrap();
            sanitize_window(&mut window);
            let bio = key.starts_with("@custom/");
            let visible = !bio
                || (if name == "Evaluator" {
                    crate::evaluator_ui::popout_visible
                } else {
                    crate::bioinsights_main::popout::visible
                })(
                    &self.settings.plugins[&name],
                    snapshot.status["GuiFocus"].as_u64().unwrap_or(0),
                    snapshot.status["Flags"].as_u64().unwrap_or(0),
                    snapshot.bio_main_view["Population"].as_u64().unwrap_or(0),
                );
            let opacity = if bio {
                let target = if visible {
                    crate::bioinsights_main::popout::opacity(&self.settings.plugins[&name])
                } else {
                    0.0
                };
                ctx.animate_value_with_time(
                    egui::Id::new(("plugin-popout-opacity", &name)),
                    target,
                    0.3,
                )
            } else {
                1.0
            };
            let mut builder = egui::ViewportBuilder::default()
                .with_title(if bio {
                    format!("{name} Popout")
                } else {
                    snapshot
                        .plugin_details
                        .get(&name)
                        .and_then(|v| v["full_name"].as_str())
                        .unwrap_or(&name)
                        .to_owned()
                })
                .with_inner_size(window.size)
                .with_min_inner_size([360.0, 240.0])
                .with_visible(visible || opacity > 0.0)
                .with_transparent(bio);
            if bio {
                builder = builder
                    .with_decorations(window.title_bar)
                    .with_window_level(if window.on_top {
                        egui::WindowLevel::AlwaysOnTop
                    } else {
                        egui::WindowLevel::Normal
                    });
            }
            if let Some(position) = window.position {
                builder = builder.with_position(position);
            }
            std::mem::swap(&mut self.filter, &mut window.filter);
            std::mem::swap(&mut self.sort, &mut window.sort);
            std::mem::swap(&mut self.descending, &mut window.descending);
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of(("plugin-window", &key)),
                builder,
                |ctx, _| {
                    if !bio {
                        egui::TopBottomPanel::bottom("popout-buttons")
                            .frame(egui::Frame::new().inner_margin(12))
                            .show(ctx, |ui| {
                                ui.horizontal(|ui| {
                                    if ui
                                        .add_sized([75.0, 23.0], egui::Button::new("Settings"))
                                        .clicked()
                                    {
                                        self.open_settings(Some(name.clone()));
                                    }
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui
                                                .add_enabled(
                                                    is_basic_ui(&name),
                                                    egui::Button::new("Clear")
                                                        .min_size(egui::vec2(75.0, 23.0)),
                                                )
                                                .clicked()
                                            {
                                                self.controller.send(Command::Clear(name.clone()));
                                            }
                                            if ui
                                                .add_enabled(
                                                    is_basic_ui(&name),
                                                    egui::Button::new("Export")
                                                        .min_size(egui::vec2(75.0, 23.0)),
                                                )
                                                .clicked()
                                            {
                                                self.export_plugin(&name);
                                            }
                                        },
                                    );
                                });
                            });
                    }
                    egui::CentralPanel::default()
                        .frame(egui::Frame::NONE)
                        .show(ctx, |ui| {
                            if bio {
                                ui.set_opacity(opacity);
                                let toolbar_id = egui::Id::new((
                                    "Plugin toolbar visibility",
                                    &name,
                                    ctx.viewport_id(),
                                ));
                                ctx.data_mut(|d| d.insert_temp(toolbar_id, window.hide_buttons));
                                self.plugin_popout_menu(ui, &name, &mut window, &mut save_position);
                            }
                            ui.push_id(("detached-plugin", &name), |ui| {
                                self.render_tab(ui, ctx, snapshot, &name)
                            });
                        });
                    self.screenshot_child(ctx, &name);
                    ctx.input(|input| {
                        let viewport = input.viewport();
                        if viewport.close_requested() {
                            if name == "BioInsights" {
                                if let Some(Ok(browser)) = &mut self.bio_browser {
                                    let _ = browser.reattach();
                                }
                            }
                            window.open = false;
                        }
                        if let Some(rect) = viewport.inner_rect {
                            window.size = [rect.width(), rect.height()];
                        }
                        if let Some(rect) = viewport.outer_rect {
                            window.position = Some([rect.min.x, rect.min.y]);
                        }
                    });
                    ctx.request_repaint_after(Duration::from_millis(100));
                },
            );
            if name == "BioInsights" && bio {
                for command in std::mem::take(&mut self.browser_chrome_actions) {
                    match command.as_str() {
                        "ToggleOnTop" => window.on_top = !window.on_top,
                        "ToggleTitleBar" => window.title_bar = !window.title_bar,
                        "ToggleButtons" => window.hide_buttons = !window.hide_buttons,
                        "Popout" => self.close_plugin_popout = Some(key.clone()),
                        _ => {}
                    }
                    if command != "Popout" {
                        save_position = true;
                    }
                }
            }
            std::mem::swap(&mut self.filter, &mut window.filter);
            std::mem::swap(&mut self.sort, &mut window.sort);
            std::mem::swap(&mut self.descending, &mut window.descending);
            if self.close_plugin_popout.as_deref() == Some(&key) {
                window.open = false;
                if name == "BioInsights" {
                    if let Some(Ok(browser)) = &mut self.bio_browser {
                        if let Err(error) = browser.reattach() {
                            self.settings_error = Some(format!("Browser reattach: {error:#}"));
                        }
                    }
                }
                self.close_plugin_popout = None;
            }
            if save_position && bio {
                if let Err(error) = self.save_custom_popout(&name, &window) {
                    self.settings_error = Some(format!("{name} popout settings: {error:#}"));
                }
            }
            self.plugin_windows.insert(key, window);
            if save_position {
                if let Some(storage) = frame.storage_mut() {
                    <Self as eframe::App>::save(self, storage);
                    storage.flush();
                }
            }
        }
    }
    fn load_custom_popout(&self, name: &str) -> Option<PluginWindow> {
        let file = std::fs::File::open(self.paths.data_dir.join(name).join("popout.json")).ok()?;
        let value: Value = serde_json::from_reader(file).ok()?;
        let mut window = PluginWindow {
            position: Some([
                value["Location"]["X"].as_f64()? as f32,
                value["Location"]["Y"].as_f64()? as f32,
            ]),
            size: [
                value["Size"]["Width"].as_f64()? as f32,
                value["Size"]["Height"].as_f64()? as f32,
            ],
            on_top: value["IsOnTop"].as_bool().unwrap_or(false),
            title_bar: value["ShowTitleBar"].as_bool().unwrap_or(true),
            hide_buttons: value["HideButtons"].as_bool().unwrap_or(false),
            ..Default::default()
        };
        sanitize_window(&mut window);
        Some(window)
    }
    fn save_custom_popout(&self, name: &str, window: &PluginWindow) -> Result<()> {
        use std::io::Write;
        let folder = self.paths.data_dir.join(name);
        std::fs::create_dir_all(&folder)?;
        let position = window.position.unwrap_or([0.0, 0.0]);
        let value = json!({"Location":{"X":position[0].round()as i32,"Y":position[1].round()as i32},"Size":{"Width":window.size[0].round()as i32,"Height":window.size[1].round()as i32},"IsOnTop":window.on_top,"ShowTitleBar":window.title_bar,"HideButtons":window.hide_buttons});
        let mut file = tempfile::NamedTempFile::new_in(&folder)?;
        file.write_all(&serde_json::to_vec(&value)?)?;
        file.as_file().sync_all()?;
        file.persist(folder.join("popout.json"))?;
        Ok(())
    }
    fn plugin_popout_menu(
        &mut self,
        ui: &mut egui::Ui,
        name: &str,
        window: &mut PluginWindow,
        save_position: &mut bool,
    ) {
        // Register the background before the source toolbar and table controls
        // so it cannot consume their primary clicks.
        ui.interact(
            ui.max_rect(),
            egui::Id::new(("Plugin popout context menu", name)),
            egui::Sense::click(),
        )
        .context_menu(|ui| {
            let mut chrome = crate::bioinsights_main::popout::Chrome {
                on_top: window.on_top,
                title_bar: window.title_bar,
                hide_buttons: window.hide_buttons,
            };
            let actions = if name == "Evaluator" {
                crate::evaluator_ui::popout_menu(ui, &mut chrome)
            } else {
                crate::bioinsights_main::popout::menu(ui, &mut chrome)
            };
            for action in actions {
                match action["command"].as_str().unwrap_or("") {
                    "SavePopoutPosition" => *save_position = true,
                    "Popout" => self.close_plugin_popout = Some(format!("@custom/{name}")),
                    command => self.controller.send(Command::PluginMessage(
                        name.into(),
                        command.into(),
                        action["payload"].clone(),
                    )),
                }
            }
            window.on_top = chrome.on_top;
            window.title_bar = chrome.title_bar;
            window.hide_buttons = chrome.hide_buttons;
        });
    }
    fn render_tab(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        snapshot: &Snapshot,
        tab: &str,
    ) {
        // Context data callbacks hold its lock. Compute the viewport key
        // before entering them to avoid recursively locking the Context.
        let grid_id = egui::Id::new(("Core grid plugin", ctx.viewport_id()));
        ctx.data_mut(|d| d.insert_temp(grid_id, tab.to_owned()));
        let version = snapshot
            .plugin_details
            .get(tab)
            .and_then(|details| details["version"].as_str())
            .unwrap_or("")
            .to_owned();
        let version_id = egui::Id::new(("Core grid version", tab));
        ctx.data_mut(|data| data.insert_temp(version_id, version));
        let sort_id = egui::Id::new(("Core source sorter", tab));
        (self.sort, self.descending) = ctx
            .data(|d| d.get_temp::<(usize, bool)>(sort_id))
            .unwrap_or((usize::MAX, false));
        let custom_key = format!("@custom/{tab}");
        let custom_id = egui::ViewportId::from_hash_of(("plugin-window", &custom_key));
        if self.plugin_windows.get(&custom_key).is_some_and(|w| w.open)
            && ctx.viewport_id() != custom_id
        {
            return;
        }
        if ctx.viewport_id() == egui::ViewportId::ROOT
            && self.plugin_windows.get(tab).is_some_and(|w| w.open)
        {
            let response=ui.add(egui::Label::new(format!("{tab} is currently in a different window.\nYou can click this text to switch to it.")).sense(egui::Sense::click()));
            if response.clicked() {
                let id = egui::ViewportId::from_hash_of(("plugin-window", tab));
                ctx.send_viewport_cmd_to(id, egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd_to(id, egui::ViewportCommand::Focus);
            }
            return;
        }
        if self.filter_open && !matches!(tab, "Core" | "Settings" | "Status" | "Errors") {
            ui.horizontal(|ui| {
                ui.label("Find:");
                ui.text_edit_singleline(&mut self.filter);
                if ui.button("Close").clicked() {
                    self.filter_open = false;
                    self.filter.clear();
                }
            });
            ui.separator();
        }
        match tab {
            "BioInsights" => {
                ctx.data_mut(|data| {
                    data.insert_temp(
                        egui::Id::new("Bio source projection revision"),
                        snapshot.bio_revision,
                    )
                });
                if !self.bio_output_enabled {
                    crate::evaluator_ui::draw_disabled_output(ui);
                    return;
                }
                let view = snapshot.bio_main_view.as_ref();
                let panel_size = [ui.max_rect().width(), ui.max_rect().height()];
                let toolbar_id =
                    egui::Id::new(("Plugin toolbar visibility", tab, ctx.viewport_id()));
                let hide_buttons = ctx
                    .data(|d| d.get_temp::<bool>(toolbar_id))
                    .unwrap_or(false);
                let mut actions = if self.bio_browser_mode {
                    if self.bio_browser.is_none() {
                        self.bio_browser = Some(
                            self.browser_parent
                                .ok_or_else(|| {
                                    "Embedded browser requires an X11/XWayland parent".to_owned()
                                })
                                .and_then(|parent| {
                                    crate::embedded_browser::Browser::new(parent)
                                        .map_err(|e| format!("{e:#}"))
                                }),
                        );
                    }
                    match self.bio_browser.as_mut().unwrap() {
                        Ok(browser) => {
                            if let Err(error) = browser.show(
                                ui,
                                &snapshot.bio_browser_html,
                                if ctx.viewport_id() == custom_id {
                                    "BioInsights Popout"
                                } else {
                                    "BioInsights"
                                },
                                snapshot.bio_summary["BrowserRoute"]
                                    .as_str()
                                    .unwrap_or("main"),
                                hide_buttons,
                            ) {
                                ui.label(format!("Embedded browser: {error:#}"));
                            }
                        }
                        Err(error) => {
                            ui.colored_label(Color32::LIGHT_RED, error.as_str());
                        }
                    }
                    Vec::new()
                } else if hide_buttons {
                    crate::bioinsights_main::show_variant(
                        ui,
                        view,
                        snapshot.bio_summary.as_ref(),
                        &self.settings.plugins["BioInsights"],
                        true,
                    )
                } else {
                    crate::bioinsights_main::show(
                        ui,
                        view,
                        snapshot.bio_summary.as_ref(),
                        &self.settings.plugins["BioInsights"],
                    )
                };
                actions.append(&mut self.pending_bio_actions);
                for action in actions {
                    let command = action["command"].as_str().unwrap_or("");
                    match command {
                        "Main" | "SelectBioInsights" => self.tab = "BioInsights".into(),
                        "Settings" => self.open_settings(Some("BioInsights".into())),
                        "Zoom" => {
                            if let Some(Ok(browser)) = &self.bio_browser {
                                browser.zoom_menu(ctx);
                            }
                        }
                        "Popout" => {
                            if ctx.viewport_id() == custom_id {
                                self.close_plugin_popout = Some(custom_key.clone());
                            } else {
                                if action["payload"]["Restore"] != false {
                                    if let Some(saved) = self.load_custom_popout("BioInsights") {
                                        self.plugin_windows.insert(custom_key.clone(), saved);
                                    }
                                }
                                let window = self
                                    .plugin_windows
                                    .entry(custom_key.clone())
                                    .or_insert_with(|| PluginWindow {
                                        size: panel_size,
                                        ..Default::default()
                                    });
                                let (position, size) = crate::bioinsights_main::popout::geometry(
                                    panel_size,
                                    action["payload"]["Restore"].as_bool().unwrap_or(true),
                                    window.position,
                                    window.size,
                                );
                                window.position = position;
                                window.size = size;
                                if action["payload"]["Restore"].as_bool() == Some(false) {
                                    window.on_top = false;
                                    window.title_bar = true;
                                    window.hide_buttons = false;
                                }
                                window.open = !window.open;
                            }
                        }
                        _ => {
                            if command == "SetZoomLevel" {
                                if let Some(zoom) = action["payload"]["ZoomLevel"].as_u64() {
                                    self.settings.plugins["BioInsights"]["ZoomLevel"] =
                                        json!(zoom.clamp(50, 600));
                                }
                            }
                            self.controller.send(Command::PluginMessage(
                                "BioInsights".into(),
                                command.into(),
                                action["payload"].clone(),
                            ));
                        }
                    }
                }
            }
            "Codex Companion" | "Unsold BioInsights" | "BioInsights - About" => {
                let route = if tab == "BioInsights - About" {
                    "About"
                } else {
                    tab
                };
                for action in crate::bioinsights_ui::open(ctx, route) {
                    self.controller.send(Command::PluginMessage(
                        "BioInsights".into(),
                        action["command"].as_str().unwrap_or("").into(),
                        action["payload"].clone(),
                    ));
                }
                self.tab = "BioInsights".into();
                self.render_tab(ui, ctx, snapshot, "BioInsights");
            }
            "Evaluator" => {
                ctx.data_mut(|data| {
                    data.insert_temp(
                        egui::Id::new("Evaluator source projection revision"),
                        snapshot.evaluator_revision,
                    )
                });
                let summary = snapshot.eval_summary.as_ref();
                let rows = snapshot
                    .plugin_rows
                    .get("Evaluator")
                    .map(|rows| rows.as_slice())
                    .unwrap_or(&[]);
                let toolbar_id =
                    egui::Id::new(("Plugin toolbar visibility", tab, ctx.viewport_id()));
                let hide_buttons = ctx
                    .data(|d| d.get_temp::<bool>(toolbar_id))
                    .unwrap_or(false);
                let actions = if hide_buttons {
                    crate::evaluator_ui::show_variant(
                        ui,
                        summary,
                        rows,
                        &self.settings.plugins["Evaluator"],
                        true,
                    )
                } else {
                    crate::evaluator_ui::show(
                        ui,
                        summary,
                        rows,
                        &self.settings.plugins["Evaluator"],
                    )
                };
                for action in actions {
                    let command = action["command"].as_str().unwrap_or("");
                    match command {
                        "Settings" => self.open_settings(Some("Evaluator".into())),
                        "Popout" => {
                            if ctx.viewport_id() == custom_id {
                                self.close_plugin_popout = Some(custom_key.clone());
                            } else {
                                if action["payload"]["Restore"] != false {
                                    if let Some(saved) = self.load_custom_popout("Evaluator") {
                                        self.plugin_windows.insert(custom_key.clone(), saved);
                                    }
                                }
                                let panel_size = [ui.max_rect().width(), ui.max_rect().height()];
                                let window = self
                                    .plugin_windows
                                    .entry(custom_key.clone())
                                    .or_insert_with(|| PluginWindow {
                                        size: panel_size,
                                        ..Default::default()
                                    });
                                (window.position, window.size) =
                                    crate::bioinsights_main::popout::geometry(
                                        panel_size,
                                        action["payload"]["Restore"].as_bool().unwrap_or(true),
                                        window.position,
                                        window.size,
                                    );
                                if action["payload"]["Restore"] == false {
                                    window.on_top = false;
                                    window.title_bar = true;
                                    window.hide_buttons = false;
                                }
                                window.open = true;
                            }
                        }
                        _ => {
                            if command == "SetZoomLevel" {
                                if let Some(zoom) = action["payload"]["ZoomLevel"].as_u64() {
                                    self.settings.plugins["Evaluator"]["ZoomLevel"] =
                                        json!(zoom.clamp(50, 600));
                                }
                            }
                            self.controller.send(Command::PluginMessage(
                                "Evaluator".into(),
                                command.into(),
                                action["payload"].clone(),
                            ));
                        }
                    }
                }
            }
            "Core" => self.core_page(ui, snapshot),
            "Settings" => self.settings(ui, ctx, snapshot),
            "Explorer" => self.results(ui, snapshot, Some("Explorer")),
            "All findings" => self.results(ui, snapshot, None),
            "Botanist" => {
                let key = format!(
                    "{}|{}|{}|{}",
                    Arc::as_ptr(&snapshot.botanist) as usize,
                    self.filter,
                    self.sort,
                    self.descending
                );
                let rows = cached_rows(ctx, egui::Id::new("Core Botanist projection"), key, || {
                    let filter = self.filter.to_lowercase();
                    let mut rows = snapshot
                        .botanist
                        .iter()
                        .map(|row| row.grid_values())
                        .filter(|row| {
                            filter.is_empty()
                                || row
                                    .iter()
                                    .any(|value| value.to_lowercase().contains(&filter))
                        })
                        .collect::<Vec<_>>();
                    sort_rows(&mut rows, self.sort, self.descending);
                    rows
                });
                let titles = [
                    "Body",
                    "Bio Total",
                    "Species",
                    "Analysed",
                    "Colony Distance",
                ];
                self.grid_exports.insert(
                    tab.into(),
                    Arc::new(crate::runtime::GridExport {
                        headers: titles.into_iter().map(str::to_owned).collect(),
                        rows: rows.clone(),
                    }),
                );
                data_table(ui, &titles, &rows, &mut self.sort, &mut self.descending);
            }
            "Communicator" => {
                let mut changed = false;
                ui.horizontal(|ui| {
                    for (label, key) in [
                        ("Local", "ShowLocal"),
                        ("System", "ShowSystem"),
                        ("Private", "ShowPrivate"),
                        ("Wing", "ShowWing"),
                        ("Squadron", "ShowSquadron"),
                        ("NPC", "ShowNPC"),
                    ] {
                        let mut selected = self.settings.plugins["Communicator"][key]
                            .as_bool()
                            .unwrap_or(true);
                        if ui.checkbox(&mut selected, label).changed() {
                            self.settings.plugins["Communicator"][key] = json!(selected);
                            changed = true;
                        }
                    }
                });
                if changed {
                    self.apply_settings();
                }
                let key = format!(
                    "{}|{}|{}|{}",
                    Arc::as_ptr(&snapshot.messages) as usize,
                    self.filter,
                    self.sort,
                    self.descending
                );
                let rows = cached_rows(
                    ctx,
                    egui::Id::new("Core Communicator projection"),
                    key,
                    || {
                        let filter = self.filter.to_lowercase();
                        let mut rows = snapshot
                            .messages
                            .iter()
                            .map(|row| {
                                vec![
                                    row.timestamp.clone(),
                                    row.channel.clone(),
                                    row.from.clone(),
                                    row.message.clone(),
                                ]
                            })
                            .filter(|row| {
                                filter.is_empty()
                                    || row
                                        .iter()
                                        .any(|value| value.to_lowercase().contains(&filter))
                            })
                            .collect::<Vec<_>>();
                        sort_rows(&mut rows, self.sort, self.descending);
                        rows
                    },
                );
                let titles = ["Timestamp", "Channel", "From", "Message"];
                self.grid_exports.insert(
                    tab.into(),
                    Arc::new(crate::runtime::GridExport {
                        headers: titles.into_iter().map(str::to_owned).collect(),
                        rows: rows.clone(),
                    }),
                );
                data_table(ui, &titles, &rows, &mut self.sort, &mut self.descending);
            }
            "Status" => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.monospace(
                        serde_json::to_string_pretty(&snapshot.status).unwrap_or_default(),
                    );
                });
            }
            "Errors" => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for error in snapshot.errors.iter().rev() {
                        ui.colored_label(Color32::LIGHT_RED, error);
                        ui.separator();
                    }
                });
            }
            _ => {
                if let Some(rows) = snapshot
                    .plugin_rows
                    .get(tab)
                    .filter(|rows| !rows.is_empty() || self.grid_columns.contains_key(tab))
                {
                    let headers = self
                        .grid_columns
                        .entry(tab.into())
                        .or_insert_with(|| {
                            snapshot
                                .plugin_details
                                .get(tab)
                                .and_then(|v| v["columns"].as_array())
                                .map(|keys| {
                                    keys.iter()
                                        .filter_map(Value::as_str)
                                        .filter(|key| *key != "ObservatoryListViewGroupID")
                                        .map(str::to_owned)
                                        .collect()
                                })
                                .unwrap_or_else(|| {
                                    rows.first()
                                        .and_then(Value::as_object)
                                        .into_iter()
                                        .flat_map(|object| object.keys())
                                        .filter(|key| key.as_str() != "ObservatoryListViewGroupID")
                                        .cloned()
                                        .collect()
                                })
                        })
                        .clone();
                    let filter = self.filter.to_lowercase();
                    let key = format!(
                        "{}|{}|{}|{}",
                        Arc::as_ptr(rows) as usize,
                        filter,
                        self.sort,
                        self.descending
                    );
                    let cells = cached_rows(
                        ctx,
                        egui::Id::new(("Core Basic grid projection", tab)),
                        key,
                        || {
                            let mut source = rows
                                .iter()
                                .filter(|row| {
                                    filter.is_empty()
                                        || headers.iter().any(|key| {
                                            grid_cell(&row[key]).to_lowercase().contains(&filter)
                                        })
                                })
                                .collect::<Vec<_>>();
                            if let Some(column) = headers.get(self.sort) {
                                crate::extensions::sort_grid_rows(
                                    &mut source,
                                    column,
                                    !self.descending,
                                );
                            }
                            source
                                .into_iter()
                                .map(|row| headers.iter().map(|key| grid_cell(&row[key])).collect())
                                .collect()
                        },
                    );
                    self.grid_exports.insert(
                        tab.into(),
                        Arc::new(crate::runtime::GridExport {
                            headers: headers.clone(),
                            rows: cells.clone(),
                        }),
                    );
                    let titles = headers.iter().map(String::as_str).collect::<Vec<_>>();
                    data_table(ui, &titles, &cells, &mut self.sort, &mut self.descending);
                } else if let Some(headers) = self.grid_columns.get(tab) {
                    let rows = Arc::new(Vec::new());
                    self.grid_exports.insert(
                        tab.into(),
                        Arc::new(crate::runtime::GridExport {
                            headers: headers.clone(),
                            rows: rows.clone(),
                        }),
                    );
                    let titles = headers.iter().map(String::as_str).collect::<Vec<_>>();
                    data_table(ui, &titles, &rows, &mut self.sort, &mut self.descending);
                } else {
                    self.results(ui, snapshot, Some(tab));
                }
            }
        }
        ctx.data_mut(|data| data.insert_temp(sort_id, (self.sort, self.descending)));
    }
}

impl eframe::App for ObservatoryApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        crate::passive_windows::shutdown();
    }
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Some(size) = self.root_size {
            self.settings
                .extra
                .insert("window_size".into(), json!(size));
            self.settings.extra.insert(
                "MainWindowSize".into(),
                json!({"Width":size[0].round()as i32,"Height":size[1].round()as i32}),
            );
        }
        if let Some(position) = self.root_position {
            self.settings
                .extra
                .insert("window_position".into(), json!(position));
            self.settings.extra.insert(
                "MainWindowPosition".into(),
                json!({"X":position[0].round()as i32,"Y":position[1].round()as i32}),
            );
        }
        let available = available_tabs(&self.controller.snapshot.lock().unwrap());
        self.tab_order = normalize_tab_order(&self.tab_order, &available);
        if let Some(index) = self.tab_order.iter().position(|tab| tab == &self.tab) {
            self.settings
                .extra
                .insert("LastTabIndex".into(), json!(index));
        }
        self.settings
            .extra
            .insert("last_tab".into(), json!(self.tab));
        self.settings
            .extra
            .insert("tab_order".into(), json!(self.tab_order));
        let core_windows = self
            .plugin_windows
            .iter()
            .filter(|(key, _)| !key.starts_with("@custom/"))
            .collect::<BTreeMap<_, _>>();
        self.settings
            .extra
            .insert("plugin_windows".into(), json!(core_windows));
        if let Some(ctx) = self.controller.snapshot.lock().unwrap().repaint.clone() {
            if let Some(widths) = ctx.data(|d| {
                d.get_temp::<Arc<Mutex<Value>>>(egui::Id::new("Core source column sizing"))
            }) {
                let mut widths = widths.lock().unwrap().clone();
                let versions = widths
                    .as_object_mut()
                    .and_then(|widths| widths.remove("__versions"))
                    .unwrap_or(json!({}));
                self.settings.extra.insert("column_sizing".into(), widths);
                self.settings
                    .extra
                    .insert("column_sizing_versions".into(), versions);
            }
        }
        self.controller
            .send(Command::Configure(self.settings.clone()));

        storage.set_string("tab", self.tab.clone());
        storage.set_string("sampler_window", "false".into());
        if let Ok(windows) = serde_json::to_string(&self.plugin_windows) {
            storage.set_string("plugin_windows", windows);
        }
        if let Ok(order) = serde_json::to_string(&self.tab_order) {
            storage.set_string("tab_order", order);
        }
    }
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.input(|i| {
            if let Some(rect) = i.viewport().inner_rect {
                self.root_size = Some([rect.width(), rect.height()]);
            }
            if let Some(rect) = i.viewport().outer_rect {
                self.root_position = Some([rect.min.x, rect.min.y]);
            }
        });
        if self.frames >= 3
            && Instant::now() >= self.capture_ready_at
            && self.screenshot.is_some()
            && self.screenshot_viewport.is_none()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        let events = ctx.input(|i| i.events.clone());
        for event in events {
            if let egui::Event::Screenshot { image, .. } = event {
                if let Some(path) = self.screenshot.take() {
                    let pixels = image
                        .pixels
                        .iter()
                        .flat_map(|p| p.to_array())
                        .collect::<Vec<_>>();
                    if let Err(error) = image::save_buffer(
                        &path,
                        &pixels,
                        image.width() as u32,
                        image.height() as u32,
                        image::ColorType::Rgba8,
                    ) {
                        self.settings_error = Some(format!("save screenshot: {error}"));
                    }
                }
            }
        }
        if let Some(result) = self.file_dialog.as_ref().and_then(|d| d.poll()) {
            let dialog = self.file_dialog.take().unwrap();
            if let Some(path) = result {
                match dialog.purpose.clone() {
                    crate::file_dialog::Purpose::Journal => {
                        self.journal_path = path.display().to_string();
                        self.apply_settings();
                    }
                    crate::file_dialog::Purpose::Export(name) => {
                        if let Some(view) = self.pending_export.take() {
                            self.controller.send(Command::ExportView(path, name, view));
                        } else {
                            self.controller
                                .send(Command::Export(path, b'\t', Some(name)));
                        }
                    }
                    crate::file_dialog::Purpose::Theme => {
                        match crate::core_theme::import(&path, &mut self.settings) {
                            Ok(()) => {
                                apply_theme(ctx, &self.settings);
                                self.apply_settings();
                            }
                            Err(error) => self.settings_error = Some(format!("Theme: {error:#}")),
                        }
                    }
                }
            }
        }
        self.frames += 1;
        if self.smoke_frames.is_some() {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
        if self.screenshot.is_some() {
            ctx.request_repaint_after(Duration::from_millis(33));
        }
        if self.smoke_frames.is_some_and(|n| self.frames >= n) {
            if self.smoke_frames == Some(self.frames) {
                eprintln!(
                    "observatory-smoke {}",
                    json!({"native_viewport_count":ctx.input(|i|i.raw.viewports.len()),"selected_tab":self.tab})
                );
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if let Some(Ok(browser)) = &mut self.bio_browser {
            browser.begin_frame();
            for url in browser.navigation() {
                if let Some(level) = url
                    .strip_prefix("observatory://bioinsights/nativezoom/")
                    .and_then(|n| n.parse::<u64>().ok())
                {
                    self.settings.plugins["BioInsights"]["ZoomLevel"] = json!(level.clamp(50, 600));
                    self.controller.send(Command::PluginMessage(
                        "BioInsights".into(),
                        "SetZoomLevel".into(),
                        json!({"ZoomLevel":level.clamp(50,600)}),
                    ));
                } else if let Some(command) =
                    url.strip_prefix("observatory://bioinsights/nativeaction/")
                {
                    self.browser_chrome_actions.push(command.to_owned());
                } else if url.starts_with("observatory://bioinsights/") {
                    self.controller.send(Command::PluginMessage(
                        "BioInsights".into(),
                        "BrowserNavigate".into(),
                        json!({"Url":url}),
                    ));
                } else {
                    let _ = ProcessCommand::new("xdg-open")
                        .arg(url)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .map(|mut child| thread::spawn(move || child.wait()));
                }
            }
        }
        let (snapshot, actions) = {
            let mut snapshot = self.controller.snapshot.lock().unwrap();
            let actions = snapshot.ui_actions.drain(..).collect::<Vec<_>>();
            (snapshot.clone(), actions)
        };
        for action in actions {
            if let Some(command) = action["CoreCommand"].as_str() {
                let payload = &action["Payload"];
                let requested = payload["Plugin"].as_str().unwrap_or("Core");
                let target = snapshot
                    .plugin_details
                    .iter()
                    .find(|(name, details)| {
                        name.as_str() == requested
                            || details["full_name"].as_str() == Some(requested)
                    })
                    .map(|(name, _)| name.as_str())
                    .unwrap_or(requested)
                    .to_owned();
                match command {
                    "OpenSettings" => self.open_settings((target != "Core").then_some(target)),
                    "OpenAbout" => {
                        self.about_plugin = (target != "Core").then_some(target);
                        self.about_open = true;
                    }
                    "FocusPlugin" if available_tabs(&snapshot).contains(&target) => {
                        self.tab = target.clone();
                        let id = if self.plugin_windows.get(&target).is_some_and(|w| w.open) {
                            egui::ViewportId::from_hash_of(("plugin-window", &target))
                        } else {
                            egui::ViewportId::ROOT
                        };
                        ctx.send_viewport_cmd_to(id, egui::ViewportCommand::Minimized(false));
                        ctx.send_viewport_cmd_to(id, egui::ViewportCommand::Focus);
                    }
                    "DisplayMessageBox" => self.message_boxes.push_back(payload.clone()),
                    _ => {}
                }
            } else {
                self.pending_bio_actions
                    .push(json!({"command":action["Action"],"payload":{"Restore":true}}));
            }
        }
        for (name, details) in &snapshot.plugin_details {
            if self.settings.plugins.get(name).is_none() {
                self.settings.plugins[name] = details["settings"].clone();
            }
        }
        self.monitor_active = snapshot.monitoring;
        if self.tab == "Settings" {
            self.open_settings(None);
            self.tab = "Core".into();
        }
        self.original_shell(ctx, &snapshot);
        let read_all_title_id = egui::Id::new("Core source ReadAllForm title");
        if let Some((journal, completed, total)) = &snapshot.read_progress {
            let title = ctx.data_mut(|data| {
                data.get_temp::<String>(read_all_title_id)
                    .unwrap_or_else(|| {
                        let draw = std::hash::BuildHasher::hash_one(
                            &std::collections::hash_map::RandomState::new(),
                            0_u8,
                        ) % 20;
                        let title = read_all_title(draw).to_owned();
                        data.insert_temp(read_all_title_id, title.clone());
                        title
                    })
            });
            let position = ctx
                .input(|i| {
                    i.viewport()
                        .outer_rect
                        .map(|r| r.min + egui::vec2(100.0, 100.0))
                })
                .unwrap_or(egui::pos2(100.0, 100.0));
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("Core ReadAllForm"),
                egui::ViewportBuilder::default()
                    .with_title(title)
                    .with_position(position)
                    .with_inner_size([395.0, 86.0])
                    .with_resizable(false),
                |ctx, _| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::NONE)
                        .show(ctx, |ui| {
                            let origin = ui.max_rect().min;
                            let rect = |x, y, w, h| {
                                egui::Rect::from_min_size(
                                    origin + egui::vec2(x, y),
                                    egui::vec2(w, h),
                                )
                            };
                            ui.put(rect(12.0, 9.0, 371.0, 15.0), egui::Label::new(journal));
                            ui.put(
                                rect(12.0, 27.0, 371.0, 23.0),
                                egui::ProgressBar::new(*completed as f32 / (*total).max(1) as f32),
                            );
                            if ui
                                .put(rect(308.0, 56.0, 75.0, 23.0), egui::Button::new("Cancel"))
                                .clicked()
                            {
                                self.controller.send(Command::CancelReplay);
                            }
                        });
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.controller.send(Command::CancelReplay);
                    }
                },
            );
        } else {
            ctx.data_mut(|data| data.remove::<String>(read_all_title_id));
        }
        self.settings_window(ctx, &snapshot);
        self.about_window(ctx, &snapshot);
        if let Some(message) = self.message_boxes.front().cloned() {
            let mut closed = false;
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("Core DisplayMessageBox"),
                egui::ViewportBuilder::default()
                    .with_title(
                        message["Title"]
                            .as_str()
                            .unwrap_or("Elite Observatory Core"),
                    )
                    .with_inner_size([420.0, 180.0])
                    .with_resizable(false),
                |ctx, _| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(120.0)
                            .show(ui, |ui| {
                                ui.label(message["Message"].as_str().unwrap_or(""));
                            });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add_sized([75.0, 23.0], egui::Button::new("OK"))
                                .clicked()
                            {
                                closed = true;
                            }
                        });
                    });
                    closed |= ctx.input(|i| i.viewport().close_requested());
                },
            );
            if closed {
                self.message_boxes.pop_front();
            }
        }

        self.render_plugin_windows(ctx, &snapshot, frame);
        if let Some(Ok(browser)) = &self.bio_browser {
            browser.end_frame();
            ctx.request_repaint_after(Duration::from_millis(33));
        }
        if self.file_dialog.is_some() {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
        crate::stellar_ui::prepare_font(ctx, &self.settings.plugins["StellarOverlay"]);
        let monitors = snapshot
            .native_popups
            .iter()
            .map(|p| {
                let rect = self
                    .game_focus
                    .monitor(&p.style.screen.to_string(), ctx.pixels_per_point());
                (p.style.screen, rect)
            })
            .collect::<BTreeMap<_, _>>();
        crate::native_popup::render(
            ctx,
            &snapshot.native_popups,
            |screen| monitors.get(&screen).copied().flatten(),
            |ctx, name| self.screenshot_child(ctx, name),
        );
        if snapshot.overlay.force_show
            || snapshot
                .overlay
                .alerts
                .iter()
                .any(|v| v["ForceShow"].as_bool() == Some(true))
            || self.game_focus.active() != Some(false)
        {
            let monitor = self.game_focus.monitor(
                snapshot.overlay.settings["SelectedMonitor"]
                    .as_str()
                    .unwrap_or("0"),
                ctx.pixels_per_point(),
            );
            crate::stellar_ui::render(ctx, &snapshot.overlay, monitor, |ctx, name| {
                self.screenshot_child(ctx, name)
            });
        }
    }
}

fn is_basic_ui(name: &str) -> bool {
    !matches!(
        name,
        "Core" | "BioInsights" | "Evaluator" | "StellarOverlay" | "Hecate"
    )
}
fn custom_themes(settings: &Settings) -> BTreeMap<String, CustomTheme> {
    settings
        .extra
        .get("themes")
        .and_then(Value::as_object)
        .map(|object| {
            object
                .iter()
                .filter_map(|(name, value)| {
                    serde_json::from_value(value.clone())
                        .ok()
                        .map(|theme| (name.clone(), theme))
                })
                .collect()
        })
        .unwrap_or_default()
}
#[cfg(test)]
fn store_theme(settings: &mut Settings, name: &str, theme: &CustomTheme) {
    let themes = settings.extra.entry("themes").or_insert(json!({}));
    if !themes.is_object() {
        *themes = json!({});
    }
    themes[name] = json!(theme);
}
fn apply_theme(ctx: &egui::Context, settings: &Settings) {
    let visuals = custom_themes(settings)
        .get(&settings.theme)
        .map(CustomTheme::visuals)
        .unwrap_or_else(|| original_visuals(&settings.theme));
    ctx.set_visuals(visuals);
    let mut style = (*ctx.style()).clone();
    style.spacing.button_padding = egui::vec2(6.0, 2.0);
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    for text_style in [egui::TextStyle::Body, egui::TextStyle::Button] {
        style
            .text_styles
            .insert(text_style, egui::FontId::proportional(13.0));
    }
    ctx.set_style(style);
    crate::core_theme::apply(ctx, settings);
}
fn original_visuals(base: &str) -> egui::Visuals {
    let light = base.eq_ignore_ascii_case("light");
    let mut visuals = if light {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };
    let background = if light {
        Color32::from_gray(240)
    } else {
        Color32::BLACK
    };
    let foreground = if light {
        Color32::BLACK
    } else {
        Color32::from_gray(211)
    };
    let button = if light {
        Color32::from_gray(227)
    } else {
        Color32::from_gray(105)
    };
    let grid = if light {
        Color32::from_gray(100)
    } else {
        Color32::from_gray(211)
    };
    visuals.override_text_color = Some(foreground);
    visuals.panel_fill = background;
    visuals.window_fill = background;
    visuals.extreme_bg_color = if light {
        Color32::WHITE
    } else {
        Color32::BLACK
    };
    visuals.faint_bg_color = if light {
        Color32::from_gray(235)
    } else {
        Color32::from_gray(20)
    };
    visuals.hyperlink_color = if light {
        Color32::BLUE
    } else {
        Color32::from_rgb(160, 160, 255)
    };
    visuals.selection.bg_fill = if light {
        Color32::from_gray(205)
    } else {
        Color32::from_gray(50)
    };
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, foreground);
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = egui::CornerRadius::ZERO;
        widget.fg_stroke = egui::Stroke::new(1.0_f32, foreground);
        widget.bg_stroke = egui::Stroke::NONE;
    }
    visuals.widgets.noninteractive.bg_fill = background;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, grid);
    visuals.widgets.inactive.bg_fill = button;
    visuals.widgets.inactive.weak_bg_fill = button;
    visuals.widgets.hovered.bg_fill = if light {
        Color32::from_gray(215)
    } else {
        Color32::from_gray(120)
    };
    visuals.widgets.active.bg_fill = if light {
        Color32::from_gray(200)
    } else {
        Color32::from_gray(169)
    };
    visuals.widgets.hovered.weak_bg_fill = visuals.widgets.hovered.bg_fill;
    visuals.widgets.active.weak_bg_fill = visuals.widgets.active.bg_fill;
    visuals.window_corner_radius = egui::CornerRadius::ZERO;
    visuals
}
fn original_tab_colors(base: &str) -> (Color32, Color32) {
    if base.eq_ignore_ascii_case("light") {
        (Color32::from_gray(227), Color32::from_gray(240))
    } else {
        (Color32::from_gray(105), Color32::from_gray(169))
    }
}
fn tab_colors(settings: &Settings) -> (Color32, Color32) {
    if let (Some(tab), Some(selected)) = (
        crate::core_theme::colour(settings, "ColourableTabControl", "TabColor"),
        crate::core_theme::colour(settings, "ColourableTabControl", "SelectedTabColor"),
    ) {
        return (tab, selected);
    }
    custom_themes(settings)
        .get(&settings.theme)
        .map(|t| {
            (
                Color32::from_rgba_unmultiplied(
                    t.widgets[0],
                    t.widgets[1],
                    t.widgets[2],
                    t.widgets[3],
                ),
                Color32::from_rgba_unmultiplied(t.accent[0], t.accent[1], t.accent[2], t.accent[3]),
            )
        })
        .unwrap_or_else(|| original_tab_colors(&settings.theme))
}
#[derive(Debug)]
struct OriginalLayout {
    tabs: egui::Rect,
    bottom: egui::Rect,
}
fn original_layout(bounds: egui::Rect) -> OriginalLayout {
    OriginalLayout {
        tabs: egui::Rect::from_min_max(
            bounds.min + egui::vec2(12.0, 12.0),
            egui::pos2(bounds.max.x - 12.0, bounds.max.y - 41.0),
        ),
        bottom: egui::Rect::from_min_max(
            egui::pos2(bounds.min.x + 12.0, bounds.max.y - 35.0),
            bounds.max - egui::vec2(12.0, 12.0),
        ),
    }
}
fn read_all_title(draw: u64) -> &'static str {
    if draw == 0 {
        "Reticulating Splines..."
    } else {
        "Read All In Progress..."
    }
}
fn format_line_total(total: u64) -> String {
    let digits = total.to_string();
    let mut output = String::new();
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            output.push(',');
        }
        output.push(character);
    }
    output
}
fn plugin_names(snapshot: &Snapshot) -> Vec<String> {
    let mut names = ["Explorer", "Botanist", "Communicator"]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    for name in snapshot
        .plugin_details
        .keys()
        .chain(snapshot.extensions.iter().map(|e| &e.0))
    {
        if !names.contains(name) {
            names.push(name.clone());
        }
    }
    names
}
fn available_tabs(snapshot: &Snapshot) -> Vec<String> {
    let mut tabs = vec!["Core".into()];
    tabs.extend(plugin_names(snapshot).into_iter().filter(|name| {
        snapshot
            .plugin_details
            .get(name)
            .and_then(|v| v["has_ui"].as_bool())
            .unwrap_or(true)
    }));
    tabs
}
fn normalize_tab_order(saved: &[String], available: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    saved
        .iter()
        .chain(available)
        .filter(|name| available.contains(name) && seen.insert((*name).clone()))
        .cloned()
        .collect()
}
fn sanitize_window(window: &mut PluginWindow) {
    for (index, dimension) in window.size.iter_mut().enumerate() {
        *dimension = if dimension.is_finite() {
            dimension.clamp(if index == 0 { 360.0 } else { 240.0 }, 8192.0)
        } else {
            if index == 0 {
                800.0
            } else {
                450.0
            }
        };
    }
    if window.position.is_some_and(|position| {
        position
            .iter()
            .any(|value| !value.is_finite() || value.abs() > 100_000.0)
    }) {
        window.position = None;
    }
}
fn enumerate_audio(shared: Arc<Mutex<AudioChoices>>) {
    thread::spawn(move || {
        let mut choices = AudioChoices::default();
        match command_text("espeak-ng", &["--voices"]) {
            Ok(text) => choices.voices = parse_voices(&text),
            Err(error) => choices
                .errors
                .push(format!("Voice list unavailable: {error}")),
        }
        match command_text("pactl", &["list", "short", "sinks"]) {
            Ok(text) => choices.devices = parse_sinks(&text),
            Err(error) => choices
                .errors
                .push(format!("Audio output list unavailable: {error}")),
        }
        choices.loaded = true;
        *shared.lock().unwrap() = choices;
    });
}
fn command_text(program: &str, args: &[&str]) -> std::result::Result<String, String> {
    let mut child = ProcessCommand::new(program)
        .args(args)
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| error.to_string())?;
    let stdout = child.stdout.take().unwrap();
    let reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout
            .take(256 * 1024)
            .read_to_end(&mut bytes)
            .map(|_| bytes)
    });
    let deadline = Instant::now() + Duration::from_secs(2);
    let result = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                break if status.success() {
                    Ok(())
                } else {
                    Err(format!("{program} exited {status}"))
                }
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(error.to_string());
            }
            Ok(None) => {}
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            break Err(format!("{program} timed out"));
        }
        thread::sleep(Duration::from_millis(10));
    };
    let bytes = reader
        .join()
        .map_err(|_| "Audio enumeration reader failed".to_owned())?
        .map_err(|error| error.to_string())?;
    result?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}
fn parse_voices(text: &str) -> Vec<(String, String)> {
    let mut seen = HashSet::new();
    text.lines()
        .filter_map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            if fields.len() < 5
                || fields[0].parse::<u32>().is_err()
                || !seen.insert(fields[4].to_owned())
            {
                return None;
            }
            Some((
                fields[4].into(),
                format!("{} ({})", fields[3].replace('_', " "), fields[1]),
            ))
        })
        .collect()
}
fn parse_sinks(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            (fields.len() >= 2).then(|| {
                (
                    fields[1].into(),
                    format!(
                        "{}{}",
                        fields[1],
                        fields
                            .last()
                            .filter(|_| fields.len() > 2)
                            .map(|state| format!(" ({state})"))
                            .unwrap_or_default()
                    ),
                )
            })
        })
        .collect()
}

fn value_editor(ui: &mut egui::Ui, name: &str, value: &mut Value) {
    match value {
        Value::Bool(value) => {
            ui.checkbox(value, name);
        }
        Value::String(value) => {
            ui.horizontal(|ui| {
                ui.label(name);
                ui.text_edit_singleline(value);
            });
        }
        Value::Number(value) => {
            let mut text = value.to_string();
            ui.horizontal(|ui| {
                ui.label(name);
                if ui.text_edit_singleline(&mut text).changed() {
                    if let Ok(number) = text.parse::<serde_json::Number>() {
                        *value = number;
                    }
                }
            });
        }
        Value::Object(object) => {
            for (key, value) in object {
                ui.push_id(key, |ui| {
                    value_editor(ui, key, value);
                });
            }
        }
        _ => {
            ui.label(format!("{name}: {value}"));
        }
    }
}
#[derive(Clone)]
struct CachedRows {
    key: String,
    rows: Arc<Vec<Vec<String>>>,
}
fn cached_rows(
    ctx: &egui::Context,
    id: egui::Id,
    key: String,
    build: impl FnOnce() -> Vec<Vec<String>>,
) -> Arc<Vec<Vec<String>>> {
    if let Some(cached) = ctx.data(|d| d.get_temp::<CachedRows>(id)) {
        if cached.key == key {
            return cached.rows;
        }
    }
    let rows = Arc::new(build());
    ctx.data_mut(|d| {
        d.insert_temp(
            id,
            CachedRows {
                key,
                rows: rows.clone(),
            },
        )
    });
    rows
}
fn grid_cell(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        Value::Bool(value) => {
            if *value {
                "True".into()
            } else {
                "False".into()
            }
        }
        value => value.to_string(),
    }
}
fn sort_rows(rows: &mut [Vec<String>], sort: usize, descending: bool) {
    if sort == usize::MAX {
        return;
    }
    rows.sort_by(|a, b| {
        let ordering = crate::extensions::compare_grid_text(
            a.get(sort).map(String::as_str).unwrap_or(""),
            b.get(sort).map(String::as_str).unwrap_or(""),
        );
        if descending {
            ordering.reverse()
        } else {
            ordering
        }
    });
}
#[cfg(test)]
fn natural_compare(left: &str, right: &str) -> std::cmp::Ordering {
    crate::extensions::compare_grid_text(left, right)
}
fn initial_column_sizing(settings: &Settings) -> Value {
    let mut widths = settings
        .extra
        .get("column_sizing")
        .filter(|value| value.is_object())
        .cloned()
        .unwrap_or(json!({}));
    widths["__versions"] = settings
        .extra
        .get("column_sizing_versions")
        .filter(|value| value.is_object())
        .cloned()
        .unwrap_or(json!({}));
    widths
}
fn source_column_widths(widths: &Value, plugin: &str, version: &str) -> Value {
    if widths["__versions"][plugin]
        .as_str()
        .is_some_and(|saved| saved != version)
    {
        Value::Null
    } else {
        widths[plugin].clone()
    }
}
#[derive(Clone, Default)]
struct GridSelection {
    selected: std::collections::HashSet<(u64, usize)>,
    anchor: Option<usize>,
    source: (usize, usize),
    keys: Arc<Vec<(u64, usize)>>,
}
fn source_row_keys(rows: &[Vec<String>]) -> Vec<(u64, usize)> {
    use std::hash::{Hash, Hasher};
    let mut counts = BTreeMap::new();
    rows.iter()
        .map(|row| {
            let mut hash = std::collections::hash_map::DefaultHasher::new();
            row.hash(&mut hash);
            let hash = hash.finish();
            let count = counts.entry(hash).or_insert(0);
            let key = (hash, *count);
            *count += 1;
            key
        })
        .collect()
}
fn selected_grid_text(rows: &[Vec<String>], selection: &GridSelection) -> String {
    let mut text = String::new();
    for (row, key) in rows.iter().zip(selection.keys.iter()) {
        if selection.selected.contains(key) {
            for cell in row {
                text.push_str(cell);
                text.push('\t');
            }
            text.push_str("\r\n");
        }
    }
    text
}
fn data_table(
    ui: &mut egui::Ui,
    headers: &[&str],
    rows: &[Vec<String>],
    sort: &mut usize,
    descending: &mut bool,
) {
    let ctx = ui.ctx().clone();
    let grid_id = egui::Id::new(("Core grid plugin", ctx.viewport_id()));
    let plugin = ctx
        .data(|data| data.get_temp::<String>(grid_id))
        .unwrap_or_default();
    let sizing = ctx.data(|data| {
        data.get_temp::<Arc<Mutex<Value>>>(egui::Id::new("Core source column sizing"))
    });
    let version_id = egui::Id::new(("Core grid version", &plugin));
    let version = ctx
        .data(|data| data.get_temp::<String>(version_id))
        .unwrap_or_default();
    let saved = sizing
        .as_ref()
        .map(|value| source_column_widths(&value.lock().unwrap(), &plugin, &version))
        .unwrap_or(Value::Null);
    let widths: Vec<f32> = headers
        .iter()
        .enumerate()
        .map(|(index, header)| {
            saved[*header]
                .as_f64()
                .filter(|n| n.is_finite() && *n >= 0.0 && *n < 16384.0)
                .map(|n| n as f32)
                .unwrap_or_else(|| match headers {
                    ["Time", "Body Name", "Description", "Details"] => {
                        [300.0, 350.0, 400.0, 600.0][index]
                    }
                    ["Body", "Bio Total", "Species", "Analysed", "Colony Distance"] => {
                        [300.0, 100.0, 300.0, 100.0, 100.0][index]
                    }
                    _ => header.encode_utf16().count() as f32 * 10.0,
                })
        })
        .collect();
    let selection_id = egui::Id::new(("Core grid selection", &plugin));
    let mut selection = ctx
        .data(|data| data.get_temp::<GridSelection>(selection_id))
        .unwrap_or_default();
    let source = (rows.as_ptr() as usize, rows.len());
    if selection.source != source {
        selection.keys = Arc::new(source_row_keys(rows));
        selection.source = source;
    }
    let height = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
    let modifiers = ui.input(|input| input.modifiers);
    let copy = ui.input(|input| {
        input
            .events
            .iter()
            .any(|event| matches!(event, egui::Event::Copy))
            || input.key_pressed(egui::Key::C)
    });
    if copy
        && modifiers.ctrl
        && !modifiers.alt
        && !modifiers.shift
        && !selection.selected.is_empty()
    {
        ctx.copy_text(selected_grid_text(rows, &selection));
    }
    egui::ScrollArea::horizontal()
        .id_salt(("Core grid horizontal", &plugin))
        .show(ui, |ui| {
            ui.set_min_width(
                widths.iter().sum::<f32>()
                    + ui.spacing().item_spacing.x * (headers.len().saturating_sub(1)) as f32,
            );
            let mut table = TableBuilder::new(ui)
                .striped(false)
                .resizable(true)
                .sense(egui::Sense::click())
                .min_scrolled_height(120.0);
            for width in &widths {
                table = table.column(Column::initial(*width).at_least(0.0).clip(true));
            }
            table
                .header(height, |mut row| {
                    for (index, title) in headers.iter().enumerate() {
                        let mut clicked = false;
                        let (_, response) = row.col(|ui| {
                            clicked = ui
                                .add_sized(
                                    ui.available_size(),
                                    egui::Label::new(RichText::new(*title).strong())
                                        .sense(egui::Sense::click()),
                                )
                                .clicked();
                        });
                        if clicked || response.clicked() {
                            if *sort == index {
                                *descending = !*descending;
                            } else {
                                *sort = index;
                                *descending = false;
                            }
                        }
                    }
                })
                .body(|body| {
                    if let Some(sizing) = &sizing {
                        let mut sizing = sizing.lock().unwrap();
                        for (header, width) in headers.iter().zip(body.widths()) {
                            sizing[&plugin][*header] = json!(width);
                        }
                        sizing["__versions"][&plugin] = json!(version);
                    }
                    body.rows(height, rows.len(), |mut row| {
                        let index = row.index();
                        let selected = selection.selected.contains(&selection.keys[index]);
                        let base = ui_grid_row_background(&ctx, index, selected);
                        for cell in &rows[index] {
                            row.col(|ui| {
                                ui.painter().rect_filled(
                                    ui.max_rect(),
                                    egui::CornerRadius::ZERO,
                                    base,
                                );
                                ui.add(egui::Label::new(cell).truncate().selectable(false));
                            });
                        }
                        if row.response().clicked() {
                            if modifiers.shift {
                                let anchor = selection
                                    .anchor
                                    .unwrap_or(index)
                                    .min(rows.len().saturating_sub(1));
                                if !modifiers.ctrl {
                                    selection.selected.clear();
                                }
                                for key in &selection.keys[anchor.min(index)..=anchor.max(index)] {
                                    selection.selected.insert(*key);
                                }
                            } else {
                                let key = selection.keys[index];
                                if modifiers.ctrl {
                                    if !selection.selected.insert(key) {
                                        selection.selected.remove(&key);
                                    }
                                } else {
                                    selection.selected.clear();
                                    selection.selected.insert(key);
                                }
                                selection.anchor = Some(index);
                            }
                            ctx.request_repaint();
                        }
                    });
                });
        });
    ctx.data_mut(|data| data.insert_temp(selection_id, selection));
}
fn ui_grid_row_background(ctx: &egui::Context, index: usize, selected: bool) -> Color32 {
    let base = ctx.style().visuals.extreme_bg_color;
    let shift = |color: u8, amount: u8| {
        if color > 127 {
            color.saturating_sub(amount)
        } else {
            color.saturating_add(amount)
        }
    };
    let delta = if index % 2 == 1 { 20 } else { 0 };
    let [r, g, b, _] = base.to_array();
    let color = [shift(r, delta), shift(g, delta), shift(b, delta)];
    if selected {
        Color32::from_rgb(
            shift(color[0], 50),
            shift(color[1], 50),
            shift(color[2], 50),
        )
    } else {
        Color32::from_rgb(color[0], color[1], color[2])
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn read_all_source_title_has_exactly_one_easter_egg_in_twenty_outcomes() {
        assert_eq!(super::read_all_title(0), "Reticulating Splines...");
        for draw in 1..20 {
            assert_eq!(super::read_all_title(draw), "Read All In Progress...");
        }
    }
    use super::*;
    #[test]
    fn source_n0_line_counter_keeps_full_integer_precision() {
        assert_eq!(format_line_total(0), "0");
        assert_eq!(format_line_total(1234567), "1,234,567");
        assert_eq!(format_line_total(u64::MAX), "18,446,744,073,709,551,615");
    }
    #[test]
    fn default_palette_matches_original_theme_manager_rgb_values() {
        let dark = original_visuals("Dark");
        assert_eq!(dark.panel_fill.to_array(), [0, 0, 0, 255]);
        assert_eq!(dark.text_color().to_array(), [211, 211, 211, 255]);
        assert_eq!(
            dark.widgets.inactive.bg_fill.to_array(),
            [105, 105, 105, 255]
        );
        assert_eq!(
            original_tab_colors("Dark").0.to_array(),
            [105, 105, 105, 255]
        );
        assert_eq!(
            original_tab_colors("Dark").1.to_array(),
            [169, 169, 169, 255]
        );
        let light = original_visuals("Light");
        assert_eq!(light.panel_fill.to_array(), [240, 240, 240, 255]);
        assert_eq!(light.text_color().to_array(), [0, 0, 0, 255]);
        assert_eq!(light.extreme_bg_color.to_array(), [255, 255, 255, 255]);
    }
    #[test]
    fn shell_rectangles_match_original_designer_coordinates() {
        let layout = original_layout(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(630.0, 361.0),
        ));
        assert_eq!(layout.tabs.min, egui::pos2(12.0, 12.0));
        assert_eq!(layout.tabs.size(), egui::vec2(606.0, 308.0));
        assert_eq!(layout.bottom.min, egui::pos2(12.0, 326.0));
        assert_eq!(layout.bottom.max, egui::pos2(618.0, 349.0));
    }
    #[test]
    fn core_is_default_and_notifiers_do_not_get_worker_tabs() {
        let mut snapshot = Snapshot::default();
        snapshot
            .plugin_details
            .insert("StellarOverlay".into(), json!({"has_ui":false}).into());
        snapshot
            .plugin_details
            .insert("BioInsights".into(), json!({"has_ui":true}).into());
        assert_eq!(available_tabs(&snapshot)[0], "Core");
        assert!(available_tabs(&snapshot).contains(&"BioInsights".to_owned()));
        assert!(!available_tabs(&snapshot).contains(&"StellarOverlay".to_owned()));
        assert!(!available_tabs(&snapshot).contains(&"Settings".to_owned()));
    }
    #[test]
    fn source_grid_width_restore_requires_matching_version_and_clipboard_keeps_source_delimiters() {
        let mut settings = Settings::default();
        settings.extra.insert(
            "column_sizing".into(),
            json!({"Explorer":{"Body Name":321}}),
        );
        settings.extra.insert(
            "column_sizing_versions".into(),
            json!({"Explorer":"1.3.0.27422"}),
        );
        let widths = initial_column_sizing(&settings);
        assert_eq!(
            source_column_widths(&widths, "Explorer", "1.3.0.27422")["Body Name"],
            321
        );
        assert!(source_column_widths(&widths, "Explorer", "2.0.0").is_null());
        let rows = vec![
            vec!["a".into(), "b".into()],
            vec!["same".into(), "row".into()],
            vec!["same".into(), "row".into()],
        ];
        let mut selection = GridSelection {
            keys: Arc::new(source_row_keys(&rows)),
            ..Default::default()
        };
        selection.selected.insert(selection.keys[0]);
        selection.selected.insert(selection.keys[2]);
        assert_eq!(
            selected_grid_text(&rows, &selection),
            "a\tb\t\r\nsame\trow\t\r\n"
        );
    }
    #[test]
    fn body_names_sort_naturally_and_large_numbers_do_not_overflow() {
        let mut rows = vec![
            vec!["Test A 10".into()],
            vec!["Test A 2".into()],
            vec!["Test A 0003".into()],
        ];
        sort_rows(&mut rows, 0, false);
        assert_eq!(
            rows,
            vec![vec!["Test A 2"], vec!["Test A 0003"], vec!["Test A 10"]]
        );
        assert!(natural_compare("Body 184467440737095516160", "Body 9").is_gt());
    }
    #[test]
    fn custom_theme_colors_round_trip_and_apply_to_native_visuals() {
        let mut settings = Settings::default();
        let mut theme = CustomTheme::from_base("dark");
        theme.panel = [12, 34, 56, 255];
        theme.text = [240, 241, 242, 255];
        store_theme(&mut settings, "Night sky", &theme);
        settings.theme = "Night sky".into();
        let serialized = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&serialized).unwrap();
        assert_eq!(custom_themes(&restored)["Night sky"], theme);
        let ctx = egui::Context::default();
        apply_theme(&ctx, &restored);
        assert_eq!(ctx.style().visuals.panel_fill.to_array(), theme.panel);
        assert_eq!(ctx.style().visuals.text_color().to_array(), theme.text);
    }
    #[test]
    fn tab_order_keeps_saved_positions_deduplicates_and_appends_new_plugins() {
        let saved = ["Botanist", "Gone", "Botanist", "Explorer"].map(str::to_owned);
        let available = ["Explorer", "Botanist", "Settings", "New extension"].map(str::to_owned);
        assert_eq!(
            normalize_tab_order(&saved, &available),
            ["Botanist", "Explorer", "Settings", "New extension"]
        );
    }
    #[test]
    fn detached_window_preferences_round_trip_and_invalid_geometry_recovers() {
        let mut window = PluginWindow {
            open: true,
            position: Some([-100.0, 25.0]),
            size: [900.0, 600.0],
            filter: "Earth-like".into(),
            sort: 3,
            descending: true,
            on_top: true,
            title_bar: false,
            hide_buttons: true,
        };
        let restored: PluginWindow =
            serde_json::from_str(&serde_json::to_string(&window).unwrap()).unwrap();
        assert!(restored.open);
        assert_eq!(restored.position, window.position);
        assert_eq!(restored.filter, window.filter);
        window.size = [f32::NAN, -1.0];
        window.position = Some([f32::INFINITY, 1.0]);
        sanitize_window(&mut window);
        assert_eq!(window.size, [800.0, 240.0]);
        assert_eq!(window.position, None);
    }
    #[test]
    fn installed_voice_and_pulseaudio_lists_produce_backend_identifiers() {
        let voices=parse_voices("Pty Language Age/Gender VoiceName File Other Languages\n 5 en-us --/M English_(America) gmw/en-US\n 5 fr --/M French roa/fr\n");
        assert_eq!(
            voices[0],
            ("gmw/en-US".into(), "English (America) (en-us)".into())
        );
        let sinks =
            parse_sinks("42\talsa_output.pci-stereo\tPipeWire\ts16le 2ch 48000Hz\tRUNNING\n");
        assert_eq!(sinks[0].0, "alsa_output.pci-stereo");
        assert!(sinks[0].1.ends_with("(RUNNING)"));
    }
}
