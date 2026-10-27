//! Original NotificationQueue.AddNotificationItem geometry and NotificationItem.Draw.
//! Layout is compared with the copied source method using a recording text measurer.
use eframe::egui::{self, Color32, Pos2, Vec2};
use observatory_plugins::stellar_overlay::OverlayNotification;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Line {
    pub text: String,
    pub font_size: f32,
    pub is_bold: bool,
    pub offset_y: i32,
    pub alignment: u8,
}
#[derive(Clone, Debug)]
pub struct Layout {
    pub size: Vec2,
    pub position: Pos2,
    pub lines: Vec<Line>,
    content_size: Vec2,
    origin: Vec2,
}
fn project(
    text: (&str, &str),
    normal: f32,
    max_width: f32,
    screen_width: i32,
    padding: Vec2,
    alignment: u8,
    measure: impl Fn(&str, f32, bool) -> Vec2,
) -> (Vec2, Vec<Line>) {
    let (title, detail) = text;
    let title = title.trim_end_matches([' ', '\r', '\n']);
    let detail = detail.trim_end_matches([' ', '\r', '\n']);
    let minimum = normal * 0.75;
    let mut title_size = normal + 2.0;
    let mut title_extent = measure(title, title_size, true);
    while title_size >= minimum && title_extent.x > max_width {
        title_size -= 1.0;
        title_extent = measure(title, title_size, true);
    }
    let mut body = detail.to_owned();
    let mut body_size = normal;
    let initial_width = measure(&body, body_size, false).x;
    if initial_width > max_width && !body.contains("\r\n") {
        let words = body.split(' ').map(str::to_owned).collect::<Vec<_>>();
        body.clear();
        let mut index = 0;
        while measure(&body, body_size, false).x < initial_width / 2.0 && index < words.len() {
            if !body.is_empty() {
                body.push(' ');
            }
            body.push_str(&words[index]);
            index += 1;
        }
        if index < words.len() {
            body.push_str("\r\n");
            body.push_str(&words[index..].join(" "));
        }
    }
    while body_size >= minimum && measure(&body, body_size, false).x > max_width {
        body_size -= 1.0;
    }
    let body_extent = measure(&body, body_size, false);
    let size = Vec2::new(
        (title_extent.x.max(body_extent.x) + padding.x * 2.0).max((screen_width / 12) as f32),
        padding.y * 2.0 + title_extent.y + body_extent.y + 5.0,
    );
    (
        size,
        vec![
            Line {
                text: title.to_owned(),
                font_size: title_size,
                is_bold: true,
                offset_y: 0,
                alignment,
            },
            Line {
                text: body,
                font_size: body_size,
                is_bold: false,
                offset_y: title_extent.y as i32,
                alignment,
            },
        ],
    )
}
fn number(settings: &Value, key: &str, default: f32) -> f32 {
    settings[key].as_f64().unwrap_or(default as f64) as f32
}
fn font(ctx: &egui::Context, size: f32, bold: bool) -> egui::FontId {
    if bold {
        crate::font_registry::font(ctx, "StellarOverlayBold", size)
    } else {
        super::font(ctx, size)
    }
}
fn measure(ctx: &egui::Context, text: &str, size: f32, bold: bool) -> Vec2 {
    if text.is_empty() {
        return Vec2::ZERO;
    }
    let selected = font(ctx, size, bold);
    ctx.fonts(|f| {
        f.layout_no_wrap(text.replace("\r\n", "\n"), selected, Color32::WHITE)
            .size()
    })
}
fn reference(settings: &Value, monitor: Vec2, font_height: f32, slot: usize) -> Vec2 {
    let count = number(settings, "MaxNotifications", 3.0);
    let spacing = number(settings, "VerticalSpacing", 15.0);
    let border = number(settings, "BorderWidth", 3.0);
    let fixed_height = font_height * 3.0 + number(settings, "YPadding", 5.0) * 2.0;
    let full = count * (fixed_height + spacing);
    let pct = number(settings, "VerticalPositionPercent", 97.0).clamp(0.0, 100.0);
    let y = if pct > 50.0 {
        monitor.y / 100.0 * pct + full
            - full / 100.0 * pct
            - border
            - (fixed_height + spacing) * slot as f32
    } else {
        monitor.y / 100.0 * pct - full / 100.0 * pct
            + fixed_height
            + border
            + (fixed_height + spacing) * slot as f32
    };
    Vec2::new(
        if settings["FromRight"].as_bool().unwrap_or(true) {
            monitor.x - number(settings, "EdgeIndentation", 20.0)
        } else {
            number(settings, "EdgeIndentation", 20.0)
        },
        y.trunc(),
    )
}
pub fn layout(
    ctx: &egui::Context,
    item: &OverlayNotification,
    settings: &Value,
    monitor: Vec2,
    slot: usize,
) -> Layout {
    let normal = (monitor.y * 16.0 / 1440.0 * number(settings, "FontScaling", 100.0) / 100.0)
        .trunc()
        .max(4.0);
    let alignment = match settings["TextAlignment"].as_str().unwrap_or("Left") {
        "Center" => 1,
        "Right" => 2,
        _ => 0,
    };
    let padding = Vec2::new(
        number(settings, "XPadding", 10.0),
        number(settings, "YPadding", 5.0),
    );
    let (size, lines) = project(
        (&item.title, &item.detail),
        normal,
        (monitor.x / 100.0 * number(settings, "MaxWidth", 25.0)).trunc(),
        monitor.x as i32,
        padding,
        alignment,
        |t, s, b| measure(ctx, t, s, b),
    );
    let position = if item.instant && item.x != -1.0 && item.y != -1.0 {
        Pos2::new(
            ((monitor.x as i32 / 100) as f32 * item.x).trunc() - (size.x / 100.0 * item.x).trunc(),
            ((monitor.y as i32 / 100) as f32 * item.y).trunc() - (size.y / 100.0 * item.y).trunc(),
        )
    } else if item.instant {
        Pos2::new(100.0, 100.0)
    } else {
        let slot = item.queue_slot.unwrap_or(slot);
        let reference = reference(
            settings,
            monitor,
            measure(ctx, "ABCDEFGHIJKLMNOPQRSTUVWXYZ", normal + 2.0, true).y,
            slot,
        );
        Pos2::new(
            reference.x
                - if settings["FromRight"].as_bool().unwrap_or(true) {
                    size.x
                } else {
                    0.0
                },
            reference.y - size.y,
        )
    };
    let inset = number(settings, "BorderWidth", 3.0).max(0.0) / 2.0;
    Layout {
        size: size + Vec2::splat(inset * 2.0),
        position: position - Vec2::splat(inset),
        lines,
        content_size: size,
        origin: Vec2::splat(inset),
    }
}
#[derive(Debug, Serialize)]
struct Command {
    op: &'static str,
    key: &'static str,
    args: Vec<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}
struct DrawStyle {
    padding: Vec2,
    closed: bool,
    corner: f32,
    border: f32,
}
fn commands(
    size: Vec2,
    origin: Pos2,
    lines: &[Line],
    style: DrawStyle,
    measure: impl Fn(&str, f32, bool) -> Vec2,
) -> Vec<Command> {
    let x = origin.x;
    let y = origin.y;
    let mut out = Vec::new();
    let mut shape = |op, key, args: Vec<f32>| {
        out.push(Command {
            op,
            key,
            args,
            text: None,
        })
    };
    shape(
        "Fill",
        "notification_background",
        vec![
            x,
            y,
            x + size.x,
            y + size.y,
            if style.closed { style.corner } else { 0.0 },
        ],
    );
    if style.border > 0.0 {
        if style.closed {
            shape(
                "Rectangle",
                "notification_border",
                vec![x, y, x + size.x, y + size.y, style.corner, style.border],
            );
        } else {
            shape(
                "Line",
                "notification_border",
                vec![x, y, x + size.x, y, style.border],
            );
            shape(
                "Line",
                "notification_border",
                vec![x, y + size.y, x + size.x, y + size.y, style.border],
            );
        }
    }
    for line in lines {
        let mut offset = y + style.padding.y + line.offset_y as f32;
        for text in line.text.split('\n') {
            let text = text.trim_end_matches(['\r', ' ', '\t']);
            let extent = measure(text, line.font_size, line.is_bold);
            let tx = x + match line.alignment {
                1 => (size.x - extent.x) / 2.0,
                2 => size.x - style.padding.x - extent.x,
                _ => style.padding.x,
            };
            out.push(Command {
                op: "Text",
                key: if line.is_bold {
                    "notification_title"
                } else {
                    "notification_text"
                },
                args: vec![tx, offset, line.font_size],
                text: Some(text.into()),
            });
            offset += extent.y;
        }
    }
    out
}
pub fn render(
    ui: &mut egui::Ui,
    item: &OverlayNotification,
    settings: &Value,
    layout: &Layout,
    alpha: f32,
) {
    let mut colors = settings.clone();
    if let Some(values) = item.colors.as_object() {
        for (k, v) in values {
            colors[k] = v.clone();
        }
    }
    let ops = commands(
        layout.content_size,
        ui.max_rect().min + layout.origin,
        &layout.lines,
        DrawStyle {
            padding: Vec2::new(
                number(settings, "XPadding", 10.0),
                number(settings, "YPadding", 5.0),
            ),
            closed: settings["NotificationShape"] == "Closed",
            corner: number(settings, "CornerRounding", 20.0),
            border: number(settings, "BorderWidth", 3.0),
        },
        |t, s, b| measure(ui.ctx(), t, s, b),
    );
    for op in ops {
        let color = match op.key {
            "notification_background" => super::color(
                &colors,
                "BackgroundColorHex",
                Color32::from_rgba_unmultiplied(0, 0, 0, 176),
            ),
            "notification_border" => {
                super::color(&colors, "BorderColorHex", Color32::from_rgb(46, 142, 255))
            }
            "notification_title" => {
                super::color(&colors, "TitleColorHex", Color32::from_rgb(255, 123, 0))
            }
            _ => super::color(&colors, "ColorHex", Color32::from_rgb(255, 99, 0)),
        }
        .gamma_multiply(alpha);
        let a = &op.args;
        match op.op {
            "Fill" => {
                ui.painter().rect_filled(
                    egui::Rect::from_min_max(Pos2::new(a[0], a[1]), Pos2::new(a[2], a[3])),
                    a[4],
                    color,
                );
            }
            "Rectangle" => {
                ui.painter().rect_stroke(
                    egui::Rect::from_min_max(Pos2::new(a[0], a[1]), Pos2::new(a[2], a[3])),
                    a[4],
                    egui::Stroke::new(a[5], color),
                    egui::StrokeKind::Middle,
                );
            }
            "Line" => {
                ui.painter().line_segment(
                    [Pos2::new(a[0], a[1]), Pos2::new(a[2], a[3])],
                    egui::Stroke::new(a[4], color),
                );
            }
            "Text" => {
                ui.painter().text(
                    Pos2::new(a[0], a[1]),
                    egui::Align2::LEFT_TOP,
                    op.text.unwrap_or_default(),
                    font(ui.ctx(), a[2], op.key == "notification_title"),
                    color,
                );
            }
            _ => {}
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn equal(actual: &Value, expected: &Value, path: &str) {
        match (actual, expected) {
            (Value::Number(a), Value::Number(b)) => assert!(
                (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 0.0001,
                "{path}: {a} !={b}"
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
    fn source_draw_matches_all_108_copied_original_method_cases() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../../fixtures/stellar-notification-layout-oracle.json"
        ))
        .unwrap();
        for (index, case) in oracle["draws"].as_array().unwrap().iter().enumerate() {
            let lines: Vec<Line> = serde_json::from_value(case["lines"].clone()).unwrap();
            let size = Vec2::new(
                case["dimensions"]["X"].as_f64().unwrap() as f32,
                case["dimensions"]["Y"].as_f64().unwrap() as f32,
            );
            let ops = commands(
                size,
                Pos2::new(12.0, 34.0),
                &lines,
                DrawStyle {
                    padding: Vec2::new(10.0, 5.0),
                    closed: case["shape"] == 0,
                    corner: 20.0,
                    border: case["border"].as_f64().unwrap() as f32,
                },
                |t, s, _| {
                    if t.is_empty() {
                        Vec2::ZERO
                    } else {
                        Vec2::new(t.len() as f32 * s * 0.5, s)
                    }
                },
            );
            equal(
                &serde_json::to_value(ops).unwrap(),
                &case["ops"],
                &format!("Draw case {index}"),
            );
        }
    }
    #[test]
    fn source_reference_slots_match_all_108_original_initialisefontheight_cases() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../../fixtures/stellar-notification-layout-oracle.json"
        ))
        .unwrap();
        for case in oracle["references"].as_array().unwrap() {
            let settings = serde_json::json!({"MaxNotifications":case["count"],"VerticalPositionPercent":case["pct"],"FromRight":case["fromRight"]});
            for (index, expected) in case["positions"].as_array().unwrap().iter().enumerate() {
                let position = reference(
                    &settings,
                    Vec2::new(1920.0, case["height"].as_f64().unwrap() as f32),
                    14.0,
                    index,
                );
                assert_eq!(position.x as f64, expected["X"].as_f64().unwrap(), "{case}");
                assert_eq!(position.y as f64, expected["Y"].as_f64().unwrap(), "{case}");
            }
        }
    }
    #[test]
    fn source_layout_matches_all_216_copied_method_cases() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../../fixtures/stellar-notification-layout-oracle.json"
        ))
        .unwrap();
        for case in oracle["cases"].as_array().unwrap() {
            let measure = |text: &str, size: f32, _| {
                if text.is_empty() {
                    Vec2::ZERO
                } else {
                    let lines = text
                        .replace("\r\n", "\n")
                        .split('\n')
                        .map(str::len)
                        .collect::<Vec<_>>();
                    Vec2::new(
                        *lines.iter().max().unwrap() as f32 * size * 0.5,
                        lines.len() as f32 * size,
                    )
                }
            };
            let (size, lines) = project(
                (
                    case["title"].as_str().unwrap(),
                    case["detail"].as_str().unwrap(),
                ),
                12.0,
                case["maxWidth"].as_f64().unwrap() as f32,
                1920,
                Vec2::new(10.0, 5.0),
                0,
                measure,
            );
            let lines = serde_json::to_value(lines).unwrap();
            for (actual, expected) in lines
                .as_array()
                .unwrap()
                .iter()
                .zip(case["output"]["Lines"].as_array().unwrap())
            {
                for key in ["Text", "IsBold", "OffsetY", "Alignment"] {
                    assert_eq!(actual[key], expected[key], "{key}: {case}");
                }
                assert_eq!(
                    actual["FontSize"].as_f64(),
                    expected["FontSize"].as_f64(),
                    "{case}"
                );
            }
            assert_eq!(
                size.x as f64,
                case["output"]["Dimensions"]["X"].as_f64().unwrap(),
                "{case}"
            );
            assert_eq!(
                size.y as f64,
                case["output"]["Dimensions"]["Y"].as_f64().unwrap(),
                "{case}"
            );
        }
    }
}
