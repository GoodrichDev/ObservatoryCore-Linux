//! Source Radar.Draw projection. Source drawing calls are independent of egui,
//! and are compared with the copied C# method's complete command trace.
use eframe::egui::{self, Align2, Color32, Pos2, Stroke, Vec2};
use observatory_plugins::stellar_overlay::{radar_radius_fraction, RadarSnapshot};
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub position: Vec2,
    pub size: Vec2,
    pub diameter: f32,
    pub font_size: f32,
    pub origin: Vec2,
}
fn geometry(monitor: Vec2, settings: &Value) -> (Vec2, f32, f32) {
    let number = |key: &str, default: f32| settings[key].as_f64().unwrap_or(default as f64) as f32;
    let diameter = (monitor.y * number("SizePct", 20.0) / 100.0).trunc();
    let anchor = |extent: f32, pct: f32| {
        ((extent as i32 / 100) as f32 * pct).trunc()
            - ((diameter as i32 / 100) as f32 * pct).trunc()
    };
    (
        Vec2::new(
            anchor(monitor.x, number("PositionXPct", 2.0)),
            anchor(monitor.y, number("PositionYPct", 50.0)),
        ),
        diameter,
        (monitor.y * 12.0 / 1440.0 * number("FontScaling", 100.0) / 100.0)
            .trunc()
            .max(4.0),
    )
}
pub fn layout(
    ctx: &egui::Context,
    radar: &RadarSnapshot,
    monitor: Vec2,
    settings: &Value,
) -> Layout {
    let (position, diameter, font_size) = geometry(monitor, settings);
    let font = super::font(ctx, font_size);
    let measure = |s: &str| {
        ctx.fonts(|f| {
            f.layout_no_wrap(s.to_owned(), font.clone(), Color32::WHITE)
                .size()
        })
    };
    let ops = project(radar, diameter / 2.0, font_size, |s| measure(s).x);
    let mut bounds = egui::Rect::from_min_size(Pos2::ZERO, Vec2::splat(diameter));
    for command in ops {
        let a = &command.args;
        let rect = match command.op {
            "Text" => egui::Rect::from_min_size(
                Pos2::new(a[0], a[1]),
                measure(command.text.as_deref().unwrap_or("")),
            ),
            "Circle" | "FillCircle" | "FillEllipse" => egui::Rect::from_center_size(
                Pos2::new(a[0], a[1]),
                Vec2::splat(a[2] * 2.0 + if a.len() > 3 { a[3] } else { 0.0 }),
            ),
            "Crosshair" => egui::Rect::from_center_size(
                Pos2::new(a[0], a[1]),
                Vec2::splat((a[2] + a[3]) * 2.0),
            ),
            "Line" | "FillRectangle" | "RectangleEdges" => {
                egui::Rect::from_two_pos(Pos2::new(a[0], a[1]), Pos2::new(a[2], a[3]))
                    .expand(if a.len() > 4 { a[4] / 2.0 } else { 0.0 })
            }
            _ => continue,
        };
        bounds = bounds.union(rect);
    }
    bounds = bounds.expand(1.0);
    Layout {
        position: position + bounds.min.to_vec2(),
        size: bounds.size().ceil(),
        diameter,
        font_size,
        origin: -bounds.min.to_vec2(),
    }
}

#[derive(Debug, Serialize)]
struct Command {
    op: &'static str,
    key: &'static str,
    args: Vec<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<&'static str>,
}
fn shape(op: &'static str, key: &'static str, args: &[f32]) -> Command {
    Command {
        op,
        key,
        args: args.to_vec(),
        text: None,
        style: None,
    }
}
fn text(key: &'static str, x: f32, y: f32, value: impl Into<String>) -> Command {
    Command {
        op: "Text",
        key,
        args: vec![x, y],
        text: Some(value.into()),
        style: None,
    }
}
fn cross(key: &'static str, x: f32, y: f32, style: &'static str) -> Command {
    Command {
        op: "Crosshair",
        key,
        args: vec![x, y, 5.0, 2.0],
        text: None,
        style: Some(style),
    }
}
fn grouped(value: f64, precision: usize) -> String {
    let formatted = format!("{value:.precision$}");
    let (integer, decimal) = formatted
        .split_once('.')
        .map(|(i, d)| (i, Some(d)))
        .unwrap_or((&formatted, None));
    let mut result = String::new();
    for (index, c) in integer.chars().enumerate() {
        if index > 0 && (integer.len() - index).is_multiple_of(3) && c != '-' {
            result.push(',');
        }
        result.push(c);
    }
    if let Some(decimal) = decimal {
        result.push('.');
        result.push_str(decimal);
    }
    result
}
fn project(radar: &RadarSnapshot, r: f32, fs: f32, measure: impl Fn(&str) -> f32) -> Vec<Command> {
    let mut out = Vec::new();
    let (cx, cy, inner, middle) = (r, r, r * 0.76, r * 0.88);
    out.push(shape("FillCircle", "radarbg", &[cx, cy, inner]));
    for radius in [inner, middle, r] {
        out.push(shape("Circle", "radar", &[cx, cy, radius, 2.0]));
    }
    out.push(shape(
        "Line",
        "radar",
        &[cx, cy - inner, cx, cy - inner + fs, 2.0],
    ));
    for radius in [inner, middle, r] {
        out.push(shape(
            "Line",
            "radar",
            &[cx, cy + radius, cx, cy + radius + (fs / 2.0).floor(), 1.0],
        ));
    }
    for (radius, label) in [(inner, "1KM"), (middle, "2KM"), (r, "10KM")] {
        out.push(text("radarwhite", cx + 2.0, cy + radius, label));
    }
    let caption_y = cy + r + fs * 3.0;
    out.push(shape(
        "FillRectangle",
        "radarbg",
        &[
            cx - r - 2.0,
            cy + r + fs * 2.0 - 2.0,
            cx + r + 2.0,
            caption_y + 4.0,
        ],
    ));
    out.push(text(
        "radarwhite",
        cx - r,
        cy + r + fs * 2.0,
        format!(
            "POSITION: ({:.4},{:.4}) @ {:.0}°",
            radar.latitude,
            radar.longitude,
            (radar.heading + 360.0) % 360.0
        ),
    ));
    if !radar.genuses.is_empty() {
        let rows = radar.genuses.len().div_ceil(2);
        out.push(shape(
            "FillRectangle",
            "radarbg",
            &[
                cx - r - 2.0,
                caption_y + 4.0,
                cx + r + 2.0,
                caption_y + fs * (rows + 1) as f32 + (fs / 2.0).floor(),
            ],
        ));
    }
    if let Some(genus) = radar.genuses.get(&radar.context_id) {
        let mut value = format!("CURRENT SAMPLE: {genus}");
        if let Some(range) = radar.colony_ranges.get(&radar.context_id) {
            value.push_str(&format!(" [{}m]", *range as i64));
        }
        value.push_str(match radar.sample_count {
            0 => "  ○○○",
            1 => "  ●○○",
            2 => "  ●●○",
            _ => "",
        });
        if radar.sample_distance_m > 0.0 {
            value.push_str(&if radar.sample_distance_m > 1000.0 {
                format!(" ({}KM)", grouped(radar.sample_distance_m / 1000.0, 2))
            } else {
                format!(" ({}M)", grouped(radar.sample_distance_m, 0))
            });
        }
        out.push(text("radarcyan", cx - r, caption_y, value));
    }
    let mut genuses = radar.genuses.iter().collect::<Vec<_>>();
    genuses.sort_by(|a, b| a.1.cmp(b.1));
    for (index, (id, name)) in genuses.into_iter().enumerate() {
        let label = if radar.seen_genuses.contains(id) {
            format!("✔ {name}")
        } else {
            name.clone()
        };
        let key = if *id == radar.context_id {
            "radarcyan"
        } else if radar.seen_genuses.contains(id) {
            "radargrey"
        } else {
            "radarwhite"
        };
        out.push(text(
            key,
            if index % 2 == 0 {
                cx - r
            } else {
                cx + r - measure(&label)
            },
            caption_y + (index / 2 + 1) as f32 * fs,
            label,
        ));
    }
    // Source draws all selected-colony collection areas before point glyphs.
    if radar.context_id >= 0 {
        for point in radar.points.iter().filter(|p| {
            p.visible
                && p.kind == "Biological"
                && p.context_id == radar.context_id
                && p.radius_m > 0.0
                && p.distance_m < 10000.0
        }) {
            let radius = radar_radius_fraction(point.distance_m, point.radius_m) * r;
            out.push(shape(
                "FillEllipse",
                "radarbgdark",
                &[cx + point.x * r, cy + point.y * r, radius, radius],
            ));
        }
    }
    for point in radar.points.iter().filter(|p| p.visible) {
        let (x, y) = (cx + point.x * r, cy + point.y * r);
        match point.kind.as_str() {
            "Vehicle" => {
                out.push(shape(
                    "RectangleEdges",
                    "radarship",
                    &[x - 5.0, y - 5.0, x + 5.0, y + 5.0, 4.0],
                ));
                if point.distance_m > 100.0 {
                    out.push(text(
                        "radarwhite",
                        x + 9.0,
                        y - fs / 2.0,
                        if point.context_id == 1 { "SRV" } else { "SHIP" },
                    ));
                }
            }
            "Waypoint" => {
                out.push(cross("radar2", x, y, "Plus"));
                out.push(text("radarwhite", x + 7.0, y - fs / 2.0, &point.label));
            }
            "Biological" => {
                out.push(cross(
                    if point.context_id == radar.context_id {
                        "radar1"
                    } else {
                        "radar0"
                    },
                    x,
                    y,
                    "Cross",
                ));
                out.push(text("radarwhite", x + 7.0, y - fs / 2.0, &point.label));
            }
            _ => {}
        }
    }
    let ready = radar.context_id < 0
        || radar.sample_distance_m <= 0.0
        || radar.minimum_distance_m <= 0.0
        || radar.sample_distance_m >= radar.minimum_distance_m;
    out.push(cross(
        if ready { "radarcyan" } else { "radargrey" },
        cx,
        cy,
        "Gap",
    ));
    out
}
fn color(key: &str) -> Color32 {
    match key {
        "radarbg" => Color32::from_rgba_unmultiplied(31, 31, 31, 95),
        "radarbgdark" => Color32::from_rgba_unmultiplied(95, 95, 111, 95),
        "radar" => Color32::from_gray(127),
        "radargrey" => Color32::from_gray(191),
        "radar0" => Color32::from_rgb(207, 63, 63),
        "radar1" => Color32::from_rgb(63, 207, 63),
        "radar2" => Color32::from_rgb(239, 95, 95),
        "radarship" => Color32::from_rgb(175, 175, 15),
        "radarcyan" => Color32::CYAN,
        _ => Color32::WHITE,
    }
}
pub fn show_layout(ui: &mut egui::Ui, radar: &RadarSnapshot, layout: Layout) {
    let font_size = layout.font_size;
    let font = super::font(ui.ctx(), font_size);
    let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
    let origin = rect.min + layout.origin;
    let r = layout.diameter / 2.0;
    let ops = project(radar, r, font_size, |s| {
        ui.painter()
            .layout_no_wrap(s.to_owned(), font.clone(), Color32::WHITE)
            .size()
            .x
    });
    let painter = ui.painter();
    for command in ops {
        let args = &command.args;
        let c = color(command.key);
        let pos = |x: f32, y: f32| origin + Vec2::new(x, y);
        match command.op {
            "FillCircle" => {
                painter.circle_filled(pos(args[0], args[1]), args[2], c);
            }
            "Circle" => {
                painter.circle_stroke(pos(args[0], args[1]), args[2], Stroke::new(args[3], c));
            }
            "Line" => {
                painter.line_segment(
                    [pos(args[0], args[1]), pos(args[2], args[3])],
                    Stroke::new(args[4], c),
                );
            }
            "FillRectangle" => {
                painter.rect_filled(
                    egui::Rect::from_min_max(pos(args[0], args[1]), pos(args[2], args[3])),
                    0.0,
                    c,
                );
            }
            "FillEllipse" => {
                painter.circle_filled(pos(args[0], args[1]), args[2], c);
            }
            "RectangleEdges" => {
                let (l, t, r, b) = (args[0], args[1], args[2], args[3]);
                let size = (((r - l + b - t) / 2.0) * 0.2).trunc();
                for (x, y, sx, sy) in [
                    (l, t, 1.0, 1.0),
                    (r, t, -1.0, 1.0),
                    (l, b, 1.0, -1.0),
                    (r, b, -1.0, -1.0),
                ] {
                    painter
                        .line_segment([pos(x, y), pos(x + sx * size, y)], Stroke::new(args[4], c));
                    painter
                        .line_segment([pos(x, y), pos(x, y + sy * size)], Stroke::new(args[4], c));
                }
            }
            "Crosshair" => {
                let (x, y, s, w) = (args[0], args[1], args[2], args[3]);
                let lines = match command.style.unwrap_or("") {
                    "Cross" => vec![(x - s, y - s, x + s, y + s), (x + s, y - s, x - s, y + s)],
                    "Plus" => vec![(x - s, y, x + s, y), (x, y - s, x, y + s)],
                    _ => vec![
                        (x - s - w, y, x - w, y),
                        (x + s + w, y, x + w, y),
                        (x, y - s - w, x, y - w),
                        (x, y + s + w, x, y + w),
                    ],
                };
                for (x, y, x2, y2) in lines {
                    painter.line_segment([pos(x, y), pos(x2, y2)], Stroke::new(w, c));
                }
            }
            "Text" => {
                let value = command.text.unwrap_or_default();
                if let Some((label, circles)) = value
                    .split_once("  ")
                    .filter(|(_, s)| s.starts_with('●') || s.starts_with('○'))
                {
                    let anchor = pos(args[0], args[1]);
                    let galley = painter.layout_no_wrap(label.to_owned(), font.clone(), c);
                    let end = anchor.x + galley.size().x + font_size;
                    painter.galley(anchor, galley, c);
                    let mut bytes = 0;
                    let mut count = 0;
                    for (index, mark) in circles
                        .chars()
                        .take_while(|c| *c == '●' || *c == '○')
                        .enumerate()
                    {
                        bytes += mark.len_utf8();
                        count += 1;
                        let center = Pos2::new(
                            end + index as f32 * font_size * 0.6,
                            anchor.y + font_size * 0.55,
                        );
                        let radius = font_size * 0.21;
                        if mark == '●' {
                            painter.circle_filled(center, radius, c);
                        } else {
                            painter.circle_stroke(center, radius, Stroke::new(1.0_f32, c));
                        }
                    }
                    painter.text(
                        Pos2::new(end + count as f32 * font_size * 0.6, anchor.y),
                        Align2::LEFT_TOP,
                        &circles[bytes..],
                        font.clone(),
                        c,
                    );
                } else {
                    painter.text(
                        pos(args[0], args[1]),
                        Align2::LEFT_TOP,
                        value,
                        font.clone(),
                        c,
                    );
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use observatory_plugins::stellar_overlay::{radar_visible, RadarPoint};
    use serde_json::Value;
    fn equal(actual: &Value, expected: &Value, path: &str) {
        match (actual, expected) {
            (Value::Number(a), Value::Number(b)) => assert!(
                (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 0.0001,
                "{path}: {a} != {b}"
            ),
            (Value::Array(a), Value::Array(b)) => {
                assert_eq!(a.len(), b.len(), "{path}");
                for (i, (a, b)) in a.iter().zip(b).enumerate() {
                    equal(a, b, &format!("{path}[{i}]"));
                }
            }
            (Value::Object(a), Value::Object(b)) => {
                assert_eq!(a.len(), b.len(), "{path}");
                for (k, v) in b {
                    equal(&a[k], v, &format!("{path}.{k}"));
                }
            }
            _ => assert_eq!(actual, expected, "{path}"),
        }
    }
    #[test]
    fn source_integer_percentage_anchoring_and_high_font_caption_bounds() {
        let (position, diameter, font_size) =
            geometry(Vec2::new(1920.0, 1080.0), &serde_json::json!({}));
        assert_eq!(position, Vec2::new(34.0, 400.0));
        assert_eq!(diameter, 216.0);
        assert_eq!(font_size, 9.0);
        let ctx = egui::Context::default();
        let mut rendered = None;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            let radar = RadarSnapshot {
                context_id: 7,
                genuses: [(7, "Stratum Tectonicas".into())].into(),
                colony_ranges: [(7, 500.0)].into(),
                sample_distance_m: 12345.6,
                ..Default::default()
            };
            rendered = Some(layout(
                ctx,
                &radar,
                Vec2::new(1280.0, 720.0),
                &serde_json::json!({"SizePct":20,"FontScaling":200}),
            ));
        });
        let rendered = rendered.unwrap();
        assert_eq!(rendered.diameter, 144.0);
        assert_eq!(rendered.font_size, 12.0);
        assert!(
            rendered.size.x > rendered.diameter,
            "source captions extend past the circle at high font scaling"
        );
        assert!(rendered.size.y > rendered.diameter + rendered.font_size * 4.0);
    }
    #[test]
    fn complete_radar_draw_matches_284_copied_original_method_cases() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../../fixtures/stellar-radar-draw-oracle.json"
        ))
        .unwrap();
        for case in oracle["cases"].as_array().unwrap() {
            let mut radar = RadarSnapshot {
                visible: true,
                context_id: 7,
                minimum_distance_m: 500.0,
                latitude: 12.345678,
                longitude: -45.678912,
                heading: 321.6,
                genuses: [
                    (7, "Stratum".into()),
                    (3, "Aleoida".into()),
                    (9, "Tussock".into()),
                ]
                .into(),
                colony_ranges: [(7, 500.0), (3, 150.0), (9, 200.0)].into(),
                seen_genuses: [3].into(),
                ..Default::default()
            };
            if case["kind"] == "visibility" {
                let m = case["mask"].as_u64().unwrap();
                let visible = radar_visible(
                    m & 2 != 0,
                    m & 4 != 0,
                    case["altitude"].as_f64().unwrap(),
                    m & 1 != 0,
                    m & 8 != 0,
                    m & 16 != 0,
                );
                assert_eq!(visible, case["result"].as_bool().unwrap(), "{case}");
                if !visible {
                    assert!(case["ops"].as_array().unwrap().is_empty());
                    continue;
                }
            } else {
                radar.sample_count = case["count"].as_u64().unwrap_or(1);
                radar.sample_distance_m = case["distance"].as_f64().unwrap();
                if case["kind"] == "points" {
                    radar.context_id = case["context"].as_i64().unwrap();
                    radar.points = vec![
                        RadarPoint {
                            kind: "Vehicle".into(),
                            context_id: 1,
                            latitude: 0.0,
                            longitude: 0.0,
                            radius_m: 0.0,
                            label: "ignored vehicle".into(),
                            measure: false,
                            distance_m: radar.sample_distance_m,
                            x: 0.2,
                            y: 0.3,
                            visible: true,
                        },
                        RadarPoint {
                            kind: "Biological".into(),
                            context_id: 7,
                            latitude: 0.0,
                            longitude: 0.0,
                            radius_m: 500.0,
                            label: "Stratum".into(),
                            measure: false,
                            distance_m: radar.sample_distance_m,
                            x: -0.1,
                            y: -0.4,
                            visible: true,
                        },
                        RadarPoint {
                            kind: "Waypoint".into(),
                            context_id: 0,
                            latitude: 0.0,
                            longitude: 0.0,
                            radius_m: 0.0,
                            label: "WP1".into(),
                            measure: false,
                            distance_m: radar.sample_distance_m,
                            x: 0.6,
                            y: -0.2,
                            visible: true,
                        },
                    ];
                }
            }
            let actual = serde_json::to_value(project(&radar, 100.0, 12.0, |s| {
                s.encode_utf16().count() as f32 * 6.0
            }))
            .unwrap();
            equal(&actual, &case["ops"], &format!("case {}", case["id"]));
        }
        assert_eq!(oracle["cases"].as_array().unwrap().len(), 284);
        assert_eq!(grouped(12345.6, 2), "12,345.60");
    }
}
