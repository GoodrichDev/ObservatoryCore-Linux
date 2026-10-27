//! Native rendering of Evaluator's original custom HTML interface and embedded themes.
use eframe::egui::{self, Align2, Color32, FontId, Id, Rect, Sense, Vec2};
use serde_json::{json, Value};

#[path = "evaluator_native.rs"]
mod native;
#[path = "evaluator_settings.rs"]
mod settings_view;
#[path = "evaluator_symbols.rs"]
mod symbols;
pub use settings_view::show as settings;

pub fn popout_visible(settings: &Value, gui_focus: u64, _flags: u64, _population: u64) -> bool {
    let enabled = |key: &str| settings[key].as_bool().unwrap_or(true);
    enabled("PopoutAlwaysShow")
        || match gui_focus {
            0 => enabled("PopoutShowNoFocus"),
            2 => enabled("PopoutShowLeftPanel"),
            1 => enabled("PopoutShowRightPanel"),
            10 => enabled("PopoutShowDSSMode"),
            4 => enabled("PopoutShowRolePanel"),
            7 => enabled("PopoutShowSysMap"),
            9 => enabled("PopoutShowFSSMode"),
            _ => false,
        }
}

pub fn popout_menu(
    ui: &mut egui::Ui,
    chrome: &mut crate::bioinsights_main::popout::Chrome,
) -> Vec<Value> {
    let mut actions = Vec::new();
    ui.menu_button("Navigate", |ui| {
        for (label, route) in [
            ("Main Window", "Main"),
            ("Show Current Data", "Unsold"),
            ("About", "About"),
        ] {
            if ui.button(label).clicked() {
                ui.ctx().data_mut(|d| {
                    d.insert_persisted(Id::new("evaluator_original_page"), route.to_owned())
                });
                if route == "Unsold" {
                    actions.push(command("Unsold", json!({})));
                }
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
            let action = match label {
                "Toggle OnTop" => {
                    chrome.on_top = !chrome.on_top;
                    command("SavePopoutPosition", json!({}))
                }
                "Lock/Unlock Position" => {
                    chrome.title_bar = !chrome.title_bar;
                    command("SavePopoutPosition", json!({}))
                }
                "Toggle Buttons" => {
                    chrome.hide_buttons = !chrome.hide_buttons;
                    command("SavePopoutPosition", json!({}))
                }
                "Close Popout" => command("Popout", json!({"Restore":true})),
                _ => command("SavePopoutPosition", json!({})),
            };
            actions.push(action);
            ui.close_menu();
        }
    }
    actions
}

struct Theme {
    data: &'static Value,
    overrides: std::collections::BTreeMap<String, Color32>,
    orange: bool,
}
impl Theme {
    fn new(settings: &Value) -> Self {
        let orange = settings["ThemeSettings"]["ThemeIndex"]
            .as_u64()
            .or_else(|| settings["ThemeSettings"]["Theme"].as_u64())
            .unwrap_or(1)
            == 1;
        use std::sync::OnceLock;
        static ORANGE: OnceLock<Value> = OnceLock::new();
        static ORIGINAL: OnceLock<Value> = OnceLock::new();
        let data = if orange {
            ORANGE.get_or_init(|| {
                serde_json::from_str(include_str!("../assets/evaluator/theme-orange.json"))
                    .expect("installed Evaluator orange theme")
            })
        } else {
            ORIGINAL.get_or_init(|| {
                serde_json::from_str(include_str!("../assets/evaluator/theme.json"))
                    .expect("installed Evaluator default theme")
            })
        };
        let overrides = if settings["ThemeSettings"]["ThemeMode"] == 1 {
            settings["ThemeSettings"]["Colors"]
                .as_object()
                .map(|colors| {
                    colors
                        .iter()
                        .map(|(name, c)| {
                            (
                                name.clone(),
                                Color32::from_rgb(
                                    c["R"].as_u64().unwrap_or(0) as u8,
                                    c["G"].as_u64().unwrap_or(0) as u8,
                                    c["B"].as_u64().unwrap_or(0) as u8,
                                ),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Default::default()
        };
        Self {
            data,
            overrides,
            orange,
        }
    }
    fn color(&self, key: &str) -> Color32 {
        if let Some(color) = self.overrides.get(key) {
            return *color;
        }
        let c = &self.data["Colors"][key];
        Color32::from_rgb(
            c["R"].as_u64().unwrap_or(255) as u8,
            c["G"].as_u64().unwrap_or(255) as u8,
            c["B"].as_u64().unwrap_or(255) as u8,
        )
    }
    fn row_height(&self) -> f32 {
        self.data["Settings"]["RowHeight"].as_f64().unwrap_or(24.0) as f32
    }
    fn spacing(&self) -> f32 {
        self.data["Settings"]["RowSpacing"].as_f64().unwrap_or(0.0) as f32
    }
    fn symbol_color(&self, key: &str, background: bool) -> Color32 {
        let symbol = &self.data["Symbols"][key];
        let field = if background {
            "ContentBackColor"
        } else {
            "ContentColor"
        };
        let color = &symbol[field];
        if color.is_object() {
            return Color32::from_rgb(
                color["R"].as_u64().unwrap_or(0) as u8,
                color["G"].as_u64().unwrap_or(0) as u8,
                color["B"].as_u64().unwrap_or(0) as u8,
            );
        }
        self.color(
            symbol[format!("{field}Key")]
                .as_str()
                .unwrap_or(if background {
                    "DefaultBackground"
                } else {
                    "DefaultText"
                }),
        )
    }
}
fn attribute_symbols(body: &Value) -> Vec<&'static str> {
    let mut symbols = Vec::new();
    for (field, key) in [
        ("IsLandable", "IsLandable"),
        ("IsHighValue", "IsHighValue"),
        ("Terraformable", "Terraformable"),
    ] {
        if body[field] == true {
            symbols.push(key);
        }
    }
    if body["WasMapped"] == true {
        symbols.push("WasMapped");
    } else if body["WasDiscovered"] == true {
        symbols.push("WasDiscovered");
    }
    if body["WasFootfalled"] == true {
        symbols.push("WasFootfalled");
    }
    if body["BioCount"].as_u64().is_some_and(|v| v > 0) {
        symbols.push("HasBiologicals");
    }
    if !body["Volcanism"].as_str().unwrap_or("").is_empty() {
        symbols.push("HasVolcanism");
    }
    for key in [
        "BioInsightsPersonalCodex",
        "BioInsightsMarked",
        "BioInsightsGalacticCodex",
        "BioInsightsHighValue",
        "PluginWorthVisiting",
    ] {
        if body[key] == true {
            symbols.push(key);
        }
    }
    symbols
}
fn star_type(kind: &str) -> String {
    use std::sync::OnceLock;
    static TYPES: OnceLock<Value> = OnceLock::new();
    TYPES.get_or_init(|| {
        serde_json::from_str(include_str!("../assets/evaluator/star-types.json"))
            .expect("original star labels")
    })[kind]
        .as_str()
        .unwrap_or(kind)
        .replace("&nbsp;", "\u{a0}")
}
fn gravity_label(g: f64) -> String {
    format!(
        "{} G",
        format!("{g:.2}")
            .trim_end_matches('0')
            .trim_end_matches('.')
    )
}
fn formatted(n: i64) -> String {
    let source = n.to_string();
    let mut text = String::new();
    for (index, c) in source.chars().enumerate() {
        if index > 0 && (source.len() - index).is_multiple_of(3) && c != '-' && !text.ends_with('-')
        {
            text.push(' ');
        }
        text.push(c);
    }
    text
}
fn symbol_data(theme: &Theme, key: &str) -> Option<(String, Vec<u8>)> {
    use base64::Engine as _;
    let symbol = &theme.data["Symbols"][key];
    let kind = symbol["ContentType"].as_str()?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(symbol["Content"].as_str()?)
        .ok()?;
    let bytes = if kind == "Base64SVG" {
        let color = theme.symbol_color(key, false);
        String::from_utf8_lossy(&bytes)
            .replace(
                "$$COLOR$$",
                &format!("{:02X}{:02X}{:02X}", color.r(), color.g(), color.b()),
            )
            .into_bytes()
    } else {
        bytes
    };
    Some((kind.into(), bytes))
}
fn symbol_text(theme: &Theme, key: &str) -> String {
    let text = theme.data["Symbols"][key]["Content"].as_str().unwrap_or("");
    if !text.starts_with('\\') {
        return text.to_owned();
    }
    text.split('\\')
        .filter(|part| !part.is_empty())
        .filter_map(|part| u32::from_str_radix(part, 16).ok().and_then(char::from_u32))
        .collect()
}
fn symbol_size(ui: &egui::Ui, theme: &Theme, key: &str, scale: f32) -> Vec2 {
    let symbol = &theme.data["Symbols"][key];
    if symbol["ContentType"] == "Text" {
        return Vec2::new(
            native::measured(
                ui,
                &symbol_text(theme, key),
                15.0 * scale,
                symbol["FontWeight"] == "bold",
                "Segoe UI Emoji",
            ),
            18.0 * scale,
        );
    }
    use std::sync::{Mutex, OnceLock};
    static SIZES: OnceLock<Mutex<std::collections::BTreeMap<(bool, String), Vec2>>> =
        OnceLock::new();
    let sizes = SIZES.get_or_init(Default::default);
    let id = (theme.orange, key.to_owned());
    if let Some(size) = sizes.lock().unwrap().get(&id).copied() {
        return size * scale;
    }
    let intrinsic = symbol_data(theme, key).and_then(|(kind, bytes)| {
        if kind == "Base64SVG" {
            crate::svg_image::intrinsic_size(&bytes).ok()
        } else {
            image::load_from_memory(&bytes)
                .ok()
                .map(|image| Vec2::new(image.width() as f32, image.height() as f32))
        }
    });
    let size = intrinsic
        .filter(|size| size.y > 0.0)
        .map(|size| Vec2::new(16.0 * size.x / size.y, 16.0))
        .unwrap_or(Vec2::splat(16.0));
    sizes.lock().unwrap().insert(id, size);
    size * scale
}
fn icon(ui: &mut egui::Ui, theme: &Theme, key: &str, r: Rect) {
    let symbol = &theme.data["Symbols"][key];
    if let Some(description) = symbol["Description"]
        .as_str()
        .filter(|text| !text.is_empty())
    {
        ui.interact(
            r,
            Id::new((
                "Evaluator source symbol title",
                key,
                r.left().to_bits(),
                r.top().to_bits(),
            )),
            Sense::hover(),
        )
        .on_hover_text(description);
    }

    if symbol["ContentType"] == "Text" {
        native::draw(
            ui,
            r,
            &symbol_text(theme, key),
            theme.symbol_color(key, false),
            Align2::CENTER_CENTER,
            r.height().min(
                15.0 * ui
                    .ctx()
                    .data(|d| d.get_temp::<f32>(Id::new("Evaluator CSS scale")))
                    .unwrap_or(1.0),
            ),
            symbol["FontWeight"] == "bold",
            "Segoe UI Emoji",
            0.0,
        );
        return;
    }
    if let Some((kind, bytes)) = symbol_data(theme, key) {
        if kind == "Base64SVG" && crate::svg_image::paint_svg(ui, &bytes, key, r).is_ok() {
            return;
        }
        if kind == "Base64PNG" {
            let id = Id::new(("Evaluator source PNG", key, theme.orange));
            let texture = ui
                .ctx()
                .data(|data| data.get_temp::<egui::TextureHandle>(id))
                .or_else(|| {
                    let image = image::load_from_memory(&bytes).ok()?.to_rgba8();
                    let texture = ui.ctx().load_texture(
                        key,
                        egui::ColorImage::from_rgba_unmultiplied(
                            [image.width() as usize, image.height() as usize],
                            image.as_raw(),
                        ),
                        egui::TextureOptions::LINEAR,
                    );
                    ui.ctx()
                        .data_mut(|data| data.insert_temp(id, texture.clone()));
                    Some(texture)
                });
            if let Some(texture) = texture {
                let size = texture.size_vec2();
                let size = size * (r.width() / size.x).min(r.height() / size.y);
                ui.painter().image(
                    texture.id(),
                    Rect::from_center_size(r.center(), size),
                    Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                return;
            }
        }
    }
    if let Some((name, bytes)) = symbols::symbol(theme.orange, key) {
        let id = Id::new(("Evaluator fallback icon", name));
        let texture = ui
            .ctx()
            .data(|data| data.get_temp::<egui::TextureHandle>(id))
            .or_else(|| {
                let image = image::load_from_memory(bytes).ok()?.to_rgba8();
                let texture = ui.ctx().load_texture(
                    name,
                    egui::ColorImage::from_rgba_unmultiplied(
                        [image.width() as usize, image.height() as usize],
                        image.as_raw(),
                    ),
                    egui::TextureOptions::LINEAR,
                );
                ui.ctx()
                    .data_mut(|data| data.insert_temp(id, texture.clone()));
                Some(texture)
            });
        if let Some(texture) = texture {
            ui.painter().image(
                texture.id(),
                r,
                Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
    }
}
fn command(name: &str, payload: Value) -> Value {
    json!({"command":name,"payload":payload})
}
fn cell(rect: Rect, widths: &[f32; 10], index: usize, span: usize) -> Rect {
    let x = rect.left() + widths[..index].iter().sum::<f32>();
    Rect::from_min_size(
        egui::pos2(x, rect.top()),
        egui::vec2(widths[index..index + span].iter().sum(), rect.height()),
    )
}
fn row(ui: &mut egui::Ui, width: f32, height: f32, background: Color32) -> Rect {
    let (r, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    if ui.is_rect_visible(r) {
        ui.painter().rect_filled(r, 0.0, background);
    }
    r
}
/// Shared installed EDJP UIBlankView text, point-size and alignment. The native
/// panel supplies bounds instead of reproducing WinForms default-label anchors.
pub(crate) fn draw_disabled_output(ui: &mut egui::Ui) {
    crate::font_registry::prepare_exact(ui.ctx(), "EDJP disabled output label", "Arial");
    ui.painter().text(
        ui.max_rect().center(),
        Align2::CENTER_CENTER,
        "Native Output Disabled",
        crate::font_registry::font(ui.ctx(), "EDJP disabled output label", 20.0 * 96.0 / 72.0),
        ui.visuals().text_color(),
    );
}
pub fn show(ui: &mut egui::Ui, summary: &Value, rows: &[Value], settings: &Value) -> Vec<Value> {
    show_variant(ui, summary, rows, settings, false)
}
pub fn show_variant(
    ui: &mut egui::Ui,
    summary: &Value,
    rows: &[Value],
    settings: &Value,
    hide_buttons: bool,
) -> Vec<Value> {
    let enabled_id = Id::new("Evaluator source NativeOutput at initialization");
    let native_enabled = ui
        .ctx()
        .data(|d| d.get_temp::<bool>(enabled_id))
        .unwrap_or_else(|| {
            let enabled = !settings["NativeOutput"]
                .as_str()
                .unwrap_or(" Enabled")
                .starts_with("Disabled");
            ui.ctx().data_mut(|d| d.insert_temp(enabled_id, enabled));
            enabled
        });
    if !native_enabled {
        draw_disabled_output(ui);
        return Vec::new();
    }
    let theme = Theme::new(settings);
    let mut actions = Vec::new();
    let mut zoom = summary["ZoomLevel"]
        .as_u64()
        .or_else(|| settings["ZoomLevel"].as_u64())
        .unwrap_or(100) as u32;
    let scale = zoom as f32 / 100.0;
    ui.ctx()
        .data_mut(|d| d.insert_temp(Id::new("Evaluator CSS scale"), scale));
    let page_id = Id::new("evaluator_original_page");
    let mut page = ui
        .ctx()
        .data_mut(|d| d.get_persisted::<String>(page_id))
        .unwrap_or_else(|| "Main".into());
    let zoom_id = Id::new("evaluator_zoom_open");
    let mut zoom_at = ui.ctx().data(|d| d.get_temp::<egui::Pos2>(zoom_id));
    egui::ScrollArea::vertical()
        .id_salt(("Evaluator source document", page.clone()))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Frame::new()
                .fill(theme.color("DefaultBackground"))
                .inner_margin(8.0 * scale)
                .show(ui, |ui| {
                    ui.visuals_mut().override_text_color = Some(theme.color("DefaultText"));
                    ui.style_mut().override_font_id = Some(FontId::proportional(15.0 * scale));
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    if !hide_buttons {
                        let size =
                            match settings["ButtonSize"].as_str().unwrap_or("  Normal").trim() {
                                "Smaller" => 16.0,
                                "Small" => 24.0,
                                "Large" => 48.0,
                                "Extra Large" => 64.0,
                                _ => 32.0,
                            } * scale;
                        let margin = size / 16.0;
                        let (bar, _) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), size + 2.0 * margin + 4.0 * scale),
                            Sense::hover(),
                        );
                        ui.painter().rect_filled(
                            Rect::from_min_max(bar.min, bar.max + Vec2::new(4.0 * scale, 0.0)),
                            0,
                            theme.color("ButtonBar"),
                        );
                        let mut x = bar.left() + 2.0 * scale + margin;
                        {
                            for (key, tooltip, action) in [
                                ("main", "Main Window", "Main"),
                                ("unsold", "Show Current Data", "Unsold"),
                                ("about", "About", "About"),
                                ("popout", "Popout Window", "Popout"),
                                ("zoom", "Zoom", "Zoom"),
                                ("settings", "Settings", "Settings"),
                            ] {
                                let rect = Rect::from_min_size(
                                    egui::pos2(x, bar.top() + 2.0 * scale + margin),
                                    Vec2::splat(size),
                                );
                                let response = ui.interact(
                                    rect,
                                    Id::new(("Evaluator source toolbar", action)),
                                    Sense::click(),
                                );
                                x += size + 2.0 * margin + 2.0 * scale;
                                ui.painter().rect_filled(
                                    rect,
                                    5.0 * scale,
                                    theme.color(if response.hovered() {
                                        "ButtonFaceHighlight"
                                    } else {
                                        "ButtonFace"
                                    }),
                                );
                                icon(
                                    ui,
                                    &theme,
                                    &format!("Button-{key}"),
                                    rect.shrink(if response.hovered() {
                                        2.0 * scale
                                    } else {
                                        4.0 * scale
                                    }),
                                );
                                if response.on_hover_text(tooltip).clicked() {
                                    match action {
                                        "Main" | "About" => page = action.into(),
                                        "Unsold" => {
                                            page = "Unsold".into();
                                            actions.push(command(action, json!({})));
                                        }
                                        "Zoom" => {
                                            zoom_at = Some(
                                                ui.ctx().pointer_latest_pos().unwrap_or_default(),
                                            )
                                        }
                                        "Popout" => actions.push(command(
                                            action,
                                            json!({"Restore":!ui.input(|i|i.modifiers.ctrl)}),
                                        )),
                                        _ => actions.push(command(action, json!({}))),
                                    }
                                }
                            }
                        }
                    }
                    egui::Frame::NONE
                        .inner_margin(egui::Margin {
                            left: (2.0 * scale).round() as i8,
                            top: (5.0 * scale).round() as i8,
                            right: 0,
                            bottom: 0,
                        })
                        .show(ui, |ui| {
                            for alert in summary["UIAlerts"]
                                .as_array()
                                .map(Vec::as_slice)
                                .unwrap_or(&[])
                            {
                                native::show_alert(ui, alert, scale, &mut actions);
                            }
                            if page == "About" {
                                native::show_about(ui, &theme, scale);
                                return;
                            }
                            if page == "Unsold" {
                                native::show_unsold(ui, summary, &theme, scale);
                                return;
                            }
                            native::show(ui, summary, rows, settings, &theme, scale, &mut actions);
                        });
                });
        });
    ui.ctx().data_mut(|d| d.insert_persisted(page_id, page));
    if let Some(at) = zoom_at {
        let popup = egui::Area::new(zoom_id)
            .order(egui::Order::Foreground)
            .fixed_pos(at)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_width(493.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().slider_width = 365.0;
                        let changed = ui
                            .add(
                                egui::Slider::new(&mut zoom, 50..=600)
                                    .step_by(5.0)
                                    .show_value(false),
                            )
                            .changed();
                        ui.label(format!("{zoom}%"));
                        let reset = ui
                            .add_sized([70.0, 37.0], egui::Button::new("Reset"))
                            .clicked();
                        if reset {
                            zoom = 100;
                        }
                        if changed || reset {
                            actions.push(command("SetZoomLevel", json!({"ZoomLevel":zoom})));
                        }
                    });
                });
            });
        if ui.input(|i| {
            i.key_pressed(egui::Key::Escape)
                || !i.focused
                || (i.pointer.any_pressed()
                    && i.pointer
                        .interact_pos()
                        .is_some_and(|p| !popup.response.rect.contains(p)))
        }) {
            zoom_at = None;
        }
    }
    ui.ctx().data_mut(|d| {
        if let Some(at) = zoom_at {
            d.insert_temp(zoom_id, at);
        } else {
            d.remove::<egui::Pos2>(zoom_id);
        }
    });
    actions
}

fn source_date(value: &str) -> String {
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let month = value
        .get(5..7)
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|m| (1..=12).contains(m));
    match (value.get(8..10), month, value.get(2..4), value.get(11..16)) {
        (Some(day), Some(month), Some(year), Some(time)) => {
            format!("{day}-{}-{year} {time}", months[month - 1])
        }
        _ => value.to_owned(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn frame(
        ctx: &egui::Context,
        events: Vec<egui::Event>,
        modifiers: egui::Modifiers,
    ) -> (Vec<Value>, egui::FullOutput) {
        let mut commands = Vec::new();
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(1400.0, 900.0),
                )),
                events,
                modifiers,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    commands = show(
                        ui,
                        &json!({"Commander":"Oracle","HecateAvailable":true,"ZoomLevel":100}),
                        &[],
                        &json!({}),
                    );
                });
            },
        );
        (commands, output)
    }
    fn custom_frame(
        ctx: &egui::Context,
        settings: &Value,
        summary: &Value,
        events: Vec<egui::Event>,
    ) -> (Vec<Value>, egui::FullOutput) {
        let mut commands = Vec::new();
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(1400.0, 900.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    commands = show(ui, summary, &[], settings);
                });
            },
        );
        (commands, output)
    }
    fn painted_text(output: &egui::FullOutput, value: &str) -> Option<egui::Pos2> {
        output.shapes.iter().find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == value => {
                Some(text.pos + text.galley.size() / 2.0)
            }
            _ => None,
        })
    }
    #[test]
    fn native_output_prefix_and_restart_latch_match_original_constructor() {
        for (initial, disabled) in [
            ("Disabled", true),
            ("Disabled (restart required)", true),
            ("disabled", false),
            (" Enabled", false),
        ] {
            let ctx = egui::Context::default();
            let summary = json!({});
            let (_, output) =
                custom_frame(&ctx, &json!({"NativeOutput":initial}), &summary, vec![]);
            assert_eq!(
                painted_text(&output, "Native Output Disabled").is_some(),
                disabled,
                "{initial}"
            );
            let (_, changed) = custom_frame(
                &ctx,
                &json!({"NativeOutput":if disabled {" Enabled"}else{"Disabled"}}),
                &summary,
                vec![],
            );
            assert_eq!(
                painted_text(&changed, "Native Output Disabled").is_some(),
                disabled,
                "restart latch {initial}"
            );
            assert_eq!(changed.viewport_output.len(), 1);
        }
    }
    #[test]
    fn source_banner_dismiss_dispatches_from_real_pointer_without_new_window() {
        let ctx = egui::Context::default();
        let settings = json!({});
        let summary = json!({"UIAlerts":[{"ID":1,"Class":"alert-info","Description":"An update is available"}]});
        let (_, output) = custom_frame(&ctx, &settings, &summary, vec![]);
        let point = painted_text(&output, "×").expect("source banner close button");
        let modifiers = egui::Modifiers::default();
        custom_frame(
            &ctx,
            &settings,
            &summary,
            vec![
                egui::Event::PointerMoved(point),
                egui::Event::PointerButton {
                    pos: point,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers,
                },
            ],
        );
        let (commands, output) = custom_frame(
            &ctx,
            &settings,
            &summary,
            vec![egui::Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers,
            }],
        );
        assert_eq!(commands, vec![command("DismissUIAlert", json!({"ID":1}))]);
        assert_eq!(output.viewport_output.len(), 1);
    }
    #[test]
    fn large_read_all_history_paints_only_visible_rows() {
        let mut systems = Vec::new();
        let mut rows = Vec::new();
        for id in 0..200 {
            systems.push(json!({"ID64":id,"System":format!("System {id}"),"Current":id==0}));
            for body in 0..20 {
                rows.push(json!({"ID64":id,"BodyID":body,"Body":body.to_string(),"Type":"Earthlike body"}));
            }
        }
        let ctx = egui::Context::default();
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(1400.0, 900.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    show(ui, &json!({"Systems":systems}), &rows, &json!({}));
                });
            },
        );
        assert!(
            output.shapes.len() < 2000,
            "4000 rows produced {} painted shapes",
            output.shapes.len()
        );
    }
    #[test]
    fn all_source_toolbar_buttons_dispatch_without_invented_child_windows() {
        for (index, action) in ["Main", "Unsold", "About", "Popout", "Zoom", "Settings"]
            .into_iter()
            .enumerate()
        {
            let ctx = egui::Context::default();
            frame(&ctx, vec![], egui::Modifiers::default());
            let p = egui::pos2(36.0 + 38.0 * index as f32, 36.0);
            let modifiers = egui::Modifiers::default();
            frame(
                &ctx,
                vec![
                    egui::Event::PointerMoved(p),
                    egui::Event::PointerButton {
                        pos: p,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers,
                    },
                ],
                modifiers,
            );
            let (commands, output) = frame(
                &ctx,
                vec![egui::Event::PointerButton {
                    pos: p,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers,
                }],
                modifiers,
            );
            assert_eq!(
                output.viewport_output.len(),
                1,
                "{action} created a child window"
            );
            match action {
                "Main" | "About" => {
                    assert!(commands.is_empty());
                    assert_eq!(
                        ctx.data_mut(
                            |d| d.get_persisted::<String>(Id::new("evaluator_original_page"))
                        )
                        .as_deref(),
                        Some(action)
                    );
                }
                "Zoom" => {
                    assert!(commands.is_empty());
                    assert!(ctx
                        .data(|d| d.get_temp::<egui::Pos2>(Id::new("evaluator_zoom_open")))
                        .is_some());
                }
                "Popout" => assert_eq!(commands, vec![command("Popout", json!({"Restore":true}))]),
                _ => assert_eq!(commands, vec![command(action, json!({}))]),
            }
        }
    }
    #[test]
    fn zoom_source_deactivate_closes_when_the_application_loses_focus() {
        let ctx = egui::Context::default();
        let zoom_id = Id::new("evaluator_zoom_open");
        ctx.data_mut(|data| data.insert_temp(zoom_id, egui::pos2(100.0, 100.0)));
        let _ = ctx.run(
            egui::RawInput {
                focused: false,
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(1200.0, 800.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    show(ui, &json!({}), &[], &json!({}));
                });
            },
        );
        assert!(ctx
            .data(|data| data.get_temp::<egui::Pos2>(zoom_id))
            .is_none());
    }
    #[test]
    fn source_focus_gates_default_to_always_and_individual_focus_settings() {
        assert!(popout_visible(&json!({}), 999, 0, 0));
        let mut settings = json!({"PopoutAlwaysShow":false,"PopoutShowNoFocus":false,"PopoutShowLeftPanel":false,"PopoutShowRightPanel":false,"PopoutShowDSSMode":false,"PopoutShowRolePanel":false,"PopoutShowSysMap":false,"PopoutShowFSSMode":false});
        for (focus, key) in [
            (0, "PopoutShowNoFocus"),
            (2, "PopoutShowLeftPanel"),
            (1, "PopoutShowRightPanel"),
            (10, "PopoutShowDSSMode"),
            (4, "PopoutShowRolePanel"),
            (7, "PopoutShowSysMap"),
            (9, "PopoutShowFSSMode"),
        ] {
            assert!(!popout_visible(&settings, focus, u64::MAX, u64::MAX));
            settings[key] = json!(true);
            assert!(popout_visible(&settings, focus, u64::MAX, u64::MAX));
            settings[key] = json!(false);
        }
    }
    #[test]
    fn original_orange_theme_geometry_and_palette() {
        let t = Theme::new(&json!({}));
        assert_eq!(t.color("DefaultText"), Color32::from_rgb(255, 111, 0));
        assert_eq!(t.color("Row1Background"), Color32::from_rgb(35, 16, 2));
        assert_eq!(t.row_height(), 24.0);
        assert_eq!(t.spacing(), 2.0);
        assert!(symbols::symbol(true, "NextBody").is_some());
        assert!(symbols::symbol(true, "Mapped").is_some());
    }
    #[test]
    fn original_default_theme_and_column_spans() {
        let t = Theme::new(&json!({"ThemeSettings":{"Theme":0}}));
        assert_eq!(t.color("HeaderBackground"), Color32::from_rgb(8, 75, 8));
        assert_eq!(t.spacing(), 0.0);
        let widths = [
            25.0, 60.0, 75.0, 80.0, 60.0, 215.0, 30.0, 35.0, 100.0, 100.0,
        ];
        let r = Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(780.0, 24.0));
        assert_eq!(cell(r, &widths, 0, 4).width(), 240.0);
        assert_eq!(cell(r, &widths, 9, 1).right(), 780.0);
    }
    #[test]
    fn source_star_labels_and_gravity_format_are_preserved() {
        let oracle: Value =
            serde_json::from_str(include_str!("../../fixtures/evaluator-star-oracle.json"))
                .unwrap();
        for (kind, label) in oracle["cases"].as_object().unwrap() {
            assert_eq!(
                star_type(kind),
                label.as_str().unwrap().replace("&nbsp;", "\u{a0}")
            );
        }
        assert_eq!(gravity_label(1.0), "1 G");
        assert_eq!(gravity_label(0.123), "0.12 G");
        assert_eq!(gravity_label(10.999), "11 G");
    }
    #[test]
    fn credits_are_grouped_without_losing_precision() {
        assert_eq!(formatted(123456789), "123 456 789");
        assert_eq!(formatted(-1000), "-1 000");
    }
}
