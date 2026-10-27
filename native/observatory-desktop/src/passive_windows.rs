//! Passive notification surfaces. Winit's `active=false` does not set ICCCM
//! input hints on X11, so establish the policy while a new surface is hidden.
use eframe::egui::{self, ViewportBuilder, ViewportId};
use raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
};
use x11rb::{
    connection::Connection,
    protocol::xproto::{AtomEnum, ConnectionExt, PropMode},
    rust_connection::RustConnection,
    wrapper::ConnectionExt as _,
};

static X11: AtomicBool = AtomicBool::new(false);
static WAYLAND: AtomicBool = AtomicBool::new(false);
static WAYLAND_ERROR: AtomicBool = AtomicBool::new(false);
static CAPABILITY_ERROR: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
#[cfg(target_os = "linux")]
#[path = "wayland_passive.rs"]
mod wayland;
thread_local! {static POLICY:RefCell<Option<Policy>>=const{RefCell::new(None)};}
struct Policy {
    connection: RustConnection,
    root: u32,
    ready: HashMap<String, u32>,
}

pub fn initialize(cc: &eframe::CreationContext<'_>) {
    WAYLAND.store(false, Ordering::Relaxed);
    WAYLAND_ERROR.store(false, Ordering::Relaxed);
    *CAPABILITY_ERROR.lock().unwrap() = None;
    X11.store(
        cc.display_handle().is_ok_and(|h| {
            matches!(
                h.as_raw(),
                RawDisplayHandle::Xlib(_) | RawDisplayHandle::Xcb(_)
            )
        }),
        Ordering::Relaxed,
    );
    #[cfg(target_os = "linux")]
    if let Ok(handle) = cc.display_handle() {
        if let RawDisplayHandle::Wayland(_) = handle.as_raw() {
            WAYLAND.store(true, Ordering::Relaxed);
            let result = cc
                .gl
                .clone()
                .ok_or_else(|| {
                    anyhow::anyhow!("Wayland passive overlays require the native Glow renderer")
                })
                .and_then(|_| wayland::initialize(&cc.egui_ctx));
            if let Err(error) = result {
                WAYLAND_ERROR.store(true, Ordering::Relaxed);
                *CAPABILITY_ERROR.lock().unwrap() = Some(format!(
                    "Wayland passive notifications unavailable: {error:#}"
                ));
                eprintln!("Wayland passive overlays unavailable: {error:#}");
            }
        }
    }
}
pub fn shutdown() {
    #[cfg(target_os = "linux")]
    wayland::shutdown();
    POLICY.with(|p| {
        p.borrow_mut().take();
    });
}
pub fn foreground_game_active() -> Option<bool> {
    #[cfg(target_os = "linux")]
    if WAYLAND.load(Ordering::Relaxed) {
        return wayland::foreground_game_active();
    }
    None
}
pub fn wayland_outputs() -> Vec<egui::Rect> {
    #[cfg(target_os = "linux")]
    if WAYLAND.load(Ordering::Relaxed) {
        return wayland::outputs();
    }
    Vec::new()
}
pub fn capability_error() -> Option<String> {
    CAPABILITY_ERROR.lock().unwrap().clone()
}

impl Policy {
    fn new() -> Option<Self> {
        let (connection, screen) = RustConnection::connect(None).ok()?;
        let root = connection.setup().roots[screen].root;
        Some(Self {
            connection,
            root,
            ready: HashMap::new(),
        })
    }
    fn atom(&self, name: &str) -> anyhow::Result<u32> {
        Ok(self
            .connection
            .intern_atom(false, name.as_bytes())?
            .reply()?
            .atom)
    }
    fn prepare(&mut self, title: &str) -> anyhow::Result<bool> {
        let pid = self.atom("_NET_WM_PID")?;
        let name = self.atom("_NET_WM_NAME")?;
        let utf8 = self.atom("UTF8_STRING")?;
        for window in self.connection.query_tree(self.root)?.reply()?.children {
            let owner = self
                .connection
                .get_property(false, window, pid, AtomEnum::CARDINAL, 0, 1)?
                .reply()?;
            if owner.value32().and_then(|mut i| i.next()) != Some(std::process::id()) {
                continue;
            }
            let text = self
                .connection
                .get_property(false, window, name, utf8, 0, 1024)?
                .reply()?;
            if text.value != title.as_bytes() {
                continue;
            }
            // InputHint is present and false. No focus request, focus restoration
            // or input event is ever sent to the foreground application.
            self.connection
                .change_property32(
                    PropMode::REPLACE,
                    window,
                    AtomEnum::WM_HINTS,
                    AtomEnum::WM_HINTS,
                    &[1, 0, 0, 0, 0, 0, 0, 0, 0],
                )?
                .check()?;
            self.connection
                .change_property32(
                    PropMode::REPLACE,
                    window,
                    self.atom("_NET_WM_USER_TIME")?,
                    AtomEnum::CARDINAL,
                    &[0],
                )?
                .check()?;
            self.connection
                .change_property32(
                    PropMode::REPLACE,
                    window,
                    self.atom("_NET_WM_WINDOW_TYPE")?,
                    AtomEnum::ATOM,
                    &[self.atom("_NET_WM_WINDOW_TYPE_NOTIFICATION")?],
                )?
                .check()?;
            let protocols = self.atom("WM_PROTOCOLS")?;
            let take_focus = self.atom("WM_TAKE_FOCUS")?;
            let old = self
                .connection
                .get_property(false, window, protocols, AtomEnum::ATOM, 0, 128)?
                .reply()?;
            let keep = old
                .value32()
                .into_iter()
                .flatten()
                .filter(|a| *a != take_focus)
                .collect::<Vec<_>>();
            self.connection
                .change_property32(PropMode::REPLACE, window, protocols, AtomEnum::ATOM, &keep)?
                .check()?;
            self.connection.flush()?;
            self.ready.insert(title.to_owned(), window);
            return Ok(true);
        }
        Ok(false)
    }
}

pub fn show(
    ctx: &egui::Context,
    id: ViewportId,
    builder: ViewportBuilder,
    mut contents: impl FnMut(&egui::Context),
) {
    #[cfg(target_os = "linux")]
    if WAYLAND.load(Ordering::Relaxed) {
        if let Err(error) = wayland::show(ctx, id, builder, contents) {
            if !WAYLAND_ERROR.swap(true, Ordering::Relaxed) {
                *CAPABILITY_ERROR.lock().unwrap() =
                    Some(format!("Wayland passive notification: {error:#}"));
                eprintln!("Wayland passive overlay: {error:#}");
            }
        }
        return;
    }
    if !X11.load(Ordering::Relaxed) {
        ctx.show_viewport_immediate(id, builder, |ctx, _| contents(ctx));
        return;
    }
    let title = format!(
        "{} [{id:?}]",
        builder
            .title
            .as_deref()
            .unwrap_or("Observatory notification")
    );
    let exists = ctx.input(|i| i.raw.viewports.contains_key(&id));
    let ready = POLICY.with(|p| {
        let mut p = p.borrow_mut();
        let policy = p.get_or_insert_with(|| Policy::new().expect("X11 display used by native UI"));
        if !exists {
            policy.ready.remove(&title);
        }
        policy.ready.contains_key(&title)
    });
    ctx.show_viewport_immediate(
        id,
        builder.with_title(&title).with_visible(ready),
        |child, _| {
            if !ready {
                let result = POLICY.with(|p| p.borrow_mut().as_mut().unwrap().prepare(&title));
                match result {
                    Ok(true) => {
                        child.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        child.request_repaint();
                    }
                    Ok(false) => child.request_repaint_after(std::time::Duration::from_millis(10)),
                    Err(error) => eprintln!("passive notification surface: {error:#}"),
                }
            }
            contents(child);
        },
    );
}
