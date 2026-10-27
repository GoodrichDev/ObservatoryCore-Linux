//! X11 foreground-window detection. Wayland leaves global focus to the compositor.
use std::{
    sync::{
        atomic::{AtomicI8, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use x11rb::protocol::randr::ConnectionExt as _;
use x11rb::{
    connection::Connection,
    protocol::xproto::{AtomEnum, ConnectionExt},
    rust_connection::RustConnection,
};
pub struct GameFocus {
    value: Arc<AtomicI8>,
    monitors: Arc<Mutex<Vec<[f32; 4]>>>,
}
impl GameFocus {
    pub fn start() -> Self {
        let value = Arc::new(AtomicI8::new(-1));
        let shared = Arc::downgrade(&value);
        let monitors = Arc::new(Mutex::new(Vec::new()));
        let monitor_output = monitors.clone();
        if crate::display_backend::detect() == crate::display_backend::Backend::X11 {
            let _ = std::thread::Builder::new()
                .name("observatory-game-focus".into())
                .spawn(move || {
                    let Ok((connection, screen)) = RustConnection::connect(None) else {
                        return;
                    };
                    let root = connection.setup().roots[screen].root;
                    let atom = |name: &[u8]| {
                        connection
                            .intern_atom(false, name)
                            .ok()?
                            .reply()
                            .ok()
                            .map(|r| r.atom)
                    };
                    let (Some(active), Some(title), Some(utf8)) = (
                        atom(b"_NET_ACTIVE_WINDOW"),
                        atom(b"_NET_WM_NAME"),
                        atom(b"UTF8_STRING"),
                    ) else {
                        return;
                    };
                    let mut frames = 0usize;
                    while let Some(shared) = shared.upgrade() {
                        if frames.is_multiple_of(20) {
                            if let Some(mut monitors) = connection
                                .randr_get_monitors(root, true)
                                .ok()
                                .and_then(|r| r.reply().ok())
                                .map(|r| r.monitors)
                            {
                                monitors.sort_by_key(|m| !m.primary);
                                *monitor_output.lock().unwrap() = monitors
                                    .into_iter()
                                    .map(|m| {
                                        [m.x as f32, m.y as f32, m.width as f32, m.height as f32]
                                    })
                                    .collect();
                            }
                        }
                        frames = frames.wrapping_add(1);
                        let focused = connection
                            .get_property(false, root, active, AtomEnum::WINDOW, 0, 1)
                            .ok()
                            .and_then(|r| r.reply().ok())
                            .and_then(|r| r.value32().and_then(|mut i| i.next()));
                        if let Some(window) = focused.filter(|w| *w != 0) {
                            let name = connection
                                .get_property(false, window, title, utf8, 0, 1024)
                                .ok()
                                .and_then(|r| r.reply().ok())
                                .map(|r| r.value)
                                .filter(|v| !v.is_empty())
                                .or_else(|| {
                                    connection
                                        .get_property(
                                            false,
                                            window,
                                            AtomEnum::WM_NAME,
                                            AtomEnum::STRING,
                                            0,
                                            1024,
                                        )
                                        .ok()
                                        .and_then(|r| r.reply().ok())
                                        .map(|r| r.value)
                                });
                            shared.store(
                                i8::from(name.is_some_and(|name| {
                                    is_game_title(&String::from_utf8_lossy(&name))
                                })),
                                Ordering::Relaxed,
                            );
                        } else if shared.load(Ordering::Relaxed) != -1 {
                            shared.store(0, Ordering::Relaxed);
                        }
                        drop(shared);
                        std::thread::sleep(Duration::from_millis(250));
                    }
                });
        }
        Self { value, monitors }
    }
    pub fn monitor_count(&self) -> usize {
        self.monitors
            .lock()
            .unwrap()
            .len()
            .max(crate::passive_windows::wayland_outputs().len())
            .max(1)
    }
    pub fn active(&self) -> Option<bool> {
        if let Some(active) = crate::passive_windows::foreground_game_active() {
            return Some(active);
        }
        match self.value.load(Ordering::Relaxed) {
            -1 => None,
            n => Some(n == 1),
        }
    }
    pub fn monitor(&self, selection: &str, ppp: f32) -> Option<eframe::egui::Rect> {
        let index = selection
            .trim()
            .split(|c: char| !c.is_ascii_digit())
            .next()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let wayland = crate::passive_windows::wayland_outputs();
        if !wayland.is_empty() {
            return wayland.get(index).or_else(|| wayland.first()).copied();
        }
        let monitors = self.monitors.lock().unwrap();
        let [x, y, w, h] = *monitors.get(index).or_else(|| monitors.first())?;
        Some(eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(x / ppp, y / ppp),
            eframe::egui::vec2(w / ppp, h / ppp),
        ))
    }
}
fn is_game_title(title: &str) -> bool {
    let title = title.to_ascii_lowercase();
    title.contains("elite - dangerous")
        || title.contains("elite dangerous")
        || title.contains("elitedangerous")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recognises_game_without_matching_observatory() {
        assert!(is_game_title("Elite - Dangerous (CLIENT)"));
        assert!(is_game_title("EliteDangerous64.exe"));
        assert!(!is_game_title("Elite Observatory Core - v2.0.0"));
    }
}
