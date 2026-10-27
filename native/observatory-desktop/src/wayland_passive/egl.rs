//! Own one EGL display/context for the overlay connection. Never mix SCTK
//! object data or GL resource ownership with winit's independent connection.
use anyhow::{bail, ensure, Result};
use eframe::glow;
use std::{
    ffi::{c_int, c_void, CString},
    ptr,
    rc::Rc,
    sync::Arc,
};
#[link(name = "EGL")]
unsafe extern "C" {
    fn eglGetDisplay(native: *mut c_void) -> *mut c_void;
    fn eglInitialize(display: *mut c_void, major: *mut c_int, minor: *mut c_int) -> u32;
    fn eglTerminate(display: *mut c_void) -> u32;
    fn eglBindAPI(api: u32) -> u32;
    fn eglCreateContext(
        display: *mut c_void,
        config: *mut c_void,
        share: *mut c_void,
        attributes: *const c_int,
    ) -> *mut c_void;
    fn eglDestroyContext(display: *mut c_void, context: *mut c_void) -> u32;
    fn eglGetProcAddress(name: *const i8) -> *const c_void;
    fn eglGetCurrentDisplay() -> *mut c_void;
    fn eglGetCurrentContext() -> *mut c_void;
    fn eglGetCurrentSurface(which: c_int) -> *mut c_void;
    fn eglChooseConfig(
        display: *mut c_void,
        attributes: *const c_int,
        configs: *mut *mut c_void,
        size: c_int,
        count: *mut c_int,
    ) -> u32;
    fn eglCreateWindowSurface(
        display: *mut c_void,
        config: *mut c_void,
        window: *mut c_void,
        attributes: *const c_int,
    ) -> *mut c_void;
    fn eglDestroySurface(display: *mut c_void, surface: *mut c_void) -> u32;
    fn eglMakeCurrent(
        display: *mut c_void,
        draw: *mut c_void,
        read: *mut c_void,
        context: *mut c_void,
    ) -> u32;
    fn eglSwapBuffers(display: *mut c_void, surface: *mut c_void) -> u32;
    fn eglSwapInterval(display: *mut c_void, interval: c_int) -> u32;
    fn eglGetError() -> c_int;
}
#[link(name = "wayland-egl")]
unsafe extern "C" {
    fn wl_egl_window_create(surface: *mut c_void, width: c_int, height: c_int) -> *mut c_void;
    fn wl_egl_window_resize(window: *mut c_void, width: c_int, height: c_int, dx: c_int, dy: c_int);
    fn wl_egl_window_destroy(window: *mut c_void);
}
const NONE: i32 = 0x3038;
fn checked(result: u32, operation: &str) -> Result<()> {
    if result == 0 {
        bail!("{operation}: EGL error 0x{:x}", unsafe { eglGetError() });
    }
    Ok(())
}
pub fn background_updates() -> Result<()> {
    let display = unsafe { eglGetCurrentDisplay() };
    ensure!(!display.is_null(), "Wayland requires EGL");
    checked(
        unsafe { eglSwapInterval(display, 0) },
        "allow background overlay updates",
    )
}
pub struct Display {
    display: *mut c_void,
    context: *mut c_void,
    config: *mut c_void,
    pub gl: Arc<glow::Context>,
    _connection: wayland_client::Connection,
}
impl Display {
    pub fn new(connection: wayland_client::Connection) -> Result<Rc<Self>> {
        let native = connection.backend().display_ptr().cast();
        let saved = Restore::capture();
        unsafe {
            let display = eglGetDisplay(native);
            ensure!(!display.is_null(), "create overlay EGL display");
            checked(
                eglInitialize(display, ptr::null_mut(), ptr::null_mut()),
                "initialize overlay EGL display",
            )?;
            checked(eglBindAPI(0x30A2), "select OpenGL API")?;
            let mut config = ptr::null_mut();
            let mut count = 0;
            let attrs = [
                0x3024, 8, 0x3023, 8, 0x3022, 8, 0x3021, 8, 0x3033, 4, 0x3040, 8, NONE,
            ];
            checked(
                eglChooseConfig(display, attrs.as_ptr(), &mut config, 1, &mut count),
                "choose alpha EGL config",
            )?;
            ensure!(
                count == 1 && !config.is_null(),
                "no alpha-capable EGL OpenGL window config"
            );
            let context = eglCreateContext(
                display,
                config,
                ptr::null_mut(),
                [0x3098, 3, 0x30FB, 3, 0x30FD, 1, NONE].as_ptr(),
            );
            if context.is_null() {
                eglTerminate(display);
                bail!("create overlay OpenGL3.3 context: 0x{:x}", eglGetError());
            }
            if let Err(error) = checked(
                eglMakeCurrent(display, ptr::null_mut(), ptr::null_mut(), context),
                "initialize surfaceless overlay GL context",
            ) {
                eglDestroyContext(display, context);
                eglTerminate(display);
                return Err(error);
            }
            let gl = Arc::new(glow::Context::from_loader_function(|name| {
                let name = CString::new(name).unwrap();
                eglGetProcAddress(name.as_ptr())
            }));
            let display = Rc::new(Self {
                display,
                context,
                config,
                gl,
                _connection: connection,
            });
            drop(saved);
            Ok(display)
        }
    }
    pub fn offscreen(&self) -> Result<Restore> {
        self.bind(ptr::null_mut())
    }
    fn bind(&self, surface: *mut c_void) -> Result<Restore> {
        let saved = Restore::capture();
        checked(
            unsafe { eglMakeCurrent(self.display, surface, surface, self.context) },
            "make layer EGL context current",
        )?;
        if !surface.is_null() {
            checked(
                unsafe { eglSwapInterval(self.display, 0) },
                "nonblocking layer swaps",
            )?;
        }
        Ok(saved)
    }
}
impl Drop for Display {
    fn drop(&mut self) {
        unsafe {
            eglDestroyContext(self.display, self.context);
            eglTerminate(self.display);
        }
    }
}
pub struct Surface {
    display: Rc<Display>,
    surface: *mut c_void,
    window: *mut c_void,
    pub size: [u32; 2],
}
impl Surface {
    pub fn new(
        display: Rc<Display>,
        surface: &wayland_client::protocol::wl_surface::WlSurface,
        size: [u32; 2],
    ) -> Result<Self> {
        unsafe {
            use wayland_client::Proxy;
            let surface = surface.id().as_ptr();
            ensure!(!surface.is_null(), "Wayland surface destroyed");
            let window = wl_egl_window_create(surface.cast(), size[0] as i32, size[1] as i32);
            ensure!(!window.is_null(), "create Wayland EGL window");
            let egl_surface =
                eglCreateWindowSurface(display.display, display.config, window, [NONE].as_ptr());
            if egl_surface.is_null() {
                wl_egl_window_destroy(window);
                bail!("create EGL layer surface: 0x{:x}", eglGetError());
            }
            Ok(Self {
                display,
                surface: egl_surface,
                window,
                size,
            })
        }
    }
    pub fn resize(&mut self, size: [u32; 2]) {
        if self.size != size {
            unsafe { wl_egl_window_resize(self.window, size[0] as i32, size[1] as i32, 0, 0) };
            self.size = size;
        }
    }
    pub fn bind(&self) -> Result<Restore> {
        self.display.bind(self.surface)
    }
    pub fn present(&self) -> Result<()> {
        checked(
            unsafe { eglSwapBuffers(self.display.display, self.surface) },
            "present passive layer",
        )
    }
}
impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            eglDestroySurface(self.display.display, self.surface);
            wl_egl_window_destroy(self.window);
        }
    }
}
pub struct Restore {
    display: *mut c_void,
    draw: *mut c_void,
    read: *mut c_void,
    context: *mut c_void,
}
impl Restore {
    fn capture() -> Self {
        unsafe {
            Self {
                display: eglGetCurrentDisplay(),
                draw: eglGetCurrentSurface(0x3059),
                read: eglGetCurrentSurface(0x305a),
                context: eglGetCurrentContext(),
            }
        }
    }
}
impl Drop for Restore {
    fn drop(&mut self) {
        if !self.display.is_null() {
            let _ = checked(
                unsafe { eglMakeCurrent(self.display, self.draw, self.read, self.context) },
                "restore application EGL context",
            );
        }
    }
}
