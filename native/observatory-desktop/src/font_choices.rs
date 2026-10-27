//! Cached installed-font families for the source notification font selector.
use eframe::egui;
use std::sync::{Arc, Mutex, OnceLock};
fn parse(output: &str) -> Vec<String> {
    let mut families = output
        .lines()
        .flat_map(|line| line.split(','))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    families.sort();
    families.dedup();
    families
}
fn load(ctx: &egui::Context) -> Option<Vec<String>> {
    static FAMILIES: OnceLock<Arc<Mutex<Option<Vec<String>>>>> = OnceLock::new();
    let families = FAMILIES.get_or_init(|| {
        let families = Arc::new(Mutex::new(None));
        let shared = families.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let list = std::process::Command::new("fc-list")
                .args(["--format=%{family}\\n"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| parse(&String::from_utf8_lossy(&o.stdout)))
                .unwrap_or_default();
            *shared.lock().unwrap() = Some(list);
            ctx.request_repaint();
        });
        families
    });
    families.lock().unwrap().clone()
}
pub fn choices(ctx: &egui::Context) -> Vec<String> {
    load(ctx).unwrap_or_default()
}
pub fn select(ui: &mut egui::Ui, current: &mut String) {
    let list = load(ui.ctx());
    egui::ComboBox::from_id_salt("Native installed font families")
        .selected_text(current.as_str())
        .show_ui(ui, |ui| {
            ui.selectable_value(current, current.clone(), current.clone());
            if let Some(list) = list {
                for family in list {
                    ui.selectable_value(current, family.clone(), family);
                }
            } else {
                ui.spinner();
            }
        });
}
#[cfg(test)]
mod tests {
    #[test]
    fn fontconfig_family_aliases_are_distinct_sorted_choices() {
        assert_eq!(
            super::parse("Noto Sans,CJK\nArial\nNoto Sans\n"),
            vec!["Arial", "CJK", "Noto Sans"]
        );
    }
}
