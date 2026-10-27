//! AlertBuilder.GenerateAlert port. Geometry and drawing order are checked
//! against the original C# method with inert drawing calls, not native snapshots.
use super::*;

#[derive(Clone, Debug)]
enum Op {
    Fill {
        y: f32,
        height: f32,
        color: &'static str,
    },
    Text {
        text: String,
        x: f32,
        y: f32,
        bold: bool,
        color: &'static str,
    },
    Symbol {
        key: &'static str,
        x: f32,
        y: f32,
    },
}
struct Plan {
    ops: Vec<Op>,
    size: Vec2,
    border: f32,
}
fn symbols(row: &Value, variant: bool) -> Vec<&'static str> {
    let fields = if variant {
        [
            "PossibleCodexEntry",
            "PossibleCodexEntryNeverSeen",
            "PersonalCodexEntry",
            "CCMark",
        ]
    } else {
        [
            "IsGalacticCodexEntry",
            "IsGalacticCodexEntryNeverSeen",
            "IsPersonalCodexEntry",
            "IsMarked",
        ]
    };
    let mut keys = Vec::new();
    if flag(row, fields[0]) {
        keys.push("PossibleCodexEntry");
    } else if flag(row, fields[1]) {
        keys.push("PossibleCodexEntryNeverSeen");
    }
    if flag(row, fields[2]) {
        keys.push("PersonalCodexEntry");
    }
    if flag(row, fields[3]) {
        keys.push("Mark");
    }
    keys
}
fn plan(
    panel: &Value,
    height: f32,
    height_b: f32,
    space: f32,
    measure: impl Fn(&str, bool) -> f32,
    symbol_width: impl Fn(&str) -> f32,
) -> Plan {
    let alert = &panel["VisualAlert"];
    let lines = array(alert, "Lines");
    let border = panel["BorderWidth"]
        .as_f64()
        .unwrap_or(2.0)
        .clamp(0.0, 32.0) as f32;
    let row_h = height + 6.0;
    let head_h = height_b + 6.0;
    let mut ops = vec![
        Op::Fill {
            y: border,
            height: head_h + 1.0,
            color: "HeaderBackground",
        },
        Op::Text {
            text: string(alert, "BodyName").to_owned(),
            x: border + 3.0,
            y: border + 3.0,
            bold: true,
            color: "HeaderText",
        },
    ];
    // The source starts the maximum with the body text's width, then compares
    // the absolute ends of species rows. Keep this small asymmetry verbatim.
    let mut end_name = measure(string(alert, "BodyName"), true);
    let mut genus = "Unicorn";
    let mut stripe = 2;
    for (i, row) in lines.iter().enumerate() {
        if genus != string(row, "Genus") {
            genus = string(row, "Genus");
            stripe = 3 - stripe;
        }
        let y = border + head_h + row_h * i as f32;
        ops.push(Op::Fill {
            y,
            height: row_h + 1.0,
            color: if stripe == 1 {
                "Row1Background"
            } else {
                "Row2Background"
            },
        });
        let mut x = border + 3.0;
        for key in symbols(row, false) {
            ops.push(Op::Symbol { key, x, y: y + 3.0 });
            x += symbol_width(key);
        }
        let label = if string(row, "Type") == "Horizons" {
            string(row, "Species").to_owned()
        } else {
            format!("{} {}", string(row, "Genus"), string(row, "Species"))
        };
        ops.push(Op::Text {
            text: label.clone(),
            x,
            y: y + 3.0,
            bold: false,
            color: "DefaultText",
        });
        end_name = end_name.max(x + measure(&label, false));
    }
    let value_x = end_name + space * 5.0;
    let count = number(alert, "BioCount");
    ops.push(Op::Text {
        text: format!(" [{count} Bio{}]", if count > 1 { "s" } else { "" }),
        x: value_x,
        y: border,
        bold: true,
        color: "HeaderText",
    });
    let mut end_value = value_x;
    for (i, row) in lines.iter().enumerate() {
        let text = millions(number(row, "BaseValue"));
        ops.push(Op::Text {
            text: text.clone(),
            x: value_x,
            y: border + head_h + row_h * i as f32 + 3.0,
            bold: false,
            color: "DefaultText",
        });
        end_value = end_value.max(value_x + measure(&text, false));
    }
    let variant_x = end_value + space * 5.0;
    let mut end = 0.0_f32;
    for (i, row) in lines.iter().enumerate() {
        let y = border + head_h + row_h * i as f32 + 3.0;
        let mut x = variant_x;
        for (index, variant) in array(row, "PossibleVariants")
            .iter()
            .filter(|v| v["ShowInUI"] != false)
            .enumerate()
        {
            if index > 0 {
                if string(row, "Type") == "SurfaceMaterial" {
                    x += space;
                    ops.push(Op::Text {
                        text: "or".into(),
                        x,
                        y,
                        bold: false,
                        color: "DefaultText",
                    });
                    x += measure("or", false);
                } else {
                    ops.push(Op::Text {
                        text: ", ".into(),
                        x,
                        y,
                        bold: false,
                        color: "DefaultText",
                    });
                    x += measure(", ", false);
                }
                x += space;
            }
            for key in symbols(variant, true) {
                ops.push(Op::Symbol { key, x, y });
                x += symbol_width(key);
            }
            let key = match string(variant, "Likelihood") {
                "Likely" => "Likely",
                "Unlikely" => "Unlikely",
                _ => "Maybe",
            };
            ops.push(Op::Symbol {
                key,
                x,
                y: y + if key == "Unlikely" { 5.0 } else { 0.0 },
            });
            x += symbol_width(key);
            let colour = string(variant, "Colour");
            ops.push(Op::Text {
                text: colour.to_owned(),
                x,
                y,
                bold: false,
                color: "DefaultText",
            });
            x += measure(colour, false);
        }
        end = end.max(x + space);
    }
    Plan {
        ops,
        size: Vec2::new(
            (end + border * 2.0).min(4000.0),
            head_h + row_h * lines.len() as f32 + border * 2.0,
        ),
        border,
    }
}
fn font_size(panel: &Value) -> f32 {
    panel["FontSize"].as_f64().unwrap_or(12.0).clamp(6.0, 72.0) as f32 * 96.0 / 72.0
}
fn symbol_text(theme: &Theme, key: &str) -> String {
    let symbol = &theme.data["Symbols"][key];
    symbol
        .get("TextOverride")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| decode_symbol_text(string(symbol, "Content")))
}
fn native_plan(ctx: &egui::Context, panel: &Value, theme: &Theme) -> Plan {
    let size = font_size(panel);
    crate::font_registry::prepare(ctx, "bio-alert-default", "Segoe UI Emoji");
    crate::font_registry::prepare(ctx, "bio-alert-bold", "Segoe UI Emoji:style=Bold");
    let normal = crate::font_registry::font(ctx, "bio-alert-default", size);
    let bold = crate::font_registry::font(ctx, "bio-alert-bold", size);
    let mut symbol_fonts = std::collections::BTreeMap::new();
    for key in [
        "PersonalCodexEntry",
        "Mark",
        "PossibleCodexEntry",
        "PossibleCodexEntryNeverSeen",
        "Likely",
        "Maybe",
        "Unlikely",
    ] {
        let selected = string(&theme.data["Symbols"][key], "TextOverrideFont");
        if !selected.is_empty() {
            let family = format!("bio-alert-symbol-{selected}");
            crate::font_registry::prepare(ctx, &family, selected);
            symbol_fonts.insert(key, crate::font_registry::font(ctx, &family, size));
        }
    }
    ctx.fonts(|fonts| {
        let measure = |text: &str, is_bold: bool| {
            fonts
                .layout_no_wrap(
                    text.to_owned(),
                    if is_bold {
                        bold.clone()
                    } else {
                        normal.clone()
                    },
                    Color32::WHITE,
                )
                .size()
                .x
                .ceil()
        };
        let height = fonts.row_height(&normal).ceil();
        let height_b = fonts.row_height(&bold).ceil();
        let space = (measure("A A", false) - measure("AA", false)).ceil();
        plan(panel, height, height_b, space, measure, |key| {
            fonts
                .layout_no_wrap(
                    symbol_text(theme, key),
                    symbol_fonts.get(key).unwrap_or(&normal).clone(),
                    Color32::WHITE,
                )
                .size()
                .x
                .ceil()
        })
    })
}
pub fn alert_size(ctx: &egui::Context, panel: &Value) -> Vec2 {
    native_plan(ctx, panel, &Theme::new(&panel["AlertSettings"])).size
}
pub fn render_alert(ui: &mut egui::Ui, panel: &Value) {
    let theme = Theme::new(&panel["AlertSettings"]);
    let plan = native_plan(ui.ctx(), panel, &theme);
    let (rect, _) = ui.allocate_exact_size(plan.size, Sense::hover());
    ui.painter().rect_filled(rect, 0.0, Color32::TRANSPARENT);
    let size = font_size(panel);
    for op in plan.ops {
        match op {
            Op::Fill { y, height, color } => {
                ui.painter().rect_filled(
                    Rect::from_min_size(
                        rect.min + Vec2::new(plan.border, y),
                        Vec2::new((rect.width() - plan.border).max(0.0), height),
                    ),
                    0.0,
                    theme.color(color),
                );
            }
            Op::Text {
                text,
                x,
                y,
                bold,
                color,
            } => {
                ui.painter().text(
                    rect.min + Vec2::new(x, y),
                    Align2::LEFT_TOP,
                    text,
                    crate::font_registry::font(
                        ui.ctx(),
                        if bold {
                            "bio-alert-bold"
                        } else {
                            "bio-alert-default"
                        },
                        size,
                    ),
                    theme.color(color),
                );
            }
            Op::Symbol { key, x, y } => {
                let symbol = &theme.data["Symbols"][key];
                let color = if symbol["TextOverrideColor"].is_object() {
                    rgb(&symbol["TextOverrideColor"])
                } else {
                    theme.symbol_color(key, false)
                };
                let selected = string(symbol, "TextOverrideFont");
                let family = if selected.is_empty() {
                    "bio-alert-default".to_owned()
                } else {
                    format!("bio-alert-symbol-{selected}")
                };
                ui.painter().text(
                    rect.min + Vec2::new(x, y),
                    Align2::LEFT_TOP,
                    symbol_text(&theme, key),
                    crate::font_registry::font(ui.ctx(), &family, size),
                    color,
                );
            }
        }
    }
    // GenerateAlert paints horizontal border lines. Its left/right crop stays
    // transparent; a surrounding generic card would change the source bitmap.
    for i in 0..plan.border as usize {
        for y in [i as f32, rect.height() - i as f32 - 1.0] {
            ui.painter().line_segment(
                [
                    rect.min + Vec2::new(plan.border, y),
                    rect.min + Vec2::new(rect.width(), y),
                ],
                egui::Stroke::new(1.0_f32, theme.color("TitleBackground")),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn geometry_and_every_symbol_match_original_alertbuilder_trace() {
        let cases: Value =
            serde_json::from_str(include_str!("../../fixtures/bioinsights-alert-oracle.json"))
                .unwrap();
        for case in cases.as_array().unwrap() {
            let f = number(case, "Flags");
            let panel = json!({"BorderWidth":2,"VisualAlert":{"BodyName":"Source 1 a","BioCount":4,"Lines":[
                {"Genus":"Bacterium","Species":"Alcyoneum","BaseValue":1658500,"IsPersonalCodexEntry":f&1!=0,"IsMarked":f&2!=0,"IsGalacticCodexEntry":f&4!=0,"IsGalacticCodexEntryNeverSeen":f&8!=0,"PossibleVariants":[{"Colour":"Green","PersonalCodexEntry":f&1!=0,"CCMark":f&2!=0,"PossibleCodexEntry":f&4!=0,"PossibleCodexEntryNeverSeen":f&8!=0,"Likelihood":if f&16!=0{"Likely"}else if f&32!=0{"Unlikely"}else{"Maybe"}}]},
                {"Genus":"Bacterium","Species":"Aurasus","BaseValue":1000000,"PossibleVariants":[{"Colour":"Red","Likelihood":"Likely"},{"Colour":"Blue","Likelihood":"Unlikely"},{"Colour":"Grey","Likelihood":"Maybe"}]},
                {"Genus":"Frutexa","Species":"Flabellum","BaseValue":1800000,"PossibleVariants":[{"Colour":"Emerald","Likelihood":"Maybe"}]},
                {"Genus":"Anemone","Species":"Luteolum Anemone","Type":"Horizons","BaseValue":500000,"PossibleVariants":[]}
            ]}});
            let p = plan(
                &panel,
                16.0,
                16.0,
                4.0,
                |text, _| text.len() as f32 * 8.0,
                |_| 8.0,
            );
            let actual:Vec<Value>=p.ops.iter().map(|op|match op{
                Op::Fill{y,height,..}=>json!({"op":"fill","x":2,"y":*y as i32,"w":4000,"h":*height as i32}),
                Op::Text{text,x,y,bold,color}=>json!({"op":"text","font":if *bold{"_bold"}else{"_default"},"text":text,"brush":if *color=="HeaderText"{"body"}else{"text"},"x":*x as i32,"y":*y as i32,"width":text.len()*8}),
                Op::Symbol{key,x,y}=>json!({"op":"symbol","symbol":key,"x":*x as i32,"y":*y as i32,"width":8}),
            }).collect();
            let expected: Vec<Value> = case["Trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|op| op["op"] != "line" && op["op"] != "crop")
                .cloned()
                .collect();
            assert_eq!(actual, expected, "flags {f}");
            let crop = case["Trace"].as_array().unwrap().last().unwrap();
            assert_eq!(
                p.size,
                Vec2::new(number(crop, "Width") as f32, number(crop, "Height") as f32)
            );
        }
    }
}
