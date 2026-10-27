//! The source BioInsights browser mode is a child control in the plugin page.
//! GtkPlug embeds WebKitGTK in the native X11/XWayland client area. Libraries
//! are loaded at runtime so headless replay has no GTK dependency.
use anyhow::{ensure, Context, Result};
use eframe::egui;
use libloading::Library;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{
    ffi::{c_char, c_int, c_ulong, c_void, CStr, CString},
    sync::mpsc,
};
use x11rb::{
    connection::Connection,
    protocol::xproto::{AtomEnum, ConfigureWindowAux, ConnectionExt},
    rust_connection::RustConnection,
};
type Ptr = *mut c_void;
struct Api {
    _gtk: Library,
    _webkit: Library,
    _gobject: Library,
    init: unsafe extern "C" fn(*mut c_int, *mut *mut *mut c_char) -> c_int,
    plug: unsafe extern "C" fn(c_ulong) -> Ptr,
    view: unsafe extern "C" fn() -> Ptr,
    add: unsafe extern "C" fn(Ptr, Ptr),
    show: unsafe extern "C" fn(Ptr),
    hide: unsafe extern "C" fn(Ptr),
    destroy: unsafe extern "C" fn(Ptr),
    size: unsafe extern "C" fn(Ptr, c_int, c_int),
    window: unsafe extern "C" fn(Ptr) -> Ptr,
    xid: unsafe extern "C" fn(Ptr) -> c_ulong,
    pending: unsafe extern "C" fn() -> c_int,
    iteration: unsafe extern "C" fn(c_int) -> c_int,
    load: unsafe extern "C" fn(Ptr, *const c_char, *const c_char),
    evaluate: unsafe extern "C" fn(
        Ptr,
        *const c_char,
        isize,
        *const c_char,
        *const c_char,
        Ptr,
        Option<unsafe extern "C" fn(Ptr, Ptr, Ptr)>,
        Ptr,
    ),
    connect: unsafe extern "C" fn(
        Ptr,
        *const c_char,
        unsafe extern "C" fn(),
        Ptr,
        Option<unsafe extern "C" fn(Ptr, Ptr)>,
        u32,
    ) -> c_ulong,
    action: unsafe extern "C" fn(Ptr) -> Ptr,
    request: unsafe extern "C" fn(Ptr) -> Ptr,
    uri: unsafe extern "C" fn(Ptr) -> *const c_char,
    ignore: unsafe extern "C" fn(Ptr),
}
impl Api {
    unsafe fn new() -> Result<Self> {
        unsafe {
            let gtk = Library::new("libgtk-3.so.0")
                .context("Install libgtk-3-0 for the embedded browser")?;
            let webkit = Library::new("libwebkit2gtk-4.1.so.0")
                .context("Install libwebkit2gtk-4.1-0 for the embedded BioInsights browser")?;
            let gobject = Library::new("libgobject-2.0.so.0")?;
            macro_rules! sym {
                ($lib:ident,$name:literal) => {
                    *$lib.get(concat!($name, "\0").as_bytes())?
                };
            }
            Ok(Self {
                init: sym!(gtk, "gtk_init_check"),
                plug: sym!(gtk, "gtk_plug_new"),
                view: sym!(webkit, "webkit_web_view_new"),
                add: sym!(gtk, "gtk_container_add"),
                show: sym!(gtk, "gtk_widget_show_all"),
                hide: sym!(gtk, "gtk_widget_hide"),
                destroy: sym!(gtk, "gtk_widget_destroy"),
                size: sym!(gtk, "gtk_widget_set_size_request"),
                window: sym!(gtk, "gtk_widget_get_window"),
                xid: sym!(gtk, "gdk_x11_window_get_xid"),
                pending: sym!(gtk, "gtk_events_pending"),
                iteration: sym!(gtk, "gtk_main_iteration_do"),
                load: sym!(webkit, "webkit_web_view_load_html"),
                evaluate: sym!(webkit, "webkit_web_view_evaluate_javascript"),
                connect: sym!(gobject, "g_signal_connect_data"),
                action: sym!(
                    webkit,
                    "webkit_navigation_policy_decision_get_navigation_action"
                ),
                request: sym!(webkit, "webkit_navigation_action_get_request"),
                uri: sym!(webkit, "webkit_uri_request_get_uri"),
                ignore: sym!(webkit, "webkit_policy_decision_ignore"),
                _gtk: gtk,
                _webkit: webkit,
                _gobject: gobject,
            })
        }
    }
}
struct Navigation {
    api: *const Api,
    tx: mpsc::Sender<String>,
}
unsafe extern "C" fn decide(_view: Ptr, decision: Ptr, kind: c_int, data: Ptr) -> c_int {
    // Only navigation decisions have a WebKitNavigationAction. Responses and
    // subresources continue through WebKit without reinterpretation.
    if kind != 0 && kind != 1 {
        return 0;
    }
    unsafe {
        let nav = &*(data.cast::<Navigation>());
        let api = &*nav.api;
        let action = (api.action)(decision);
        if action.is_null() {
            return 0;
        }
        let request = (api.request)(action);
        if request.is_null() {
            return 0;
        }
        let uri = (api.uri)(request);
        if uri.is_null() {
            return 0;
        }
        let uri = CStr::from_ptr(uri).to_string_lossy();
        if uri.starts_with("observatory://bioinsights/")
            || uri.starts_with("https://")
            || uri.starts_with("http://")
        {
            (api.ignore)(decision);
            let _ = nav.tx.send(uri.into_owned());
            return 1;
        }
    }
    0
}
pub struct Browser {
    api: Box<Api>,
    plug: Ptr,
    view: Ptr,
    _navigation: Box<Navigation>,
    rx: mpsc::Receiver<String>,
    connection: RustConnection,
    root: u32,
    root_parent: u32,
    child: u32,
    parent: u32,
    html: std::sync::Arc<String>,
    visible: bool,
    route: String,
    chrome: (bool, bool),
    focus: Option<(u32, u32, bool)>,
}
fn find_parent(connection: &RustConnection, root: u32, preferred: u32, title: &str) -> Result<u32> {
    let atom =
        |name: &[u8]| -> Result<u32> { Ok(connection.intern_atom(false, name)?.reply()?.atom) };
    let pid = atom(b"_NET_WM_PID")?;
    let name = atom(b"_NET_WM_NAME")?;
    let utf8 = atom(b"UTF8_STRING")?;
    let matches = |window: u32| -> bool {
        let Some(owner) = connection
            .get_property(false, window, pid, AtomEnum::CARDINAL, 0, 1)
            .ok()
            .and_then(|c| c.reply().ok())
        else {
            return false;
        };
        owner.value32().and_then(|mut v| v.next()) == Some(std::process::id())
            && connection
                .get_property(false, window, name, utf8, 0, 1024)
                .ok()
                .and_then(|c| c.reply().ok())
                .is_some_and(|r| r.value == title.as_bytes())
    };
    if matches(preferred) {
        return Ok(preferred);
    }
    // Reparenting WMs put the application client below a decoration frame.
    // EWMH names the actual client; root children alone name only those frames.
    let clients = atom(b"_NET_CLIENT_LIST")?;
    if let Ok(reply) = connection
        .get_property(false, root, clients, AtomEnum::WINDOW, 0, 4096)?
        .reply()
    {
        if let Some(windows) = reply.value32() {
            for window in windows {
                if matches(window) {
                    return Ok(window);
                }
            }
        }
    }
    // Xvfb and small WMs need not publish EWMH. Search their actual hierarchy,
    // bounded independently of other clients' depth/count and tolerant of races.
    let mut pending = std::collections::VecDeque::from([(root, 0u8)]);
    for _ in 0..4096 {
        let Some((window, depth)) = pending.pop_front() else {
            break;
        };
        if window != root && matches(window) {
            return Ok(window);
        }
        if depth < 32 {
            if let Some(tree) = connection
                .query_tree(window)
                .ok()
                .and_then(|c| c.reply().ok())
            {
                pending.extend(tree.children.into_iter().map(|child| (child, depth + 1)));
            }
        }
    }
    anyhow::bail!("Embedded browser parent has not been created yet")
}
impl Browser {
    pub fn root_window(cc: &eframe::CreationContext<'_>) -> Option<u32> {
        match cc.window_handle().ok()?.as_raw() {
            RawWindowHandle::Xlib(h) => Some(h.window as u32),
            RawWindowHandle::Xcb(h) => Some(h.window.get()),
            _ => None,
        }
    }
    pub fn new(root_parent: u32) -> Result<Self> {
        let (connection, screen) = RustConnection::connect(None)?;
        let root = connection.setup().roots[screen].root;
        unsafe {
            // The application selected the verified X11 display before creating
            // its window; GTK must use that same backend for GtkPlug.
            std::env::set_var("GDK_BACKEND", "x11");
            let api = Box::new(Api::new()?);
            ensure!(
                (api.init)(std::ptr::null_mut(), std::ptr::null_mut()) != 0,
                "GTK could not connect to the X11 display"
            );
            let plug = (api.plug)(0);
            ensure!(!plug.is_null(), "GTK could not create the embedded child");
            let view = (api.view)();
            ensure!(
                !view.is_null(),
                "WebKit could not create the embedded browser"
            );
            (api.add)(plug, view);
            let (tx, rx) = mpsc::channel();
            let navigation = Box::new(Navigation { api: &*api, tx });
            let callback = std::mem::transmute::<
                unsafe extern "C" fn(Ptr, Ptr, c_int, Ptr) -> c_int,
                unsafe extern "C" fn(),
            >(decide);
            (api.connect)(
                view,
                c"decide-policy".as_ptr(),
                callback,
                (&*navigation as *const Navigation).cast_mut().cast(),
                None,
                0,
            );
            (api.show)(plug);
            let window = (api.window)(plug);
            ensure!(!window.is_null(), "GTK child was not realized");
            let child = (api.xid)(window) as u32;
            connection
                .reparent_window(child, root_parent, 0, 0)?
                .check()?;
            (api.hide)(plug);
            Ok(Self {
                api,
                plug,
                view,
                _navigation: navigation,
                rx,
                connection,
                root,
                root_parent,
                child,
                parent: root_parent,
                html: Default::default(),
                visible: false,
                route: String::new(),
                chrome: (false, false),
                focus: None,
            })
        }
    }
    fn parent(&self, ctx: &egui::Context, title: &str) -> Result<u32> {
        if ctx.viewport_id() == egui::ViewportId::ROOT {
            return Ok(self.root_parent);
        }
        find_parent(&self.connection, self.root, self.parent, title)
    }
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        html: &std::sync::Arc<String>,
        title: &str,
        route: &str,
        hide_buttons: bool,
    ) -> Result<()> {
        let rect = ui.available_rect_before_wrap();
        let ppp = ui.ctx().pixels_per_point();
        let parent = self.parent(ui.ctx(), title)?;
        if self.parent != parent {
            self.connection
                .reparent_window(self.child, parent, 0, 0)?
                .check()?;
            self.parent = parent;
        }
        let width = (rect.width() * ppp).round().max(1.0) as u32;
        let height = (rect.height() * ppp).round().max(1.0) as u32;
        unsafe {
            (self.api.size)(self.view, width as c_int, height as c_int);
            (self.api.size)(self.plug, width as c_int, height as c_int);
        }
        self.connection.configure_window(
            self.child,
            &ConfigureWindowAux::new()
                .x((rect.left() * ppp).round() as i32)
                .y((rect.top() * ppp).round() as i32)
                .width(width)
                .height(height),
        )?;
        self.connection.map_window(self.child)?;
        let is_popout = title.ends_with(" Popout");
        if !std::sync::Arc::ptr_eq(&self.html, html) {
            if self.html.is_empty() || self.route != route {
                let boot = include_str!("embedded_browser.js")
                    .replace("$$POPOUT$$", if is_popout { "true" } else { "false" })
                    .replace(
                        "$$HIDE_BUTTONS$$",
                        if hide_buttons { "true" } else { "false" },
                    );
                let document = format!("{}<script>{}</script>", html, boot);
                let value = CString::new(document)?;
                unsafe {
                    (self.api.load)(self.view, value.as_ptr(), c"about:blank".as_ptr());
                }
            } else {
                let document = serde_json::to_string(html.as_str())?;
                self.evaluate(
                    &include_str!("embedded_browser_update.js").replace("$$DOCUMENT$$", &document),
                )?;
            }
            self.html = html.clone();
            self.route = route.to_owned();
            self.chrome = (is_popout, hide_buttons);
        }
        if self.chrome != (is_popout, hide_buttons) {
            self.evaluate(&format!("window.__observatoryPopout={is_popout};document.querySelector('.buttonheader').style.display={};",if hide_buttons{"'none'"}else{"''"}))?;
            self.chrome = (is_popout, hide_buttons);
        }

        unsafe {
            (self.api.show)(self.plug);
        }
        self.connection.flush()?;
        self.visible = true;
        ui.allocate_rect(rect, egui::Sense::hover());
        Ok(())
    }
    fn focus_is_owned(&self, mut window: u32) -> bool {
        // GtkPlug has nested input windows. Moving focus into one of those is
        // still focus inside this source plugin window, not Deactivate.
        for _ in 0..64 {
            if window == self.parent || window == self.child {
                return true;
            }
            if window <= 1 || window == self.root {
                return false;
            }
            let Some(parent) = self
                .connection
                .query_tree(window)
                .ok()
                .and_then(|reply| reply.reply().ok())
                .map(|reply| reply.parent)
            else {
                return false;
            };
            if parent == window {
                return false;
            }
            window = parent;
        }
        false
    }
    fn sync_owner_focus(&mut self) {
        let Some(window) = self
            .connection
            .get_input_focus()
            .ok()
            .and_then(|reply| reply.reply().ok())
            .map(|reply| reply.focus)
        else {
            return;
        };
        if self
            .focus
            .is_some_and(|(parent, focus, _)| parent == self.parent && focus == window)
        {
            return;
        }
        let owned = self.focus_is_owned(window);
        if self.focus.is_some_and(|(_, _, owned)| owned) && !owned {
            // A manually reparented GtkPlug has no GtkSocket/XEmbed focus
            // bridge, so WebKit otherwise never receives the real owner blur.
            let _ = self.evaluate("window.dispatchEvent(new Event('blur'));");
        }
        self.focus = Some((self.parent, window, owned));
    }
    pub fn begin_frame(&mut self) {
        self.sync_owner_focus();
        self.visible = false;
        for _ in 0..64 {
            unsafe {
                if (self.api.pending)() == 0 {
                    break;
                }
                (self.api.iteration)(0);
            }
        }
    }
    pub fn reattach(&mut self) -> Result<()> {
        unsafe {
            (self.api.hide)(self.plug);
        }
        self.connection
            .reparent_window(self.child, self.root_parent, 0, 0)?
            .check()?;
        self.connection.flush()?;
        self.parent = self.root_parent;
        self.visible = false;
        Ok(())
    }
    pub fn end_frame(&self) {
        if !self.visible {
            unsafe {
                (self.api.hide)(self.plug);
            }
        }
    }
    fn evaluate(&self, script: &str) -> Result<()> {
        let script = CString::new(script)?;
        unsafe {
            (self.api.evaluate)(
                self.view,
                script.as_ptr(),
                -1,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
            );
        }
        Ok(())
    }
    pub fn zoom_menu(&self, ctx: &egui::Context) {
        let to_css = |c: egui::Color32| format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b());
        let background = to_css(ctx.style().visuals.window_fill);
        let foreground = to_css(ctx.style().visuals.text_color());
        let script = include_str!("embedded_zoom.js")
            .replace("$$BACKGROUND$$", &background)
            .replace("$$FOREGROUND$$", &foreground);
        let _ = self.evaluate(&script);
    }
    pub fn navigation(&self) -> Vec<String> {
        self.rx.try_iter().take(128).collect()
    }
}
impl Drop for Browser {
    fn drop(&mut self) {
        unsafe {
            (self.api.destroy)(self.plug);
            for _ in 0..16 {
                if (self.api.pending)() == 0 {
                    break;
                }
                (self.api.iteration)(0);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use x11rb::{
        protocol::xproto::{CreateWindowAux, PropMode, WindowClass},
        wrapper::ConnectionExt as _,
    };
    #[test]
    #[ignore = "requires an owned private X11 display; run embedded_browser_parent_test.py"]
    fn locates_real_client_below_reparenting_frames_with_and_without_ewmh() {
        let (connection, screen) = RustConnection::connect(None).unwrap();
        let root = connection.setup().roots[screen].root;
        let atom = |s: &[u8]| {
            connection
                .intern_atom(false, s)
                .unwrap()
                .reply()
                .unwrap()
                .atom
        };
        let pid = atom(b"_NET_WM_PID");
        let name = atom(b"_NET_WM_NAME");
        let utf8 = atom(b"UTF8_STRING");
        let clients = atom(b"_NET_CLIENT_LIST");
        let mut parent = root;
        let mut windows = Vec::new();
        for _ in 0..3 {
            let window = connection.generate_id().unwrap();
            connection
                .create_window(
                    x11rb::COPY_DEPTH_FROM_PARENT,
                    window,
                    parent,
                    0,
                    0,
                    100,
                    100,
                    0,
                    WindowClass::INPUT_OUTPUT,
                    0,
                    &CreateWindowAux::new(),
                )
                .unwrap()
                .check()
                .unwrap();
            windows.push(window);
            parent = window;
        }
        let title = "BioInsights source owner — reparented";
        connection
            .change_property32(
                PropMode::REPLACE,
                parent,
                pid,
                AtomEnum::CARDINAL,
                &[std::process::id()],
            )
            .unwrap()
            .check()
            .unwrap();
        connection
            .change_property8(PropMode::REPLACE, parent, name, utf8, title.as_bytes())
            .unwrap()
            .check()
            .unwrap();
        connection
            .delete_property(root, clients)
            .unwrap()
            .check()
            .unwrap();
        assert_eq!(find_parent(&connection, root, root, title).unwrap(), parent);
        connection
            .change_property32(
                PropMode::REPLACE,
                root,
                clients,
                AtomEnum::WINDOW,
                &[parent],
            )
            .unwrap()
            .check()
            .unwrap();
        assert_eq!(find_parent(&connection, root, root, title).unwrap(), parent);
        assert_eq!(
            find_parent(&connection, root, parent, title).unwrap(),
            parent
        );
        connection
            .delete_property(root, clients)
            .unwrap()
            .check()
            .unwrap();
        connection
            .destroy_window(windows[0])
            .unwrap()
            .check()
            .unwrap();
        assert!(
            find_parent(&connection, root, parent, title).is_err(),
            "a destroyed owner cannot be reused"
        );
    }
}
