//! BioInsights' custom main view, following the installed HTML table and theme.
//! The generic exported grid is a separate presentation in the original plugin.
use eframe::egui::{self, Align2, Color32, FontId, Id, Rect, Sense, Vec2};
use serde_json::{json, Value};
#[path = "css_text.rs"]
pub(crate) mod css_text;

#[path = "bioinsights_pages.rs"]
mod pages;
#[path = "../assets/bioinsights/symbols.rs"]
mod symbols;
pub use pages::navigate;
#[path = "bioinsights_alert.rs"]
mod alert;
#[path = "bioinsights_popout.rs"]
pub mod popout;
pub use alert::{alert_size, render_alert};

const CELL_PADDING: f32 = 5.0;
#[derive(Clone)]
struct CssFonts {
    table: String,
    bold: String,
    italic: String,
    label: String,
    label_bold: String,
    body: String,
    body_bold: String,
}
fn prepare_fonts(ctx: &egui::Context) {
    let installed = crate::font_choices::choices(ctx);
    let selected = |name: &str, fallback: &str| {
        installed
            .iter()
            .find(|family| family.eq_ignore_ascii_case(name))
            .cloned()
            .unwrap_or_else(|| fallback.to_owned())
    };
    // Resolve the CSS family through Fontconfig even when it is absent. This
    // uses the same system aliases as WebKit instead of choosing a new face.
    let table = css_text::css_family(&selected("Segoe UI Emoji", "Segoe UI Emoji"));
    let label = selected("Calibri", "Calibri");
    let body = selected("Verdana", "Verdana");
    let fonts = CssFonts {
        table: format!("BioTable:{table}"),
        bold: format!("BioTableBold:{table}"),
        italic: format!("BioTableItalic:{table}"),
        label: format!("BioLabel:{label}"),
        label_bold: format!("BioLabelBold:{label}"),
        body: format!("BioBody:{body}"),
        body_bold: format!("BioBodyBold:{body}"),
    };
    for (family, face) in [
        (&fonts.table, table.clone()),
        (&fonts.bold, format!("{table}:style=Bold")),
        (&fonts.italic, format!("{table}:slant=italic")),
        (&fonts.label, label.clone()),
        (&fonts.label_bold, format!("{label}:style=Bold")),
        (&fonts.body, body.clone()),
        (&fonts.body_bold, format!("{body}:style=Bold")),
    ] {
        crate::font_registry::prepare_exact(ctx, family, &face);
    }
    ctx.data_mut(|data| data.insert_temp(Id::new("Bio source CSS fonts"), fonts));
}
fn source_font(ui: &egui::Ui, size: f32, kind: &str) -> FontId {
    let fonts = ui
        .ctx()
        .data(|data| data.get_temp::<CssFonts>(Id::new("Bio source CSS fonts")));
    let family = fonts.as_ref().map(|fonts| match kind {
        "bold" => &fonts.bold,
        "italic" => &fonts.italic,
        "label" => &fonts.label,
        "label-bold" => &fonts.label_bold,
        "body" => &fonts.body,
        "body-bold" => &fonts.body_bold,
        _ => &fonts.table,
    });
    family
        .map(|family| crate::font_registry::font(ui.ctx(), family, size))
        .unwrap_or_else(|| FontId::proportional(size))
}
fn css_scale(ui: &egui::Ui) -> f32 {
    ui.style()
        .text_styles
        .get(&egui::TextStyle::Body)
        .map(|font| font.size / 15.0)
        .unwrap_or(1.0)
}
fn css_line_height(ui: &egui::Ui, size: f32, kind: &str) -> f32 {
    let font = source_font(ui, size, kind);
    css_text::line_height(ui, &font)
        .unwrap_or_else(|| ui.fonts(|fonts| fonts.row_height(&font)).ceil())
}
fn css_header_height(ui: &egui::Ui, theme: &Theme, scale: f32, subheaders: bool) -> f32 {
    let border = theme.data["Settings"]["BorderWidth"]
        .as_f64()
        .unwrap_or(0.0) as f32
        * scale;
    let heading = css_line_height(ui, 15.0 * scale, "bold") + 2.0 * scale + 2.0 * border;
    if subheaders {
        heading.max(css_line_height(ui, 15.0 * scale, "table") + 4.0 * scale)
    } else {
        heading
    }
}
fn css_alert(ui: &mut egui::Ui, alert: &Value, scale: f32, actions: &mut Vec<Value>) {
    let background = match string(alert, "Class") {
        "alert-danger" => Color32::from_rgb(245, 99, 88),
        "alert-success" => Color32::from_rgb(46, 184, 133),
        "alert-warning" => Color32::from_rgb(255, 169, 43),
        _ => Color32::from_rgb(70, 167, 245),
    };
    let normal = source_font(ui, 13.0 * scale, "body");
    let bold = source_font(ui, 13.0 * scale, "body-bold");
    let mut job = egui::text::LayoutJob::default();
    let mut rest = string(alert, "Description");
    while let Some((before, strong)) = rest.split_once("<strong>") {
        job.append(
            before,
            0.0,
            egui::TextFormat::simple(normal.clone(), Color32::WHITE),
        );
        let (part, after) = strong.split_once("</strong>").unwrap_or((strong, ""));
        job.append(
            part,
            0.0,
            egui::TextFormat::simple(bold.clone(), Color32::WHITE),
        );
        rest = after;
    }
    job.append(rest, 0.0, egui::TextFormat::simple(normal, Color32::WHITE));
    job.wrap.max_width = (ui.available_width() - 55.0 * scale).max(1.0);
    let galley = ui.fonts(|fonts| fonts.layout_job(job));
    let height = galley.size().y.ceil() + 20.0 * scale;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    ui.painter().rect_filled(rect, 7.0 * scale, background);
    ui.painter()
        .galley(rect.min + Vec2::splat(10.0 * scale), galley, Color32::WHITE);
    let close = Rect::from_min_size(
        egui::pos2(rect.right() - 30.0 * scale, rect.top() + 7.0 * scale),
        Vec2::splat(20.0 * scale),
    );
    let response = ui.interact(
        close,
        ui.make_persistent_id(("Bio source alert close", alert["ID"].to_string())),
        Sense::click(),
    );
    ui.painter().text(
        close.center(),
        Align2::CENTER_CENTER,
        "×",
        source_font(ui, 22.0 * scale, "body-bold"),
        if response.hovered() {
            Color32::BLACK
        } else {
            Color32::WHITE
        },
    );
    if response.clicked() {
        actions.push(action("DismissUIAlert", json!({"ID":alert["ID"]})));
    }
    ui.add_space(css_line_height(ui, 16.0 * scale, "body"));
}
fn image_symbol(theme: &Theme, key: &str) -> bool {
    matches!(
        theme.data["Symbols"][key]["ContentType"].as_str(),
        Some("Base64SVG" | "Base64PNG" | "Image" | "ThemeImage" | "BuiltIn")
    )
}
fn species_height(ui: &egui::Ui, theme: &Theme, species: &Value, scale: f32) -> f32 {
    let prediction = prediction_symbol(string(species, "PredictionType"), string(species, "Seen"));
    let image = flag_keys(species)
        .iter()
        .any(|key| image_symbol(theme, key))
        || image_symbol(theme, prediction)
        || image_symbol(
            theme,
            if string(species, "Seen") == "Seen" {
                "Tick"
            } else {
                "Cross"
            },
        )
        || image_symbol(theme, sample_symbol(string(species, "SampleState")))
        || array(species, "Variants")
            .iter()
            .any(|v| v["ShowInUI"] != false);
    let line = css_line_height(ui, 15.0 * scale, "table");
    let content = if image {
        line.max((theme.row_height() - 1.0) * scale)
    } else {
        line
    };
    content + 4.0 * scale
}
const HEADERS: [(&str, usize, usize); 5] = [
    ("Body", 0, 4),
    ("Type", 4, 1),
    ("Signals", 5, 1),
    ("Radiant Star", 6, 3),
    ("Body Value", 9, 2),
];
const SUBHEADERS: [(&str, usize, usize); 7] = [
    ("", 0, 1),
    ("Genus", 1, 1),
    ("Species", 2, 2),
    ("Status", 4, 2),
    ("Variants", 6, 2),
    ("Value", 8, 1),
    ("Dist", 9, 2),
];
const EXTRA_HEADERS: [(&str, usize, usize); 6] = [
    ("", 0, 1),
    ("Atmosphere Type", 1, 3),
    ("Pressure", 4, 2),
    ("Volcanism", 6, 2),
    (" Gravity", 8, 1),
    ("Temp", 9, 2),
];
const SPECIES_SPANS: [usize; 9] = [1, 1, 1, 1, 1, 1, 2, 1, 2];
#[derive(Debug, PartialEq)]
struct HeaderCell {
    text: String,
    start: usize,
    span: usize,
    align: Align2,
}
struct HeaderRow {
    cells: Vec<HeaderCell>,
    fill: &'static str,
    text: &'static str,
}
fn header_align(title: &str) -> Align2 {
    match title {
        "Body Value" => Align2::RIGHT_CENTER,
        "Type" | "Signals" | "Value" | "Dist" => Align2::LEFT_CENTER,
        _ => Align2::CENTER_CENTER,
    }
}
fn grouped_credits(value: u64) -> String {
    let digits = value.to_string();
    let mut result = String::new();
    for (index, c) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            result.push(' ');
        }
        result.push(c);
    }
    result
}
fn header_rows(settings: &Value, commander: &str, total: u64) -> Vec<HeaderRow> {
    let show = settings["ShowHeaders"].as_bool().unwrap_or(true);
    let mut rows = Vec::new();
    let banner = |text: String, align| HeaderRow {
        cells: vec![HeaderCell {
            text,
            start: 0,
            span: 11,
            align,
        }],
        fill: "HeaderBackground",
        text: "HeaderText",
    };
    if show {
        rows.push(banner(format!("CMDR {commander}"), Align2::CENTER_CENTER));
    }
    if settings["ShowTotal"].as_bool().unwrap_or(false) {
        rows.push(banner(
            format!("Unsold Data: {}", grouped_credits(total)),
            Align2::RIGHT_CENTER,
        ));
        if !show {
            rows.push(HeaderRow {
                cells: vec![HeaderCell {
                    text: String::new(),
                    start: 0,
                    span: 11,
                    align: Align2::LEFT_CENTER,
                }],
                fill: "DefaultBackground",
                text: "DefaultText",
            });
        }
    }
    if show {
        let standard = |source: &[(&str, usize, usize)], extra: bool| HeaderRow {
            cells: source
                .iter()
                .filter(|(_, _, span)| *span > 0)
                .map(|(text, start, span)| HeaderCell {
                    text: (*text).into(),
                    start: *start,
                    span: *span,
                    align: if extra {
                        Align2::RIGHT_CENTER
                    } else {
                        header_align(text)
                    },
                })
                .collect(),
            fill: "TitleBackground",
            text: "TitleText",
        };
        rows.push(standard(&HEADERS, false));
        if settings["ShowBodyFields"].as_bool().unwrap_or(false) {
            rows.push(standard(&EXTRA_HEADERS, true));
        }
        rows.push(standard(&SUBHEADERS, false));
    }
    rows
}
fn prediction_symbol(prediction: &str, seen: &str) -> &'static str {
    if seen == "NotPresent" {
        return "Unknown";
    }
    match prediction {
        "ParentStar" => "StarType",
        "SurfaceMaterial" => "Material",
        "Horizons" => "Horizons",
        "NotPredicted" => "NotPredicted",
        _ => "Unknown",
    }
}
fn sample_symbol(sample: &str) -> &'static str {
    match sample {
        "Logged" => "BiologicalLogged",
        "Sampled" => "BiologicalSampled",
        "Complete" => "BiologicalComplete",
        "Incomplete" => "Aborted",
        _ => "Unknown",
    }
}
fn variant_separator(prediction: &str) -> &'static str {
    if prediction == "SurfaceMaterial" {
        " or "
    } else {
        ", "
    }
}
fn signal_text(count: u64) -> String {
    format!("{count} bio{}", if count == 1 { "" } else { "s" })
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
            .map(|index| index != 0)
            .unwrap_or(settings["Theme"] != "Original");
        use std::sync::OnceLock;
        static ORANGE: OnceLock<Value> = OnceLock::new();
        static ORIGINAL: OnceLock<Value> = OnceLock::new();
        let data = if orange {
            ORANGE.get_or_init(|| {
                serde_json::from_str(include_str!(
                    "../../observatory-plugins/src/bioinsights/theme-orange.json"
                ))
                .expect("installed BioInsights orange theme")
            })
        } else {
            ORIGINAL.get_or_init(|| {
                serde_json::from_str(include_str!(
                    "../../observatory-plugins/src/bioinsights/theme.json"
                ))
                .expect("installed BioInsights original theme")
            })
        };
        let overrides = settings["ThemeSettings"]["Colors"]
            .as_object()
            .filter(|_| {
                settings["ThemeSettings"]["ThemeMode"]
                    .as_u64()
                    .or_else(|| settings["ThemeMode"].as_u64())
                    == Some(1)
            })
            .map(|colors| {
                colors
                    .iter()
                    .map(|(key, value)| (key.clone(), rgb(value)))
                    .collect()
            })
            .unwrap_or_default();
        Self {
            data,
            overrides,
            orange,
        }
    }
    fn color(&self, key: &str) -> Color32 {
        self.overrides
            .get(key)
            .copied()
            .unwrap_or_else(|| rgb(&self.data["Colors"][key]))
    }
    fn symbol_color(&self, key: &str, background: bool) -> Color32 {
        let symbol = &self.data["Symbols"][key];
        let field = if background {
            "ContentBackColor"
        } else {
            "ContentColor"
        };
        if symbol[field].is_object() {
            return rgb(&symbol[field]);
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
    fn row_height(&self) -> f32 {
        self.data["Settings"]["RowHeight"].as_f64().unwrap_or(20.0) as f32
    }
    fn highlight_row(&self) -> bool {
        self.data["Settings"]["UseHighlightRow"]
            .as_bool()
            .unwrap_or(false)
    }
    fn border_width(&self) -> f32 {
        self.data["Settings"]["BorderWidth"].as_f64().unwrap_or(0.0) as f32
    }
    fn spacing(&self) -> f32 {
        self.data["Settings"]["RowSpacing"].as_f64().unwrap_or(2.0) as f32
    }
}
fn rgb(value: &Value) -> Color32 {
    Color32::from_rgb(
        value["R"].as_u64().unwrap_or(0) as u8,
        value["G"].as_u64().unwrap_or(0) as u8,
        value["B"].as_u64().unwrap_or(0) as u8,
    )
}
fn string<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or("")
}
fn number(value: &Value, key: &str) -> u64 {
    value[key].as_u64().unwrap_or(0)
}
fn flag(value: &Value, key: &str) -> bool {
    value[key].as_bool().unwrap_or(false)
}
fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value[key].as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn action(command: &str, payload: Value) -> Value {
    json!({"command":command,"payload":payload})
}
fn short_decimal(number: f64) -> String {
    format!("{number:.2}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}
fn millions(value: u64) -> String {
    format!("{}M", short_decimal(value as f64 / 1_000_000.0))
}
fn value_range(min: u64, max: u64) -> String {
    if min == max {
        millions(min)
    } else {
        format!("{} - {}", millions(min), millions(max))
    }
}
fn decode_symbol_text(text: &str) -> String {
    if !text.starts_with('\\') {
        return text.to_owned();
    }
    text.split('\\')
        .filter(|part| !part.is_empty())
        .filter_map(|part| u32::from_str_radix(part, 16).ok().and_then(char::from_u32))
        .collect()
}
fn measured(ui: &egui::Ui, text: &str, size: f32) -> f32 {
    measured_with(ui, text, source_font(ui, size, "table"))
}
fn measured_with(ui: &egui::Ui, text: &str, font: FontId) -> f32 {
    css_text::measure(ui, text, &font)
        .map(|size| size.x)
        .unwrap_or_else(|| {
            ui.fonts(|fonts| {
                fonts
                    .layout_no_wrap(text.to_owned(), font, Color32::WHITE)
                    .size()
                    .x
            })
        })
}
fn cell(row: Rect, widths: &[f32], start: usize, span: usize) -> Rect {
    Rect::from_min_size(
        row.min + Vec2::new(widths[..start].iter().sum(), 0.0),
        Vec2::new(widths[start..start + span].iter().sum(), row.height()),
    )
}
fn species_cell(row: Rect, widths: &[f32; 11], index: usize) -> Rect {
    cell(
        row,
        widths,
        SPECIES_SPANS[..index].iter().sum(),
        SPECIES_SPANS[index],
    )
}
fn body_cell(row: Rect, widths: &[f32; 11], index: usize) -> Rect {
    cell(row, widths, HEADERS[index].1, HEADERS[index].2)
}
fn label(ui: &egui::Ui, rect: Rect, text: &str, color: Color32, size: f32, align: Align2) {
    label_with(ui, rect, text, color, source_font(ui, size, "table"), align);
}
fn label_bold(ui: &egui::Ui, rect: Rect, text: &str, color: Color32, size: f32, align: Align2) {
    label_with(ui, rect, text, color, source_font(ui, size, "bold"), align);
}
fn label_with(ui: &egui::Ui, rect: Rect, text: &str, color: Color32, font: FontId, align: Align2) {
    let padding = CELL_PADDING * css_scale(ui);
    let position = match align {
        Align2::RIGHT_CENTER => rect.right_center() - Vec2::new(padding, 0.0),
        Align2::CENTER_CENTER => rect.center(),
        _ => rect.left_center() + Vec2::new(padding, 0.0),
    };
    if !css_text::paint(ui, text, &font, position, align, color, rect) {
        ui.painter()
            .with_clip_rect(rect)
            .text(position, align, text, font, color);
    }
}
fn original_svg(theme: &Theme, key: &str) -> Option<(Vec<u8>, Option<Vec2>)> {
    use base64::Engine as _;
    let symbol = &theme.data["Symbols"][key];
    if symbol["ContentType"] == "Base64SVG" {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(symbol["Content"].as_str()?)
            .ok()?;
        let svg = String::from_utf8(bytes).ok()?;
        let color = theme.symbol_color(key, false);
        return Some((
            svg.replace(
                "$$COLOR$$",
                &format!("{:02X}{:02X}{:02X}", color.r(), color.g(), color.b()),
            )
            .into_bytes(),
            None,
        ));
    }
    if let Some(button) = key.strip_prefix("Button-") {
        type Templates = std::collections::BTreeMap<String, (String, Vec2)>;
        static TEMPLATES: std::sync::OnceLock<Templates> = std::sync::OnceLock::new();
        let templates = TEMPLATES.get_or_init(|| {
            let css = include_str!("../../observatory-plugins/src/bioinsights/buttons.css");
            [
                "main",
                "unsold",
                "about",
                "codexcompanion",
                "popout",
                "zoom",
                "settings",
            ]
            .into_iter()
            .filter_map(|key| {
                let rest = css.split_once(&format!("#{key} {{"))?.1.split_once('}')?.0;
                let encoded = rest
                    .split_once("data:image/svg+xml,")?
                    .1
                    .split_once("\"")?
                    .0;
                let mut bytes = Vec::with_capacity(encoded.len());
                let raw = encoded.as_bytes();
                let mut i = 0;
                while i < raw.len() {
                    if raw[i] == b'%' && i + 2 < raw.len() {
                        let number = std::str::from_utf8(&raw[i + 1..i + 3])
                            .ok()
                            .and_then(|s| u8::from_str_radix(s, 16).ok());
                        if let Some(byte) = number {
                            bytes.push(byte);
                            i += 3;
                            continue;
                        }
                    }
                    bytes.push(raw[i]);
                    i += 1;
                }
                let svg = String::from_utf8(bytes).ok()?;
                let size = crate::svg_image::intrinsic_size(
                    svg.replace("$$BUTTON$$", "000000").as_bytes(),
                )
                .unwrap_or(Vec2::splat(1.0));
                Some((key.to_owned(), (svg, size)))
            })
            .collect()
        });
        let (svg, size) = templates.get(button)?;
        let color = theme.color("Button");
        return Some((
            svg.replace(
                "$$BUTTON$$",
                &format!("{:02X}{:02X}{:02X}", color.r(), color.g(), color.b()),
            )
            .into_bytes(),
            Some(*size),
        ));
    }
    None
}
fn source_symbol(ui: &egui::Ui, theme: &Theme, key: &str, rect: Rect) {
    source_symbol_inherit(ui, theme, key, rect, false);
}
fn source_symbol_inherit(ui: &egui::Ui, theme: &Theme, key: &str, rect: Rect, bold: bool) {
    let vector = original_svg(theme, key).is_some_and(|(bytes, size)| {
        let target = if let Some(size) = size {
            let ratio = (rect.width() / size.x).min(rect.height() / size.y);
            Rect::from_min_size(rect.min, size * ratio)
        } else {
            Rect::from_center_size(rect.center(), Vec2::splat(rect.width().min(rect.height())))
        };
        crate::svg_image::paint_svg(ui, &bytes, key, target).is_ok()
    });
    if vector {
    } else if let Some((name, bytes)) = symbols::symbol(theme.orange, key) {
        let id = Id::new(("bioinsights_original_symbol", name));
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
            ui.painter().with_clip_rect(rect).image(
                texture.id(),
                Rect::from_center_size(rect.center(), Vec2::splat(rect.width().min(rect.height()))),
                Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
    } else {
        let symbol = &theme.data["Symbols"][key];
        let scale = css_scale(ui);
        let is_label = symbol["IsLabel"] == true;
        let size = symbol["FontSize"]
            .as_f64()
            .unwrap_or(if is_label { 14.0 } else { 15.0 }) as f32
            * scale;
        let font = source_font(
            ui,
            size,
            if is_label {
                if bold {
                    "label-bold"
                } else {
                    "label"
                }
            } else if bold || symbol["FontWeight"] == "bold" {
                "bold"
            } else {
                "table"
            },
        );
        let text = decode_symbol_text(symbol["Content"].as_str().unwrap_or(""));
        let target = if is_label {
            let line = ui.fonts(|fonts| fonts.row_height(&font)).ceil();
            let badge =
                Rect::from_center_size(rect.center(), Vec2::new(rect.width(), line + 2.0 * scale));
            ui.painter()
                .rect_filled(badge, 7.0 * scale, theme.symbol_color(key, true));
            badge
        } else {
            rect
        };
        if !css_text::paint(
            ui,
            &text,
            &font,
            target.center(),
            Align2::CENTER_CENTER,
            theme.symbol_color(key, false),
            ui.clip_rect(),
        ) {
            ui.painter().with_clip_rect(ui.clip_rect()).text(
                target.center(),
                Align2::CENTER_CENTER,
                text,
                font,
                theme.symbol_color(key, false),
            );
        }
    }
    let description = theme.data["Symbols"][key]["Description"]
        .as_str()
        .unwrap_or("");
    if !description.is_empty() && theme.data["Symbols"][key]["SupressMouseOver"] != true {
        ui.interact(
            rect,
            Id::new((
                "bioinsights-symbol-tooltip",
                key,
                rect.min.x.to_bits(),
                rect.min.y.to_bits(),
            )),
            Sense::hover(),
        )
        .on_hover_text(description);
    }
}

/// Native equivalent of the installed table's intrinsic column sizing. Spanning
/// cells constrain the same eleven logical columns as the original HTML.
struct WidthCache {
    view: Value,
    pixels_per_point: u32,
    columns: [f32; 11],
    fonts: Vec<egui::FontFamily>,
    theme: Value,
    source_revision: std::sync::atomic::AtomicU64,
    header_mode: [bool; 3],
}

fn widths(
    ui: &egui::Ui,
    view: &Value,
    theme: &Theme,
    settings: &Value,
    available: f32,
    scale: f32,
) -> [f32; 11] {
    use std::sync::Arc;
    let id = Id::new("BioInsights intrinsic source table widths");
    let cache = ui.ctx().data(|data| data.get_temp::<Arc<WidthCache>>(id));
    let revision = ui
        .ctx()
        .data(|data| data.get_temp::<u64>(Id::new("Bio source projection revision")));
    let header_mode = [
        settings["ShowHeaders"].as_bool().unwrap_or(true),
        settings["ShowTotal"].as_bool().unwrap_or(false),
        settings["ShowBodyFields"].as_bool().unwrap_or(false),
    ];
    let pixels_per_point = ui.ctx().pixels_per_point().to_bits();
    let fonts = ["table", "bold", "italic", "label", "label-bold"]
        .map(|kind| source_font(ui, 15.0, kind).family)
        .to_vec();
    let mut columns = if let Some(cache) = cache.filter(|cache| {
        cache.pixels_per_point == pixels_per_point
            && (revision.is_some_and(|revision| {
                cache
                    .source_revision
                    .load(std::sync::atomic::Ordering::Relaxed)
                    == revision
            }) || cache.view == *view)
            && cache.fonts == fonts
            && cache.header_mode == header_mode
            && cache.theme == *theme.data
    }) {
        if let Some(revision) = revision {
            cache
                .source_revision
                .store(revision, std::sync::atomic::Ordering::Relaxed);
        }
        cache.columns
    } else {
        let columns = intrinsic_widths(ui, view, theme, settings);

        ui.ctx().data_mut(|data| {
            data.insert_temp(
                id,
                Arc::new(WidthCache {
                    view: view.clone(),
                    pixels_per_point,
                    columns,
                    fonts,
                    theme: theme.data.clone(),
                    header_mode,
                    source_revision: std::sync::atomic::AtomicU64::new(
                        revision.unwrap_or(u64::MAX),
                    ),
                }),
            )
        });
        columns
    };
    for column in &mut columns {
        *column *= scale;
    }
    let natural: f32 = columns.iter().sum();
    if available > natural {
        // CSS auto table layout distributes surplus width in proportion to
        // intrinsic widths when every cell is nowrap and no columns are fixed.
        for column in &mut columns {
            *column *= available / natural;
        }
    }
    columns
}

fn intrinsic_widths(ui: &egui::Ui, view: &Value, theme: &Theme, settings: &Value) -> [f32; 11] {
    let mut columns = [0.0_f32; 11];
    columns[0] = measured(ui, "\u{a0}\u{a0}", 15.0).round() + 10.0;
    let mut constraints = Vec::new();
    let text_width = |text: &str, size: f32, kind: &str| {
        let font = source_font(ui, size, kind);
        css_text::measure(ui, text, &font)
            .map(|size| size.x)
            .unwrap_or_else(|| {
                ui.fonts(|fonts| {
                    fonts
                        .layout_no_wrap(text.to_owned(), font, Color32::WHITE)
                        .size()
                        .x
                })
            })
            .round()
    };
    for header in header_rows(
        settings,
        string(view, "Commander"),
        number(view, "UnsoldTotal"),
    ) {
        let bordered = header.cells.len() > 1;
        for cell in header.cells {
            constraints.push((
                cell.start,
                cell.span,
                text_width(
                    &cell.text,
                    15.0,
                    if header.fill == "DefaultBackground" {
                        "table"
                    } else {
                        "bold"
                    },
                ) + 10.0
                    + if bordered {
                        2.0 * theme.border_width()
                    } else {
                        0.0
                    },
            ));
        }
    }
    for group in array(view, "Groups") {
        let flags = flag_keys(group)
            .iter()
            .map(|key| symbol_width(ui, theme, key, 1.0))
            .sum::<f32>();
        constraints.push((
            0,
            4,
            text_width(string(group, "BodyName"), 16.0, "bold") + flags + 11.0,
        ));
        constraints.push((
            4,
            1,
            symbol_width_inherit(ui, theme, string(group, "PlanetSymbol"), 1.0, true).round()
                + 10.0,
        ));
        let signals = if group["SignalSymbol"] == "BioCountAutoscan" {
            "autoscan".to_owned()
        } else {
            signal_text(number(group, "Signals"))
        };
        constraints.push((5, 1, text_width(&signals, 14.0, "label-bold") + 30.0));
        constraints.push((
            6,
            3,
            text_width(
                &format!("Star: {}", string(group, "RadiantStar")),
                16.0,
                "bold",
            ) + 10.0,
        ));
        let value = group["ValueText"]
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| {
                value_range(number(group, "MinimumValue"), number(group, "MaximumValue"))
            });
        constraints.push((9, 2, text_width(&value, 16.0, "bold") + 10.0));
        if settings["ShowBodyFields"] == true {
            let extra = &group["ExtraFields"];
            for (field, start, span) in [
                ("Atmosphere", 1, 3),
                ("Pressure", 4, 2),
                ("Volcanism", 6, 2),
                ("Gravity", 8, 1),
                ("Temperature", 9, 2),
            ] {
                constraints.push((
                    start,
                    span,
                    text_width(string(extra, field), 16.0, "bold")
                        + 11.0
                        + if start == 1 || start == 9 {
                            theme.border_width()
                        } else {
                            0.0
                        },
                ));
            }
        }
        for species in array(group, "Rows") {
            let absent = string(species, "Seen") == "NotPresent";
            let kind = if absent { "italic" } else { "table" };
            let flags = flag_keys(species)
                .iter()
                .filter(|key| !matches!(**key, "CurrentBody" | "HighValue"))
                .map(|key| symbol_width(ui, theme, key, 1.0))
                .sum::<f32>();
            constraints.push((
                1,
                1,
                text_width(string(species, "Genus"), 15.0, kind) + flags + 10.0,
            ));
            let prediction =
                prediction_symbol(string(species, "PredictionType"), string(species, "Seen"));
            constraints.push((
                2,
                1,
                if image_symbol(theme, prediction) {
                    symbol_width(ui, theme, prediction, 1.0) + 5.0
                } else {
                    measured(ui, "\u{a0}", 15.0).round() + 5.0
                },
            ));
            constraints.push((
                3,
                1,
                text_width(string(species, "Species"), 15.0, kind) + 5.0,
            ));
            let status = match string(species, "Seen") {
                "Seen" => "Tick",
                "NotPresent" => "Cross",
                _ => "Unknown",
            };
            constraints.push((
                4,
                1,
                measured(
                    ui,
                    if status == "Unknown" {
                        "\u{a0}\u{a0}"
                    } else {
                        "\u{a0}"
                    },
                    15.0,
                )
                .round()
                    + symbol_width(ui, theme, status, 1.0).round()
                    + 10.0,
            ));
            constraints.push((
                5,
                1,
                symbol_width(
                    ui,
                    theme,
                    sample_symbol(string(species, "SampleState")),
                    1.0,
                )
                .round()
                    + 10.0,
            ));
            let mut length = 0.0;
            for (index, variant) in array(species, "Variants")
                .iter()
                .filter(|v| v["ShowInUI"] != false)
                .enumerate()
            {
                if index > 0 {
                    length += text_width(
                        variant_separator(string(species, "PredictionType")),
                        15.0,
                        kind,
                    );
                }
                length += flag_keys(variant)
                    .iter()
                    .filter(|key| !matches!(**key, "CurrentBody" | "HighValue"))
                    .map(|key| symbol_width(ui, theme, key, 1.0))
                    .sum::<f32>();
                let likelihood = match variant["Likelihood"].as_str().unwrap_or("") {
                    "Likely" | "LikelyInCurrentRegion" => "Likely",
                    "Unlikely" => "Unlikely",
                    _ => "Maybe",
                };
                length += symbol_width(ui, theme, likelihood, 1.0)
                    + text_width(
                        string(variant, "Colour"),
                        15.0,
                        if absent || flag(variant, "NotPresent") {
                            "italic"
                        } else {
                            "table"
                        },
                    );
            }
            constraints.push((6, 2, length + 10.0));
            constraints.push((
                8,
                1,
                text_width(&millions(number(species, "BaseValue")), 15.0, kind) + 10.0,
            ));
            constraints.push((
                9,
                2,
                text_width(&format!("{}m", number(species, "Distance")), 15.0, kind) + 10.0,
            ));
        }
    }
    constraints.sort_by_key(|(_, span, _)| *span);
    for (start, span, width) in constraints {
        let total = columns[start..start + span].iter().sum::<f32>();
        if width > total {
            for column in &mut columns[start..start + span] {
                *column += if total > 0.0 {
                    (width - total) * *column / total
                } else {
                    width / span as f32
                };
            }
        }
    }
    columns
}

fn source_border(ui: &egui::Ui, rect: Rect, theme: &Theme, scale: f32) {
    let border = theme.border_width() * scale;
    if border > 0.0 {
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(border, theme.color("Border")),
            egui::StrokeKind::Inside,
        );
    }
}
fn row(ui: &mut egui::Ui, width: f32, height: f32, background: Color32) -> Rect {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, background);
    rect
}
fn flags(ui: &egui::Ui, theme: &Theme, view: &Value, rect: Rect, scale: f32, body: bool) -> f32 {
    let mut x = rect.left() + CELL_PADDING * scale;
    for key in flag_keys(view)
        .into_iter()
        .filter(|key| body || !matches!(*key, "CurrentBody" | "HighValue"))
    {
        let width = symbol_width(ui, theme, key, scale);
        source_symbol(
            ui,
            theme,
            key,
            Rect::from_min_size(
                egui::pos2(
                    x,
                    rect.center().y - (theme.row_height() - 1.0) * scale / 2.0,
                ),
                Vec2::new(width, (theme.row_height() - 1.0) * scale),
            ),
        );
        x += width;
    }
    x
}
fn flag_keys(view: &Value) -> Vec<&'static str> {
    [
        ("Current", "CurrentBody"),
        ("HighValue", "HighValue"),
        ("PersonalCodex", "PersonalCodexEntry"),
        ("Marked", "Mark"),
        ("GalacticNeverSeen", "PossibleCodexEntryNeverSeen"),
        ("GalacticCodex", "PossibleCodexEntry"),
    ]
    .into_iter()
    .filter(|(field, _)| {
        flag(view, field) && !(*field == "GalacticCodex" && flag(view, "GalacticNeverSeen"))
    })
    .map(|(_, key)| key)
    .collect()
}
struct TableLayout<'a> {
    theme: &'a Theme,
    widths: [f32; 11],
    scale: f32,
}
fn symbol_width(ui: &egui::Ui, theme: &Theme, key: &str, scale: f32) -> f32 {
    symbol_width_inherit(ui, theme, key, scale, false)
}
fn symbol_width_inherit(ui: &egui::Ui, theme: &Theme, key: &str, scale: f32, bold: bool) -> f32 {
    let symbol = &theme.data["Symbols"][key];
    let font = source_font(
        ui,
        if symbol["IsLabel"] == true {
            14.0
        } else {
            15.0
        } * scale,
        if symbol["IsLabel"] == true {
            if bold {
                "label-bold"
            } else {
                "label"
            }
        } else if bold || symbol["FontWeight"] == "bold" {
            "bold"
        } else {
            "table"
        },
    );
    match symbol["ContentType"].as_str().unwrap_or("") {
        "Text" => {
            ({
                let text = decode_symbol_text(symbol["Content"].as_str().unwrap_or(""));
                css_text::measure(ui, &text, &font)
                    .map(|size| size.x)
                    .unwrap_or_else(|| {
                        ui.fonts(|fonts| fonts.layout_no_wrap(text, font, Color32::WHITE).size().x)
                    })
            }) + if symbol["IsLabel"] == true {
                20.0 * scale
            } else {
                0.0
            }
        }
        "None" => 0.0,
        "Base64PNG" => (theme.row_height() + 1.0) * scale,
        _ => (theme.row_height() - 1.0) * scale,
    }
}
fn inline_symbol(
    ui: &egui::Ui,
    theme: &Theme,
    key: &str,
    cell: Rect,
    scale: f32,
    leading_space: bool,
) {
    if key == "Unknown" {
        return;
    }
    let width = symbol_width(ui, theme, key, scale);
    source_symbol(
        ui,
        theme,
        key,
        Rect::from_min_size(
            cell.left_center()
                + Vec2::new(
                    if leading_space {
                        measured(ui, "\u{a0}", 15.0 * scale)
                    } else {
                        0.0
                    } + 5.0 * scale,
                    -(theme.row_height() - 1.0) * scale / 2.0,
                ),
            Vec2::new(width, (theme.row_height() - 1.0) * scale),
        ),
    );
}
fn species_row(
    ui: &mut egui::Ui,
    layout: &TableLayout<'_>,
    group: &Value,
    species: &Value,
    index: usize,
    actions: &mut Vec<Value>,
) {
    let theme = layout.theme;
    let widths = &layout.widths;
    let height = species_height(ui, theme, species, layout.scale);
    let scale = layout.scale;
    let width: f32 = widths.iter().sum();
    let absent = string(species, "Seen") == "NotPresent";
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    ui.painter().rect_filled(
        Rect::from_min_max(rect.min + Vec2::new(widths[0], 0.0), rect.max),
        0.0,
        theme.color(
            if species["Stripe"]
                .as_u64()
                .unwrap_or(index as u64)
                .is_multiple_of(2)
            {
                "Row1Background"
            } else {
                "Row2Background"
            },
        ),
    );
    let foreground = theme.color(if absent {
        "NotPresentText"
    } else {
        "DefaultText"
    });
    let genus = species_cell(rect, widths, 1);
    let start = flags(ui, theme, species, genus, scale, false);
    label_with(
        ui,
        Rect::from_min_max(
            egui::pos2(start - CELL_PADDING * scale, genus.top()),
            genus.max,
        ),
        string(species, "Genus"),
        foreground,
        source_font(ui, 15.0 * scale, if absent { "italic" } else { "table" }),
        Align2::LEFT_CENTER,
    );
    let prediction = prediction_symbol(string(species, "PredictionType"), string(species, "Seen"));
    source_symbol(
        ui,
        theme,
        prediction,
        Rect::from_min_size(
            species_cell(rect, widths, 2).left_center()
                + Vec2::new(5.0 * scale, -(theme.row_height() - 1.0) * scale / 2.0),
            Vec2::splat((theme.row_height() - 1.0) * scale),
        ),
    );
    let species_rect = species_cell(rect, widths, 3);
    let species_font = source_font(ui, 15.0 * scale, if absent { "italic" } else { "table" });
    let text_size = css_text::measure(ui, string(species, "Species"), &species_font)
        .unwrap_or_else(|| {
            ui.fonts(|fonts| {
                fonts
                    .layout_no_wrap(
                        string(species, "Species").to_owned(),
                        species_font.clone(),
                        foreground,
                    )
                    .size()
            })
        });
    let link = Rect::from_min_size(
        species_rect.left_center() - Vec2::new(0.0, text_size.y / 2.0),
        text_size,
    );
    let response = ui
        .interact(
            link,
            Id::new((
                "bioinsights-species",
                number(group, "ID64"),
                number(group, "BodyID"),
                string(species, "SpeciesID"),
            )),
            Sense::click(),
        )
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    if response.hovered() {
        ui.painter()
            .rect_filled(link, 0.0, theme.color("DefaultText"));
    }
    label_with(
        ui,
        species_rect.translate(Vec2::new(-CELL_PADDING * scale, 0.0)),
        string(species, "Species"),
        if response.hovered() {
            theme.color("DefaultBackground")
        } else {
            foreground
        },
        species_font,
        Align2::LEFT_CENTER,
    );
    if response.clicked() {
        actions.push(action(
            "CodexCompanion",
            json!({"SpeciesID":species["SpeciesID"]}),
        ));
    }
    match string(species, "Seen") {
        "Seen" => inline_symbol(
            ui,
            theme,
            "Tick",
            species_cell(rect, widths, 4),
            scale,
            true,
        ),
        "NotPresent" => inline_symbol(
            ui,
            theme,
            "Cross",
            species_cell(rect, widths, 4),
            scale,
            true,
        ),
        _ => {}
    }
    let sampled = sample_symbol(string(species, "SampleState"));
    inline_symbol(
        ui,
        theme,
        sampled,
        species_cell(rect, widths, 5),
        scale,
        false,
    );
    let variants = species_cell(rect, widths, 6);
    let mut x = variants.left() + 5.0 * scale;
    for (index, variant) in array(species, "Variants")
        .iter()
        .filter(|v| v["ShowInUI"] != false)
        .enumerate()
    {
        if x >= variants.right() - 5.0 * scale {
            break;
        }
        if index > 0 {
            let separator = variant_separator(string(species, "PredictionType"));
            let width = measured(ui, separator, 15.0 * scale);
            label(
                ui,
                Rect::from_min_size(
                    egui::pos2(x, variants.top()),
                    Vec2::new(width, variants.height()),
                ),
                separator,
                foreground,
                15.0 * scale,
                Align2::LEFT_CENTER,
            );
            x += width;
        }
        x = flags(
            ui,
            theme,
            variant,
            Rect::from_min_max(egui::pos2(x - 5.0 * scale, variants.top()), variants.max),
            scale,
            false,
        );
        let likelihood = match variant
            .get("Likelihood")
            .and_then(Value::as_str)
            .unwrap_or("")
        {
            "Likely" | "LikelyInCurrentRegion" => "Likely",
            "Unlikely" => "Unlikely",
            _ => "Maybe",
        };
        let icon = symbol_width(ui, theme, likelihood, scale);
        source_symbol(
            ui,
            theme,
            likelihood,
            Rect::from_min_size(
                egui::pos2(
                    x,
                    variants.center().y - (theme.row_height() - 1.0) * scale / 2.0,
                ),
                Vec2::new(icon, (theme.row_height() - 1.0) * scale),
            ),
        );
        x += icon;
        let text = string(variant, "Colour");
        let width = measured(ui, text, 15.0 * scale) + 5.0 * scale;
        let vrect = Rect::from_min_size(
            egui::pos2(x, variants.top()),
            Vec2::new(width, variants.height()),
        )
        .intersect(variants);
        label_with(
            ui,
            vrect.translate(Vec2::new(-CELL_PADDING * scale, 0.0)),
            text,
            if flag(variant, "NotPresent") {
                theme.color("NotPresentText")
            } else {
                foreground
            },
            source_font(
                ui,
                15.0 * scale,
                if absent || flag(variant, "NotPresent") {
                    "italic"
                } else {
                    "table"
                },
            ),
            Align2::LEFT_CENTER,
        );
        ui.interact(
            vrect,
            Id::new((
                "bioinsights-variant-reason",
                number(group, "ID64"),
                number(group, "BodyID"),
                string(species, "SpeciesID"),
                text,
            )),
            Sense::hover(),
        )
        .on_hover_text(string(variant, "Reason"));
        x += width;
    }
    let value = species["ValueText"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| millions(number(species, "BaseValue")));
    label(
        ui,
        species_cell(rect, widths, 7),
        &value,
        foreground,
        15.0 * scale,
        Align2::LEFT_CENTER,
    );
    label_with(
        ui,
        species_cell(rect, widths, 8),
        &format!("{}m", number(species, "Distance")),
        foreground,
        source_font(ui, 15.0 * scale, if absent { "italic" } else { "table" }),
        Align2::LEFT_CENTER,
    );
}

pub fn show(ui: &mut egui::Ui, view: &Value, summary: &Value, settings: &Value) -> Vec<Value> {
    show_variant(ui, view, summary, settings, false)
}

pub fn show_variant(
    ui: &mut egui::Ui,
    view: &Value,
    summary: &Value,
    settings: &Value,
    hide_buttons: bool,
) -> Vec<Value> {
    prepare_fonts(ui.ctx());
    let theme = Theme::new(settings);
    let mut actions = Vec::new();
    let mut page = pages::State::load(ui.ctx());
    page.synchronize_commander(view);
    let mut zoom = view["ZoomLevel"]
        .as_u64()
        .or_else(|| settings["ZoomLevel"].as_u64())
        .unwrap_or(100)
        .clamp(50, 600) as u32;
    let scale = zoom as f32 / 100.0;
    egui::ScrollArea::vertical().id_salt(("Bio source document",page.scroll_id())).auto_shrink([false,false]).show(ui,|ui|{
    ui.spacing_mut().scroll.floating=true;
    egui::Frame::NONE
        .fill(theme.color("DefaultBackground"))
        .inner_margin(8.0 * scale)
        .show(ui, |ui| {
            let table_font=source_font(ui,15.0*scale,"table");
            ui.style_mut().text_styles.insert(egui::TextStyle::Body,table_font);
            ui.spacing_mut().item_spacing = Vec2::new(2.0 * scale, 0.0);
            if view["DependencyNag"]==true {
                ui.heading("Plugin Missing or Disabled - Hecate");
                ui.add_space(16.0);
                ui.heading("What is Hecate?");
                ui.label("Hecate is a Data Vault plugin whose purpose is the storing and retrieving data for other plugins.");
                ui.label("This means that instead of each plugin having to maintain it's own background database of information, it can be store in a centralised location.");
                ui.label("It also means API calls to spansh or similar services can be made just once and shared between plugins instead of each plugin making it's own requests.");
                ui.add_space(12.0);ui.strong("Do I need it?");
                ui.label("It's not required, but certain features won't work without it.");
                ui.add_space(24.0);
                if ui.button("Enable Hecate").clicked(){actions.push(json!({"command":"InstallHecate","payload":{}}));}
                ui.small("Hecate is included with this Linux build.");
                if ui.button("Skip").clicked(){actions.push(json!({"command":"BrowserNavigate","payload":{"Url":"observatory://bioinsights/skiphecate"}}));}
                return;
            }
            if !hide_buttons {
                egui::Frame::NONE.fill(theme.color("ButtonBar")).inner_margin(2.0 * scale).show(ui,|ui| ui.horizontal(|ui| {
                    for (key, tooltip, command) in [
                        ("main", "Main Window", "Main"),
                        ("unsold", "Show Current Data", "Unsold"),
                        ("about", "About", "About"),
                        ("codexcompanion", "Codex Companion", "CodexCompanion"),
                        ("popout", "Popout Window", "Popout"),
                        ("zoom", "Zoom", "Zoom"),
                        ("settings", "Settings", "Settings"),
                    ] {
                        let size = match settings["ButtonSize"].as_str().unwrap_or("Normal").trim()
                        {
                            "Smaller" => 16.0,
                            "Small" => 24.0,
                            "Large" => 48.0,
                            "Extra Large" => 64.0,
                            _ => 32.0,
                        } * scale;
                        // buttons.css has a size/16 margin on every side in
                        // addition to the two-pixel flex gap and bar padding.
                        let margin=size/16.0;
                        let (rect, response) =
                            ui.allocate_exact_size(Vec2::splat(size+2.0*margin), Sense::click());
                        let rect=rect.shrink(margin);
                        ui.painter().rect_filled(
                            rect,
                            5.0 * scale,
                            theme.color(if response.hovered() {
                                "ButtonFaceHighlight"
                            } else {
                                "ButtonFace"
                            }),
                        );
                        source_symbol(
                            ui,
                            &theme,
                            &format!("Button-{key}"),
                            rect.shrink(if response.hovered(){2.0*scale}else{4.0*scale}),
                        );
                        if response.on_hover_text(tooltip).clicked() {
                            actions.extend(page.route(command, &json!({}), ui.ctx()));
                        }
                    }
                }));
            }
            egui::Frame::NONE.inner_margin(egui::Margin {left:(2.0*scale).round()as i8,top:(5.0*scale).round()as i8,right:0,bottom:0}).show(ui,|ui|{
                for alert in array(view,"UIAlerts") {css_alert(ui,alert,scale,&mut actions);}
            if page.is_main() {
                egui::ScrollArea::horizontal()
                    .id_salt("bioinsights-original-main-table")
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        egui::Frame::NONE.inner_margin(egui::Margin {left:(5.0*scale).round()as i8,right:(5.0*scale).round()as i8,top:(2.0*scale).round()as i8,bottom:(2.0*scale).round()as i8}).show(ui,|ui|{
                        ui.spacing_mut().item_spacing.y=theme.spacing()*scale;
                        ui.add_space(theme.spacing()*scale);
                        let columns = widths(
                            ui,
                            view,
                            &theme,
                            settings,
                            if settings["ExpandOutput"].as_bool().unwrap_or(true) {
                                ui.available_width()
                            } else {
                                0.0
                            },
                            scale,
                        );
                        let width = columns.iter().sum();
                        let height = css_header_height(ui,&theme,scale,false);
                        let layout = TableLayout {
                            theme: &theme,
                            widths: columns,
                            scale,
                        };
                        ui.set_min_width(width);
                        for header in header_rows(
                            settings,
                            string(view, "Commander"),
                            number(view, "UnsoldTotal"),
                        ) {
                            let header_height=if header.fill=="DefaultBackground"{css_line_height(ui,15.0*scale,"table")+4.0*scale}else if header.cells.len()==1{css_line_height(ui,15.0*scale,"bold")+2.0*scale}else{css_header_height(ui,&theme,scale,header.cells[0].text.is_empty())};
                            let rect = row(ui, width, header_height, theme.color(header.fill));
                            if header.cells.len() > 1 && header.cells[0].text.is_empty() {
                                ui.painter().rect_filled(
                                    cell(rect, &columns, 0, header.cells[0].span),
                                    0.0,
                                    theme.color("DefaultBackground"),
                                );
                            }
                            let bordered=header.cells.len()>1;
                            for heading in header.cells {
                                if bordered&&!heading.text.is_empty(){source_border(ui,cell(rect,&columns,heading.start,heading.span),&theme,scale);}
                                label_bold(
                                    ui,
                                    cell(rect, &columns, heading.start, heading.span),
                                    &heading.text,
                                    theme.color(header.text),
                                    15.0 * scale,
                                    heading.align,
                                );
                            }
                        }
                        for group in array(view, "Groups") {
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::new(width,css_line_height(ui,16.0*scale,"bold").max(if flag_keys(group).iter().any(|key|image_symbol(&theme,key)){(theme.row_height()-1.0)*scale}else{0.0})+4.0*scale), Sense::hover());
                            ui.painter().rect_filled(
                                rect,
                                egui::CornerRadius {
                                    nw: (10.0 * scale) as u8,
                                    sw: (10.0 * scale) as u8,
                                    ne: 0,
                                    se: 0,
                                },
                                theme.color(
                                    if flag(group, "Current") && theme.highlight_row()
                                    {
                                        "HighlightBackground"
                                    } else {
                                        "HeaderBackground"
                                    },
                                ),
                            );
                            let foreground = theme.color("HeaderText");
                            let body = body_cell(rect, &columns, 0);
                            ui.painter().rect_filled(Rect::from_min_max(egui::pos2(body.right()-scale,body.top()),body.max),0.0,theme.color("HeaderBackground"));
                            let x = flags(ui, &theme, group, body, scale,true);
                            label_bold(
                                ui,
                                Rect::from_min_max(
                                    egui::pos2(x - CELL_PADDING * scale, body.top()),
                                    body.max,
                                ),
                                string(group, "BodyName"),
                                foreground,
                                16.0 * scale,
                                Align2::LEFT_CENTER,
                            );
                            let planet = body_cell(rect, &columns, 1);
                            let badge_width = symbol_width_inherit(ui,&theme,string(group,"PlanetSymbol"),scale,true);
                            source_symbol_inherit(
                                ui,
                                &theme,
                                string(group, "PlanetSymbol"),
                                Rect::from_min_size(
                                    planet.min + Vec2::new(5.0 * scale, 0.0),
                                    Vec2::new(
                                        badge_width.min((planet.width() - 10.0 * scale).max(1.0)),
                                        planet.height(),
                                    ),
                                ),
                                true,
                            );
                            let signal_key = group["SignalSymbol"].as_str().unwrap_or("BioCount");
                            let signals = body_cell(rect, &columns, 2);
                            let count = if signal_key == "BioCountAutoscan" {
                                "autoscan".into()
                            } else {
                                group
                                    .get("Signals")
                                    .and_then(Value::as_u64)
                                    .map(signal_text)
                                    .unwrap_or_default()
                            };
                            let signal_font=source_font(ui,14.0*scale,"label-bold");
                            let signal_line=ui.fonts(|fonts| fonts.row_height(&signal_font)).ceil();
                            let signal_width = (css_text::measure(ui,&count,&signal_font).map(|size|size.x).unwrap_or_else(||ui.fonts(|fonts|fonts.layout_no_wrap(count.clone(),signal_font.clone(),Color32::WHITE).size().x)) + 20.0 * scale)
                                .min(signals.width() - 10.0 * scale);
                            let badge = Rect::from_min_size(
                                signals.left_center() + Vec2::new(5.0 * scale, -(signal_line+2.0*scale)/2.0),
                                Vec2::new(signal_width, signal_line+2.0*scale),
                            );
                            ui.painter().rect_filled(
                                badge,
                                7.0 * scale,
                                theme.symbol_color(signal_key, true),
                            );
                            label_with(
                                ui,
                                badge,
                                &count,
                                theme.symbol_color(signal_key, false),
                                signal_font,
                                Align2::CENTER_CENTER,
                            );
                            label_bold(
                                ui,
                                body_cell(rect, &columns, 3),
                                &format!("Star: {}", string(group, "RadiantStar")),
                                foreground,
                                16.0 * scale,
                                Align2::LEFT_CENTER,
                            );
                            let value = group["ValueText"]
                                .as_str()
                                .map(str::to_owned)
                                .unwrap_or_else(|| {
                                    value_range(
                                        number(group, "MinimumValue"),
                                        number(group, "MaximumValue"),
                                    )
                                });
                            label_bold(
                                ui,
                                body_cell(rect, &columns, 4),
                                &value,
                                foreground,
                                16.0 * scale,
                                Align2::RIGHT_CENTER,
                            );
                            if settings["ShowBodyFields"].as_bool().unwrap_or(false) {
                                let detail = row(ui, width, css_line_height(ui,16.0*scale,"bold")+4.0*scale, theme.color("HeaderBackground"));
                                ui.painter().rect_filled(cell(detail,&columns,0,1),0.0,theme.color("DefaultBackground"));
                                let extra=&group["ExtraFields"];
                                for (text, start, span) in [
                                    (string(extra,"Atmosphere").to_owned(),1,3),
                                    (string(extra,"Pressure").to_owned(),4,2),
                                    (string(extra,"Volcanism").to_owned(),6,2),
                                    (string(extra,"Gravity").to_owned(),8,1),
                                    (string(extra,"Temperature").to_owned(),9,2),
                                ] {
                                    label(
                                        ui,
                                        cell(detail, &columns, start, span),
                                        &text,
                                        theme.color("HeaderText"),
                                        15.0 * scale,
                                        Align2::LEFT_CENTER,
                                    );
                                }
                            }
                            if array(group, "Rows").is_empty() {
                                let no_predictions =
                                    row(ui, width, css_line_height(ui,15.0*scale,"table")+4.0*scale, theme.color("Row1Background"));
                                label_with(ui,cell(no_predictions,&columns,1,10),"No predictions",theme.color("NotPresentText"),source_font(ui,15.0*scale,"italic"),Align2::CENTER_CENTER);
                            }
                            for (index, species) in array(group, "Rows").iter().enumerate() {
                                species_row(ui, &layout, group, species, index, &mut actions);
                            }
                        }
                        if settings["ShowHeaders"].as_bool().unwrap_or(true) {
                            row(ui, width, css_line_height(ui,15.0*scale,"bold")+2.0*scale, theme.color("HeaderBackground"));
                        }
                        let footer=row(ui, width, height, theme.color("TitleBackground"));source_border(ui,footer,&theme,scale);
                        ui.add_space(theme.spacing()*scale);
                        });
                    });
            } else {
                page.show(
                    ui,
                    view,
                    summary,
                    pages::Presentation {
                        theme: &theme,
                        settings,
                        scale,
                    },
                    &mut actions,
                );
            }
            });
        });
    });
    page.zoom_selector(ui.ctx(), &mut zoom, &mut actions);
    let mut host_actions = Vec::new();
    for emitted in actions {
        let command = emitted["command"].as_str().unwrap_or("");
        if matches!(command, "Main" | "Unsold" | "About" | "CodexCompanion") {
            host_actions.extend(page.route(command, &emitted["payload"], ui.ctx()));
        } else {
            host_actions.push(emitted);
        }
    }
    page.save(ui.ctx());
    host_actions
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_theme_inherits_sources_disabled_highlight_default() {
        assert!(!Theme::new(&json!({"ThemeSettings":{"ThemeIndex":0}})).highlight_row());
        assert!(Theme::new(&json!({"ThemeSettings":{"ThemeIndex":1}})).highlight_row());
    }
    #[test]
    fn header_spans_match_installed_table() {
        assert_eq!(HEADERS.iter().map(|(_, _, span)| span).sum::<usize>(), 11);
        assert_eq!(
            SUBHEADERS.iter().map(|(_, _, span)| span).sum::<usize>(),
            11
        );
        assert_eq!(HEADERS[0], ("Body", 0, 4));
        assert_eq!(SUBHEADERS[4], ("Variants", 6, 2));
    }
    #[test]
    fn source_theme_and_numeric_labels() {
        let theme = Theme::new(&json!({}));
        assert_eq!(theme.color("DefaultText"), Color32::from_rgb(255, 111, 0));
        assert_eq!(theme.color("Row1Background"), Color32::from_rgb(35, 16, 2));
        assert_eq!(theme.row_height(), 24.0);
        assert_eq!(millions(1_660_000), "1.66M");
        assert_eq!(value_range(6_900_000, 12_940_000), "6.9M - 12.94M");
        assert_eq!(decode_symbol_text("\\0025CF\\0025CB\\0025CB"), "●○○");
    }

    #[test]
    fn source_custom_colour_overrides_require_custom_theme_mode() {
        let mut settings = json!({"ThemeMode":1,"ThemeSettings":{"ThemeIndex":1,"ThemeMode":0,"Colors":{"DefaultText":{"R":1,"G":2,"B":3}}}});
        assert_eq!(
            Theme::new(&settings).color("DefaultText"),
            Color32::from_rgb(255, 111, 0)
        );
        settings["ThemeSettings"]["ThemeMode"] = json!(1);
        settings["ThemeMode"] = json!(0);
        assert_eq!(
            Theme::new(&settings).color("DefaultText"),
            Color32::from_rgb(1, 2, 3)
        );
        settings["ThemeSettings"]["ThemeMode"] = json!(2);
        settings["ThemeMode"] = json!(1);
        assert_eq!(
            Theme::new(&settings).color("DefaultText"),
            Color32::from_rgb(255, 111, 0)
        );
        // The older native port stored this flag at the plugin root. Retain
        // that fallback only when the source/form setting is absent.
        settings["ThemeSettings"]
            .as_object_mut()
            .unwrap()
            .remove("ThemeMode");
        assert_eq!(
            Theme::new(&settings).color("DefaultText"),
            Color32::from_rgb(1, 2, 3)
        );
    }
    #[test]
    fn toolbar_uses_original_embedded_images() {
        for name in [
            "main",
            "unsold",
            "about",
            "codexcompanion",
            "popout",
            "zoom",
            "settings",
        ] {
            assert!(
                symbols::symbol(true, &format!("Button-{name}")).is_some(),
                "{name}"
            );
        }
        let theme = Theme::new(&json!({}));
        assert!(original_svg(&theme, "Button-main").is_some());
        assert!(original_svg(&theme, "Button-unsold").is_some());
        assert!(
            original_svg(&theme, "Button-codexcompanion").is_none(),
            "Codex uses its original embedded PNG; do not consume a later CSS rule's SVG"
        );
    }
    #[test]
    fn sample_states_use_original_unicode_content_and_theme_colors() {
        let theme = Theme::new(&json!({}));
        for (key, text) in [
            ("BiologicalLogged", "●○○"),
            ("BiologicalSampled", "●●○"),
            ("BiologicalComplete", "●●●"),
            ("Aborted", "○○○"),
        ] {
            assert_eq!(
                decode_symbol_text(theme.data["Symbols"][key]["Content"].as_str().unwrap()),
                text
            );
        }
        assert_eq!(theme.symbol_color("Aborted", false), Color32::RED);
    }
    fn source_cells(html: &str) -> Vec<(usize, String)> {
        let mut remaining = html;
        let mut cells = Vec::new();
        loop {
            let start = match (remaining.find("<td"), remaining.find("<th")) {
                (Some(a), Some(b)) => a.min(b),
                (Some(a), None) | (None, Some(a)) => a,
                (None, None) => break,
            };
            remaining = &remaining[start..];
            let tag = if remaining.starts_with("<td") {
                "td"
            } else {
                "th"
            };
            let end = remaining.find('>').unwrap();
            let attributes = &remaining[..end];
            let span = attributes
                .split_once("colspan=")
                .and_then(|(_, value)| {
                    value
                        .trim_start_matches(['\"', '\''])
                        .split(|c: char| !c.is_ascii_digit())
                        .next()
                })
                .and_then(|n| n.parse().ok())
                .unwrap_or(1);
            remaining = &remaining[end + 1..];
            let close = format!("</{tag}>");
            let end = remaining.find(&close).unwrap();
            let inside = &remaining[..end];
            let mut text = String::new();
            let mut in_tag = false;
            for c in inside.chars() {
                match c {
                    '<' => in_tag = true,
                    '>' => in_tag = false,
                    _ if !in_tag => text.push(c),
                    _ => {}
                }
            }
            cells.push((span, text.replace("&nbsp;", "").trim().to_owned()));
            remaining = &remaining[end + close.len()..];
        }
        cells
    }
    #[test]
    fn all_eight_header_modes_match_independently_compiled_source() {
        let cases: Value =
            serde_json::from_str(include_str!("../assets/bioinsights/header-oracle.json")).unwrap();
        for case in cases.as_array().unwrap() {
            let settings = json!({"ShowHeaders":case["show_headers"],"ShowBodyFields":case["show_body_fields"],"ShowTotal":case["show_total"]});
            let native: Vec<_> = header_rows(&settings, "Reference Commander", 123456789)
                .into_iter()
                .flat_map(|row| {
                    row.cells
                        .into_iter()
                        .map(|cell| (cell.span, cell.text.trim().to_owned()))
                })
                .collect();
            assert_eq!(
                native,
                source_cells(case["html"].as_str().unwrap()),
                "{settings}"
            );
        }
    }
    #[test]
    fn all_seventy_five_species_states_match_source_cells() {
        let cases: Value =
            serde_json::from_str(include_str!("../assets/bioinsights/row-oracle.json")).unwrap();
        assert_eq!(cases.as_array().unwrap().len(), 75);
        for case in cases.as_array().unwrap() {
            let cells = source_cells(case["html"].as_str().unwrap());
            assert_eq!(
                cells.iter().map(|(span, _)| *span).collect::<Vec<_>>(),
                SPECIES_SPANS
            );
            let kind = case["type"].as_str().unwrap();
            let seen = case["seen"].as_str().unwrap();
            let sample = case["samples"].as_str().unwrap();
            let symbol = prediction_symbol(kind, seen);
            assert_eq!(
                cells[2].1,
                if symbol == "Unknown" {
                    String::new()
                } else {
                    format!("[{symbol}]")
                },
                "{kind}/{seen}"
            );
            assert_eq!(
                cells[4].1,
                match seen {
                    "Seen" => "[Tick]",
                    "NotPresent" => "[Cross]",
                    _ => "",
                }
            );
            let sample = sample_symbol(sample);
            assert_eq!(
                cells[5].1,
                if sample == "Unknown" {
                    String::new()
                } else {
                    format!("[{sample}]")
                }
            );
            assert!(
                cells[6]
                    .1
                    .contains(&format!("Emerald{}[Mark]", variant_separator(kind))),
                "{kind}: {}",
                cells[6].1
            );
            assert_eq!(cells[7].1, millions(1_658_500));
            assert_eq!(cells[8].1, "500m");
        }
    }
    #[test]
    fn all_twelve_body_headers_preserve_source_spans_flag_order_and_labels() {
        let cases: Value =
            serde_json::from_str(include_str!("../assets/bioinsights/body-oracle.json")).unwrap();
        assert_eq!(cases.as_array().unwrap().len(), 12);
        for case in cases.as_array().unwrap() {
            let cells = source_cells(case["html"].as_str().unwrap());
            assert_eq!(
                cells.iter().map(|(span, _)| *span).collect::<Vec<_>>(),
                HEADERS.map(|(_, _, span)| span)
            );
            let group = json!({"Current":case["current"],"HighValue":case["flags"],"PersonalCodex":case["flags"],"Marked":case["flags"],"GalacticCodex":case["flags"]});
            let prefix = flag_keys(&group)
                .into_iter()
                .map(|key| format!("[{key}]"))
                .collect::<String>();
            assert_eq!(cells[0].1, format!("{prefix}Reference System 1"));
            assert_eq!(
                cells[2].1,
                format!(
                    "[BioCount:{}]",
                    signal_text(case["signals"].as_u64().unwrap())
                )
            );
            assert_eq!(cells[3].1, "Star: G");
            assert_eq!(cells[4].1, value_range(6_900_000, 12_940_000));
        }
    }
}
