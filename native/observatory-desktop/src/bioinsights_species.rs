//! Native layout of the original four-column Codex species table.
use super::*;

fn width(ui: &egui::Ui, text: &str, font: &FontId) -> f32 {
    if let Some(advance) = css_text::measure_advance(ui, text, font) {
        return advance;
    }
    css_text::measure(ui, text, font)
        .map(|v| v.x)
        .unwrap_or_else(|| {
            ui.fonts(|f| {
                f.layout_no_wrap(text.to_owned(), font.clone(), Color32::WHITE)
                    .size()
                    .x
            })
        })
}
fn extents(ui: &egui::Ui, font: &FontId) -> (f32, f32, f32) {
    css_text::font_extents(ui, font).unwrap_or((font.size * 0.8, font.size * 0.2, 0.0))
}
fn text_at(ui: &egui::Ui, text: &str, font: &FontId, at: egui::Pos2, color: Color32) {
    if !css_text::paint(ui, text, font, at, Align2::LEFT_TOP, color, ui.clip_rect()) {
        ui.painter()
            .text(at, Align2::LEFT_TOP, text, font.clone(), color);
    }
}
fn wrap(ui: &egui::Ui, text: &str, font: &FontId, available: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_inclusive(' ') {
            let next = format!("{line}{word}");
            if !line.is_empty() && width(ui, next.trim_end_matches(' '), font) > available {
                lines.push(std::mem::take(&mut line));
            }
            line.push_str(word);
        }
        lines.push(line);
    }
    lines
}
fn requirements(species: &Value) -> Vec<String> {
    static SOURCE: std::sync::OnceLock<BTreeMap<String, Vec<String>>> = std::sync::OnceLock::new();
    let source = SOURCE.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../observatory-plugins/src/bioinsights/companion-page-details.json"
        ))
        .expect("source species details")
    });
    let key = format!(
        "{}/{}",
        number(species, "GenusNumber"),
        number(species, "SpeciesNumber")
    );
    if let Some(lines) = source.get(&key) {
        return lines.clone();
    }
    let mut result = array(species, "Requirements")
        .iter()
        .filter_map(Value::as_str)
        .flat_map(|s| s.split('\n'))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if !array(species, "RegionNames").is_empty() {
        result.push(format!(
            "  ● Regions: {}",
            array(species, "RegionNames")
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let cause = match string(species, "VariantType") {
        "ParentStar" => Some("Radiant Star"),
        "SurfaceMaterial" => Some("Surface Materials"),
        _ => None,
    };
    if let Some(cause) = cause {
        result.push(String::new());
        result.push(format!("  ● Variant Colours: Determined by {cause}"));
    }
    result.extend([String::new(), String::new()]);
    result
}
fn variant_columns(ui: &egui::Ui, species: &Value, theme: &Theme, scale: f32) -> [f32; 5] {
    let header = source_font(ui, 15.0 * scale, "bold");
    let body = source_font(ui, 15.0 * scale, "body");
    let table = source_font(ui, 15.0 * scale, "table");
    let border = theme.data["Settings"]["BorderWidth"]
        .as_f64()
        .unwrap_or(0.0) as f32
        * scale;
    let cause = match string(species, "VariantType") {
        "ParentStar" => "Radiant Star",
        "SurfaceMaterial" => "Surface Material",
        _ => "\u{a0}",
    };
    let mut widths = ["Colour", cause, "Mark", "Seen?", "BioForge"]
        .map(|s| width(ui, s, &header) + 10.0 * scale + 2.0 * border);
    for variant in array(species, "Variants") {
        let count = number(variant, "SeenRegionCount");
        let seen = match count {
            0 => "No".to_owned(),
            1 => "1 region".to_owned(),
            n => format!("{n} regions"),
        };
        widths[0] = widths[0].max(width(ui, string(variant, "Colour"), &body) + 10.0 * scale);
        widths[1] = widths[1].max(
            width(
                ui,
                if string(variant, "Cause").is_empty() {
                    "\u{a0}"
                } else {
                    string(variant, "Cause")
                },
                &table,
            ) + 10.0 * scale,
        );
        widths[2] = widths[2].max(
            symbol_width(
                ui,
                theme,
                if flag(variant, "Marked") {
                    "OptionOn"
                } else {
                    "OptionOff"
                },
                scale,
            ) + 10.0 * scale,
        );
        widths[3] = widths[3].max(width(ui, &seen, &body) + 10.0 * scale);
        widths[4] = widths[4].max(width(ui, "Launch", &body) + 10.0 * scale);
    }
    widths
}
#[derive(Debug)]
struct Geometry {
    columns: [f32; 4],
    widths: [f32; 5],
    description: Vec<String>,
    requirements: Vec<String>,
    header: f32,
    description_height: f32,
    blank: f32,
    overview: f32,
    variants: f32,
    variant_table: f32,
    variant_header: f32,
    variant_row: f32,
    width: f32,
    height: f32,
}
fn geometry(
    ui: &egui::Ui,
    species: &Value,
    theme: &Theme,
    available: f32,
    scale: f32,
    gallery_caption: bool,
) -> Geometry {
    let body = source_font(ui, 15.0 * scale, "body");
    let large = source_font(ui, 22.0 * scale, "bold");
    let heading = source_font(ui, 40.0 * scale, "body-bold");
    let desc = source_font(ui, 16.0 * scale, "body");
    let criteria = source_font(ui, 13.0 * scale, "body");
    let parent_line = css_line_height(ui, 15.0 * scale, "body");
    let large_line = css_line_height(ui, 22.0 * scale, "bold");
    let table_line = css_line_height(ui, 15.0 * scale, "table");
    let requirements = requirements(species);
    let mut first = requirements
        .iter()
        .map(|s| width(ui, s, &criteria))
        .fold(0.0f32, f32::max)
        + 10.0 * scale;
    for (name, value) in stats(species) {
        first = first.max(width(ui, &format!("{name} {value}"), &large) + 10.0 * scale);
    }
    let spacer = width(ui, "\u{a0}", &body) + 10.0 * scale;
    let widths = variant_columns(ui, species, theme, scale);
    // Auto-layout expands the first three columns proportionally to satisfy
    // their shared nested-table constraint, then all four for spanning cells.
    let shared = widths.iter().sum::<f32>() + 50.0 * scale;
    let multiplier = (shared / (first + spacer)).max(1.0);
    let mut columns = [
        first * 0.5 * multiplier,
        first * 0.5 * multiplier,
        spacer * multiplier,
        490.0 * scale,
    ];
    let minimum = columns.iter().sum::<f32>();
    let maximum = minimum
        .max(width(ui, string(species, "Description"), &desc) + 10.0 * scale)
        .max(135.0 * scale + width(ui, string(species, "FullName"), &heading) + 10.0 * scale);
    let content = (available - 40.0 * scale).min(maximum).max(minimum);
    for column in &mut columns {
        *column *= content / minimum;
    }
    let description = wrap(
        ui,
        string(species, "Description"),
        &desc,
        content - 10.0 * scale,
    );
    // display:table-cell spans form an anonymous inline table, including the
    // inherited vertical border-spacing above and below their image baseline.
    let header =
        135.0 * scale + extents(ui, &heading).1 + 2.0 * theme.spacing() * scale + 2.0 * scale;
    let description_height = parent_line
        + description.len() as f32 * css_line_height(ui, 16.0 * scale, "body")
        + 4.0 * scale;
    let blank = parent_line + 4.0 * scale;
    let left_overview =
        3.0 * (large_line + parent_line) + large_line + requirements.len() as f32 * parent_line;
    let right_overview = 270.0 * scale
        + extents(ui, &body).1
        + if gallery_caption {
            large_line + css_line_height(ui, 15.0 * scale, "body")
        } else {
            0.0
        };
    let overview = left_overview.max(right_overview) + 4.0 * scale;
    let variant_header = css_header_height(ui, theme, scale, false);
    let variant_row = parent_line.max((theme.row_height() - 1.0) * scale) + 4.0 * scale;
    let n = array(species, "Variants").len() as f32;
    let spacing = theme.spacing() * scale;
    let variant_table = 30.0 * scale + (n + 2.0) * spacing + variant_header + n * variant_row;
    let variants = (large_line + variant_table)
        .max(400.0 * scale + extents(ui, &source_font(ui, 15.0 * scale, "table")).1)
        + 4.0 * scale;
    let caption = table_line + 4.0 * scale;
    let height = 30.0 * scale
        + 7.0 * spacing
        + header
        + description_height
        + blank
        + overview
        + variants
        + caption;
    Geometry {
        columns,
        widths,
        description,
        requirements,
        header,
        description_height,
        blank,
        overview,
        variants,
        variant_table,
        variant_header,
        variant_row,
        width: content + 30.0 * scale,
        height,
    }
}
fn stats(species: &Value) -> [(&'static str, String); 3] {
    [
        ("Base Value:", grouped_credits(number(species, "BaseValue"))),
        (
            "First Logged:",
            grouped_credits(number(species, "FirstLoggedValue")),
        ),
        (
            "Colony Range:",
            format!("{}m", number(species, "MinimumSampleDistance")),
        ),
    ]
}
fn media_rects(
    ui: &egui::Ui,
    g: &Geometry,
    theme: &Theme,
    scale: f32,
    start: egui::Pos2,
    has_caption: bool,
) -> [Rect; 3] {
    let left = start.x + 25.0 * scale;
    let spacing = theme.spacing() * scale;
    let top = start.y + 15.0 * scale + spacing;
    let body = source_font(ui, 15.0 * scale, "body");
    let overview = top + g.header + spacing + g.description_height + spacing + g.blank + spacing;
    let gallery_height = 270.0 * scale
        + extents(ui, &body).1
        + if has_caption {
            css_line_height(ui, 22.0 * scale, "bold") + css_line_height(ui, 15.0 * scale, "body")
        } else {
            0.0
        };
    let variant = overview + g.overview + spacing;
    [
        Rect::from_min_size(
            egui::pos2(left + 5.0 * scale, top + (1.0 + theme.spacing()) * scale),
            Vec2::splat(135.0 * scale),
        ),
        Rect::from_min_size(
            egui::pos2(
                left + g.columns[..3].iter().sum::<f32>() + (g.columns[3] - 480.0 * scale) * 0.5,
                overview + (g.overview - gallery_height) * 0.5,
            ),
            Vec2::new(480.0, 270.0) * scale,
        ),
        Rect::from_min_size(
            egui::pos2(
                left + g.columns[..3].iter().sum::<f32>() + (g.columns[3] - 400.0 * scale) * 0.5,
                variant
                    + (g.variants
                        - (400.0 * scale + extents(ui, &source_font(ui, 15.0 * scale, "table")).1))
                        * 0.5,
            ),
            Vec2::splat(400.0 * scale),
        ),
    ]
}
fn image_at(ui: &mut egui::Ui, url: &str, rect: Rect) {
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.spacing_mut().item_spacing = Vec2::ZERO;
    images::show(&mut child, url, rect.size());
}
impl State {
    pub(super) fn species_detail(
        &mut self,
        ui: &mut egui::Ui,
        species: &Value,
        _summary: &Value,
        theme: &Theme,
        scale: f32,
        actions: &mut Vec<Value>,
    ) {
        let full_name = string(species, "FullName");
        let gallery = images::gallery(ui.ctx(), full_name);
        let has_caption = gallery.as_ref().is_some_and(|g| !g.name.is_empty());
        let g = geometry(ui, species, theme, ui.available_width(), scale, has_caption);
        let start = ui.cursor().min;
        let media = media_rects(ui, &g, theme, scale, start, has_caption);
        let back_font = source_font(ui, 16.0 * scale, "body");
        let back_size = css_text::measure(ui, "⋘", &back_font).unwrap_or(Vec2::splat(19.0 * scale));
        let back = Rect::from_min_size(start - Vec2::new(10.0 * scale, 0.0), back_size);
        let response = ui
            .interact(
                back,
                Id::new("BioInsights source species back"),
                Sense::click(),
            )
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if response.hovered() {
            ui.painter().rect_filled(back, 0.0, theme.color("LinkText"));
        }
        text_at(
            ui,
            "⋘",
            &back_font,
            back.min,
            theme.color(if response.hovered() {
                "DefaultBackground"
            } else {
                "LinkText"
            }),
        );
        if response.clicked() {
            self.page = Page::Main;
        }
        ui.allocate_exact_size(Vec2::new(g.width + 10.0 * scale, g.height), Sense::hover());
        let left = start.x + 25.0 * scale;
        let mut y = start.y + 15.0 * scale + theme.spacing() * scale;
        let color = theme.color("DefaultText");
        let heading_font = source_font(ui, 40.0 * scale, "body-bold");
        let heading_inset = (1.0 + theme.spacing()) * scale;
        image_at(ui, string(species, "BioscanURL"), media[0]);
        let title_h = css_line_height(ui, 40.0 * scale, "body-bold");
        text_at(
            ui,
            full_name,
            &heading_font,
            egui::pos2(
                left + 140.0 * scale,
                y + heading_inset + (135.0 * scale + extents(ui, &heading_font).1 - title_h) * 0.5,
            ),
            color,
        );
        y += g.header + theme.spacing() * scale;
        let desc_font = source_font(ui, 16.0 * scale, "body");
        let parent = source_font(ui, 15.0 * scale, "body");
        let parent_line = css_line_height(ui, 15.0 * scale, "body");
        for (i, line) in g.description.iter().enumerate() {
            text_at(
                ui,
                line,
                &desc_font,
                egui::pos2(
                    left + 5.0 * scale,
                    y + 2.0 * scale
                        + parent_line
                        + i as f32 * css_line_height(ui, 16.0 * scale, "body"),
                ),
                color,
            );
        }
        y += g.description_height + theme.spacing() * scale + g.blank + theme.spacing() * scale;
        let overview_top = y;
        let large = source_font(ui, 22.0 * scale, "bold");
        let large_line = css_line_height(ui, 22.0 * scale, "bold");
        for (title, value) in stats(species) {
            text_at(
                ui,
                &format!("{title} {value}"),
                &large,
                egui::pos2(left + 5.0 * scale, y + 2.0 * scale),
                color,
            );
            y += large_line + parent_line;
        }
        text_at(
            ui,
            "Requirements",
            &large,
            egui::pos2(left + 5.0 * scale, y + 2.0 * scale),
            color,
        );
        y += large_line;
        let criteria = source_font(ui, 13.0 * scale, "body");
        let baseline_offset = extents(ui, &parent).0 - extents(ui, &criteria).0;
        for (i, line) in g.requirements.iter().enumerate() {
            text_at(
                ui,
                line,
                &criteria,
                egui::pos2(
                    left + 5.0 * scale,
                    y + 2.0 * scale + baseline_offset + i as f32 * parent_line,
                ),
                color,
            );
        }
        let image_x = media[1].left();
        let image_y = media[1].top();
        image_at(
            ui,
            gallery
                .as_ref()
                .map(|g| g.url.as_str())
                .unwrap_or("source:Unknown"),
            media[1],
        );
        if let Some(gallery) = gallery.filter(|g| !g.name.is_empty()) {
            let at = image_y + 270.0 * scale + extents(ui, &parent).1;
            text_at(
                ui,
                &gallery.name,
                &large,
                egui::pos2(
                    image_x + (480.0 * scale - width(ui, &gallery.name, &large)) * 0.5,
                    at,
                ),
                color,
            );
            let italic = css_text::font("Verdana", 15.0 * scale, false, true);
            let credit = format!("CMDR {}", gallery.cmdr);
            text_at(
                ui,
                &credit,
                &italic,
                egui::pos2(
                    image_x + (480.0 * scale - width(ui, &credit, &italic)) * 0.5,
                    at + large_line,
                ),
                color,
            );
        }
        y = overview_top + g.overview + theme.spacing() * scale;
        let variant_x = left + 30.0 * scale;
        let variant_y = y + (g.variants - (large_line + g.variant_table)) * 0.5;
        text_at(
            ui,
            "Variants",
            &large,
            egui::pos2(left + 5.0 * scale, variant_y),
            color,
        );
        self.variant_table_at(
            ui,
            species,
            theme,
            scale,
            actions,
            egui::pos2(
                variant_x,
                variant_y + large_line + 15.0 * scale + theme.spacing() * scale,
            ),
            &g,
        );
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(media[2])
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        let seen = array(species, "Variants")
            .iter()
            .find(|v| v["Colour"] == self.map_variant)
            .map(|v| number(v, "SeenRegionMask"))
            .unwrap_or(0);
        images::map(
            &mut child,
            number(species, "RegionMask"),
            seen,
            color,
            400.0 * scale,
        );
        y += g.variants + theme.spacing() * scale;
        let name = if self.map_variant.is_empty() || flag(species, "Horizons") {
            full_name.to_owned()
        } else {
            format!("{full_name} - {}", self.map_variant)
        };
        let caption_x = left
            + g.columns[..2].iter().sum::<f32>()
            + (g.columns[2] + g.columns[3] - width(ui, &name, &criteria)) * 0.5;
        text_at(
            ui,
            &name,
            &criteria,
            egui::pos2(
                caption_x,
                y + 2.0 * scale + extents(ui, &source_font(ui, 15.0 * scale, "table")).0
                    - extents(ui, &criteria).0,
            ),
            color,
        );
    }
    #[cfg(test)]
    pub(super) fn variant_table(
        &mut self,
        ui: &mut egui::Ui,
        species: &Value,
        theme: &Theme,
        scale: f32,
        actions: &mut Vec<Value>,
    ) {
        let g = geometry(ui, species, theme, ui.available_width(), scale, false);
        let at = ui.cursor().min;
        ui.allocate_exact_size(
            Vec2::new(g.widths.iter().sum(), g.variant_table),
            Sense::hover(),
        );
        self.variant_table_at(ui, species, theme, scale, actions, at, &g);
    }
    #[allow(clippy::too_many_arguments)]
    fn variant_table_at(
        &mut self,
        ui: &mut egui::Ui,
        species: &Value,
        theme: &Theme,
        scale: f32,
        actions: &mut Vec<Value>,
        at: egui::Pos2,
        g: &Geometry,
    ) {
        let cause = match string(species, "VariantType") {
            "ParentStar" => "Radiant Star",
            "SurfaceMaterial" => "Surface Material",
            _ => "",
        };
        let total = g.widths.iter().sum();
        let header = Rect::from_min_size(at, Vec2::new(total, g.variant_header));
        ui.painter()
            .rect_filled(header, 0.0, theme.color("TitleBackground"));
        for (i, title) in ["Colour", cause, "Mark", "Seen?", "BioForge"]
            .iter()
            .enumerate()
        {
            let border = theme.data["Settings"]["BorderWidth"]
                .as_f64()
                .unwrap_or(0.0) as f32
                * scale;
            if border > 0.0 {
                ui.painter().rect_stroke(
                    cell(header, &g.widths, i, 1),
                    0.0,
                    egui::Stroke::new(border, theme.color("Border")),
                    egui::StrokeKind::Inside,
                );
            }
            label_bold(
                ui,
                cell(header, &g.widths, i, 1),
                title,
                theme.color("TitleText"),
                15.0 * scale,
                Align2::CENTER_CENTER,
            );
        }
        for (i, variant) in array(species, "Variants").iter().enumerate() {
            let rect = Rect::from_min_size(
                at + Vec2::new(
                    0.0,
                    g.variant_header
                        + theme.spacing() * scale
                        + i as f32 * (g.variant_row + theme.spacing() * scale),
                ),
                Vec2::new(total, g.variant_row),
            );
            ui.painter().rect_filled(
                rect,
                0.0,
                theme.color(if i.is_multiple_of(2) {
                    "Row1Background"
                } else {
                    "Row2Background"
                }),
            );
            let colour = string(variant, "Colour");
            for (col, text, font) in [
                (0, colour, source_font(ui, 15.0 * scale, "body")),
                (
                    1,
                    string(variant, "Cause"),
                    source_font(ui, 15.0 * scale, "table"),
                ),
            ] {
                text_at(
                    ui,
                    text,
                    &font,
                    cell(rect, &g.widths, col, 1).min + Vec2::new(5.0, 2.0) * scale,
                    theme.color("DefaultText"),
                );
            }
            let count = number(variant, "SeenRegionCount");
            let seen = match count {
                0 => "No".to_owned(),
                1 => "1 region".to_owned(),
                n => format!("{n} regions"),
            };
            let body = source_font(ui, 15.0 * scale, "body");
            for (col, text, kind) in [(3, seen.as_str(), "seen"), (4, "Launch", "forge")] {
                let point = cell(rect, &g.widths, col, 1).min + Vec2::new(5.0, 2.0) * scale;
                let link = Rect::from_min_size(
                    point,
                    Vec2::new(
                        width(ui, text, &body),
                        css_line_height(ui, 15.0 * scale, "body"),
                    ),
                );
                let response = ui
                    .interact(
                        link,
                        Id::new(("BioInsights CC link", &self.species, colour, kind)),
                        Sense::click(),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if response.hovered() {
                    ui.painter().rect_filled(link, 0.0, theme.color("LinkText"));
                }
                text_at(
                    ui,
                    text,
                    &body,
                    point,
                    theme.color(if response.hovered() {
                        "DefaultBackground"
                    } else {
                        "LinkText"
                    }),
                );
                if response.clicked() {
                    if col == 3 {
                        self.map_variant = colour.to_owned();
                    } else {
                        ui.ctx()
                            .open_url(egui::OpenUrl::new_tab(string(variant, "BioForgeURL")));
                    }
                }
            }
            let key = format!("{}:{colour}", self.species);
            let source_mark = flag(variant, "Marked");
            if self.marks.get(&key) == Some(&source_mark) {
                self.marks.remove(&key);
            }
            let marked = self.marks.get(&key).copied().unwrap_or(source_mark);
            let toggle = Rect::from_min_size(
                cell(rect, &g.widths, 2, 1).min + Vec2::new(5.0, 2.0) * scale,
                Vec2::splat((theme.row_height() - 1.0) * scale),
            );
            source_symbol(
                ui,
                theme,
                if marked { "OptionOn" } else { "OptionOff" },
                toggle,
            );
            if ui
                .interact(
                    toggle,
                    Id::new(("BioInsights source mark", key.as_str())),
                    Sense::click(),
                )
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                self.marks.insert(key, !marked);
                actions.push(action(
                    "MarkVariant",
                    json!({"SpeciesID":self.species,"Variant":colour,"Marked":!marked}),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_species_table_constraints_and_rows_match_actual_webkit_css() {
        let cases: Value =
            serde_json::from_str(include_str!("../../fixtures/bioinsights-species-css.json"))
                .unwrap();
        let worker = observatory_plugins::bioinsights::BioInsights::new(
            json!({"GalacticCodexRemote":false}),
        )
        .unwrap();
        let catalogue = worker.codex_companion();
        for case in cases.as_array().unwrap() {
            let parts = case["page"]
                .as_str()
                .unwrap()
                .split('/')
                .collect::<Vec<_>>();
            let genus = parts[1].parse::<u64>().unwrap();
            let species = parts[2].parse::<u64>().unwrap();
            let row = catalogue
                .iter()
                .find(|r| r["GenusNumber"] == genus && r["SpeciesNumber"] == species)
                .unwrap();
            let context = egui::Context::default();
            let mut actual = None;
            for _ in 0..200 {
                let _ = context.run(egui::RawInput::default(), |ctx| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::NONE)
                        .show(ctx, |ui| {
                            prepare_fonts(ctx);
                            if ["body", "body-bold", "table", "bold"].iter().all(|kind| {
                                matches!(
                                    source_font(ui, 15.0, kind).family,
                                    egui::FontFamily::Name(_)
                                )
                            }) {
                                let theme = Theme::new(
                                    &json!({"ThemeSettings":{"ThemeIndex":case["theme_index"]}}),
                                );
                                let caption = case["gallery_caption"] == true;
                                let g = geometry(
                                    ui,
                                    row,
                                    &theme,
                                    case["content"]["rect"]["width"].as_f64().unwrap() as f32 - 2.0,
                                    1.0,
                                    caption,
                                );
                                let start = egui::pos2(
                                    case["content"]["rect"]["x"].as_f64().unwrap() as f32 + 2.0,
                                    case["content"]["rect"]["y"].as_f64().unwrap() as f32 + 5.0,
                                );
                                let media = media_rects(ui, &g, &theme, 1.0, start, caption);
                                actual = Some((g, media));
                            }
                        });
                });
                if actual.is_some() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            let (actual, media) = actual.unwrap();
            let rows = case["tables"][0]["rows"].as_array().unwrap();
            let target = [
                (
                    "width",
                    actual.width,
                    case["tables"][0]["rect"]["width"].as_f64().unwrap() as f32,
                ),
                (
                    "height",
                    actual.height,
                    case["tables"][0]["rect"]["height"].as_f64().unwrap() as f32,
                ),
                (
                    "header",
                    actual.header,
                    rows[0]["rect"]["height"].as_f64().unwrap() as f32,
                ),
                (
                    "description",
                    actual.description_height,
                    rows[1]["rect"]["height"].as_f64().unwrap() as f32,
                ),
                (
                    "overview",
                    actual.overview,
                    rows[3]["rect"]["height"].as_f64().unwrap() as f32,
                ),
                (
                    "variants",
                    actual.variants,
                    rows[4]["rect"]["height"].as_f64().unwrap() as f32,
                ),
            ];
            println!(
                "{} {} {target:?} nativecol={:?}, sourcecol={:?}",
                case["page"],
                case["width"],
                actual.columns,
                rows[3]["cells"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|c| c["rect"]["width"].as_f64().unwrap())
                    .collect::<Vec<_>>()
            );
            for (name, native, source) in target {
                assert!(
                    (native - source).abs() < 1.0 / 16.0,
                    "{} {} {name}: native={native},source={source}",
                    case["page"],
                    case["width"]
                );
            }
            for (i, span) in [
                actual.columns[0] + actual.columns[1],
                actual.columns[2],
                actual.columns[3],
            ]
            .iter()
            .enumerate()
            {
                assert!(
                    (*span - rows[3]["cells"][i]["rect"]["width"].as_f64().unwrap() as f32).abs()
                        < 1.0 / 16.0
                );
            }
            for (i, w) in actual.widths.iter().enumerate() {
                assert!(
                    (*w - case["tables"][1]["rows"][1]["cells"][i]["rect"]["width"]
                        .as_f64()
                        .unwrap() as f32)
                        .abs()
                        < 1.0 / 16.0,
                    "variant column {i}"
                );
            }
            for (i, rect) in media.iter().enumerate() {
                let source = &case["media"][i]["rect"];
                for (name, value) in [
                    ("x", rect.left()),
                    ("y", rect.top()),
                    ("width", rect.width()),
                    ("height", rect.height()),
                ] {
                    assert!(
                        (value - source[name].as_f64().unwrap() as f32).abs() < 1.0 / 16.0,
                        "{} theme={} media{i} {name}: {value} vs {}",
                        case["page"],
                        case["theme_index"],
                        source[name]
                    );
                }
            }
        }
    }
}
