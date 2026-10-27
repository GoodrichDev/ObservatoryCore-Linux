//! Non-activating Wayland overlays. Each surface has the layer-shell role,
//! keyboard interactivity NONE and an empty pointer/touch input region before
//! its first buffer commit. An owned EGL context keeps compositor proxies and
//! graphics resources independent of winit; root GL state is restored after drawing.
#[path = "wayland_passive/egl.rs"]
mod egl;
use anyhow::{ensure, Context, Result};
use eframe::{
    egui::{self, Pos2, Rect, Vec2, ViewportId},
    glow::{self, HasContext},
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    os::fd::{AsFd, AsRawFd},
    rc::Rc,
    sync::Arc,
};
use wayland_client::{
    delegate_noop,
    protocol::{wl_compositor, wl_output, wl_region, wl_registry, wl_surface},
    Connection, Dispatch, EventQueue, Proxy, QueueHandle,
};
use wayland_protocols::xdg::xdg_output::zv1::client::{zxdg_output_manager_v1, zxdg_output_v1};
use wayland_protocols_wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1, zwlr_foreign_toplevel_manager_v1,
};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};
thread_local! {static DRIVER:RefCell<Option<Driver>>=const{RefCell::new(None)};}

#[derive(Clone)]
struct Output {
    proxy: wl_output::WlOutput,
    logical: Rect,
    pixels: [i32; 2],
    scale: i32,
    xdg: Option<zxdg_output_v1::ZxdgOutputV1>,
}
struct Layer {
    surface: wl_surface::WlSurface,
    role: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    egl: Option<egl::Surface>,
    output: u32,
    logical_size: [u32; 2],
    scale: i32,
    configured: bool,
    closed: bool,
    last_seen: u64,
}
impl Drop for Layer {
    fn drop(&mut self) {
        self.egl.take();
        self.role.destroy();
        self.surface.destroy();
    }
}
#[derive(Default)]
struct ForeignWindow {
    title: String,
    app_id: String,
    active: bool,
}
#[derive(Default)]
struct State {
    foreign_manager: Option<zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1>,
    windows: HashMap<u32, ForeignWindow>,
    compositor: Option<wl_compositor::WlCompositor>,
    shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    output_manager: Option<zxdg_output_manager_v1::ZxdgOutputManagerV1>,
    outputs: BTreeMap<u32, Output>,
    layers: HashMap<ViewportId, Layer>,
}
// Drop owned EGL before disconnecting the Wayland client that owns its surfaces.
struct Driver {
    painter: eframe::egui_glow::Painter,
    state: State,
    egl: Rc<egl::Display>,
    context: egui::Context,
    fonts: egui::FontDefinitions,
    queue: EventQueue<State>,
    connection: Connection,
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => match interface.as_str() {
                "zwlr_foreign_toplevel_manager_v1" => {
                    state.foreign_manager = Some(registry.bind(name, version.min(3), qh, ()))
                }
                "wl_compositor" => {
                    state.compositor = Some(registry.bind(name, version.min(4), qh, ()))
                }
                "zwlr_layer_shell_v1" => {
                    state.shell = Some(registry.bind(name, version.min(4), qh, ()))
                }
                "zxdg_output_manager_v1" => {
                    state.output_manager = Some(registry.bind(name, version.min(3), qh, ()))
                }
                "wl_output" => {
                    state.outputs.insert(
                        name,
                        Output {
                            proxy: registry.bind(name, version.min(4), qh, name),
                            logical: Rect::from_min_size(Pos2::ZERO, Vec2::ZERO),
                            pixels: [0, 0],
                            scale: 1,
                            xdg: None,
                        },
                    );
                }
                _ => {}
            },
            wl_registry::Event::GlobalRemove { name } => {
                state.layers.retain(|_, l| l.output != name);
                state.outputs.remove(&name);
            }
            _ => {}
        }
    }
}
impl Dispatch<wl_output::WlOutput, u32> for State {
    fn event(
        state: &mut Self,
        _: &wl_output::WlOutput,
        event: wl_output::Event,
        id: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let Some(output) = state.outputs.get_mut(id) else {
            return;
        };
        match event {
            wl_output::Event::Geometry { x, y, .. } => {
                if output.xdg.is_none() {
                    output.logical =
                        Rect::from_min_size(Pos2::new(x as f32, y as f32), output.logical.size());
                }
            }
            wl_output::Event::Mode {
                width,
                height,
                flags,
                ..
            } if flags
                .into_result()
                .is_ok_and(|f| f.contains(wl_output::Mode::Current)) =>
            {
                output.pixels = [width, height];
            }
            wl_output::Event::Scale { factor } => output.scale = factor.max(1),
            wl_output::Event::Done if output.xdg.is_none() => {
                output.logical = Rect::from_min_size(
                    output.logical.min,
                    Vec2::new(output.pixels[0] as f32, output.pixels[1] as f32)
                        / output.scale as f32,
                );
            }
            _ => {}
        }
    }
}
impl Dispatch<zxdg_output_v1::ZxdgOutputV1, u32> for State {
    fn event(
        state: &mut Self,
        _: &zxdg_output_v1::ZxdgOutputV1,
        event: zxdg_output_v1::Event,
        id: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let Some(output) = state.outputs.get_mut(id) {
            match event {
                zxdg_output_v1::Event::LogicalPosition { x, y } => {
                    output.logical =
                        Rect::from_min_size(Pos2::new(x as f32, y as f32), output.logical.size())
                }
                zxdg_output_v1::Event::LogicalSize { width, height } => {
                    output.logical = Rect::from_min_size(
                        output.logical.min,
                        Vec2::new(width as f32, height as f32),
                    )
                }
                _ => {}
            }
        }
    }
}
impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ViewportId> for State {
    fn event(
        state: &mut Self,
        role: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        id: &ViewportId,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let Some(layer) = state.layers.get_mut(id) {
            match event {
                zwlr_layer_surface_v1::Event::Configure { serial, .. } => {
                    role.ack_configure(serial);
                    layer.configured = true;
                }
                zwlr_layer_surface_v1::Event::Closed => layer.closed = true,
                _ => {}
            }
        }
    }
}
impl Dispatch<zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, ()> for State {
    wayland_client::event_created_child!(State,zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,[0=>(zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1,())]);
    fn event(
        state: &mut Self,
        _: &zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,
        event: zwlr_foreign_toplevel_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } => {
                state
                    .windows
                    .entry(toplevel.id().protocol_id())
                    .or_default();
            }
            zwlr_foreign_toplevel_manager_v1::Event::Finished => {
                state.foreign_manager = None;
                state.windows.clear();
            }
            _ => {}
        }
    }
}
fn elite_window(title: &str, app_id: &str) -> bool {
    [title, app_id].iter().any(|name| {
        let name = name.to_ascii_lowercase();
        name.contains("elite - dangerous")
            || name.contains("elite dangerous")
            || name.contains("elitedangerous")
    })
}
impl State {
    fn active_game(&self) -> Option<bool> {
        self.foreign_manager.as_ref().map(|_| {
            self.windows
                .values()
                .any(|w| w.active && elite_window(&w.title, &w.app_id))
        })
    }
}
impl Dispatch<zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1, ()> for State {
    fn event(
        state: &mut Self,
        handle: &zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1,
        event: zwlr_foreign_toplevel_handle_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let id = handle.id().protocol_id();
        if let zwlr_foreign_toplevel_handle_v1::Event::Closed = event {
            state.windows.remove(&id);
            handle.destroy();
            return;
        }
        let window = state.windows.entry(id).or_default();
        match event {
            zwlr_foreign_toplevel_handle_v1::Event::Title { title } => window.title = title,
            zwlr_foreign_toplevel_handle_v1::Event::AppId { app_id } => window.app_id = app_id,
            zwlr_foreign_toplevel_handle_v1::Event::State { state } => {
                window.active = state
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .any(|s| u32::from_ne_bytes(*s) == 2)
            }
            _ => {}
        }
    }
}
delegate_noop!(State:ignore wl_compositor::WlCompositor);
delegate_noop!(State:ignore wl_surface::WlSurface);
delegate_noop!(State:ignore wl_region::WlRegion);
delegate_noop!(State:ignore zwlr_layer_shell_v1::ZwlrLayerShellV1);
delegate_noop!(State:ignore zxdg_output_manager_v1::ZxdgOutputManagerV1);

impl Driver {
    /// `display` remains owned by eframe. shutdown() runs before that owner exits.
    fn new(root: &egui::Context) -> Result<Self> {
        let connection = Connection::connect_to_env().context("open passive Wayland connection")?;
        let mut queue = connection.new_event_queue();
        let qh = queue.handle();
        connection.display().get_registry(&qh, ());
        let mut state = State::default();
        queue
            .roundtrip(&mut state)
            .context("read Wayland layer globals")?;
        ensure!(
            state.compositor.is_some(),
            "Wayland compositor does not expose wl_compositor"
        );
        ensure!(state.shell.is_some(),"Wayland compositor does not support wlr-layer-shell; passive notifications cannot use xdg_toplevel safely");
        if let Some(manager) = &state.output_manager {
            for (id, output) in &mut state.outputs {
                output.xdg = Some(manager.get_xdg_output(&output.proxy, &qh, *id));
            }
        }
        queue
            .roundtrip(&mut state)
            .context("read Wayland logical outputs")?;
        ensure!(!state.outputs.is_empty(), "Wayland display has no outputs");
        egl::background_updates()?;
        let context = egui::Context::default();
        let repaint = root.clone();
        context.set_request_repaint_callback(move |info| repaint.request_repaint_after(info.delay));
        let egl = egl::Display::new(connection.clone())?;
        let saved = egl.offscreen()?;
        let painter = eframe::egui_glow::Painter::new(egl.gl.clone(), "", None, false)
            .map_err(|e| anyhow::anyhow!("create passive layer painter: {e}"))?;
        drop(saved);
        Ok(Self {
            egl,
            connection,
            queue,
            state,
            context,
            fonts: egui::FontDefinitions::default(),
            painter,
        })
    }
    fn prepare(
        &mut self,
        id: ViewportId,
        builder: &egui::ViewportBuilder,
        pass: u64,
    ) -> Result<(egui::Context, egui::RawInput)> {
        self.poll()?;
        let position = builder.position.unwrap_or(Pos2::ZERO);
        let size = builder.inner_size.unwrap_or(Vec2::new(355.0, 145.0));
        let output_id = self
            .state
            .outputs
            .iter()
            .min_by(|(_, a), (_, b)| {
                a.logical
                    .distance_sq_to_pos(position)
                    .total_cmp(&b.logical.distance_sq_to_pos(position))
            })
            .map(|(id, _)| *id)
            .context("Wayland output removed")?;
        let output = self.state.outputs[&output_id].clone();
        if self
            .state
            .layers
            .get(&id)
            .is_some_and(|l| l.closed || l.output != output_id)
        {
            self.state.layers.remove(&id);
        }
        let logical_size = [
            size.x.ceil().clamp(1.0, 32768.0) as u32,
            size.y.ceil().clamp(1.0, 32768.0) as u32,
        ];
        if !self.state.layers.contains_key(&id) {
            let qh = self.queue.handle();
            let surface = self
                .state
                .compositor
                .as_ref()
                .unwrap()
                .create_surface(&qh, ());
            let empty = self
                .state
                .compositor
                .as_ref()
                .unwrap()
                .create_region(&qh, ());
            surface.set_input_region(Some(&empty));
            empty.destroy();
            let namespace = format!(
                "Observatory passive {}",
                builder.title.as_deref().unwrap_or("notification")
            );
            let role = self.state.shell.as_ref().unwrap().get_layer_surface(
                &surface,
                Some(&output.proxy),
                zwlr_layer_shell_v1::Layer::Overlay,
                namespace,
                &qh,
                id,
            );
            role.set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::None);
            role.set_exclusive_zone(-1);
            role.set_anchor(
                zwlr_layer_surface_v1::Anchor::Top | zwlr_layer_surface_v1::Anchor::Left,
            );
            self.state.layers.insert(
                id,
                Layer {
                    surface,
                    role,
                    egl: None,
                    output: output_id,
                    logical_size,
                    scale: output.scale,
                    configured: false,
                    closed: false,
                    last_seen: pass,
                },
            );
        }
        let layer = self.state.layers.get_mut(&id).unwrap();
        layer.last_seen = pass;
        layer.logical_size = logical_size;
        layer.scale = output.scale;
        layer.surface.set_buffer_scale(output.scale);
        layer.role.set_size(logical_size[0], logical_size[1]);
        layer.role.set_margin(
            (position.y - output.logical.min.y).round() as i32,
            0,
            0,
            (position.x - output.logical.min.x).round() as i32,
        );
        layer.surface.commit();
        if !layer.configured {
            self.queue
                .roundtrip(&mut self.state)
                .context("configure passive layer")?;
        }
        self.connection.flush()?;
        let layer = &self.state.layers[&id];
        ensure!(
            layer.configured && !layer.closed,
            "Wayland layer was not configured"
        );
        let mut raw = egui::RawInput {
            viewport_id: id,
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            focused: false,
            ..Default::default()
        };
        for (id, layer) in &self.state.layers {
            raw.viewports.insert(
                *id,
                egui::ViewportInfo {
                    parent: Some(ViewportId::ROOT),
                    native_pixels_per_point: Some(layer.scale as f32),
                    focused: Some(false),
                    ..Default::default()
                },
            );
        }
        Ok((self.context.clone(), raw))
    }
    fn paint(&mut self, id: ViewportId, frame: &Frame) -> Result<()> {
        let Some(layer) = self.state.layers.get_mut(&id) else {
            return Ok(());
        };
        let size = [
            (layer.logical_size[0] * layer.scale as u32).max(1),
            (layer.logical_size[1] * layer.scale as u32).max(1),
        ];
        if layer.egl.is_none() {
            layer.egl = Some(egl::Surface::new(self.egl.clone(), &layer.surface, size)?);
        }
        let surface = layer.egl.as_mut().unwrap();
        surface.resize(size);
        let gl = self.painter.gl().clone();
        let saved = surface.bind()?;
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
        self.painter.clear(size, [0.0; 4]);
        self.painter.paint_and_update_textures(
            size,
            frame.pixels_per_point,
            &frame.primitives,
            &frame.textures,
        );
        let result = surface.present();
        drop(saved);

        result
    }
    fn poll(&mut self) -> Result<()> {
        self.queue.dispatch_pending(&mut self.state)?;
        if let Some(guard) = self.connection.prepare_read() {
            let mut poll = libc::pollfd {
                fd: self.connection.as_fd().as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            if unsafe { libc::poll(&mut poll, 1, 0) } > 0 {
                guard.read()?;
            }
        }
        self.queue.dispatch_pending(&mut self.state)?;
        Ok(())
    }
    fn end_pass(&mut self, pass: u64) {
        self.state.layers.retain(|_, layer| layer.last_seen == pass);
        let _ = self.poll();
        let _ = self.connection.flush();
    }
}
impl Drop for Driver {
    fn drop(&mut self) {
        self.state.layers.clear();
        let _ = self.connection.flush();
        if let Ok(saved) = self.egl.offscreen() {
            self.painter.destroy();
            drop(saved);
        }
    }
}
struct Frame {
    primitives: Vec<egui::ClippedPrimitive>,
    textures: egui::TexturesDelta,
    pixels_per_point: f32,
}

pub fn initialize(ctx: &egui::Context) -> Result<()> {
    let driver = Driver::new(ctx)?;
    DRIVER.with(|d| *d.borrow_mut() = Some(driver));
    ctx.on_end_pass(
        "passive Wayland lifecycle",
        Arc::new(|ctx| {
            if ctx.viewport_id() == ViewportId::ROOT {
                DRIVER.with(|d| {
                    if let Some(driver) = d.borrow_mut().as_mut() {
                        driver.end_pass(ctx.cumulative_pass_nr());
                    }
                });
            }
        }),
    );
    Ok(())
}
pub fn foreground_game_active() -> Option<bool> {
    DRIVER.with(|d| {
        let mut d = d.borrow_mut();
        let driver = d.as_mut()?;
        if driver.poll().is_err() {
            return None;
        }
        driver.state.active_game()
    })
}
pub fn outputs() -> Vec<Rect> {
    DRIVER.with(|d| {
        d.borrow()
            .as_ref()
            .map(|d| d.state.outputs.values().map(|o| o.logical).collect())
            .unwrap_or_default()
    })
}
pub fn shutdown() {
    DRIVER.with(|d| {
        d.borrow_mut().take();
    });
}
pub fn show(
    root: &egui::Context,
    id: ViewportId,
    builder: egui::ViewportBuilder,
    mut contents: impl FnMut(&egui::Context),
) -> Result<()> {
    let fonts = root.fonts(|f| f.lock().fonts.definitions().clone());
    let (child, mut input) = DRIVER.with(|d| -> Result<_> {
        let mut driver = d.borrow_mut();
        let driver = driver
            .as_mut()
            .context("Wayland passive layer backend unavailable")?;
        if driver.fonts != fonts {
            driver.context.set_fonts(fonts.clone());
            driver.fonts = fonts;
        }
        driver.context.set_style(root.style().as_ref().clone());
        driver.context.set_zoom_factor(root.zoom_factor());
        driver.prepare(id, &builder, root.cumulative_pass_nr())
    })?;
    input.time = root.input(|i| i.time).into();
    let output = child.run(input, |ctx| contents(ctx));
    let ppp = output.pixels_per_point;
    let frame = Frame {
        primitives: child.tessellate(output.shapes, ppp),
        textures: output.textures_delta,
        pixels_per_point: ppp,
    };
    let rect = root.screen_rect();
    root.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        // The shared font atlas emits incremental updates in UI order.
        // One root layer preserves that order for all rendering callbacks.
        egui::Id::new("Wayland passive frames"),
    ))
    .add(egui::PaintCallback {
        rect,
        callback: Arc::new(eframe::egui_glow::CallbackFn::new(move |_, _| {
            DRIVER.with(|d| {
                if let Some(driver) = d.borrow_mut().as_mut() {
                    if let Err(error) = driver.paint(id, &frame) {
                        eprintln!("Wayland passive layer render: {error:#}");
                    }
                }
            });
        })),
    });
    Ok(())
}

#[cfg(test)]
#[path = "wayland_passive/tests.rs"]
mod tests;
