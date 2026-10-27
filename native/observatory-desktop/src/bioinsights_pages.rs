//! Pages from UIHandler, rendered inside the same custom output control.
//! Only the source Popout and Core settings routes are delegated to the host.
use super::*;
use std::collections::BTreeMap;
#[path = "bioinsights_species.rs"]
mod species;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Page {
    #[default]
    Main,
    Unsold,
    About,
    Codex,
}

#[derive(Clone, Default)]
pub(super) struct State {
    page: Page,
    genus: String,
    species: String,
    map_variant: String,
    zoom_at: Option<egui::Pos2>,
    marks: BTreeMap<String, bool>,
    bonuses: BTreeMap<u64, bool>,
    commander: Option<(Value, String)>,
}

pub(super) struct Presentation<'a> {
    pub theme: &'a Theme,
    pub settings: &'a Value,
    pub scale: f32,
}

fn state_id() -> Id {
    Id::new("BioInsights custom output page")
}

/// Legacy launch aliases and settings buttons select the existing plugin page.
/// This function deliberately has no viewport/window creation or focus command.
pub fn navigate(ctx: &egui::Context, route: &str, payload: &Value) -> Vec<Value> {
    let mut state = State::load(ctx);
    let actions = state.route(route, payload, ctx);
    state.save(ctx);
    ctx.request_repaint();
    actions
}

impl State {
    pub(super) fn load(ctx: &egui::Context) -> Self {
        ctx.data(|d| d.get_temp(state_id())).unwrap_or_default()
    }
    pub(super) fn save(self, ctx: &egui::Context) {
        ctx.data_mut(|d| d.insert_temp(state_id(), self));
    }
    pub(super) fn synchronize_commander(&mut self, view: &Value) {
        let commander = (view["FID"].clone(), string(view, "Commander").to_owned());
        if self.commander.as_ref() != Some(&commander) {
            self.marks.clear();
            self.bonuses.clear();
            self.commander = Some(commander);
        }
    }
    pub(super) fn scroll_id(&self) -> u8 {
        self.page as u8
    }
    pub(super) fn is_main(&self) -> bool {
        self.page == Page::Main
    }
    pub(super) fn route(
        &mut self,
        route: &str,
        payload: &Value,
        ctx: &egui::Context,
    ) -> Vec<Value> {
        match route {
            "Main" | "main" => self.page = Page::Main,
            "Unsold"
            | "unsold"
            | "Unsold Data"
            | "Unsold BioInsights"
            | "BioInsights - Unsold Data" => {
                self.page = Page::Unsold;
                return vec![action("RefreshUnsold", json!({}))];
            }
            "About" | "about" => self.page = Page::About,
            "CodexCompanion"
            | "codexcompanion"
            | "codex"
            | "Codex Companion"
            | "BioInsights - Codex Companion" => {
                self.page = Page::Codex;
                // /codexcompanion has Unknown genus/species in UIHandler;
                // the main species link supplies the explicit species route.
                self.genus.clear();
                self.species.clear();
                self.map_variant.clear();
                if let Some(species) = payload["SpeciesID"].as_str() {
                    self.species = species.to_owned();
                }
            }
            "Zoom" | "zoom" => {
                self.zoom_at = if self.zoom_at.is_some() {
                    None
                } else {
                    Some(ctx.pointer_latest_pos().unwrap_or_default())
                };
            }
            "Popout" => {
                return vec![action(
                    "Popout",
                    json!({"Restore":!ctx.input(|input| input.modifiers.ctrl)}),
                )]
            }
            "Settings" => return vec![action(route, payload.clone())],
            _ => {}
        }
        Vec::new()
    }

    pub(super) fn zoom_selector(
        &mut self,
        ctx: &egui::Context,
        zoom: &mut u32,
        actions: &mut Vec<Value>,
    ) {
        let Some(at) = self.zoom_at else { return };
        // FZoomSelector: cursor-anchored, 505x54 client area, 50..600%, reset 100.
        let popup = egui::Area::new(Id::new("BioInsights source zoom selector"))
            .order(egui::Order::Foreground)
            .fixed_pos(at)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_width(493.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().slider_width = 365.0;
                        let changed = ui
                            .add(
                                egui::Slider::new(zoom, 50..=600)
                                    .step_by(5.0)
                                    .show_value(false),
                            )
                            .changed();
                        ui.label(format!("{zoom}%"));
                        let reset = ui
                            .add_sized([70.0, 37.0], egui::Button::new("Reset"))
                            .clicked();
                        if reset {
                            *zoom = 100;
                        }
                        if changed || reset {
                            actions.push(action("SetZoomLevel", json!({"ZoomLevel":zoom})));
                        }
                    });
                });
            });
        if ctx.input(|i| !i.focused || i.key_pressed(egui::Key::Escape))
            || ctx.input(|i| {
                i.pointer.any_pressed()
                    && i.pointer
                        .interact_pos()
                        .is_some_and(|p| !popup.response.rect.contains(p))
            })
        {
            self.zoom_at = None;
        }
    }

    pub(super) fn show(
        &mut self,
        ui: &mut egui::Ui,
        view: &Value,
        summary: &Value,
        presentation: Presentation<'_>,
        actions: &mut Vec<Value>,
    ) {
        let Presentation {
            theme,
            settings,
            scale,
        } = presentation;
        ui.style_mut().visuals.override_text_color = Some(theme.color("DefaultText"));
        ui.style_mut().visuals.widgets.active.fg_stroke.color = theme.color("DefaultText");
        ui.style_mut().visuals.hyperlink_color = theme.color("LinkText");
        let body_font = source_font(ui, 15.0 * scale, "table");
        ui.style_mut()
            .text_styles
            .insert(egui::TextStyle::Body, body_font);
        match self.page {
            Page::Unsold => self.unsold(ui, summary, settings, theme, scale, actions),
            Page::About => about(ui, theme, scale),
            Page::Codex => self.codex(ui, view, summary, theme, scale, actions),
            Page::Main => {}
        }
    }

    fn unsold(
        &mut self,
        ui: &mut egui::Ui,
        summary: &Value,
        _settings: &Value,
        theme: &Theme,
        scale: f32,
        actions: &mut Vec<Value>,
    ) {
        let rows = array(summary, "Unsold");
        let bonus_for = |record: &Value| {
            self.bonuses
                .get(&number(record, "ID"))
                .copied()
                .unwrap_or(flag(record, "FirstFootfall"))
        };
        let total = rows
            .iter()
            .map(|record| {
                number(record, "BaseValue").saturating_mul(if bonus_for(record) { 5 } else { 1 })
            })
            .fold(0u64, u64::saturating_add);
        let text_width = |text: &str, bold: bool| {
            let font = source_font(ui, 13.0 * scale, if bold { "bold" } else { "table" });
            css_text::measure(ui, text, &font)
                .map(|s| s.x)
                .unwrap_or_else(|| measured(ui, text, 13.0 * scale))
        };
        let mut widths = [
            "Date",
            "Body",
            "Species",
            "Base Value",
            "Logged",
            "Expected Value",
        ]
        .map(|text| text_width(text, true) + 10.0 * scale);
        widths[4] = widths[4].max((theme.row_height() - 1.0) * scale + 10.0 * scale);
        widths[5] = widths[5].max(text_width(&grouped_credits(total), true) + 10.0 * scale);
        for record in rows {
            for (index, text) in [
                source_date(string(record, "DateTime")),
                string(record, "BodyName").to_owned(),
                string(record, "FullName").to_owned(),
                grouped_credits(number(record, "BaseValue")),
                String::new(),
                grouped_credits(
                    number(record, "BaseValue").saturating_mul(if bonus_for(record) {
                        5
                    } else {
                        1
                    }),
                ),
            ]
            .iter()
            .enumerate()
            {
                widths[index] = widths[index].max(text_width(text, false) + 10.0 * scale);
            }
        }
        egui::Frame::NONE
            .inner_margin(egui::Margin {
                left: (5.0 * scale).round() as i8,
                right: (5.0 * scale).round() as i8,
                top: (2.0 * scale).round() as i8,
                bottom: (2.0 * scale).round() as i8,
            })
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = theme.spacing() * scale;
                ui.add_space(theme.spacing() * scale);
                let width = widths.iter().sum();
                let total_row = row(
                    ui,
                    width,
                    css_line_height(ui, 13.0 * scale, "bold") + 4.0 * scale,
                    theme.color("DefaultBackground"),
                );
                label_bold(
                    ui,
                    cell(total_row, &widths, 0, 5),
                    "Total",
                    theme.color("DefaultText"),
                    13.0 * scale,
                    Align2::RIGHT_CENTER,
                );
                label_bold(
                    ui,
                    cell(total_row, &widths, 5, 1),
                    &grouped_credits(total),
                    theme.color("DefaultText"),
                    13.0 * scale,
                    Align2::RIGHT_CENTER,
                );
                row(
                    ui,
                    width,
                    css_line_height(ui, 15.0 * scale, "table") + 4.0 * scale,
                    theme.color("DefaultBackground"),
                );
                let header = row(
                    ui,
                    width,
                    css_line_height(ui, 13.0 * scale, "bold") * 2.0
                        + 2.0 * scale
                        + 2.0 * theme.border_width() * scale,
                    theme.color("TitleBackground"),
                );
                for (index, heading) in [
                    "Date",
                    "Body",
                    "Species",
                    "Base Value",
                    "First\nLogged",
                    "Expected Value",
                ]
                .iter()
                .enumerate()
                {
                    label_bold(
                        ui,
                        cell(header, &widths, index, 1),
                        heading,
                        theme.color("TitleText"),
                        13.0 * scale,
                        Align2::CENTER_CENTER,
                    );
                }
                let mut previous = "";
                let mut stripe = 1usize;
                for record in rows {
                    let body = string(record, "BodyName");
                    if !previous.is_empty() && previous != body {
                        stripe += 1;
                    }
                    previous = body;
                    let id = number(record, "ID");
                    let source = flag(record, "FirstFootfall");
                    if self.bonuses.get(&id) == Some(&source) {
                        self.bonuses.remove(&id);
                    }
                    let bonus = self.bonuses.get(&id).copied().unwrap_or(source);
                    let base = number(record, "BaseValue");
                    let height = css_line_height(ui, 13.0 * scale, "table")
                        .max((theme.row_height() - 1.0) * scale)
                        + 4.0 * scale;
                    let rect = row(
                        ui,
                        width,
                        height,
                        theme.color(if stripe.is_multiple_of(2) {
                            "Row1Background"
                        } else {
                            "Row2Background"
                        }),
                    );
                    for (index, text) in [
                        source_date(string(record, "DateTime")),
                        body.to_owned(),
                        string(record, "FullName").to_owned(),
                        grouped_credits(base),
                        String::new(),
                        grouped_credits(base.saturating_mul(if bonus { 5 } else { 1 })),
                    ]
                    .iter()
                    .enumerate()
                    {
                        if index != 4 {
                            label(
                                ui,
                                cell(rect, &widths, index, 1),
                                text,
                                theme.color("DefaultText"),
                                13.0 * scale,
                                if index >= 3 {
                                    Align2::RIGHT_CENTER
                                } else {
                                    Align2::LEFT_CENTER
                                },
                            );
                        }
                    }
                    let toggle = Rect::from_min_size(
                        cell(rect, &widths, 4, 1).min + Vec2::new(5.0, 2.0) * scale,
                        Vec2::splat((theme.row_height() - 1.0) * scale),
                    );
                    source_symbol(
                        ui,
                        theme,
                        if bonus { "OptionOn" } else { "OptionOff" },
                        toggle,
                    );
                    if record["ID"].is_u64()
                        && ui
                            .interact(
                                toggle,
                                Id::new(("BioInsights unsold bonus", id)),
                                Sense::click(),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                    {
                        self.bonuses.insert(id, !bonus);
                        actions.push(action("SetBonus", json!({"ID":id,"Value":!bonus})));
                    }
                }
                ui.add_space(theme.spacing() * scale);
            });
    }

    fn codex(
        &mut self,
        ui: &mut egui::Ui,
        view: &Value,
        summary: &Value,
        theme: &Theme,
        scale: f32,
        actions: &mut Vec<Value>,
    ) {
        let catalogue = array(summary, "CodexCompanion");
        if let Some(species) = catalogue
            .iter()
            .find(|species| species["SpeciesID"] == self.species)
        {
            self.genus = string(species, "Genus").to_owned();
            self.species_detail(ui, species, summary, theme, scale, actions);
            return;
        }
        // Original buildBasePage/buildGenusPage: h3 followed by separate floated
        // cc tables. Species pages return directly, without these navigation lists.
        let h3_size = 18.72 * scale;
        ui.add_space(h3_size);
        let heading_font = source_font(ui, h3_size, "body-bold");
        let heading_height = css_text::line_height(ui, &heading_font).unwrap_or(21.0 * scale);
        let (heading, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), heading_height),
            Sense::hover(),
        );
        label_with(
            ui,
            heading.translate(Vec2::new(-CELL_PADDING * scale, 0.0)),
            &format!("Codex Companion - CMDR {}", string(view, "Commander")),
            theme.color("DefaultText"),
            heading_font,
            Align2::LEFT_CENTER,
        );
        ui.add_space(h3_size);
        let name = |row: &Value| {
            match number(row, "GenusNumber") {
                20 => "Anemone",
                21 => "Sinuous Tubers",
                _ => string(row, "Genus"),
            }
            .to_owned()
        };
        let nav_width = |items: &[&Value], field: &str, genus: bool| {
            let body = source_font(ui, 15.0 * scale, "body");
            let header = source_font(ui, 15.0 * scale, "bold");
            let mut width = 130.0 * scale;
            for heading in if genus {
                vec!["Odyssey Genuses", "Horizons Genuses"]
            } else {
                vec!["Species"]
            } {
                width = width.max(
                    css_text::measure(ui, heading, &header)
                        .map(|s| s.x)
                        .unwrap_or_else(|| measured(ui, heading, 15.0 * scale))
                        + 10.0 * scale,
                );
            }
            for row in items {
                let text = format!(
                    "\u{a0}{}\u{a0}",
                    if genus {
                        name(row)
                    } else {
                        string(row, field).to_owned()
                    }
                );
                width = width.max(
                    css_text::measure(ui, &text, &body)
                        .map(|s| s.x)
                        .unwrap_or_else(|| measured(ui, &text, 15.0 * scale))
                        + 10.0 * scale,
                );
            }
            width
        };
        let mut genuses: Vec<_> = catalogue.iter().collect();
        genuses.sort_by_key(|row| number(row, "GenusNumber"));
        genuses.dedup_by_key(|row| number(row, "GenusNumber"));
        let genus_width = nav_width(&genuses, "Genus", true);
        let mut species_rows: Vec<_> = catalogue
            .iter()
            .filter(|row| row["Genus"] == self.genus)
            .collect();
        species_rows.sort_by_key(|row| string(row, "Species"));
        let species_width = nav_width(&species_rows, "Species", false);
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.horizontal_wrapped(|ui| {
                egui::Frame::NONE
                    .inner_margin(egui::Margin {
                        left: (25.0 * scale).round() as i8,
                        right: (15.0 * scale).round() as i8,
                        top: (15.0 * scale).round() as i8,
                        bottom: (15.0 * scale).round() as i8,
                    })
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                            ui.spacing_mut().item_spacing.y = theme.spacing() * scale;
                            ui.add_space(theme.spacing() * scale);
                            for horizons in [false, true] {
                                let rect = row(
                                    ui,
                                    genus_width,
                                    css_line_height(ui, 15.0 * scale, "bold") + 2.0 * scale,
                                    theme.color("TitleBackground"),
                                );
                                label_bold(
                                    ui,
                                    rect,
                                    if horizons {
                                        "Horizons Genuses"
                                    } else {
                                        "Odyssey Genuses"
                                    },
                                    theme.color("TitleText"),
                                    15.0 * scale,
                                    Align2::CENTER_CENTER,
                                );
                                for (stripe, entry) in genuses
                                    .iter()
                                    .filter(|entry| flag(entry, "Horizons") == horizons)
                                    .enumerate()
                                {
                                    let raw = string(entry, "Genus");
                                    if nav_row(
                                        ui,
                                        theme,
                                        &name(entry),
                                        self.genus == raw,
                                        genus_width,
                                        scale,
                                        stripe,
                                    ) {
                                        self.genus = raw.to_owned();
                                        self.species.clear();
                                        self.map_variant.clear();
                                    }
                                }
                            }
                            ui.add_space(theme.spacing() * scale);
                        });
                    });
                if !self.genus.is_empty() {
                    egui::Frame::NONE
                        .inner_margin(egui::Margin {
                            left: (25.0 * scale).round() as i8,
                            right: (15.0 * scale).round() as i8,
                            top: (15.0 * scale).round() as i8,
                            bottom: (15.0 * scale).round() as i8,
                        })
                        .show(ui, |ui| {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                ui.spacing_mut().item_spacing.y = theme.spacing() * scale;
                                ui.add_space(theme.spacing() * scale);
                                let rect = row(
                                    ui,
                                    species_width,
                                    css_line_height(ui, 15.0 * scale, "bold") + 2.0 * scale,
                                    theme.color("TitleBackground"),
                                );
                                label_bold(
                                    ui,
                                    rect,
                                    "Species",
                                    theme.color("TitleText"),
                                    15.0 * scale,
                                    Align2::CENTER_CENTER,
                                );
                                for (stripe, species) in species_rows.iter().enumerate() {
                                    let id = string(species, "SpeciesID");
                                    if nav_row(
                                        ui,
                                        theme,
                                        string(species, "Species"),
                                        false,
                                        species_width,
                                        scale,
                                        stripe,
                                    ) {
                                        self.species = id.to_owned();
                                        self.map_variant.clear();
                                    }
                                }
                                ui.add_space(theme.spacing() * scale);
                            });
                        });
                }
            });
        });
    }
}

fn source_date(value: &str) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    if value.len() >= 16 {
        let month = value
            .get(5..7)
            .and_then(|v| v.parse::<usize>().ok())
            .and_then(|m| MONTHS.get(m.wrapping_sub(1)));
        if let (Some(month), Some(year), Some(day), Some(time)) =
            (month, value.get(..4), value.get(8..10), value.get(11..16))
        {
            return format!("{day}-{month}-{year} {time}");
        }
    }
    value.to_owned()
}

fn nav_row(
    ui: &mut egui::Ui,
    theme: &Theme,
    text: &str,
    selected: bool,
    width: f32,
    scale: f32,
    stripe: usize,
) -> bool {
    let height = css_line_height(ui, 15.0 * scale, "body") + 4.0 * scale;
    let rect = row(
        ui,
        width,
        height,
        theme.color(if selected {
            "LinkText"
        } else if stripe.is_multiple_of(2) {
            "Row1Background"
        } else {
            "Row2Background"
        }),
    );
    let font = source_font(ui, 15.0 * scale, "body");
    let text = if selected {
        text.to_owned()
    } else {
        format!("\u{a0}{text}\u{a0}")
    };
    let size = css_text::measure(ui, &text, &font).unwrap_or_else(|| {
        ui.fonts(|fonts| {
            fonts
                .layout_no_wrap(text.clone(), font.clone(), Color32::WHITE)
                .size()
        })
    });
    let link = Rect::from_center_size(rect.center(), size);
    let response = ui.interact(
        link,
        Id::new(("Bio source Codex navigation", &text, rect.min.y.to_bits())),
        if selected {
            Sense::hover()
        } else {
            Sense::click()
        },
    );
    let hover = !selected && response.hovered();
    if hover {
        ui.painter().rect_filled(link, 0.0, theme.color("LinkText"));
    }
    label_with(
        ui,
        rect,
        &text,
        theme.color(if selected || hover {
            "DefaultBackground"
        } else {
            "LinkText"
        }),
        font,
        Align2::CENTER_CENTER,
    );
    if !selected {
        response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
    } else {
        false
    }
}

fn css_block(ui: &mut egui::Ui, text: &str, size: f32, kind: &str, color: Color32) -> Rect {
    let font = source_font(ui, size, kind);
    let dimensions = css_text::measure(ui, text, &font).unwrap_or_else(|| {
        ui.fonts(|fonts| {
            fonts
                .layout_no_wrap(text.to_owned(), font.clone(), color)
                .size()
        })
    });
    let (rect, _) = ui.allocate_exact_size(dimensions, Sense::hover());
    if !css_text::paint(
        ui,
        text,
        &font,
        rect.left_top(),
        Align2::LEFT_TOP,
        color,
        ui.clip_rect(),
    ) {
        ui.painter()
            .text(rect.left_top(), Align2::LEFT_TOP, text, font, color);
    }
    rect
}
fn css_link(ui: &mut egui::Ui, text: &str, url: &str, rect: Rect, font: FontId, color: Color32) {
    let response = ui
        .interact(rect, Id::new(("Bio source link", url)), Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    if !css_text::paint(
        ui,
        text,
        &font,
        rect.left_center(),
        Align2::LEFT_CENTER,
        color,
        ui.clip_rect(),
    ) {
        ui.painter()
            .text(rect.left_center(), Align2::LEFT_CENTER, text, font, color);
    }
    if response.clicked() {
        ui.ctx().open_url(egui::OpenUrl::new_tab(url));
    }
}
fn css_inline(
    ui: &mut egui::Ui,
    segments: &[(&str, bool, bool, Option<&str>)],
    theme: &Theme,
    scale: f32,
) {
    let metrics: Vec<_> = segments
        .iter()
        .map(|(text, bold, italic, _)| {
            let font = css_text::font("Verdana", 14.0 * scale, *bold, *italic);
            let size = css_text::measure(ui, text, &font).unwrap_or_else(|| {
                ui.fonts(|fonts| {
                    fonts
                        .layout_no_wrap((*text).into(), font.clone(), Color32::WHITE)
                        .size()
                })
            });
            (font, size)
        })
        .collect();
    let (line, _) = ui.allocate_exact_size(
        Vec2::new(metrics.iter().map(|(_, size)| size.x).sum(), 19.0 * scale),
        Sense::hover(),
    );
    let mut x = line.left();
    for ((text, _, _, url), (font, size)) in segments.iter().zip(metrics) {
        let rect = Rect::from_min_size(egui::pos2(x, line.center().y - size.y / 2.0), size);
        let mut color = theme.color("DefaultText");
        if let Some(url) = url {
            let response = ui
                .interact(rect, Id::new(("Bio source credit", url)), Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if response.hovered() {
                ui.painter().rect_filled(rect, 0.0, theme.color("LinkText"));
                color = theme.color("DefaultBackground");
            } else {
                color = theme.color("LinkText");
            }
            if response.clicked() {
                ui.ctx().open_url(egui::OpenUrl::new_tab(*url));
            }
        }
        if !css_text::paint(
            ui,
            text,
            &font,
            rect.left_top(),
            Align2::LEFT_TOP,
            color,
            ui.clip_rect(),
        ) {
            ui.painter()
                .text(rect.left_top(), Align2::LEFT_TOP, text, font, color);
        }
        x += size.x;
    }
}
fn about(ui: &mut egui::Ui, theme: &Theme, scale: f32) {
    let normal = source_font(ui, 14.0 * scale, "body");
    let bold = source_font(ui, 14.0 * scale, "body-bold");
    let prefix = "Enjoying BioInsights? ";
    let width = css_text::measure(ui, prefix, &normal)
        .map(|size| size.x)
        .unwrap_or(153.0 * scale);
    let link = "Buy me a coffee";
    let link_size = css_text::measure(ui, link, &bold).unwrap_or(Vec2::new(126.0, 17.0) * scale);
    let (line, _) = ui.allocate_exact_size(
        Vec2::new(width + link_size.x + 8.0 * scale, 19.0 * scale),
        Sense::hover(),
    );
    label_with(
        ui,
        line.translate(Vec2::new(-CELL_PADDING * scale, 0.0)),
        prefix,
        theme.color("DefaultText"),
        normal,
        Align2::LEFT_CENTER,
    );
    let badge = Rect::from_center_size(
        egui::pos2(
            line.left() + width + (link_size.x + 8.0 * scale) / 2.0,
            line.center().y,
        ),
        link_size + Vec2::splat(8.0 * scale),
    );
    let hover = ui.rect_contains_pointer(badge);
    ui.painter().rect_filled(
        badge,
        5.0 * scale,
        theme.color(if hover {
            "ButtonFaceHighlight"
        } else {
            "ButtonFace"
        }),
    );
    css_link(
        ui,
        link,
        "https://ko-fi.com/explorationtools",
        badge.shrink(4.0 * scale),
        bold,
        theme.color("Button"),
    );
    ui.add_space(19.0 * scale);
    // Exact UIHandler.SymbolList order; the three signal badges follow it.
    const SYMBOLS: [&str; 29] = [
        "PersonalCodexEntry",
        "Mark",
        "PossibleCodexEntry",
        "PossibleCodexEntryNeverSeen",
        "HighValue",
        "StarType",
        "Material",
        "Horizons",
        "Tick",
        "Cross",
        "Likely",
        "Maybe",
        "Unlikely",
        "NotPredicted",
        "BiologicalLogged",
        "BiologicalSampled",
        "BiologicalComplete",
        "Aborted",
        "ColourIsMissing",
        "Canonn",
        "HMC",
        "RB",
        "IB",
        "RIB",
        "MR",
        "CurrentBody",
        "BioCount",
        "BioCountAutoscan",
        "BioCountMismatch",
    ];
    egui::Frame::NONE
        .inner_margin(egui::Margin {
            left: (5.0 * scale).round() as i8,
            right: (5.0 * scale).round() as i8,
            top: (2.0 * scale).round() as i8,
            bottom: (2.0 * scale).round() as i8,
        })
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = theme.spacing() * scale;
            ui.add_space(theme.spacing() * scale);
            let text_width = |text: &str, font: FontId| {
                css_text::measure(ui, text, &font)
                    .map(|s| s.x)
                    .unwrap_or_else(|| measured(ui, text, font.size))
            };
            let mut first =
                text_width("Symbol", source_font(ui, 15.0 * scale, "bold")) + 10.0 * scale;
            for key in SYMBOLS {
                first = first.max(symbol_width(ui, theme, key, scale) + 10.0 * scale);
            }
            let second = SYMBOLS
                .iter()
                .map(|key| {
                    text_width(
                        string(&theme.data["Symbols"][key], "Description"),
                        source_font(ui, 15.0 * scale, "table"),
                    ) + 10.0 * scale
                })
                .fold(
                    text_width("Meaning", source_font(ui, 15.0 * scale, "bold")) + 10.0 * scale,
                    f32::max,
                );
            let widths = [first, second];
            let width = first + second;
            let header = css_header_height(ui, theme, scale, false);
            let title = row(ui, width, header, theme.color("TitleBackground"));
            label_bold(
                ui,
                title,
                "Symbol Legend",
                theme.color("TitleText"),
                15.0 * scale,
                Align2::CENTER_CENTER,
            );
            let headings = row(ui, width, header, theme.color("TitleBackground"));
            for (index, text) in ["Symbol", "Meaning"].iter().enumerate() {
                label_bold(
                    ui,
                    cell(headings, &widths, index, 1),
                    text,
                    theme.color("TitleText"),
                    15.0 * scale,
                    Align2::CENTER_CENTER,
                );
            }
            for key in SYMBOLS {
                let height =
                    css_line_height(ui, 15.0 * scale, "table").max(if image_symbol(theme, key) {
                        (theme.row_height() - 1.0) * scale
                    } else {
                        0.0
                    }) + 4.0 * scale;
                let rect = row(ui, width, height, theme.color("DefaultBackground"));
                let symbol_cell = cell(rect, &widths, 0, 1);
                let symbol = Rect::from_center_size(
                    symbol_cell.center(),
                    Vec2::new(
                        symbol_width(ui, theme, key, scale),
                        (theme.row_height() - 1.0) * scale,
                    ),
                );
                if matches!(key, "BioCount" | "BioCountMismatch") {
                    let text = if key == "BioCount" {
                        "5 bios"
                    } else {
                        "2 bios"
                    };
                    let font = source_font(ui, 14.0 * scale, "label");
                    let size = css_text::measure(ui, text, &font)
                        .unwrap_or(Vec2::new(43.0, 17.0) * scale)
                        + Vec2::new(20.0, 2.0) * scale;
                    let badge = Rect::from_center_size(symbol_cell.center(), size);
                    ui.painter()
                        .rect_filled(badge, 7.0 * scale, theme.symbol_color(key, true));
                    label_with(
                        ui,
                        badge,
                        text,
                        theme.symbol_color(key, false),
                        font,
                        Align2::CENTER_CENTER,
                    );
                } else {
                    source_symbol(ui, theme, key, symbol);
                }
                label(
                    ui,
                    cell(rect, &widths, 1, 1),
                    string(&theme.data["Symbols"][key], "Description"),
                    theme.color("DefaultText"),
                    15.0 * scale,
                    Align2::LEFT_CENTER,
                );
            }
        });
    ui.add_space(19.0 * scale);
    ui.add_space(19.0 * scale);
    css_block(
        ui,
        "Thanks",
        24.0 * scale,
        "body",
        theme.color("DefaultText"),
    );
    for line in [
        vec![
            ("CMDR Coddiwompler", true, false, None),
            (" and members of ", false, false, None),
            ("IGAU", true, false, None),
            (
                " for their assistance creating BioInsights",
                false,
                false,
                None,
            ),
        ],
        vec![
            ("CMDR Vithigar", true, false, None),
            (" for creating ", false, false, None),
            (
                "Observatory",
                false,
                false,
                Some("https://observatory.xjph.net/"),
            ),
            (
                " and listening to my feedback and requests",
                false,
                false,
                None,
            ),
        ],
        vec![
            ("CMDR Marxanthius", true, false, None),
            (
                " whose spreadsheet powers the possible New Galactic Codex entry notifications",
                false,
                false,
                None,
            ),
        ],
        vec![
            ("CMDR Arras DeCairo", true, false, None),
            (
                " for the Bio Scan images in the Codex Companion",
                false,
                false,
                None,
            ),
        ],
        vec![
            ("DSN", true, false, None),
            (", ", false, false, None),
            ("Cannon", true, false, None),
            (" and ", false, false, None),
            ("spansh", true, false, None),
            (" whose data helps improve predictions", false, false, None),
        ],
        vec![
            ("Everyone", true, false, None),
            (" who submits bug reports or ", false, false, None),
            ("unexpected bio", false, true, None),
            (
                " reports that all help improve BioInsights",
                false,
                false,
                None,
            ),
        ],
    ] {
        css_inline(ui, &line, theme, scale);
    }
    ui.add_space(19.0 * scale);
    css_block(
        ui,
        "If I've forgotten to add you, please let me know!",
        14.0 * scale,
        "body",
        theme.color("DefaultText"),
    );
    ui.add_space(19.0 * scale);
    css_block(
        ui,
        "Need assistance?",
        24.0 * scale,
        "body",
        theme.color("DefaultText"),
    );
    css_inline(
        ui,
        &[
            ("Try the ", false, false, None),
            (
                "Observatory Discord",
                false,
                false,
                Some("https://discord.gg/4J55fNdU7B"),
            ),
        ],
        theme,
        scale,
    );
}

#[path = "bioinsights_images.rs"]
mod images;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_codex_float_tables_keep_their_rows_vertical() {
        let ctx = egui::Context::default();
        let theme = Theme::new(&json!({}));
        let summary = json!({"CodexCompanion":[
            {"Genus":"Aleoida","GenusNumber":1,"Species":"Arcus","SpeciesID":"a"},
            {"Genus":"Bacterium","GenusNumber":2,"Species":"Alcyoneum","SpeciesID":"b"},
            {"Genus":"Bacterium","GenusNumber":2,"Species":"Acytis","SpeciesID":"c"},
            {"Genus":"Anemone","GenusNumber":20,"Horizons":true,"Species":"Luteolum","SpeciesID":"d"}
        ]});
        let mut state = State {
            genus: "Bacterium".into(),
            ..State::default()
        };
        let result = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(1200.0, 800.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    state.codex(
                        ui,
                        &json!({"Commander":"Test"}),
                        &summary,
                        &theme,
                        1.0,
                        &mut Vec::new(),
                    );
                });
            },
        );
        let rows: Vec<_> = result
            .shapes
            .iter()
            .filter_map(|shape| {
                if let egui::epaint::Shape::Rect(shape) = &shape.shape {
                    let rect = shape.rect;
                    (rect.width() >= 130.0
                        && rect.width() < 300.0
                        && (20.0..=23.0).contains(&rect.height()))
                    .then_some(rect)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            rows.len(),
            8,
            "two headers and three genus rows; one header and two species rows: {rows:?}"
        );
        assert!(
            rows[..5].iter().all(|rect| rect.left() == rows[0].left()),
            "genus rows must form one vertical table: {rows:?}"
        );
        assert!(
            rows[5..].iter().all(|rect| rect.left() == rows[5].left()),
            "species rows must form one vertical table: {rows:?}"
        );
        assert_eq!(
            rows[0].top(),
            rows[5].top(),
            "floated tables share the top edge"
        );
        assert!(rows[5].left() > rows[0].right());
        assert!(rows[..5]
            .windows(2)
            .all(|pair| pair[1].top() >= pair[0].bottom()));
    }
    #[test]
    fn optimistic_switches_do_not_cross_commander_namespaces() {
        let mut state = State {
            page: Page::Codex,
            species: "Codex_Ent_Bacterial_01_Name".into(),
            ..State::default()
        };
        state.synchronize_commander(&json!({"FID":"F123","Commander":"First"}));
        state.marks.insert("species:green".into(), true);
        state.bonuses.insert(7, true);
        state.synchronize_commander(&json!({"FID":"F123","Commander":"First"}));
        assert_eq!(state.marks.len(), 1);
        assert_eq!(state.bonuses.len(), 1);
        state.synchronize_commander(&json!({"FID":"F456","Commander":"Second"}));
        assert!(state.marks.is_empty() && state.bonuses.is_empty());
        assert_eq!(state.page, Page::Codex);
        assert_eq!(state.species, "Codex_Ent_Bacterial_01_Name");
    }
    #[test]
    fn source_navigation_preserves_viewport_and_returns_only_host_exceptions() {
        let ctx = egui::Context::default();
        let result = ctx.run(egui::RawInput::default(), |ctx| {
            let mut state = State::default();
            for (command, page) in [
                ("Main", Page::Main),
                ("Unsold", Page::Unsold),
                ("About", Page::About),
                ("CodexCompanion", Page::Codex),
            ] {
                let actions = state.route(command, &json!({}), ctx);
                assert_eq!(state.page, page);
                assert_eq!(
                    actions
                        .iter()
                        .map(|a| a["command"].as_str().unwrap())
                        .collect::<Vec<_>>(),
                    if command == "Unsold" {
                        vec!["RefreshUnsold"]
                    } else {
                        vec![]
                    }
                );
            }
            assert!(state.route("Zoom", &json!({}), ctx).is_empty());
            assert!(state.zoom_at.is_some());
            for command in ["Popout", "Settings"] {
                assert_eq!(state.route(command, &json!({}), ctx)[0]["command"], command);
                assert_eq!(state.page, Page::Codex);
            }
        });
        assert_eq!(result.viewport_output.len(), 1);
        assert!(result.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .is_empty());
    }
    #[test]
    fn species_navigation_and_main_follow_source_routes() {
        let ctx = egui::Context::default();
        navigate(
            &ctx,
            "CodexCompanion",
            &json!({"SpeciesID":"Bacterium_Acytis"}),
        );
        assert_eq!(State::load(&ctx).species, "Bacterium_Acytis");
        navigate(&ctx, "Main", &json!({}));
        assert!(State::load(&ctx).is_main());
        navigate(&ctx, "Codex Companion", &json!({}));
        assert!(State::load(&ctx).species.is_empty());
        assert_eq!(State::load(&ctx).page, Page::Codex);
    }
    #[test]
    fn control_popout_bypasses_saved_geometry_without_changing_page() {
        let ctx = egui::Context::default();
        let mut state = State {
            page: Page::Unsold,
            ..Default::default()
        };
        let _ = ctx.run(
            egui::RawInput {
                modifiers: egui::Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            |ctx| {
                assert_eq!(
                    state.route("Popout", &json!({}), ctx),
                    vec![action("Popout", json!({"Restore":false}))]
                );
                assert_eq!(state.page, Page::Unsold);
            },
        );
    }
    #[test]
    fn source_dates_preserve_utc_minute_and_original_month_name() {
        assert_eq!(source_date("2026-10-25T21:13:00Z"), "25-Oct-2026 21:13");
    }

    fn click_input(point: egui::Pos2) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                Vec2::new(1400.0, 900.0),
            )),
            events: vec![
                egui::Event::PointerMoved(point),
                egui::Event::PointerButton {
                    pos: point,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                },
                egui::Event::PointerButton {
                    pos: point,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::default(),
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn original_unsold_switch_emits_vault_bonus_mutation_in_same_viewport() {
        let ctx = egui::Context::default();
        let theme = Theme::new(&json!({}));
        let mut state = State::default();
        let summary = json!({"Unsold":[{"ID":7,"DateTime":"2026-10-25T21:13:00Z","BodyName":"Reference 1 a","FullName":"Bacterium Acytis","BaseValue":1000000,"FirstFootfall":false}]});
        let mut actions = Vec::new();
        let mut initial = click_input(egui::Pos2::ZERO);
        initial.events.clear();
        let _ = ctx.run(initial, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                state.unsold(ui, &summary, &json!({}), &theme, 1.0, &mut actions)
            });
        });
        let toggle = ctx
            .read_response(Id::new(("BioInsights unsold bonus", 7u64)))
            .expect("source bonus switch")
            .rect
            .center();
        let result = ctx.run(click_input(toggle), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                state.unsold(ui, &summary, &json!({}), &theme, 1.0, &mut actions)
            });
        });
        assert_eq!(
            actions,
            vec![action("SetBonus", json!({"ID":7,"Value":true}))]
        );
        assert_eq!(state.bonuses.get(&7), Some(&true));
        assert_eq!(result.viewport_output.len(), 1);
    }

    #[test]
    fn original_codex_mark_switch_persists_command_and_seen_link_selects_map() {
        let ctx = egui::Context::default();
        let theme = Theme::new(&json!({}));
        let mut state = State {
            species: "Codex_Ent_Bacterial_06_Name".into(),
            ..Default::default()
        };
        let species = json!({"FullName":"Bacterium Alcyoneum","VariantType":"ParentStar","Variants":[{"Colour":"Emerald","Cause":"F star / G star","Marked":false,"SeenRegionCount":2,"SeenRegionMask":6}]});
        let mut actions = Vec::new();
        let mut initial = click_input(egui::Pos2::ZERO);
        initial.events.clear();
        let _ = ctx.run(initial, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                state.variant_table(ui, &species, &theme, 1.0, &mut actions)
            });
        });
        let key = format!("{}:Emerald", state.species);
        let mark = ctx
            .read_response(Id::new(("BioInsights source mark", key.as_str())))
            .expect("source mark switch")
            .rect
            .center();
        let _ = ctx.run(click_input(mark), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                state.variant_table(ui, &species, &theme, 1.0, &mut actions)
            });
        });
        assert_eq!(
            actions,
            vec![action(
                "MarkVariant",
                json!({"SpeciesID":state.species,"Variant":"Emerald","Marked":true})
            )]
        );
        let seen = ctx
            .read_response(Id::new((
                "BioInsights CC link",
                &state.species,
                "Emerald",
                "seen",
            )))
            .expect("source seen link")
            .rect
            .center();
        let result = ctx.run(click_input(seen), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                state.variant_table(ui, &species, &theme, 1.0, &mut actions)
            });
        });
        assert_eq!(state.map_variant, "Emerald");
        assert_eq!(
            actions.len(),
            1,
            "Seen updates the existing map and does not send a worker command"
        );
        assert_eq!(result.viewport_output.len(), 1);
    }
}
