//! BioInsights host settings and legacy launch aliases. Content pages stay in
//! the existing custom view; UIHandler only delegates explicit Popout/settings.
use eframe::egui;
use serde_json::{json, Value};

pub fn open(ctx: &egui::Context, which: &str) -> Vec<Value> {
    crate::bioinsights_main::navigate(ctx, which, &json!({}))
}
