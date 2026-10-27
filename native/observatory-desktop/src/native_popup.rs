//! Native Core NotificationForm, independent of StellarOverlay and of a desktop
//! notification daemon. Geometry and lifecycle follow the installed Core.
use eframe::egui::{self, Color32, Pos2, Rect, Vec2};
use observatory_io::Settings;
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Debug, Serialize)]
pub struct Style {
    pub colour: u32,
    pub scale: f32,
    pub font_scale: f32,
    pub font: String,
    pub transparent: bool,
    pub corner: u8,
    pub screen: i64,
    pub timeout: i64,
}
fn setting<'a>(settings: &'a Settings, key: &str) -> Option<&'a Value> {
    settings.extra.get(key).or_else(|| {
        settings
            .extra
            .get("legacy_settings")
            .and_then(|v| v.get(key))
    })
}
impl Style {
    pub fn from_settings(s: &Settings) -> Self {
        let number =
            |key: &str, default: i64| setting(s, key).and_then(Value::as_i64).unwrap_or(default);
        Self {
            colour: setting(s, "NativeNotifyColour")
                .and_then(Value::as_u64)
                .unwrap_or(4_294_944_000) as u32,
            scale: number("NativeNotifyScale", 100).clamp(1, 500) as f32 / 100.0,
            font_scale: number("NativeNotifyFontScale", 100).clamp(1, 500) as f32 / 100.0,
            font: setting(s, "NativeNotifyFont")
                .and_then(Value::as_str)
                .unwrap_or("Segoe UI")
                .to_owned(),
            transparent: setting(s, "NativeNotifyTransparent")
                .and_then(Value::as_bool)
                .unwrap_or(true),
            corner: u8::try_from(number("NativeNotifyCorner", 0))
                .ok()
                .filter(|n| *n <= 7)
                .unwrap_or(0),
            screen: number("NativeNotifyScreen", -1),
            timeout: setting(s, "NativeNotifyTimeout")
                .and_then(Value::as_i64)
                .or_else(|| {
                    s.extra
                        .get("notification_timeout_ms")
                        .and_then(Value::as_i64)
                })
                .unwrap_or(8000),
        }
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Popup {
    pub id: String,
    pub title: String,
    pub detail: String,
    pub created: u64,
    pub expires: Option<u64>,
    pub style: Style,
    pub x: f32,
    pub y: f32,
    pub offset: f32,
}
impl Popup {
    pub fn size(&self) -> Vec2 {
        Vec2::new(
            (355.0 * self.style.scale).round_ties_even(),
            (145.0 * self.style.scale).round_ties_even(),
        )
    }
    fn positioned(&self) -> bool {
        self.x >= 0.0 && self.y >= 0.0
    }
    pub fn position(&self, screen: Rect) -> Pos2 {
        let size = self.size();
        if self.positioned() {
            return screen.min
                + Vec2::new(
                    (screen.width() as f64 * self.x as f64 / 100.0).round_ties_even() as f32,
                    (screen.height() as f64 * self.y as f64 / 100.0).round_ties_even() as f32,
                );
        }
        let (x, y) = match self.style.corner {
            1 => (screen.left() + 50.0, screen.bottom() - size.y - 50.0),
            2 => (screen.right() - size.x - 50.0, screen.top() + 50.0),
            3 => (screen.left() + 50.0, screen.top() + 50.0),
            4 => (
                (screen.right() as i32 / 2 - size.x as i32 / 2) as f32,
                screen.top() + 50.0,
            ),
            5 => (
                (screen.right() as i32 / 2 - size.x as i32 / 2) as f32,
                screen.bottom() - size.y - 50.0,
            ),
            6 => (
                screen.left() + 50.0,
                (screen.bottom() as i32 / 2 - size.y as i32 / 2) as f32,
            ),
            7 => (
                screen.right() - size.x - 50.0,
                (screen.bottom() as i32 / 2 - size.y as i32 / 2) as f32,
            ),
            _ => (
                screen.right() - size.x - 50.0,
                screen.bottom() - size.y - 50.0,
            ),
        };
        Pos2::new(x, y + self.offset)
    }
}
#[derive(Default)]
pub struct Manager {
    items: Vec<Popup>,
    sequence: u64,
    current_corner: u8,
}
impl Manager {
    pub fn set_settings(&mut self, settings: &Settings) {
        self.current_corner = Style::from_settings(settings).corner;
    }
    pub fn snapshot(&self) -> Vec<Popup> {
        self.items.clone()
    }
    pub fn tick(&mut self, now: u64) {
        let expired = self
            .items
            .iter()
            .filter(|p| p.expires.is_some_and(|e| now >= e))
            .map(|p| p.id.clone())
            .collect::<Vec<_>>();
        for id in expired {
            self.close(&id);
        }
    }
    fn close(&mut self, id: &str) {
        let Some(index) = self.items.iter().position(|p| p.id == id) else {
            return;
        };
        let closed = self.items.remove(index);
        let corner = self.current_corner;
        for older in self
            .items
            .iter_mut()
            .filter(|p| p.created < closed.created && !p.positioned())
        {
            if older.offset != 0.0 {
                older.offset += older.size().y * if corner <= 1 { 1.0 } else { -1.0 };
            }
        }
    }
    pub fn lifecycle(
        &mut self,
        value: &Value,
        settings: &Settings,
        now: u64,
    ) -> anyhow::Result<()> {
        self.set_settings(settings);
        let id = value["id"].as_str().unwrap_or("");
        match value["action"].as_str().unwrap_or("show") {
            "cancel" => self.close(id),
            "update" => {
                anyhow::ensure!(!id.is_empty(), "Cannot update notification without Guid");
                if let Some(item) = self.items.iter_mut().find(|p| p.id == id) {
                    item.title = value["title"].as_str().unwrap_or("").to_owned();
                    item.detail = value["detail"].as_str().unwrap_or("").to_owned();
                    // Core NotificationForm.Update changes text, not timeout/position.
                }
            }
            _ => {
                self.sequence += 1;
                let id = if id.is_empty() {
                    format!("core.{now}.{}", self.sequence)
                } else {
                    id.to_owned()
                };
                anyhow::ensure!(
                    !self.items.iter().any(|p| p.id == id),
                    "Notification {id} already exists; use update"
                );
                for old in self.items.iter_mut().filter(|p| !p.positioned()) {
                    old.offset += old.size().y * if self.current_corner >= 2 { 1.0 } else { -1.0 };
                }
                let style = Style::from_settings(settings);
                let timeout = value["timeout"].as_i64().unwrap_or(-1);
                let timeout = if timeout == -1 {
                    style.timeout
                } else {
                    timeout
                };
                self.items.push(Popup {
                    id,
                    title: value["title"].as_str().unwrap_or("").into(),
                    detail: value["detail"].as_str().unwrap_or("").into(),
                    created: self.sequence,
                    expires: (timeout > 0).then_some(now + timeout.max(0) as u64),
                    style,
                    x: value["x"].as_f64().unwrap_or(-1.0) as f32,
                    y: value["y"].as_f64().unwrap_or(-1.0) as f32,
                    offset: 0.0,
                });
            }
        }
        Ok(())
    }
}

pub fn render(
    ctx: &egui::Context,
    popups: &[Popup],
    mut monitor: impl FnMut(i64) -> Option<Rect>,
    mut capture: impl FnMut(&egui::Context, &str),
) {
    for popup in popups {
        crate::font_registry::prepare(ctx, "CoreNotifications", &popup.style.font);
        let screen = monitor(popup.style.screen).unwrap_or_else(|| {
            Rect::from_min_size(
                Pos2::ZERO,
                ctx.input(|i| i.viewport().monitor_size)
                    .unwrap_or(Vec2::new(1920.0, 1080.0)),
            )
        });
        let size = popup.size();
        let builder = egui::ViewportBuilder::default()
            .with_title("Observatory Core Notification")
            .with_position(popup.position(screen))
            .with_inner_size(size)
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_mouse_passthrough(true)
            .with_resizable(false)
            .with_taskbar(false)
            .with_active(false);
        crate::passive_windows::show(
            ctx,
            egui::ViewportId::from_hash_of(("core-notification", &popup.id)),
            builder,
            |ctx| {
                let fill = if popup.style.transparent {
                    Color32::TRANSPARENT
                } else {
                    Color32::from_rgb(25, 30, 30)
                };
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE.fill(fill))
                    .show(ctx, |ui| {
                        let rect = ui.max_rect();
                        let colour = Color32::from_rgba_unmultiplied(
                            (popup.style.colour >> 16) as u8,
                            (popup.style.colour >> 8) as u8,
                            popup.style.colour as u8,
                            (popup.style.colour >> 24) as u8,
                        );
                        ui.painter().rect_stroke(
                            rect,
                            0,
                            egui::Stroke::new(6.0_f32, colour),
                            egui::StrokeKind::Middle,
                        );
                        let scale = popup.style.scale;
                        let font = popup.style.font_scale;
                        let scaled = |value: f32| (value * scale).round_ties_even();
                        let title_height = scaled(35.0);
                        let title = Rect::from_min_size(
                            rect.min + Vec2::new(scaled(5.0), scaled(5.0)),
                            Vec2::new(scaled(338.0), title_height),
                        );
                        // WinForms scales integer control bounds first; the
                        // constructor then rounds its extra font-height offset.
                        let body_y =
                            (scaled(40.0) + title_height * font - title_height).round_ties_even();
                        let body = Rect::from_min_size(
                            rect.min + Vec2::new(scaled(12.0), body_y),
                            Vec2::new(scaled(320.0), scaled(85.0)),
                        );
                        for (text, area, points) in
                            [(&popup.title, title, 18.0), (&popup.detail, body, 14.0)]
                        {
                            let galley = ui.painter().layout(
                                text.clone(),
                                crate::font_registry::font(
                                    ctx,
                                    "CoreNotifications",
                                    points * 96.0 / 72.0 * scale * font,
                                ),
                                colour,
                                area.width(),
                            );
                            ui.painter()
                                .with_clip_rect(area)
                                .galley(area.min, galley, colour);
                        }
                    });
                capture(ctx, "Core Notification");
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn all_480_original_geometry_and_changed_corner_stack_vectors_match() {
        let vectors: Vec<Value> =
            serde_json::from_str(include_str!("../../fixtures/core-popup-oracle.json")).unwrap();
        assert_eq!(vectors.len(), 480);
        for v in vectors {
            let mut settings = Settings::default();
            settings
                .extra
                .insert("NativeNotifyCorner".into(), v["Corner"].clone());
            let size = v["Size"].as_array().unwrap();
            let scale = match size[0].as_u64().unwrap() {
                355 => 100,
                444 => 125,
                178 => 50,
                _ => unreachable!(),
            };
            settings
                .extra
                .insert("NativeNotifyScale".into(), json!(scale));
            let screen = &v["Screen"];
            let rect = Rect::from_min_size(
                Pos2::new(
                    screen[0].as_i64().unwrap() as f32,
                    screen[1].as_i64().unwrap() as f32,
                ),
                Vec2::new(
                    screen[2].as_i64().unwrap() as f32,
                    screen[3].as_i64().unwrap() as f32,
                ),
            );
            let mut manager = Manager::default();
            let payload = |id: &str| json!({"id":id,"timeout":0,"x":v["Position"][0].as_f64().unwrap()*100.0,"y":v["Position"][1].as_f64().unwrap()*100.0});
            manager.lifecycle(&payload("first"), &settings, 0).unwrap();
            let point = |key: &str| {
                Pos2::new(
                    v[key][0].as_i64().unwrap() as f32,
                    v[key][1].as_i64().unwrap() as f32,
                )
            };
            assert_eq!(manager.items[0].position(rect), point("Initial"), "{v}");
            manager.lifecycle(&payload("second"), &settings, 1).unwrap();
            assert_eq!(manager.items[0].position(rect), point("Offset1"), "{v}");
            manager.lifecycle(&payload("third"), &settings, 2).unwrap();
            assert_eq!(manager.items[0].position(rect), point("Offset2"), "{v}");
            settings
                .extra
                .insert("NativeNotifyCorner".into(), v["ChangedCorner"].clone());
            manager.set_settings(&settings);
            manager
                .lifecycle(&json!({"action":"cancel","id":"third"}), &settings, 3)
                .unwrap();
            assert_eq!(manager.items[0].position(rect), point("Closed"), "{v}");
        }
    }
    #[test]
    fn core_source_corners_explicit_percent_and_stack_lifecycle() {
        let mut settings = Settings::default();
        let mut manager = Manager::default();
        manager
            .lifecycle(
                &json!({"id":"one","title":"First","timeout":0}),
                &settings,
                100,
            )
            .unwrap();
        manager
            .lifecycle(&json!({"id":"two","timeout":0}), &settings, 200)
            .unwrap();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(1920.0, 1080.0));
        assert_eq!(manager.items[1].position(screen), Pos2::new(1515.0, 885.0));
        assert_eq!(manager.items[0].position(screen), Pos2::new(1515.0, 740.0));
        manager
            .lifecycle(&json!({"id":"two","action":"cancel"}), &settings, 300)
            .unwrap();
        assert_eq!(manager.items[0].position(screen), Pos2::new(1515.0, 885.0));
        settings.extra.insert("NativeNotifyCorner".into(), json!(3));
        manager
            .lifecycle(
                &json!({"id":"three","timeout":100,"x":10,"y":20}),
                &settings,
                400,
            )
            .unwrap();
        assert_eq!(manager.items[1].position(screen), Pos2::new(192.0, 216.0));
        manager.tick(500);
        assert_eq!(manager.items.len(), 1);
        manager
            .lifecycle(
                &json!({"id":"one","action":"update","title":"Changed","timeout":1}),
                &settings,
                500,
            )
            .unwrap();
        manager.tick(9999);
        assert_eq!(manager.items.len(), 1);
        assert_eq!(manager.items[0].title, "Changed");
    }
}
