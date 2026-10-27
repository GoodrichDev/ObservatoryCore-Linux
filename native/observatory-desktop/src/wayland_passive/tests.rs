//! Ignored integration probes; run only against an isolated headless compositor.
use super::*;
use observatory_plugins::stellar_overlay::StellarOverlay;
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};
fn options(app_id: &str) -> eframe::NativeOptions {
    eframe::NativeOptions {
        event_loop_builder: Some(Box::new(|b| {
            use winit::platform::wayland::EventLoopBuilderExtWayland;
            b.with_wayland().with_any_thread(true);
        })),
        viewport: egui::ViewportBuilder::default()
            .with_app_id(app_id)
            .with_inner_size([1000.0, 700.0]),
        vsync: false,
        ..Default::default()
    }
}
fn directory() -> PathBuf {
    PathBuf::from(
        std::env::var_os("OBSERVATORY_WAYLAND_QA").expect("isolated QA directory required"),
    )
}
fn probe_clock() -> Option<Instant> {
    std::env::var_os("OBSERVATORY_WAYLAND_WAIT_FOR_START")
        .is_none()
        .then(Instant::now)
}
fn probe_elapsed(start: &mut Option<Instant>, directory: &Path) -> Duration {
    if start.is_none() && directory.join("probe-start").is_file() {
        *start = Some(Instant::now());
    }
    start.as_ref().map(Instant::elapsed).unwrap_or_default()
}
fn write_report(directory: &Path, name: &str, report: &Value) {
    let path = directory.join(name);
    let temporary = path.with_extension("tmp");
    std::fs::write(&temporary, report.to_string()).unwrap();
    std::fs::rename(temporary, path).unwrap();
}
fn monitor() -> Rect {
    crate::passive_windows::wayland_outputs()
        .first()
        .copied()
        .unwrap_or_else(|| Rect::from_min_size(Pos2::ZERO, Vec2::new(1920.0, 1080.0)))
}
struct Game {
    start: Option<Instant>,
    directory: PathBuf,
    report: Value,
}
impl eframe::App for Game {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let elapsed = probe_elapsed(&mut self.start, &self.directory);
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(12,20,36))).show(ctx,|ui|{
   ui.label("Fullscreen game surrogate: passive overlays must not take keyboard or pointer input.");
   let target=Rect::from_min_size(Pos2::new(100.0,100.0),Vec2::new(360.0,190.0));
   if ui.put(target,egui::Button::new("GAME INPUT TARGET")).clicked(){self.report["clicks"]=json!(self.report["clicks"].as_u64().unwrap()+1);}
  });
        let game_title = if elapsed.as_secs_f64() > 8.0 {
            "Game input probe"
        } else if elapsed.as_secs_f64() > 2.0 {
            "Elite - Dangerous (QA)"
        } else {
            "Game input probe"
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(game_title.into()));
        let events = ctx.input(|i| i.events.clone());
        for event in events {
            if let egui::Event::Text(text) = event {
                let mut texts = self.report["text"].as_str().unwrap_or("").to_owned();
                texts.push_str(&text);
                self.report["text"] = json!(texts);
            }
        }
        self.report["focused"] = json!(ctx.input(|i| i.focused));
        self.report["seconds"] = json!(elapsed.as_secs_f64());
        write_report(&self.directory, "game.json", &self.report);
        ctx.request_repaint_after(Duration::from_millis(20));
        if elapsed > Duration::from_secs(16) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
struct Overlay {
    start: Option<Instant>,
    directory: PathBuf,
    model: StellarOverlay,
    core: crate::native_popup::Manager,
    captured: Arc<Mutex<Vec<String>>>,
    phase: u64,
    negative: bool,
    report: Value,
}
fn capture(ctx: &egui::Context, name: &str, directory: &Path, captured: &Arc<Mutex<Vec<String>>>) {
    let name = name.replace(' ', "-");
    let mut names = captured.lock().unwrap();
    if names.contains(&name) {
        return;
    }
    names.push(name.clone());
    drop(names);
    let path = directory.join(format!("{name}.png"));
    ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("layer-capture"),
    ))
    .add(egui::PaintCallback {
        rect: ctx.screen_rect(),
        callback: Arc::new(eframe::egui_glow::CallbackFn::new(move |info, painter| {
            let image = painter.read_screen_rgba(info.screen_size_px);
            let pixels = image
                .pixels
                .iter()
                .flat_map(|p| p.to_array())
                .collect::<Vec<_>>();
            image::save_buffer(
                &path,
                &pixels,
                image.width() as u32,
                image.height() as u32,
                image::ColorType::Rgba8,
            )
            .unwrap();
        })),
    });
}
impl eframe::App for Overlay {
    fn on_exit(&mut self, _: Option<&glow::Context>) {
        crate::passive_windows::shutdown();
    }
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Observatory Core background window");
        });
        let elapsed = probe_elapsed(&mut self.start, &self.directory);
        if elapsed > Duration::from_secs(2) && elapsed < Duration::from_secs(9) {
            let phase = elapsed.as_millis() as u64 / 400;
            if phase != self.phase {
                self.phase = phase;
                let scale = std::env::var("OBSERVATORY_QA_SCALE")
                    .ok()
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(1);
                self.model.on_notification(&json!({"id":format!("notification-{phase}"),"Title":"Native notification","Detail":"Actual Stellar source layout, with keyboard focus retained by the game.","Timeout":600,"XPos":10*scale,"YPos":15*scale}));
            }
            self.model.tick();
            if self.negative {
                ctx.show_viewport_immediate(
                    ViewportId::from_hash_of(("unsafe-test-toplevel", phase)),
                    egui::ViewportBuilder::default()
                        .with_title("Unsafe negative control")
                        .with_inner_size([250.0, 120.0])
                        .with_active(false),
                    |ctx, _| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.label("xdg_toplevel negative control");
                        });
                    },
                );
            } else {
                crate::stellar_ui::prepare_font(ctx, self.model.settings());
                let snapshot = self.model.snapshot();
                let dir = self.directory.clone();
                let captured = self.captured.clone();
                crate::stellar_ui::render(ctx, &snapshot, Some(monitor()), |child, name| {
                    if elapsed > Duration::from_secs(3) {
                        capture(child, name, &dir, &captured);
                    }
                });
                crate::native_popup::render(
                    ctx,
                    &self.core.snapshot(),
                    |_| Some(monitor()),
                    |child, name| {
                        if elapsed > Duration::from_secs(3) {
                            capture(child, name, &self.directory, &self.captured);
                        }
                    },
                );
            }
        }
        DRIVER.with(|d| {
            if let Some(driver) = d.borrow().as_ref() {
                let current = driver.state.layers.len() as u64;
                let active = driver.state.active_game();
                if active == Some(true) {
                    self.report["foreign_seen_game"] = json!(true);
                }
                if self.report["foreign_seen_game"] == true && active == Some(false) {
                    self.report["foreign_seen_other"] = json!(true);
                }
                self.report["max_layers"] =
                    json!(self.report["max_layers"].as_u64().unwrap().max(current));
            }
        });
        self.report["capability_error"] = json!(crate::passive_windows::capability_error());
        self.report["seconds"] = json!(elapsed.as_secs_f64());
        self.report["phase"] = json!(self.phase);
        self.report["main_focused"] = json!(ctx.input(|i| i.focused));
        write_report(
            &self.directory,
            if self.negative {
                "negative.json"
            } else {
                "overlay.json"
            },
            &self.report,
        );
        ctx.request_repaint_after(Duration::from_millis(15));
        if elapsed > Duration::from_secs(10) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
fn run_overlay(negative: bool) {
    let dir = directory();
    std::fs::create_dir_all(&dir).unwrap();
    let png = dir.join("native-bitmap.png");
    let mut image = image::RgbaImage::new(128, 72);
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        *pixel = if (x / 8 + y / 8) % 2 == 0 {
            image::Rgba([240, 20, 180, 255])
        } else {
            image::Rgba([20, 230, 90, 255])
        };
    }
    image.save(&png).unwrap();
    eframe::run_native("Observatory overlay probe",options("observatory-overlay-probe"),Box::new(move|cc|{
  crate::passive_windows::initialize(cc);
  let mut model=StellarOverlay::new(json!({"DisplayOverride":true,"FontScaling":125,"RadarSettings":{"PositionXPct":2,"PositionYPct":50,"SizePct":25,"FontScaling":150}}));
  model.on_event(&observatory_domain::JournalEvent::parse("{\"timestamp\":\"2026-01-01T00:00:00Z\",\"event\":\"LoadGame\"}").unwrap());
  model.handle_message("BioInsights","RadarSetPlanet",&json!({"ID64":42,"BodyID":3,"RadiusKM":1500,"Genuses":{"7":"Stratum","3":"Aleoida"}})).unwrap();
  model.handle_message("BioInsights","RadarSetContextID",&json!({"ContextID":7,"Count":1,"Radius":500})).unwrap();
  model.handle_message("BioInsights","SetColonyRanges",&json!({"7":500,"3":150})).unwrap();
  model.on_status(&json!({"Latitude":12.3,"Longitude":45.6,"Heading":30,"Altitude":100}));
  model.handle_message("BioInsights","RadarAddBiological",&json!({"UseCurrentPosition":true,"ContextID":7,"Radius":500,"Label":"Stratum"})).unwrap();
  model.handle_message("BioInsights","ShowBitmapNotification",&json!({"ImageKey":"native-image","Filename":png,"XPos":70,"YPos":30,"TTLms":0,"ForceShow":true})).unwrap();
  let mut settings=observatory_io::Settings::default();settings.extra.insert("NativeNotifyTransparent".into(),json!(false));settings.extra.insert("NativeNotifyTimeout".into(),json!(0));
  let mut core=crate::native_popup::Manager::default();core.lifecycle(&json!({"id":"core","title":"Core notification","detail":"Actual native Core notification surface","timeout":0}),&settings,0).unwrap();
  Ok(Box::new(Overlay{start:probe_clock(),directory:dir,model,core,captured:Arc::default(),phase:0,negative,report:json!({"max_layers":0})}))
 })).unwrap();
}
#[test]
#[ignore = "requires an isolated layer-shell Wayland compositor"]
fn passive_overlay_native_rendering_probe() {
    run_overlay(false);
}
#[test]
#[ignore = "negative control requires an isolated Wayland compositor"]
fn unsafe_xdg_toplevel_negative_control() {
    run_overlay(true);
}
#[test]
#[ignore = "requires isolated compositor; launched by the Wayland focus test"]
fn fullscreen_game_input_probe() {
    let dir = directory();
    eframe::run_native(
        "Game input probe",
        options("observatory-game-probe"),
        Box::new(move |_| {
            Ok(Box::new(Game {
                start: probe_clock(),
                directory: dir,
                report: json!({"clicks":0,"text":""}),
            }))
        }),
    )
    .unwrap();
}

mod virtual_input {
    use super::*;
    use wayland_client::protocol::{wl_pointer, wl_seat};
    use wayland_protocols_wlr::virtual_pointer::v1::client::{
        zwlr_virtual_pointer_manager_v1, zwlr_virtual_pointer_v1,
    };
    #[derive(Default)]
    struct Input {
        manager: Option<zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1>,
        seat: Option<wl_seat::WlSeat>,
    }
    impl Dispatch<wl_registry::WlRegistry, ()> for Input {
        fn event(
            state: &mut Self,
            registry: &wl_registry::WlRegistry,
            event: wl_registry::Event,
            _: &(),
            _: &Connection,
            qh: &QueueHandle<Self>,
        ) {
            if let wl_registry::Event::Global {
                name,
                interface,
                version,
            } = event
            {
                match interface.as_str() {
                    "zwlr_virtual_pointer_manager_v1" => {
                        state.manager = Some(registry.bind(name, version.min(2), qh, ()))
                    }
                    "wl_seat" => state.seat = Some(registry.bind(name, version.min(7), qh, ())),
                    _ => {}
                }
            }
        }
    }
    delegate_noop!(Input:ignore wl_seat::WlSeat);
    delegate_noop!(Input:ignore zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1);
    delegate_noop!(Input:ignore zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1);
    #[test]
    #[ignore = "virtual input requires the isolated test compositor"]
    fn game_pointer_probe() {
        let _ = directory();
        let connection = Connection::connect_to_env().unwrap();
        let mut queue = connection.new_event_queue();
        let qh = queue.handle();
        connection.display().get_registry(&qh, ());
        let mut state = Input::default();
        queue.roundtrip(&mut state).unwrap();
        let pointer =
            state
                .manager
                .as_ref()
                .unwrap()
                .create_virtual_pointer(state.seat.as_ref(), &qh, ());
        let start = Instant::now();
        for _ in 0..9 {
            let time = start.elapsed().as_millis() as u32;
            let scale = std::env::var("OBSERVATORY_QA_SCALE")
                .ok()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(1);
            pointer.motion_absolute(time, 200 * scale, 160 * scale, 1920, 1080);
            pointer.frame();
            connection.flush().unwrap();
            std::thread::sleep(Duration::from_millis(150));
            pointer.button(time + 150, 272, wl_pointer::ButtonState::Pressed);
            pointer.frame();
            connection.flush().unwrap();
            std::thread::sleep(Duration::from_millis(80));
            pointer.button(time + 230, 272, wl_pointer::ButtonState::Released);
            pointer.frame();
            connection.flush().unwrap();
            std::thread::sleep(Duration::from_millis(500));
        }
        pointer.destroy();
        connection.flush().unwrap();
    }
}
