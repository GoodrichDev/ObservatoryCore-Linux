//! Original AboutForm fields, links and geometry.
use eframe::egui;
use serde_json::{json, Value};
pub fn metadata(name: &str) -> Value {
    if name == "Evaluator" {
        return observatory_plugins::evaluator::about_info();
    }
    if name == "Core" {
        return json!({"FullName":"Elite Observatory Core","ShortName":"Core","AuthorName":"Vithigar","Description":concat!("A tool for reading/monitoring Elite Dangerous journals for interesting objects.\n\nIf you like this tool, consider making a one-time donation via PayPal, or ongoing via Patreon!\n\nWith special thanks to my Patrons:\n","  1971gtrxu1\n  Anto\n  Arx\n  CMDR Reise Lang\n  CMDR SpaceTrash67\n  CMDR Xariss\n  David D\n  David T\n  Gareth dW\n  Hannes M\n  Heinrich S\n  Jacob G\n  Keith\n  McMuttons\n  Me, I'm Counting\n  NirevS\n  ObediahJoel\n  PeoplesHero\n  pepčok\n  Rich\n  Scott Gries\n  Slegnor\n  Trevor R\n  Valshia\n  William K"),"Links":[{"Text":"github","Url":"https://github.com/Xjph/ObservatoryCore"},{"Text":"Documentation","Url":"https://observatory.xjph.net/"},{"Text":"Donate via Paypal","Url":"https://www.paypal.com/donate/?hosted_button_id=XYQWYQ337TBP4"},{"Text":"Donate via Patreon","Url":"https://www.patreon.com/vithigar"}]});
    }
    let(full,description)=match name{
        "BioInsights"=>("BioInsights","BioInsights predicts which biologicals will be present on a body, informs you about potential new codex entries, and keeps track of your unsold exobiology data."),
        "Hecate"=>("Hecate - Data Vault","Hecate is a background plugin that stores and retrieves data for other plugins"),
        "StellarOverlay"=>("Stellar Overlay","An overlay for Observatory to provide notifications and other things"),
        _=>return json!({"FullName":name,"ShortName":name,"AuthorName":"","Description":"","Links":[]}),
    };
    json!({"FullName":full,"ShortName":name,"AuthorName":"Matt G","Description":description,"Links":[{"Text":"Website","Url":"https://edjp.colacube.net/observatory"},{"Text":"Buy me a coffee","Url":"https://ko-fi.com/explorationtools"}]})
}
#[allow(deprecated)]
pub fn show(ui: &mut egui::Ui, metadata: &Value) {
    let origin = ui.max_rect().min;
    let rect = |x: f32, y: f32, w: f32, h: f32| {
        egui::Rect::from_min_size(origin + egui::vec2(x, y), egui::vec2(w, h))
    };
    for (label, key, y) in [
        ("Full Name:", "FullName", 8.0),
        ("Short Name (Tab Name):", "ShortName", 32.0),
        ("Author:", "AuthorName", 56.0),
    ] {
        ui.put(rect(8.0, y, 152.0, 15.0), egui::Label::new(label));
        ui.put(
            rect(176.0, y, 328.0, 15.0),
            egui::Label::new(metadata[key].as_str().unwrap_or("")),
        );
    }
    ui.put(
        rect(8.0, 80.0, 152.0, 15.0),
        egui::Label::new("Description:"),
    );
    ui.allocate_ui_at_rect(rect(176.0, 80.0, 328.0, 128.0), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::Label::new(metadata["Description"].as_str().unwrap_or("")).selectable(true),
            );
        });
    });
    for (index, link) in metadata["Links"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|v| {
            v["Text"].as_str().is_some_and(|s| !s.is_empty())
                && v["Url"]
                    .as_str()
                    .is_some_and(|s| s.starts_with("https://") || s.starts_with("http://"))
        })
        .take(4)
        .enumerate()
    {
        let x = if index % 2 == 0 { 8.0 } else { 264.0 };
        let y = 216.0 + 24.0 * (index / 2) as f32;
        ui.allocate_ui_at_rect(rect(x, y, 240.0, 16.0), |ui| {
            ui.hyperlink_to(
                link["Text"].as_str().unwrap(),
                link["Url"].as_str().unwrap(),
            );
        });
    }
}
