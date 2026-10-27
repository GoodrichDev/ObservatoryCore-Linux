//! Resolve selected native fonts off the UI thread, retaining both notification
//! families when either Core or StellarOverlay changes its selection.
use eframe::egui::{self, FontData, FontDefinitions, FontFamily, FontId};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};
type Loaded = (String, String, Option<Vec<u8>>);
#[derive(Default)]
struct Registry {
    requests: BTreeMap<String, String>,
    exact: BTreeSet<String>,
    fonts: BTreeMap<String, Arc<FontData>>,
    loaded: Arc<Mutex<Vec<Loaded>>>,
}
fn registry(ctx: &egui::Context) -> Arc<Mutex<Registry>> {
    let id = egui::Id::new("native notification font registry");
    if let Some(value) = ctx.data(|d| d.get_temp::<Arc<Mutex<Registry>>>(id)) {
        return value;
    }
    let value = Arc::new(Mutex::new(Registry::default()));
    ctx.data_mut(|d| d.insert_temp(id, value.clone()));
    value
}
pub fn prepare(ctx: &egui::Context, family: &str, selected: &str) {
    prepare_family(ctx, family, selected, false);
}
/// CSS families use the platform's resolved face without adding application
/// emoji fonts which were never present in the source browser's fallback list.
pub fn prepare_exact(ctx: &egui::Context, family: &str, selected: &str) {
    prepare_family(ctx, family, selected, true);
}
fn prepare_family(ctx: &egui::Context, family: &str, selected: &str, exact: bool) {
    let registry = registry(ctx);
    let mut state = registry.lock().unwrap();
    if exact {
        state.exact.insert(family.into());
    }
    if state.requests.get(family).map(String::as_str) != Some(selected) {
        state.requests.insert(family.into(), selected.into());
        let family = family.to_owned();
        let selected = selected.to_owned();
        let loaded = state.loaded.clone();
        let repaint = ctx.clone();
        let _ = std::thread::Builder::new()
            .name("observatory-font".into())
            .spawn(move || {
                let bytes = std::process::Command::new("fc-match")
                    .args(["--format=%{file}", &selected])
                    .output()
                    .ok()
                    .filter(|o| o.status.success())
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .filter(|file| {
                        std::fs::metadata(file).is_ok_and(|m| m.len() <= 16 * 1024 * 1024)
                    })
                    .and_then(|file| std::fs::read(file).ok());
                loaded.lock().unwrap().push((family, selected, bytes));
                repaint.request_repaint();
            });
    }
    let completed = std::mem::take(&mut *state.loaded.lock().unwrap());
    let mut changed = false;
    for (family, selected, bytes) in completed {
        if state.requests.get(&family) != Some(&selected) {
            continue;
        }
        if let Some(bytes) = bytes {
            state
                .fonts
                .insert(family, Arc::new(FontData::from_owned(bytes)));
            changed = true;
        }
    }
    if !changed {
        return;
    }
    let mut fonts = FontDefinitions::default();
    for (family, data) in &state.fonts {
        fonts.font_data.insert(family.clone(), data.clone());
        let mut fallbacks = vec![family.clone()];
        if !state.exact.contains(family) {
            fallbacks.extend(fonts.families[&FontFamily::Proportional].clone());
        }
        fonts
            .families
            .insert(FontFamily::Name(family.clone().into()), fallbacks);
    }
    drop(state);
    ctx.set_fonts(fonts);
    ctx.request_repaint();
}
pub fn font(ctx: &egui::Context, family: &str, size: f32) -> FontId {
    let name = FontFamily::Name(family.to_owned().into());
    let available = ctx.fonts(|fonts| fonts.families().contains(&name));
    FontId::new(
        size,
        if available {
            name
        } else {
            FontFamily::Proportional
        },
    )
}
