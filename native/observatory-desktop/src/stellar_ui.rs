//! Compositor-native transparent StellarOverlay windows.
use base64::Engine as _;
use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};
use observatory_plugins::stellar_overlay::{
    now_ms, relative_position, OverlaySnapshot, RadarSnapshot,
};
use serde_json::Value;

fn color(value: &Value, key: &str, fallback: Color32) -> Color32 {
    let Some(raw) = value[key].as_str() else {
        return fallback;
    };
    let raw = raw.trim_start_matches('#');
    let Ok(n) = u32::from_str_radix(raw, 16) else {
        return fallback;
    };
    match raw.len() {
        8 => Color32::from_rgba_unmultiplied(
            (n >> 16) as u8,
            (n >> 8) as u8,
            n as u8,
            (n >> 24) as u8,
        ),
        6 => Color32::from_rgb((n >> 16) as u8, (n >> 8) as u8, n as u8),
        _ => fallback,
    }
}
fn number(v: &Value, k: &str, default: f32) -> f32 {
    v[k].as_f64()
        .filter(|n| n.is_finite())
        .unwrap_or(default as f64) as f32
}
fn builder(title: &str, position: Pos2, size: Vec2) -> egui::ViewportBuilder {
    egui::ViewportBuilder::default()
        .with_title(title)
        .with_position(position)
        .with_inner_size(size)
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top()
        .with_mouse_passthrough(true)
        .with_resizable(false)
        .with_taskbar(false)
        .with_active(false)
}
pub fn prepare_font(ctx: &egui::Context, settings: &Value) {
    let selected = settings["SelectedFont"].as_str().unwrap_or("Arial");
    crate::font_registry::prepare(ctx, "StellarOverlay", selected);
    crate::font_registry::prepare(ctx, "StellarOverlayBold", &format!("{selected}:style=Bold"));
}
fn font(ctx: &egui::Context, size: f32) -> egui::FontId {
    crate::font_registry::font(ctx, "StellarOverlay", size)
}
fn transition(name: &str, progress: f32, size: Vec2, entry: bool, bitmap: bool) -> (Vec2, f32) {
    let remaining = if entry { 1.0 - progress } else { progress }.clamp(0.0, 1.0);
    if name == "Fade" {
        return (Vec2::ZERO, 1.0 - remaining);
    }
    let sign = if entry && !bitmap { -1.0 } else { 1.0 };
    let offset = match name {
        "Left" => Vec2::new(-size.x, 0.0),
        "Right" => Vec2::new(size.x, 0.0),
        "Up" => Vec2::new(0.0, -size.y),
        "Down" => Vec2::new(0.0, size.y),
        _ => Vec2::ZERO,
    };
    (offset * remaining * sign, 1.0)
}
fn pose(
    created: u64,
    expires: Option<u64>,
    duration: u64,
    entry: &str,
    exit: &str,
    size: Vec2,
    bitmap: bool,
) -> (Vec2, f32, bool) {
    let now = now_ms();
    let age = now.saturating_sub(created);
    let duration = duration.max(1);
    if age < duration {
        let (offset, alpha) = transition(entry, age as f32 / duration as f32, size, true, bitmap);
        return (offset, alpha, entry != "None");
    }
    if let Some(remaining) = expires
        .map(|end| end.saturating_sub(now))
        .filter(|remaining| *remaining < duration)
    {
        let (offset, alpha) = transition(
            exit,
            1.0 - remaining as f32 / duration as f32,
            size,
            false,
            bitmap,
        );
        return (offset, alpha, exit != "None");
    }
    (Vec2::ZERO, 1.0, false)
}
#[derive(Clone)]
struct RadarMotion {
    from: RadarSnapshot,
    to: RadarSnapshot,
    changed_ms: u64,
}
fn angle_between(from: f64, to: f64) -> f64 {
    (to - from + 180.0).rem_euclid(360.0) - 180.0
}
fn interpolate_radar(motion: &RadarMotion, now: u64) -> RadarSnapshot {
    let p = (now.saturating_sub(motion.changed_ms) as f64 / 300.0).clamp(0.0, 1.0);
    let mut current = motion.to.clone();
    current.latitude = motion.from.latitude + (motion.to.latitude - motion.from.latitude) * p;
    current.longitude = (motion.from.longitude
        + angle_between(motion.from.longitude, motion.to.longitude) * p
        + 180.0)
        .rem_euclid(360.0)
        - 180.0;
    current.heading = (motion.from.heading
        + angle_between(motion.from.heading, motion.to.heading) * p)
        .rem_euclid(360.0);
    for point in &mut current.points {
        let (distance, x, y) = relative_position(
            current.latitude,
            current.longitude,
            current.heading,
            point.latitude,
            point.longitude,
            current.radius_m,
        );
        point.distance_m = distance;
        point.x = x;
        point.y = y;
        point.visible = distance < 20000.0;
    }
    current
}
fn smooth_radar(ctx: &egui::Context, target: &RadarSnapshot) -> RadarSnapshot {
    let id = egui::Id::new("stellar-radar-motion");
    let now = now_ms();
    let mut motion = ctx
        .data(|d| d.get_temp::<RadarMotion>(id))
        .unwrap_or_else(|| RadarMotion {
            from: target.clone(),
            to: target.clone(),
            changed_ms: now.saturating_sub(300),
        });
    if (motion.to.system_address, motion.to.body_id) != (target.system_address, target.body_id) {
        motion.from = target.clone();
        motion.changed_ms = now.saturating_sub(300);
    } else if (motion.to.latitude, motion.to.longitude, motion.to.heading)
        != (target.latitude, target.longitude, target.heading)
    {
        motion.from = interpolate_radar(&motion, now);
        motion.changed_ms = now;
    }
    motion.to = target.clone();
    let current = interpolate_radar(&motion, now);
    if now.saturating_sub(motion.changed_ms) < 300 {
        ctx.request_repaint_after(std::time::Duration::from_millis(8));
    }
    ctx.data_mut(|d| d.insert_temp(id, motion));
    current
}
fn bitmap_image(
    ctx: &egui::Context,
    panel: &Value,
) -> Option<(egui::Id, std::sync::Arc<egui::ColorImage>)> {
    let encoded = panel
        .get("Bitmap")
        .and_then(|b| b.as_str().or_else(|| b["png_base64"].as_str()));
    let filename = panel["Filename"].as_str();
    let metadata = filename.and_then(|file| std::fs::metadata(file).ok());
    let key = if let Some(encoded) = encoded {
        if encoded.len() > 16 * 1024 * 1024 {
            return None;
        }
        egui::Id::new(("stellar-bitmap", encoded))
    } else {
        egui::Id::new((
            "stellar-file",
            filename?,
            metadata.as_ref()?.modified().ok(),
            metadata.as_ref()?.len(),
        ))
    };
    let image_key = key.with("decoded image");
    if let Some(image) = ctx.data(|d| d.get_temp::<std::sync::Arc<egui::ColorImage>>(image_key)) {
        return Some((key, image));
    }
    let bytes = if let Some(encoded) = encoded {
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .ok()?
    } else {
        if metadata?.len() > 16 * 1024 * 1024 {
            return None;
        }
        std::fs::read(filename?).ok()?
    };
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    reader.limits(limits);
    let image = reader.decode().ok()?.to_rgba8();
    let image = std::sync::Arc::new(egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        image.as_raw(),
    ));
    ctx.data_mut(|d| d.insert_temp(image_key, image.clone()));
    Some((key, image))
}
fn bitmap_texture(
    ctx: &egui::Context,
    key: egui::Id,
    image: &egui::ColorImage,
) -> egui::TextureHandle {
    if let Some(texture) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(key)) {
        return texture;
    }
    let texture = ctx.load_texture(
        "StellarOverlay bitmap",
        image.clone(),
        egui::TextureOptions::LINEAR,
    );
    ctx.data_mut(|d| d.insert_temp(key, texture.clone()));
    texture
}

#[path = "stellar_settings.rs"]
mod settings_form;
pub fn settings(ui: &mut egui::Ui, value: &mut Value) -> Vec<Value> {
    settings_form::show(ui, value)
}
pub fn render(
    ctx: &egui::Context,
    snapshot: &OverlaySnapshot,
    monitor_rect: Option<egui::Rect>,
    mut capture: impl FnMut(&egui::Context, &str),
) {
    // The host additionally gates on X11 foreground focus when available.
    let normal_visible = snapshot.game_running || snapshot.force_show;
    if !normal_visible
        && !snapshot
            .alerts
            .iter()
            .any(|v| v["ForceShow"].as_bool() == Some(true))
    {
        return;
    }
    let monitor_rect = monitor_rect.unwrap_or_else(|| {
        egui::Rect::from_min_size(
            Pos2::ZERO,
            ctx.input(|i| i.viewport().monitor_size)
                .unwrap_or(Vec2::new(1920.0, 1080.0)),
        )
    });
    let monitor = monitor_rect.size();
    let origin = monitor_rect.min.to_vec2();
    let settings = &snapshot.settings;
    let mut slot = 0usize;
    for item in snapshot.notifications.iter().filter(|_| normal_visible) {
        let layout = notification_layout(
            ctx,
            item,
            settings,
            monitor,
            item.queue_slot.unwrap_or(slot),
        );
        if !item.instant {
            slot += 1;
        }
        let (offset, alpha, animated) = pose(
            item.created_ms,
            item.expires_ms,
            item.transition_ms,
            &item.entry_transition,
            &item.exit_transition,
            layout.size,
            false,
        );
        if animated {
            ctx.request_repaint_after(std::time::Duration::from_secs_f32(
                1.0 / number(settings, "FPS", 120.0).clamp(10.0, 300.0),
            ));
        }
        let alpha = alpha * item.opacity / 100.0;
        crate::passive_windows::show(
            ctx,
            egui::ViewportId::from_hash_of(("stellar", &item.id)),
            builder(
                "StellarOverlay",
                layout.position + origin + offset,
                layout.size,
            ),
            |ctx| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show(ctx, |ui| {
                        notification_render(ui, item, settings, &layout, alpha)
                    });
                capture(ctx, "StellarOverlay");
            },
        );
    }
    if normal_visible
        && snapshot.radar.visible
        && settings["RadarSettings"]["Enabled"]
            .as_bool()
            .unwrap_or(true)
    {
        let config = &settings["RadarSettings"];
        let layout = radar_layout(ctx, &snapshot.radar, monitor, config);
        crate::passive_windows::show(
            ctx,
            egui::ViewportId::from_hash_of("stellar-radar"),
            builder(
                "StellarOverlay Radar",
                layout.position.to_pos2() + origin,
                layout.size,
            ),
            |ctx| {
                let current = smooth_radar(ctx, &snapshot.radar);
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show(ctx, |ui| radar_with_layout(ui, &current, layout));
                capture(ctx, "StellarOverlay Radar");
            },
        );
    }
    for (index, panel) in snapshot
        .alerts
        .iter()
        .filter(|v| {
            v["Enabled"].as_bool() != Some(false)
                && (normal_visible || v["ForceShow"].as_bool() == Some(true))
        })
        .enumerate()
    {
        let x = number(panel, "XPos", 2.0);
        let y = number(panel, "YPos", 5.0 + index as f32 * 25.0);
        let rows = panel["Predictions"].as_array().map_or(0, Vec::len);
        let bitmap = bitmap_image(ctx, panel);
        let size = bitmap
            .as_ref()
            .map(|(_, image)| Vec2::new(image.width() as f32, image.height() as f32))
            .or_else(|| {
                (panel["SourceBitmap"] == "BioInsights")
                    .then(|| crate::bioinsights_main::alert_size(ctx, panel))
            })
            .unwrap_or_else(|| {
                Vec2::new(
                    number(panel, "Width", 410.0),
                    number(
                        panel,
                        "Height",
                        if rows > 0 {
                            85.0 + rows as f32 * 19.0
                        } else if panel.get("Sampler").is_some() {
                            100.0
                        } else {
                            90.0
                        },
                    ),
                )
            });
        let (offset, alpha, animated) = pose(
            panel["created_ms"].as_u64().unwrap_or(snapshot.sampled_ms),
            panel["expires_ms"].as_u64(),
            panel["TransitionMS"].as_u64().unwrap_or(300),
            panel["EntryTransition"].as_str().unwrap_or("Fade"),
            panel["ExitTransition"].as_str().unwrap_or("Fade"),
            size,
            true,
        );
        if animated {
            ctx.request_repaint_after(std::time::Duration::from_secs_f32(
                1.0 / number(settings, "FPS", 120.0).clamp(10.0, 300.0),
            ));
        }
        let alpha = alpha * number(panel, "Opacity", 100.0) / 100.0;
        let fill = color(
            panel,
            "BackgroundColorHex",
            Color32::from_rgba_unmultiplied(0, 0, 0, 176),
        )
        .gamma_multiply(alpha);
        let border =
            color(panel, "BorderColorHex", Color32::from_rgb(255, 61, 58)).gamma_multiply(alpha);
        crate::passive_windows::show(
            ctx,
            egui::ViewportId::from_hash_of((
                "stellar-panel",
                panel["Key"].as_str().unwrap_or("information"),
                index,
            )),
            builder(
                "StellarOverlay Information",
                Pos2::new(
                    (monitor.x - size.x) * x / 100.0,
                    (monitor.y - size.y) * y / 100.0,
                ) + origin
                    + offset,
                size,
            ),
            |ctx| {
                let bitmap = bitmap
                    .as_ref()
                    .map(|(key, image)| bitmap_texture(ctx, *key, image));
                let frame = if bitmap.is_some() || panel["SourceBitmap"] == "BioInsights" {
                    egui::Frame::NONE
                } else {
                    egui::Frame::NONE
                        .fill(fill)
                        .stroke(Stroke::new(number(panel, "BorderWidth", 2.0), border))
                        .inner_margin(number(panel, "Padding", 3.0) as i8)
                };
                egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
                    ui.set_opacity(alpha);
                    if let Some(texture) = &bitmap {
                        ui.image((texture.id(), texture.size_vec2()));
                    } else if panel["SourceBitmap"] == "BioInsights" {
                        crate::bioinsights_main::render_alert(ui, panel);
                    } else {
                        information(ui, panel);
                    }
                });
            },
        );
    }
}
#[path = "stellar_radar.rs"]
mod radar_view;
pub fn radar_layout(
    ctx: &egui::Context,
    radar: &RadarSnapshot,
    monitor: Vec2,
    settings: &Value,
) -> radar_view::Layout {
    radar_view::layout(ctx, radar, monitor, settings)
}
pub fn radar_with_layout(ui: &mut egui::Ui, radar: &RadarSnapshot, layout: radar_view::Layout) {
    radar_view::show_layout(ui, radar, layout)
}
fn credits(v: &Value, key: &str) -> String {
    v[key]
        .as_u64()
        .map(|n| format!("{n} Cr"))
        .unwrap_or_default()
}
fn information(ui: &mut egui::Ui, panel: &Value) {
    ui.visuals_mut().override_text_color =
        Some(color(panel, "ColorHex", Color32::from_rgb(255, 99, 0)));
    let size = number(panel, "FontSize", 12.0);
    ui.style_mut().override_font_id = Some(font(ui.ctx(), size));
    ui.label(
        egui::RichText::new(
            panel["Title"]
                .as_str()
                .or_else(|| panel["BodyName"].as_str())
                .unwrap_or("Elite Observatory"),
        )
        .strong()
        .color(color(
            panel,
            "TitleColorHex",
            Color32::from_rgb(255, 123, 58),
        )),
    );
    if let Some(detail) = panel["Detail"].as_str() {
        ui.label(detail);
    }
    if panel.get("MinimumValue").is_some() {
        ui.label(format!(
            "Biological value: {} – {}",
            credits(panel, "MinimumValue"),
            credits(panel, "MaximumValue")
        ));
    }
    if let Some(predictions) = panel["Predictions"].as_array() {
        for species in predictions.iter().take(8) {
            ui.label(format!(
                "{} {}  {}",
                species["Genus"].as_str().unwrap_or(""),
                species["Species"].as_str().unwrap_or(""),
                credits(species, "BaseValue")
            ));
        }
    }
    if let Some(sampler) = panel.get("Sampler") {
        ui.label(format!(
            "{}  sample {} / 3",
            sampler["Species"].as_str().unwrap_or(""),
            sampler["NextSample"]
        ));
        if let Some(distances) = sampler["Distances"].as_array() {
            ui.label(
                distances
                    .iter()
                    .filter_map(Value::as_f64)
                    .map(|n| format!("{n:.0} m"))
                    .collect::<Vec<_>>()
                    .join(" / "),
            );
        }
    }
    if panel.get("UnsoldTotal").is_some() {
        ui.label(format!("Unsold: {}", credits(panel, "UnsoldTotal")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn argb_alpha_and_rgb_are_preserved() {
        assert_eq!(
            color(&serde_json::json!({"c":"B0000000"}), "c", Color32::RED),
            Color32::from_rgba_unmultiplied(0, 0, 0, 176)
        );
        assert_eq!(
            color(&serde_json::json!({"c":"FF6300"}), "c", Color32::RED),
            Color32::from_rgb(255, 99, 0)
        );
    }
    #[test]
    fn source_linear_transitions_distinguish_queue_and_bitmap_entry() {
        assert_eq!(
            transition("Left", 0.5, Vec2::new(400.0, 80.0), true, false),
            (Vec2::new(200.0, 0.0), 1.0)
        );
        assert_eq!(
            transition("Left", 0.5, Vec2::new(400.0, 80.0), true, true),
            (Vec2::new(-200.0, 0.0), 1.0)
        );
        assert_eq!(
            transition("Fade", 0.25, Vec2::splat(400.0), true, true),
            (Vec2::ZERO, 0.25)
        );
        assert_eq!(
            transition("Fade", 0.25, Vec2::splat(400.0), false, true),
            (Vec2::ZERO, 0.75)
        );
    }
    #[test]
    fn radar_smoothing_takes_short_path_across_dateline_and_heading_zero() {
        let from = RadarSnapshot {
            latitude: 0.0,
            longitude: 179.9,
            heading: 350.0,
            ..Default::default()
        };
        let to = RadarSnapshot {
            latitude: 0.0,
            longitude: -179.9,
            heading: 10.0,
            ..Default::default()
        };
        let current = interpolate_radar(
            &RadarMotion {
                from,
                to,
                changed_ms: 1000,
            },
            1150,
        );
        assert!((current.longitude.abs() - 180.0).abs() < 0.0001);
        assert!(current.heading.abs() < 0.0001);
    }
}

#[path = "stellar_notification_layout.rs"]
mod notification_view;
pub fn notification_layout(
    ctx: &egui::Context,
    item: &observatory_plugins::stellar_overlay::OverlayNotification,
    settings: &Value,
    monitor: Vec2,
    slot: usize,
) -> notification_view::Layout {
    notification_view::layout(ctx, item, settings, monitor, slot)
}
pub fn notification_render(
    ui: &mut egui::Ui,
    item: &observatory_plugins::stellar_overlay::OverlayNotification,
    settings: &Value,
    layout: &notification_view::Layout,
    alpha: f32,
) {
    notification_view::render(ui, item, settings, layout, alpha)
}
