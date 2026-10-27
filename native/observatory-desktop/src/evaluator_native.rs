//! Evaluator's fixed ten-column HTML table, sized from source CSS metrics.
use super::*;
use crate::bioinsights_main::css_text;
fn font(size: f32, bold: bool, family: &str) -> FontId {
    css_text::font(family, size, bold, false)
}
pub(super) fn measured(ui: &egui::Ui, value: &str, size: f32, bold: bool, family: &str) -> f32 {
    css_text::measure_advance(ui, value, &font(size, bold, family))
        .or_else(|| css_text::measure(ui, value, &font(size, bold, family)).map(|size| size.x))
        .unwrap_or_else(|| {
            ui.painter()
                .layout_no_wrap(value.into(), FontId::proportional(size), Color32::WHITE)
                .size()
                .x
        })
}
#[allow(clippy::too_many_arguments)] // These are the independent CSS paint properties.
pub(super) fn draw(
    ui: &egui::Ui,
    rect: Rect,
    value: &str,
    color: Color32,
    align: Align2,
    size: f32,
    bold: bool,
    family: &str,
    padding: f32,
) {
    let at = match align {
        Align2::CENTER_CENTER => rect.center(),
        Align2::RIGHT_CENTER => rect.right_center() - Vec2::new(padding, 0.0),
        _ => rect.left_center() + Vec2::new(padding, 0.0),
    };
    if !css_text::paint(ui, value, &font(size, bold, family), at, align, color, rect) {
        ui.painter()
            .with_clip_rect(rect)
            .text(at, align, value, FontId::proportional(size), color);
    }
}
fn span_minimum(columns: &mut [f32; 10], start: usize, span: usize, width: f32) {
    let sum: f32 = columns[start..start + span].iter().sum();
    if width > sum {
        for column in &mut columns[start..start + span] {
            *column = if sum > 0.0 {
                *column * width / sum
            } else {
                width / span as f32
            };
        }
    }
}
#[derive(Clone)]
struct WidthCache {
    revision: Option<u64>,
    rows: usize,
    summary: usize,
    count: usize,
    headers: bool,
    orange: bool,
    pixels_per_point: f32,
    columns: [f32; 10],
}
pub(super) fn widths(
    ui: &egui::Ui,
    summary: &Value,
    rows: &[Value],
    theme: &Theme,
    available: f32,
    scale: f32,
    settings: &Value,
) -> [f32; 10] {
    let headers = settings["ShowHeaders"].as_bool().unwrap_or(true);
    let expand = settings["ExpandOutput"].as_bool().unwrap_or(true);
    let revision = ui
        .ctx()
        .data(|data| data.get_temp::<u64>(Id::new("Evaluator source projection revision")));
    let id = Id::new("Evaluator source intrinsic widths");
    let cache = ui.ctx().data(|data| data.get_temp::<WidthCache>(id));
    let current = cache.filter(|cache| {
        cache.revision == revision
            && cache.rows == rows.as_ptr() as usize
            && cache.summary == summary as *const Value as usize
            && cache.count == rows.len()
            && cache.headers == headers
            && cache.pixels_per_point == ui.ctx().pixels_per_point()
            && cache.orange == theme.orange
    });
    let mut columns = if let Some(cache) = current {
        cache.columns
    } else {
        let columns = intrinsic_widths(ui, summary, rows, theme, headers);
        let pixels_per_point = ui.ctx().pixels_per_point();
        ui.ctx().data_mut(|data| {
            data.insert_temp(
                id,
                WidthCache {
                    revision,
                    rows: rows.as_ptr() as usize,
                    summary: summary as *const Value as usize,
                    count: rows.len(),
                    headers,
                    orange: theme.orange,
                    pixels_per_point,
                    columns,
                },
            )
        });
        columns
    };
    for column in &mut columns {
        *column *= scale;
    }
    let intrinsic: f32 = columns.iter().sum();
    if available > intrinsic {
        let rest: f32 = columns[1..].iter().sum();
        columns[0] = (42.0 * scale).min(available - rest).max(columns[0]);
        if expand && rest > 0.0 {
            let ratio = ((available - columns[0]) / rest).max(1.0);
            for column in &mut columns[1..] {
                *column *= ratio;
            }
        }
    }
    columns
}
fn intrinsic_widths(
    ui: &egui::Ui,
    summary: &Value,
    rows: &[Value],
    theme: &Theme,
    headers: bool,
) -> [f32; 10] {
    let mut columns = [0.0f32; 10];
    columns[0] = measured(
        ui,
        "\u{a0}\u{a0}\u{a0}\u{a0}",
        15.0,
        false,
        "Segoe UI Emoji",
    ) + 10.0;
    if headers {
        for (index, label) in [
            (1, "Body"),
            (2, "Type"),
            (3, "Attributes"),
            (4, "Distance"),
            (5, "Gravity"),
            (8, "Current"),
            (9, "Mapped"),
        ] {
            columns[index] = measured(ui, label, 16.0, true, "Segoe UI Emoji") + 12.0;
        }
    }
    for body in rows {
        columns[1] = columns[1].max(
            measured(
                ui,
                body["Body"].as_str().unwrap_or(""),
                15.0,
                false,
                "Segoe UI Emoji",
            ) + 10.0,
        );
        if body["IsStar"] != true {
            let short = type_label(body);
            let symbol = &theme.data["Symbols"][&short];
            let label = measured(
                ui,
                symbol["Content"].as_str().unwrap_or(&short),
                15.0,
                symbol["FontWeight"] == "bold",
                "Segoe UI Emoji",
            ) + symbol["PaddingLeft"].as_f64().unwrap_or(4.0) as f32
                + symbol["PaddingRight"].as_f64().unwrap_or(4.0) as f32;
            columns[2] = columns[2].max(label + 10.0);
            let attributes = attribute_symbols(body);
            let attributes_width: f32 = attributes
                .iter()
                .map(|key| symbol_size(ui, theme, key, 1.0).x)
                .sum();
            columns[3] = columns[3].max(
                attributes_width + measured(ui, "\u{a0}", 15.0, false, "Segoe UI Emoji") + 10.0,
            );
            columns[6] = columns[6].max(
                symbol_size(
                    ui,
                    theme,
                    if body["Map"] == true {
                        "WorthMapping"
                    } else {
                        "NotWorthMapping"
                    },
                    1.0,
                )
                .x + 10.0,
            );
            columns[7] = columns[7]
                .max(40.0)
                .max(if body["Order"].as_u64().unwrap_or(0) > 0 {
                    measured(
                        ui,
                        &format!("[{}]", body["Order"]),
                        15.0,
                        false,
                        "Times New Roman",
                    ) + 10.0
                } else {
                    0.0
                });
        }
        columns[4] = columns[4].max(60.0).max(
            measured(
                ui,
                &format!(
                    "{} LS",
                    formatted(body["Distance"].as_f64().unwrap_or(0.0).round_ties_even() as i64)
                ),
                15.0,
                false,
                "Segoe UI Emoji",
            ) + 10.0,
        );
        columns[5] = columns[5].max(60.0).max(
            measured(
                ui,
                &gravity_label(body["Gravity"].as_f64().unwrap_or(0.0)),
                15.0,
                false,
                "Segoe UI Emoji",
            ) + 10.0,
        );
        columns[8] = columns[8].max(
            measured(
                ui,
                &formatted(body["CurrentValue"].as_i64().unwrap_or(0)),
                15.0,
                false,
                "Segoe UI Emoji",
            ) + 10.0,
        );
        columns[9] = columns[9].max(
            measured(
                ui,
                &formatted(body["PotentialMaxValue"].as_i64().unwrap_or(0)),
                15.0,
                false,
                "Segoe UI Emoji",
            ) + 20.0,
        );
    }
    if headers {
        columns[4] = columns[4].max(measured(ui, "Progress", 16.0, true, "Segoe UI Emoji") + 12.0);
        columns[5] = columns[5].max(measured(ui, "Filter", 16.0, true, "Segoe UI Emoji") + 12.0);
        columns[9] =
            columns[9].max(measured(ui, "Potential Value", 16.0, true, "Segoe UI Emoji") + 12.0);
        span_minimum(
            &mut columns,
            6,
            2,
            measured(ui, "Visit / Map", 16.0, true, "Segoe UI Emoji") + 12.0,
        );
    }
    for body in rows {
        if body["IsStar"] == true {
            span_minimum(
                &mut columns,
                2,
                2,
                measured(
                    ui,
                    &star_type(body["Type"].as_str().unwrap_or("")),
                    15.0,
                    true,
                    "Segoe UI Emoji",
                ) + 30.0,
            );
        }
    }
    if let Some(systems) = summary["Systems"].as_array() {
        for system in systems {
            span_minimum(
                &mut columns,
                0,
                4,
                measured(
                    ui,
                    system["System"].as_str().unwrap_or(""),
                    15.0,
                    true,
                    "Verdana",
                ) + 12.0,
            );
            columns[4] = columns[4].max(
                measured(
                    ui,
                    &format!("{} / {}", system["ScannedBodies"], system["TotalBodies"]),
                    15.0,
                    true,
                    "Verdana",
                ) + 10.0,
            );
            columns[5] = columns[5].max(
                ["FilterMapped", "FilterDefault", "FilterAll"]
                    .iter()
                    .map(|key| symbol_size(ui, theme, key, 1.0).x)
                    .sum::<f32>()
                    + 10.0,
            );
            span_minimum(
                &mut columns,
                6,
                3,
                measured(
                    ui,
                    &formatted(system["SystemValue"].as_i64().unwrap_or(0)),
                    15.0,
                    true,
                    "Verdana",
                ) + 10.0,
            );
            columns[9] = columns[9].max(
                measured(
                    ui,
                    &formatted(system["PotentialValue"].as_i64().unwrap_or(0)),
                    15.0,
                    true,
                    "Verdana",
                ) + 10.0,
            );
        }
    }
    if headers {
        span_minimum(
            &mut columns,
            6,
            3,
            measured(ui, "System Value", 16.0, true, "Segoe UI Emoji") + 12.0,
        );
    }
    columns
}
fn line(ui: &mut egui::Ui, width: f32, height: f32, bg: Color32, spacing: f32) -> Rect {
    ui.add_space(spacing);
    row(ui, width, height, bg)
}
pub(super) fn show(
    ui: &mut egui::Ui,
    summary: &Value,
    rows: &[Value],
    settings: &Value,
    theme: &Theme,
    scale: f32,
    actions: &mut Vec<Value>,
) {
    let viewport = ui.ctx().viewport_id();
    let revision = ui
        .ctx()
        .data(|data| data.get_temp::<u64>(Id::new("Evaluator source projection revision")))
        .unwrap_or(rows.len() as u64);
    let scroll_id = Id::new(("Evaluator source current-system scroll", viewport));
    let scroll_current = ui.ctx().data(|data| data.get_temp::<u64>(scroll_id)) != Some(revision);
    if scroll_current {
        ui.ctx()
            .data_mut(|data| data.insert_temp(scroll_id, revision));
    }
    let headers = settings["ShowHeaders"].as_bool().unwrap_or(true);
    let available = ui.available_width();
    let columns = widths(ui, summary, rows, theme, available, scale, settings);
    let width = columns.iter().sum();
    let spacing = theme.spacing() * scale;
    let padding = 5.0 * scale;
    let title_height = (theme.row_height() + 4.0) * scale;
    let system_height = (theme.row_height() + 2.0) * scale;
    let body_height = (theme.row_height() + 4.0) * scale;
    let blank_height = 22.0 * scale;
    ui.spacing_mut().item_spacing = Vec2::ZERO;
    egui::ScrollArea::horizontal().id_salt("Evaluator source main scroll").auto_shrink([false,true]).show(ui,|ui|{
        ui.set_min_width(width);
        let title=|ui:&mut egui::Ui,label:&str,right:bool|{
            let r=line(ui,width,title_height,theme.color("HeaderBackground"),spacing);
            ui.painter().rect_stroke(r,0,egui::Stroke::new(scale,theme.color("Border")),egui::StrokeKind::Inside);
            draw(ui,r,label,theme.color("HeaderText"),if right{Align2::RIGHT_CENTER}else{Align2::CENTER_CENTER},16.0*scale,true,"Segoe UI Emoji",6.0*scale);
        };
        if headers{title(ui,&format!("CMDR {}",summary["Commander"].as_str().unwrap_or("")),false);}
        if settings["ShowTotal"].as_bool().unwrap_or(false){title(ui,&format!("Unsold Data (Max): {}",formatted(summary["UnsoldTotal"].as_i64().unwrap_or(0))),true);if !headers{line(ui,width,blank_height,theme.color("DefaultBackground"),spacing);}}
        if headers{
            let r=line(ui,width,title_height,theme.color("TitleBackground"),spacing);
            for(index,span,label)in[(0,4,"System"),(4,1,"Progress"),(5,1,"Filter"),(6,3,"System Value"),(9,1,"Potential Value")]{
                let rect=cell(r,&columns,index,span);ui.painter().rect_stroke(rect,0,egui::Stroke::new(scale,theme.color("Border")),egui::StrokeKind::Inside);
                let tip=match index{8=>"Estimated current value of body",9=>"Estimated maximum value of body if mapped",_=>""};
                if !tip.is_empty(){ui.interact(rect,Id::new(("Evaluator source body header title",index)),Sense::hover()).on_hover_text(tip);}

                let tip=match index{4=>"Scanned bodies (FSS) / Total bodies",5=>"Show mapped only | Default | Show all",6=>"Estimated current total value of system",9=>"Estimated maximum value of system if all bodies worth mapping are mapped",_=>""};
                if !tip.is_empty(){ui.interact(rect,Id::new(("Evaluator source header title",index)),Sense::hover()).on_hover_text(tip);}

                draw(ui,rect,label,theme.color("TitleText"),if index>=6{Align2::RIGHT_CENTER}else if index==0{Align2::LEFT_CENTER}else{Align2::CENTER_CENTER},16.0*scale,true,"Segoe UI Emoji",6.0*scale);
            }
            let r=line(ui,width,title_height,theme.color("TitleBackground"),spacing);ui.painter().rect_filled(cell(r,&columns,0,1),0,theme.color("DefaultBackground"));
            for(index,span,label)in[(1,1,"Body"),(2,1,"Type"),(3,1,"Attributes"),(4,1,"Distance"),(5,1,"Gravity"),(6,2,"Visit / Map"),(8,1,"Current"),(9,1,"Mapped")]{
                let rect=cell(r,&columns,index,span);ui.painter().rect_stroke(rect,0,egui::Stroke::new(scale,theme.color("Border")),egui::StrokeKind::Inside);
                draw(ui,rect,label,theme.color("TitleText"),if index>=8{Align2::RIGHT_CENTER}else{Align2::CENTER_CENTER},16.0*scale,true,"Segoe UI Emoji",6.0*scale);
            }
        }
        let mut by_visit=std::collections::BTreeMap::<(u64,u64),Vec<&Value>>::new();
        for body in rows{by_visit.entry((body["ID64"].as_u64().unwrap_or(0),body["VisitID"].as_u64().unwrap_or(0))).or_default().push(body);}
        let mut scrolled_current=false;
        for system in summary["Systems"].as_array().map(Vec::as_slice).unwrap_or(&[]){
            let id=system["ID64"].as_u64().unwrap_or(0);let visit=system["VisitID"].as_u64().unwrap_or(0);let current=system["Current"]==true;
            let r=line(ui,width,system_height,theme.color("DefaultBackground"),spacing);
            if current&&scroll_current&&!scrolled_current {scrolled_current=true;ui.scroll_to_rect(r,Some(egui::Align::Min));}
            if ui.is_rect_visible(r){
                ui.painter().rect_filled(r,(10.0*scale).round()as u8,theme.color("HeaderBackground"));
                draw(ui,cell(r,&columns,0,4),system["System"].as_str().unwrap_or(""),theme.color("HeaderText"),Align2::LEFT_CENTER,15.0*scale,true,"Verdana",padding);
                draw(ui,cell(r,&columns,4,1),&format!("{} / {}",system["ScannedBodies"],system["TotalBodies"]),theme.symbol_color(if system["ScannedBodies"]==system["TotalBodies"]{"BodiesAllScanned"}else{"BodiesNotAllScanned"},false),Align2::CENTER_CENTER,15.0*scale,true,"Verdana",padding);
                let filter=cell(r,&columns,5,1);let filter_width=["FilterMapped","FilterDefault","FilterAll"].iter().map(|key|symbol_size(ui, theme,key,scale).x).sum::<f32>();let mut x=filter.center().x-filter_width/2.0;
                if current{for(name,key)in[("MappingOnly","FilterMapped"),("Default","FilterDefault"),("All","FilterAll")]{
                    let selected=summary["Filter"].as_str().unwrap_or("Default")==name;let key=format!("{key}{}",if selected{"Selected"}else{""});let size=symbol_size(ui, theme,&key,scale);let rect=Rect::from_min_size(egui::pos2(x,r.center().y-size.y/2.0-2.0*scale),size);icon(ui,theme,&key,rect);x+=size.x;
                    if ui.interact(rect,Id::new(("Evaluator source filter",id,visit,name)),Sense::click()).on_hover_text(theme.data["Symbols"][&key]["Description"].as_str().unwrap_or("")).clicked(){actions.push(command("SetFilter",json!({"Filter":name})));}
                }}
                draw(ui,cell(r,&columns,6,3),&formatted(system["SystemValue"].as_i64().unwrap_or(0)),theme.color("HeaderText"),Align2::RIGHT_CENTER,15.0*scale,true,"Verdana",padding);
                draw(ui,cell(r,&columns,9,1),&formatted(system["PotentialValue"].as_i64().unwrap_or(0)),theme.color("HeaderText"),Align2::RIGHT_CENTER,15.0*scale,true,"Verdana",padding);
            }
            for(index,body)in by_visit.get(&(id,visit)).into_iter().flatten().enumerate(){
                let next=body["IsNextToVisit"]==true;
                let background=theme.color(if next&&theme.data["Settings"]["UseHighlightRow"]==true{"HighlightBackground"}else if index%2==0{"Row1Background"}else{"Row2Background"});
                let r=line(ui,width,body_height,theme.color("DefaultBackground"),spacing);if !ui.is_rect_visible(r){continue;}
                let color=theme.color("DefaultText");
                // The first marker cell has no row color in the source.
                let colored=Rect::from_min_max(cell(r,&columns,1,1).min,r.max);ui.painter().rect_filled(colored,0,background);
                if next{let c=cell(r,&columns,0,1);let size=symbol_size(ui, theme,"NextBody",scale);icon(ui,theme,"NextBody",Rect::from_min_size(c.left_center()+Vec2::new(padding,-size.y/2.0-2.0*scale),size));}
                draw(ui,cell(r,&columns,1,1),body["Body"].as_str().unwrap_or(""),color,Align2::LEFT_CENTER,15.0*scale,false,"Segoe UI Emoji",padding);
                let star=body["IsStar"]==true;let short=type_label(body);let symbol=if star{"STAR"}else{short.as_str()};let type_cell=cell(r,&columns,2,if star{2}else{1});let label=if star{short.as_str()}else{theme.data["Symbols"][symbol]["Content"].as_str().unwrap_or(&short)};
                if theme.data["Symbols"][symbol].is_object(){let bold=theme.data["Symbols"][symbol]["FontWeight"]=="bold";let left=theme.data["Symbols"][symbol]["PaddingLeft"].as_f64().unwrap_or(4.0)as f32*scale;let right=theme.data["Symbols"][symbol]["PaddingRight"].as_f64().unwrap_or(4.0)as f32*scale;let w=measured(ui,label,15.0*scale,bold,"Segoe UI Emoji")+left+right;let rect=Rect::from_min_size(type_cell.left_center()+Vec2::new(padding,-9.0*scale),Vec2::new(w,18.0*scale));ui.painter().rect_filled(rect,5.0*scale,theme.symbol_color(symbol,true));draw(ui,rect,label,theme.symbol_color(symbol,false),Align2::LEFT_CENTER,15.0*scale,bold,"Segoe UI Emoji",left);}
                if !star{let attrs=cell(r,&columns,3,1);let mut x=attrs.left()+padding;let mut separator=false;for key in attribute_symbols(body){if !separator&&(key.starts_with("BioInsights")||key=="PluginWorthVisiting"){x+=measured(ui,"\u{a0}",15.0*scale,false,"Segoe UI Emoji");separator=true;}let size=symbol_size(ui, theme,key,scale);icon(ui,theme,key,Rect::from_min_size(egui::pos2(x,r.center().y-size.y/2.0-2.0*scale),size));x+=size.x;}}
                draw(ui,cell(r,&columns,4,1),&format!("{} LS",formatted(body["Distance"].as_f64().unwrap_or(0.0).round_ties_even()as i64)),color,Align2::RIGHT_CENTER,15.0*scale,false,"Segoe UI Emoji",padding);
                let gravity=body["Gravity"].as_f64().unwrap_or(0.0);if gravity>0.0{let key=if gravity>=2.0{"Alert"}else if gravity>=1.0{"Warn1"}else if gravity>=0.5{"Warn0"}else{"OK"};draw(ui,cell(r,&columns,5,1),&gravity_label(gravity),theme.symbol_color(key,false),Align2::RIGHT_CENTER,15.0*scale,theme.data["Symbols"][key]["FontWeight"]=="bold","Segoe UI Emoji",padding);}
                if !star&&(body["Done"]==true||current){let key=if body["Done"]==true{"Mapped"}else if body["Map"]==true{"WorthMapping"}else{"NotWorthMapping"};let c=cell(r,&columns,6,1);let size=symbol_size(ui, theme,key,scale);let rect=Rect::from_min_size(c.left_center()+Vec2::new(padding,-size.y/2.0-2.0*scale),size);icon(ui,theme,key,rect);if current&&body["Done"]!=true&&ui.interact(rect,Id::new(("Evaluator source map",id,visit,body["BodyID"].as_u64().unwrap_or(0))),Sense::click()).on_hover_text(theme.data["Symbols"][key]["Description"].as_str().unwrap_or("")).clicked(){actions.push(command("SetWorthMapping",json!({"ID64":id,"BodyID":body["BodyID"],"IsWorthMapping":body["Map"]!=true})));}}
                if !star&&current&&body["Done"]!=true&&body["Order"].as_u64().unwrap_or(0)>0{draw(ui,cell(r,&columns,7,1),&format!("[{}]",body["Order"]),theme.color("WorthMappingText"),Align2::LEFT_CENTER,15.0*scale,false,"Times New Roman",padding);}
                draw(ui,cell(r,&columns,8,1),&formatted(body["CurrentValue"].as_i64().unwrap_or(0)),color,Align2::RIGHT_CENTER,15.0*scale,false,"Segoe UI Emoji",padding);
                draw(ui,cell(r,&columns,9,1),&formatted(body["PotentialMaxValue"].as_i64().unwrap_or(0)),color,Align2::RIGHT_CENTER,15.0*scale,false,"Segoe UI Emoji",padding);
            }
            if headers{let rect=line(ui,width,title_height,theme.color("HeaderBackground"),spacing);ui.painter().rect_stroke(rect,0,egui::Stroke::new(scale,theme.color("Border")),egui::StrokeKind::Inside);}
            line(ui,width,blank_height,theme.color("DefaultBackground"),spacing);
        }
        line(ui,width,blank_height,theme.color("DefaultBackground"),spacing);ui.add_space(spacing+19.0*scale);
    });
}
fn type_label(body: &Value) -> String {
    if body["IsStar"] == true {
        return star_type(body["Type"].as_str().unwrap_or(""));
    }
    match body["Type"].as_str().unwrap_or("") {
        "Earthlike body" => "ELW",
        "Water world" => "WW",
        "Ammonia world" => "AW",
        "High metal content body" => "HMC",
        "Metal rich body" => "MR",
        "Icy body" => "ICY",
        "Rocky body" => "ROCKY",
        "Rocky ice body" => "RIB",
        "Sudarsky class I gas giant" => "GG1",
        "Sudarsky class II gas giant" => "GG2",
        "Sudarsky class III gas giant" => "GG3",
        "Sudarsky class IV gas giant" => "GG4",
        "Sudarsky class V gas giant" => "GG5",
        "Gas giant with water based life" => "GGWL",
        "Gas giant with ammonia based life" => "GGAL",
        "Helium rich gas giant" => "HRGG",
        "Helium gas giant" => "HGG",
        "Water giant" => "WG",
        other => other,
    }
    .into()
}

/// Original `.alert` normal-flow text; the floated close anchor does not add height.
pub(super) fn show_alert(ui: &mut egui::Ui, alert: &Value, scale: f32, actions: &mut Vec<Value>) {
    let background = match alert["Class"].as_str().unwrap_or("") {
        "alert-danger" => Color32::from_rgb(245, 99, 88),
        "alert-success" => Color32::from_rgb(46, 184, 133),
        "alert-warning" => Color32::from_rgb(255, 169, 43),
        _ => Color32::from_rgb(70, 167, 245),
    };
    let description = alert["Description"]
        .as_str()
        .unwrap_or("")
        .replace("<strong>", "\u{1}")
        .replace("</strong>", "\u{2}")
        .replace("<b>", "\u{1}")
        .replace("</b>", "\u{2}")
        .replace("&amp;", "&")
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">");
    let mut words = Vec::<(String, bool, bool)>::new();
    let mut word = String::new();
    let mut bold = false;
    let mut gap = false;
    for ch in description.chars().chain(std::iter::once(' ')) {
        if ch.is_whitespace() || ch == '\u{1}' || ch == '\u{2}' {
            if !word.is_empty() {
                words.push((std::mem::take(&mut word), bold, gap));
                gap = false;
            }
            if ch.is_whitespace() {
                gap = true;
            } else {
                bold = ch == '\u{1}';
            }
        } else {
            word.push(ch);
        }
    }
    let line_height =
        css_text::line_height(ui, &font(13.0 * scale, false, "Verdana")).unwrap_or(15.0 * scale);
    let close_width = measured(ui, "×", 22.0 * scale, true, "Verdana");
    let width = ui.available_width();
    let mut runs = Vec::new();
    let (mut x, mut line) = (0.0, 0_usize);
    for (word, bold, gap) in words {
        let word_width = measured(ui, &word, 13.0 * scale, bold, "Verdana");
        let mut space = if gap && x > 0.0 {
            measured(ui, " ", 13.0 * scale, bold, "Verdana")
        } else {
            0.0
        };
        let reserve = if line as f32 * line_height < 20.0 * scale {
            close_width + 15.0 * scale
        } else {
            0.0
        };
        if gap && x > 0.0 && x + space + word_width > width - 20.0 * scale - reserve {
            line += 1;
            x = 0.0;
            space = 0.0;
        }
        x += space;
        runs.push((word, bold, x, line));
        x += word_width;
    }
    let height = (line + 1) as f32 * line_height + 20.0 * scale;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    ui.painter().rect_filled(rect, 7.0 * scale, background);
    for (word, bold, x, line) in runs {
        let r = Rect::from_min_size(
            rect.min + Vec2::new(10.0 * scale + x, 10.0 * scale + line as f32 * line_height),
            Vec2::new((rect.width() - 20.0 * scale - x).max(0.0), line_height),
        );
        draw(
            ui,
            r,
            &word,
            Color32::WHITE,
            Align2::LEFT_CENTER,
            13.0 * scale,
            bold,
            "Verdana",
            0.0,
        );
    }
    let close = Rect::from_min_size(
        egui::pos2(
            rect.right() - 10.0 * scale - close_width,
            rect.top() + 7.0 * scale,
        ),
        Vec2::new(close_width, 20.0 * scale),
    );
    let response = ui.interact(
        close,
        ui.make_persistent_id(("Evaluator source alert close", alert["ID"].to_string())),
        Sense::click(),
    );
    crate::font_registry::prepare_exact(
        ui.ctx(),
        "Evaluator source alert close",
        "Verdana:style=Bold",
    );
    ui.painter().text(
        close.center(),
        Align2::CENTER_CENTER,
        "×",
        crate::font_registry::font(ui.ctx(), "Evaluator source alert close", 22.0 * scale),
        if response.hovered() {
            Color32::BLACK
        } else {
            Color32::WHITE
        },
    );
    if response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
    {
        actions.push(command("DismissUIAlert", json!({"ID":alert["ID"]})));
    }
    ui.add_space(
        css_text::line_height(ui, &font(16.0 * scale, false, "Verdana")).unwrap_or(19.0 * scale),
    );
}
const LEGEND: [&str; 18] = [
    "WorthMapping",
    "Mapped",
    "Terraformable",
    "IsHighValue",
    "IsLandable",
    "HasBiologicals",
    "BioInsightsPersonalCodex",
    "BioInsightsGalacticCodex",
    "BioInsightsMarked",
    "BioInsightsHighValue",
    "HasVolcanism",
    "WasDiscovered",
    "WasMapped",
    "WasFootfalled",
    "PluginWorthVisiting",
    "NextBody",
    "FilterDefault",
    "FilterDefaultSelected",
];
fn legend_description<'a>(theme: &'a Theme, key: &str) -> &'a str {
    match key {
        "FilterDefault" => "Filter bodies (mapped / default / all)",
        "FilterDefaultSelected" => "Currently selected body filter",
        _ => theme.data["Symbols"][key]["Description"]
            .as_str()
            .unwrap_or(""),
    }
}
fn about_columns(ui: &egui::Ui, theme: &Theme, scale: f32) -> [f32; 2] {
    [
        (measured(ui, "Symbol", 16.0, true, "Segoe UI Emoji") + 12.0) * scale,
        LEGEND
            .iter()
            .map(|key| {
                measured(
                    ui,
                    legend_description(theme, key),
                    15.0,
                    false,
                    "Segoe UI Emoji",
                ) + 10.0
            })
            .fold(
                measured(ui, "Meaning", 16.0, true, "Segoe UI Emoji") + 12.0,
                f32::max,
            )
            * scale,
    ]
}
fn part(rect: Rect, widths: &[f32], index: usize) -> Rect {
    Rect::from_min_size(
        rect.min + Vec2::new(widths[..index].iter().sum(), 0.0),
        Vec2::new(widths[index], rect.height()),
    )
}
fn link(
    ui: &mut egui::Ui,
    rect: Rect,
    text: &str,
    url: &str,
    theme: &Theme,
    scale: f32,
    button: bool,
) {
    let response = ui.interact(
        rect,
        Id::new(("Evaluator source hyperlink", url)),
        Sense::click(),
    );
    let hovered = response.hovered();
    if button || hovered {
        ui.painter().rect_filled(
            rect,
            if button { 5.0 * scale } else { 0.0 },
            theme.color(if button {
                if hovered {
                    "ButtonFaceHighlight"
                } else {
                    "ButtonFace"
                }
            } else {
                "LinkText"
            }),
        );
    }
    draw(
        ui,
        rect,
        text,
        theme.color(if button {
            "Button"
        } else if hovered {
            "DefaultBackground"
        } else {
            "LinkText"
        }),
        Align2::CENTER_CENTER,
        14.0 * scale,
        true,
        "Verdana",
        0.0,
    );
    if response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
    {
        ui.ctx().open_url(egui::OpenUrl::new_tab(url));
    }
}
fn rich_line(
    ui: &mut egui::Ui,
    theme: &Theme,
    scale: f32,
    segments: &[(&str, bool, Option<&str>)],
) {
    let width = segments
        .iter()
        .map(|(text, bold, _)| measured(ui, text, 14.0 * scale, *bold, "Verdana"))
        .sum();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 19.0 * scale), Sense::hover());
    let mut x = rect.left();
    for (text, bold, url) in segments {
        let width = measured(ui, text, 14.0 * scale, *bold, "Verdana");
        let r = Rect::from_min_size(egui::pos2(x, rect.top()), Vec2::new(width, 19.0 * scale));
        if let Some(url) = url {
            link(ui, r, text, url, theme, scale, false);
        } else {
            draw(
                ui,
                r,
                text,
                theme.color("DefaultText"),
                Align2::LEFT_CENTER,
                14.0 * scale,
                *bold,
                "Verdana",
                0.0,
            );
        }
        x += width;
    }
}
pub(super) fn show_about(ui: &mut egui::Ui, theme: &Theme, scale: f32) {
    {
        let columns = about_columns(ui, theme, scale);
        let width = columns.iter().sum();
        let spacing = theme.spacing() * scale;
        let (intro, _) = ui.allocate_exact_size(Vec2::new(width, 38.0 * scale), Sense::hover());
        let lead = "Enjoying Evaluator? ";
        let x = intro.left() + measured(ui, lead, 14.0 * scale, false, "Verdana");
        draw(
            ui,
            Rect::from_min_size(intro.min, Vec2::new(x - intro.left(), 19.0 * scale)),
            lead,
            theme.color("DefaultText"),
            Align2::LEFT_CENTER,
            14.0 * scale,
            false,
            "Verdana",
            0.0,
        );
        let link_width =
            measured(ui, "Buy me a coffee", 14.0 * scale, true, "Verdana") + 8.0 * scale;
        link(
            ui,
            Rect::from_min_size(
                egui::pos2(x, intro.top() - 2.0 * scale),
                Vec2::new(link_width, 24.0 * scale),
            ),
            "Buy me a coffee",
            "https://ko-fi.com/explorationtools",
            theme,
            scale,
            true,
        );
        let head = (theme.row_height() + 4.0) * scale;
        let r = line(ui, width, head, theme.color("TitleBackground"), spacing);
        ui.painter().rect_stroke(
            r,
            0,
            egui::Stroke::new(scale, theme.color("Border")),
            egui::StrokeKind::Inside,
        );
        draw(
            ui,
            r,
            "Symbol Legend",
            theme.color("TitleText"),
            Align2::CENTER_CENTER,
            16.0 * scale,
            true,
            "Segoe UI Emoji",
            6.0 * scale,
        );
        let r = line(ui, width, head, theme.color("TitleBackground"), spacing);
        for (index, text) in ["Symbol", "Meaning"].iter().enumerate() {
            let c = part(r, &columns, index);
            ui.painter().rect_stroke(
                c,
                0,
                egui::Stroke::new(scale, theme.color("Border")),
                egui::StrokeKind::Inside,
            );
            draw(
                ui,
                c,
                text,
                theme.color("TitleText"),
                Align2::CENTER_CENTER,
                16.0 * scale,
                true,
                "Segoe UI Emoji",
                6.0 * scale,
            );
        }
        for key in LEGEND {
            let text = theme.data["Symbols"][key]["ContentType"] == "Text";
            let r = line(
                ui,
                width,
                if key == "WorthMapping" || text {
                    22.0
                } else {
                    24.0
                } * scale,
                theme.color("DefaultBackground"),
                spacing,
            );
            if ui.is_rect_visible(r) {
                let c = part(r, &columns, 0);
                if key == "WorthMapping" {
                    let box_rect = Rect::from_min_size(
                        egui::pos2(c.center().x - 12.5 * scale, r.top() + 2.0 * scale),
                        Vec2::splat(18.0 * scale),
                    );
                    let id = Id::new("Evaluator source About checkbox demo");
                    let mut checked = ui
                        .ctx()
                        .data(|data| data.get_temp::<bool>(id))
                        .unwrap_or(true);
                    if ui
                        .interact(
                            Rect::from_min_size(box_rect.min, Vec2::new(50.0, 15.0) * scale),
                            id,
                            Sense::click(),
                        )
                        .clicked()
                    {
                        checked = !checked;
                        ui.ctx().data_mut(|data| data.insert_temp(id, checked));
                    }
                    ui.painter().rect_filled(
                        box_rect,
                        2.0 * scale,
                        theme.color(if checked {
                            "WorthMappingText"
                        } else {
                            "DefaultBackground"
                        }),
                    );
                    ui.painter().rect_stroke(
                        box_rect,
                        2.0 * scale,
                        egui::Stroke::new(2.0 * scale, theme.color("DefaultText")),
                        egui::StrokeKind::Inside,
                    );
                } else {
                    let size = symbol_size(ui, theme, key, scale);
                    icon(
                        ui,
                        theme,
                        key,
                        Rect::from_center_size(
                            c.center() - Vec2::new(0.0, if text { 0.0 } else { -2.0 * scale }),
                            size,
                        ),
                    );
                }
                draw(
                    ui,
                    part(r, &columns, 1),
                    legend_description(theme, key),
                    theme.color("DefaultText"),
                    Align2::LEFT_CENTER,
                    15.0 * scale,
                    false,
                    "Segoe UI Emoji",
                    5.0 * scale,
                );
            }
        }
        ui.add_space(spacing + 19.0 * scale);
        let (r, _) = ui.allocate_exact_size(Vec2::new(width, 29.0 * scale), Sense::hover());
        draw(
            ui,
            r,
            "Thanks",
            theme.color("DefaultText"),
            Align2::LEFT_CENTER,
            24.0 * scale,
            false,
            "Verdana",
            0.0,
        );
        rich_line(
            ui,
            theme,
            scale,
            &[
                ("CMDR Vithigar", true, None),
                (" for creating ", false, None),
                ("Observatory", true, Some("https://observatory.xjph.net/")),
                (" and listening to my feedback and requests", false, None),
            ],
        );
        rich_line(
            ui,
            theme,
            scale,
            &[
                (
                    "Evaluator uses some .SVGs from Iain Collins's ",
                    false,
                    None,
                ),
                (
                    "Icarus Terminal",
                    true,
                    Some("https://github.com/iaincollins/icarus"),
                ),
            ],
        );
        ui.add_space(38.0 * scale);
        let (r, _) = ui.allocate_exact_size(Vec2::new(width, 29.0 * scale), Sense::hover());
        draw(
            ui,
            r,
            "Need assistance?",
            theme.color("DefaultText"),
            Align2::LEFT_CENTER,
            24.0 * scale,
            false,
            "Verdana",
            0.0,
        );
        rich_line(
            ui,
            theme,
            scale,
            &[
                ("Try the ", false, None),
                (
                    "Observatory Discord",
                    true,
                    Some("https://discord.gg/4J55fNdU7B"),
                ),
            ],
        );
        ui.add_space(38.0 * scale);
    }
}
fn unsold_columns(ui: &egui::Ui, rows: &[Value], total: i64, scale: f32) -> [f32; 3] {
    let mut widths = [
        measured(ui, "Total Unsold Data (max)", 16.0, true, "Segoe UI Emoji") + 10.0,
        measured(ui, "System", 16.0, true, "Segoe UI Emoji") + 10.0,
        (measured(ui, "Value", 16.0, true, "Segoe UI Emoji") + 10.0)
            .max(measured(ui, &formatted(total), 16.0, true, "Segoe UI Emoji") + 10.0),
    ];
    for line in rows.iter().filter(|line| line[2].as_i64().unwrap_or(0) > 0) {
        for (index, text) in [
            source_date(line[1].as_str().unwrap_or("")),
            line[0].as_str().unwrap_or("").to_owned(),
            formatted(line[2].as_i64().unwrap_or(0)),
        ]
        .iter()
        .enumerate()
        {
            widths[index] =
                widths[index].max(measured(ui, text, 15.0, false, "Segoe UI Emoji") + 10.0);
        }
    }
    widths.map(|width| width * scale)
}
pub(super) fn show_unsold(ui: &mut egui::Ui, summary: &Value, theme: &Theme, scale: f32) {
    {
        if summary["HecateAvailable"] == false {
            let text = "This requires the Hecate - Data Vault plugin";
            let (r, _) = ui.allocate_exact_size(
                Vec2::new(
                    measured(ui, text, 16.0 * scale, false, "Verdana"),
                    19.0 * scale,
                ),
                Sense::hover(),
            );
            draw(
                ui,
                r,
                text,
                theme.color("DefaultText"),
                Align2::LEFT_CENTER,
                16.0 * scale,
                false,
                "Verdana",
                0.0,
            );
            return;
        }
        let rows = summary["UnsoldData"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let total: i64 = rows.iter().map(|line| line[2].as_i64().unwrap_or(0)).sum();
        let widths = unsold_columns(ui, rows, total, scale);
        let width = widths.iter().sum();
        let spacing = theme.spacing() * scale;
        let head = (theme.row_height() + 2.0) * scale;
        line(ui, width, head, theme.color("TitleBackground"), spacing);
        let r = line(ui, width, head, theme.color("TitleBackground"), spacing);
        draw(
            ui,
            part(r, &widths, 0),
            "Total Unsold Data (max)",
            theme.color("TitleText"),
            Align2::LEFT_CENTER,
            16.0 * scale,
            true,
            "Segoe UI Emoji",
            5.0 * scale,
        );
        draw(
            ui,
            part(r, &widths, 2),
            &formatted(total),
            theme.color("TitleText"),
            Align2::RIGHT_CENTER,
            16.0 * scale,
            true,
            "Segoe UI Emoji",
            5.0 * scale,
        );
        line(ui, width, head, theme.color("TitleBackground"), spacing);
        for _ in 0..3 {
            line(
                ui,
                width,
                22.0 * scale,
                theme.color("DefaultBackground"),
                spacing,
            );
        }
        let r = line(ui, width, head, theme.color("TitleBackground"), spacing);
        draw(
            ui,
            r,
            "Breakdown",
            theme.color("TitleText"),
            Align2::CENTER_CENTER,
            16.0 * scale,
            true,
            "Segoe UI Emoji",
            5.0 * scale,
        );
        let r = line(ui, width, head, theme.color("TitleBackground"), spacing);
        for (index, text) in ["Date", "System", "Value"].iter().enumerate() {
            draw(
                ui,
                part(r, &widths, index),
                text,
                theme.color("TitleText"),
                Align2::CENTER_CENTER,
                16.0 * scale,
                true,
                "Segoe UI Emoji",
                5.0 * scale,
            );
        }
        for (stripe, line_data) in rows
            .iter()
            .filter(|line| line[2].as_i64().unwrap_or(0) > 0)
            .enumerate()
        {
            let r = line(
                ui,
                width,
                (theme.row_height() + 4.0) * scale,
                theme.color(if stripe % 2 == 0 {
                    "Row2Background"
                } else {
                    "Row1Background"
                }),
                spacing,
            );
            if !ui.is_rect_visible(r) {
                continue;
            }
            for (index, text) in [
                source_date(line_data[1].as_str().unwrap_or("")),
                line_data[0].as_str().unwrap_or("").to_owned(),
                formatted(line_data[2].as_i64().unwrap_or(0)),
            ]
            .iter()
            .enumerate()
            {
                draw(
                    ui,
                    part(r, &widths, index),
                    text,
                    theme.color("DefaultText"),
                    if index == 2 {
                        Align2::RIGHT_CENTER
                    } else {
                        Align2::LEFT_CENTER
                    },
                    15.0 * scale,
                    false,
                    "Segoe UI Emoji",
                    5.0 * scale,
                );
            }
        }
        ui.add_space(spacing);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "explicit release-mode CPU profile of 4000 source rows"]
    fn profile_evaluator_native_paint() {
        let fixture: Value =
            serde_json::from_str(include_str!("../../fixtures/evaluator-css-layout.json")).unwrap();
        let projection = &fixture["projection"];
        let template = projection["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["IsStar"] != true)
            .unwrap();
        let rows = (0..4000)
            .map(|index| {
                let mut row = template.clone();
                row["BodyID"] = json!(index + 1);
                row["Body"] = json!(format!("A {}", index + 1));
                row
            })
            .collect::<Vec<_>>();
        let mut receipt = Vec::new();
        for size in [Vec2::new(1200.0, 800.0), Vec2::new(3440.0, 1440.0)] {
            let context = egui::Context::default();
            let mut cold = 0.0;
            let mut samples = Vec::new();
            let mut paint_batches = 0;
            context.data_mut(|data| {
                data.insert_temp(Id::new("Evaluator source projection revision"), 1_u64)
            });
            for frame in 0..160 {
                let started = std::time::Instant::now();
                let output = context.run(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, size)),
                        time: Some(frame as f64 / 60.0),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default()
                            .frame(egui::Frame::NONE)
                            .show(ctx, |ui| {
                                super::super::show(
                                    ui,
                                    &projection["summary"],
                                    &rows,
                                    &projection["settings"],
                                );
                            });
                    },
                );
                paint_batches = context
                    .tessellate(output.shapes, output.pixels_per_point)
                    .len();
                let elapsed = started.elapsed().as_secs_f64() * 1000.0;
                if frame == 0 {
                    cold = elapsed;
                } else if frame >= 20 {
                    samples.push(elapsed);
                }
            }
            samples.sort_by(f64::total_cmp);
            let mean = samples.iter().sum::<f64>() / samples.len() as f64;
            let p95 = samples[samples.len() * 95 / 100];
            receipt.push(json!({"width":size.x,"height":size.y,"source_rows":rows.len(),"first_frame_ms":cold,"mean_ms":mean,"p95_ms":p95,"max_ms":samples.last(),"samples":samples.len(),"paint_batches":paint_batches}));
            assert!(
                mean < 16.0 && p95 < 33.0,
                "warm native Evaluator exceeds frame budget: mean={mean} p95={p95}"
            );
        }
        let encoded = serde_json::to_string_pretty(&receipt).unwrap();
        println!("{encoded}");
        if let Ok(path) = std::env::var("OBSERVATORY_EVALUATOR_PROFILE_OUTPUT") {
            std::fs::write(path, encoded).unwrap();
        }
    }
    #[test]
    fn source_alert_float_linebox_padding_and_following_header_match_webkit() {
        let source: Value = serde_json::from_str(include_str!(
            "../../fixtures/evaluator-alert-css-layout.json"
        ))
        .unwrap();
        let context = egui::Context::default();
        let output=context.run(egui::RawInput{screen_rect:Some(Rect::from_min_size(egui::Pos2::ZERO,Vec2::new(1170.0,900.0))),..Default::default()},|ctx|{egui::CentralPanel::default().frame(egui::Frame::NONE).show(ctx,|ui|{super::super::show(ui,&json!({"UIAlerts":[{"ID":3,"Class":"alert-success","Description":"Read All completed"}]}),&[],&json!({}));});});
        let rects = output
            .shapes
            .iter()
            .filter_map(|shape| {
                if let egui::epaint::Shape::Rect(rect) = &shape.shape {
                    Some(rect)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let alert = rects
            .iter()
            .find(|rect| rect.fill == Color32::from_rgb(46, 184, 133))
            .unwrap();
        for (actual, key) in [
            (alert.rect.left(), "x"),
            (alert.rect.top(), "y"),
            (alert.rect.width(), "width"),
            (alert.rect.height(), "height"),
        ] {
            assert!(
                (actual - source["alerts"][0]["rect"][key].as_f64().unwrap() as f32).abs() < 0.1,
                "alert{key}:native={actual} source={}",
                source["alerts"][0]["rect"][key]
            );
        }
        let theme = Theme::new(&json!({}));
        let header = rects
            .iter()
            .find(|rect| rect.fill == theme.color("HeaderBackground") && rect.rect.width() > 500.0)
            .unwrap();
        assert!(
            (header.rect.top() - source["first_header"]["y"].as_f64().unwrap() as f32).abs() < 0.1
        );
    }
    #[test]
    fn all_source_header_and_unsold_banner_settings_match_webkit_columns() {
        let captures: Value = serde_json::from_str(include_str!(
            "../../fixtures/evaluator-header-css-layouts.json"
        ))
        .unwrap();
        for capture in captures.as_array().unwrap() {
            let context = egui::Context::default();
            let projection = &capture["projection"];
            let mut actual = [0.0; 10];
            let output = context.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(1170.0, 900.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::NONE)
                        .show(ctx, |ui| {
                            actual = widths(
                                ui,
                                &projection["summary"],
                                projection["rows"].as_array().unwrap(),
                                &Theme::new(&projection["settings"]),
                                1152.0,
                                1.0,
                                &projection["settings"],
                            );
                            super::super::show(
                                ui,
                                &projection["summary"],
                                projection["rows"].as_array().unwrap(),
                                &projection["settings"],
                            );
                        });
                },
            );
            let theme = Theme::new(&projection["settings"]);
            let system = output
                .shapes
                .iter()
                .filter_map(|shape| {
                    if let egui::epaint::Shape::Rect(rect) = &shape.shape {
                        Some(rect)
                    } else {
                        None
                    }
                })
                .find(|rect| {
                    rect.fill == theme.color("HeaderBackground")
                        && (rect.rect.height() - 26.0).abs() < 0.1
                        && rect.rect.width() > 500.0
                })
                .unwrap();
            let source_system = capture["rows"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["text"][0] == "Native Smoke")
                .unwrap();
            assert!(
                (system.rect.top() - source_system["y"].as_f64().unwrap() as f32).abs() < 0.1,
                "{} system row native={} source={}",
                capture["name"],
                system.rect.top(),
                source_system["y"]
            );
            println!(
                "{} native={actual:?} source={}",
                capture["name"], capture["columns"]
            );
            for (index, (actual, expected)) in actual
                .into_iter()
                .zip(capture["columns"].as_array().unwrap())
                .enumerate()
            {
                assert!(
                    (actual - expected.as_f64().unwrap() as f32).abs() < 0.03125,
                    "{} col{index}:native={actual} source={expected}",
                    capture["name"]
                );
            }
        }
    }
    #[test]
    fn about_and_unsold_geometry_match_original_webkit_css() {
        let source: Value = serde_json::from_str(include_str!(
            "../../fixtures/evaluator-ancillary-css-layout.json"
        ))
        .unwrap();
        for page in ["About", "Unsold"] {
            let context = egui::Context::default();
            context.data_mut(|data| {
                data.insert_persisted(Id::new("evaluator_original_page"), page.to_owned())
            });
            let mut widths = Vec::new();
            let output = context.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(1170.0, 900.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::NONE)
                        .show(ctx, |ui| {
                            let theme = Theme::new(&json!({}));
                            widths = if page == "About" {
                                about_columns(ui, &theme, 1.0).to_vec()
                            } else {
                                unsold_columns(ui, &[], 0, 1.0).to_vec()
                            };
                            super::super::show(
                                ui,
                                &json!({"HecateAvailable":true}),
                                &[],
                                &json!({}),
                            );
                        });
                },
            );
            let table = &source[page.to_lowercase()]["tables"][0];
            let cells = table["rows"][1]["cells"].as_array().unwrap();
            println!(
                "{page} native widths={widths:?} source widths={:?}",
                cells
                    .iter()
                    .map(|cell| cell["rect"]["width"].as_f64().unwrap())
                    .collect::<Vec<_>>()
            );
            for (width, cell) in widths.iter().zip(cells) {
                assert!(
                    (*width - cell["rect"]["width"].as_f64().unwrap() as f32).abs() < 0.03125,
                    "{page} native={width} source={}",
                    cell["rect"]["width"]
                );
            }
            let theme = Theme::new(&json!({}));
            let first = output
                .shapes
                .iter()
                .filter_map(|shape| {
                    if let egui::epaint::Shape::Rect(rect) = &shape.shape {
                        Some(rect)
                    } else {
                        None
                    }
                })
                .find(|rect| {
                    rect.fill == theme.color("TitleBackground") && rect.rect.width() > 300.0
                })
                .unwrap();
            let expected = &table["rows"][0]["rect"];
            assert!((first.rect.left() - expected["x"].as_f64().unwrap() as f32).abs() < 0.1);
            assert!(
                (first.rect.top() - expected["y"].as_f64().unwrap() as f32).abs() < 0.1,
                "{page} {:?} source={expected}",
                first.rect
            );
            assert!(
                (first.rect.height() - expected["height"].as_f64().unwrap() as f32).abs() < 0.1
            );
        }
    }
    #[test]
    fn native_table_geometry_matches_actual_original_css_webkit_capture() {
        let data: Value =
            serde_json::from_str(include_str!("../../fixtures/evaluator-css-layout.json")).unwrap();
        let projection = &data["projection"];
        let context = egui::Context::default();
        let mut actual = [0.0; 10];
        let _ = context.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(1170.0, 800.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show(ctx, |ui| {
                        actual = widths(
                            ui,
                            &projection["summary"],
                            projection["rows"].as_array().unwrap(),
                            &Theme::new(&projection["settings"]),
                            1152.0,
                            1.0,
                            &json!({}),
                        );
                    });
            },
        );
        let expected = data["dom"]["tables"][0]["rows"][5]["cells"]
            .as_array()
            .unwrap()
            .iter()
            .map(|cell| cell["rect"]["width"].as_f64().unwrap() as f32)
            .collect::<Vec<_>>();
        println!("native={actual:?} source={expected:?}");
        for (index, (actual, expected)) in actual.into_iter().zip(expected).enumerate() {
            assert!(
                (actual - expected).abs() < 0.03125,
                "column{index}: native={actual} source={expected}"
            );
        }
    }
}
