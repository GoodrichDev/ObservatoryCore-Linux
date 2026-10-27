mod about;
mod bioinsights_main;
mod bioinsights_settings;
mod bioinsights_ui;
mod core_controls;
mod core_notification;
mod core_theme;
mod display_backend;
mod embedded_browser;
mod evaluator_ui;
mod extensions;
mod file_dialog;
mod font_choices;
mod font_registry;
mod game_focus;
mod hecate_settings;
mod http;
mod legacy_data;
mod legacy_settings;
mod native_popup;
mod native_updates;
mod notifications;
mod passive_windows;
mod runtime;
mod stellar_ui;
mod store;
mod svg_image;
mod table_export;
mod ui;

use anyhow::Result;
use clap::Parser;
use observatory_io::{ConfigPaths, Settings};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "observatory-core",
    version,
    about = "Native Linux Elite Observatory Core"
)]
struct Args {
    /// Override the journal directory (also supports ELITE_JOURNAL_DIR).
    #[arg(long)]
    journal_dir: Option<PathBuf>,
    /// Alternate settings file; data remains in XDG_DATA_HOME.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Import original portable Observatory.config or installed user.config and exit.
    #[arg(long="import-settings",conflicts_with_all=["headless","discover","journal_dir"])]
    import_settings: Option<PathBuf>,
    /// Compatible native Linux plugin update manifest (saved in this profile).
    #[arg(long)]
    native_update_manifest: Option<String>,
    /// Replay all journals without opening a window, printing results as JSON.
    #[arg(long)]
    headless: bool,
    /// Monitor live journals for a bounded interval after context initialization.
    #[arg(long,requires="headless",conflicts_with="export",value_parser=clap::value_parser!(u64).range(1..=86400))]
    watch_seconds: Option<u64>,
    /// Export all persisted results to CSV after headless replay.
    #[arg(long, requires = "headless")]
    export: Option<PathBuf>,
    /// Print detected journal directories and exit.
    #[arg(long)]
    discover: bool,
    /// Open a named page, such as Settings or Botanist.
    #[arg(long)]
    view: Option<String>,
    /// Open a plugin in a separate native window.
    #[arg(long)]
    detach: Option<String>,
    /// Launch and close the real window after N rendered frames (CI smoke check).
    #[arg(long, hide = true)]
    smoke_frames: Option<usize>,
    #[arg(long, hide = true)]
    screenshot: Option<PathBuf>,
    #[arg(long, hide = true)]
    screenshot_viewport: Option<String>,
    #[arg(long, hide = true, default_value_t = 0)]
    screenshot_delay_ms: u64,
    /// Initial client area in logical pixels, for example 1600x900.
    #[arg(long,value_parser=parse_window_size)]
    window_size: Option<[f32; 2]>,
}

fn parse_window_size(value: &str) -> std::result::Result<[f32; 2], String> {
    let (width, height) = value
        .split_once('x')
        .ok_or("Use WIDTHxHEIGHT, for example 1600x900")?;
    let width: u32 = width.parse().map_err(|_| "Invalid window width")?;
    let height: u32 = height.parse().map_err(|_| "Invalid window height")?;
    if !(592..=8192).contains(&width) || !(269..=8192).contains(&height) {
        return Err("Window size must be between 592x269 and 8192x8192".into());
    }
    Ok([width as f32, height as f32])
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.discover {
        println!(
            "{}",
            serde_json::to_string_pretty(&observatory_io::discover_journal_dirs())?
        );
        return Ok(());
    }
    let mut paths = ConfigPaths::discover()?;
    if let Some(config) = args.config {
        paths.settings_file = config;
    }
    paths.create_dirs()?;
    let _data_lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(paths.data_dir.join("core.lock"))?;
    _data_lock.try_lock().map_err(|error| {
        anyhow::anyhow!(
            "Another Observatory instance is using {}: {error}",
            paths.data_dir.display()
        )
    })?;
    let mut settings = Settings::load(&paths.settings_file)?;
    if let Some(source) = args.import_settings {
        anyhow::ensure!(
            std::fs::canonicalize(&source)
                .ok()
                .zip(std::fs::canonicalize(&paths.settings_file).ok())
                .is_none_or(|(source, destination)| source != destination),
            "The original settings file cannot also be the native import destination"
        );
        settings = legacy_settings::import(&source, settings)?;
        runtime::default_plugin_settings(&mut settings);
        let receipt = legacy_data::import_data(&source, &paths)?;
        receipt.apply_to(&mut settings);
        settings.save(&paths.settings_file)?;
        println!("{}", serde_json::to_string_pretty(&receipt)?);
        println!("Imported settings into {}", paths.settings_file.display());
        if let Some(notes) = settings.extra.get("legacy_import_notes") {
            println!("{}", serde_json::to_string_pretty(notes)?);
        }
        return Ok(());
    }
    if let Some(dir) = args.journal_dir {
        settings.journal_dir = Some(dir);
    }
    if let Some(url) = args.native_update_manifest {
        settings
            .extra
            .insert("native_update_manifest_url".into(), serde_json::json!(url));
        settings.save(&paths.settings_file)?;
    }
    runtime::default_plugin_settings(&mut settings);
    if args.headless {
        if let Some(seconds) = args.watch_seconds {
            println!(
                "{}",
                serde_json::to_string_pretty(&runtime::watch(settings, paths, seconds)?)?
            );
            return Ok(());
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&runtime::replay(settings, paths, args.export)?)?
        );
        return Ok(());
    }
    let icon = image::load_from_memory(include_bytes!("../../packaging/observatory-core.png"))?
        .into_rgba8();
    let icon = eframe::egui::IconData {
        width: icon.width(),
        height: icon.height(),
        rgba: icon.into_raw(),
    };
    let initial_size = args.window_size.or_else(|| {
        settings
            .extra
            .get("window_size")
            .and_then(|value| serde_json::from_value::<[f32; 2]>(value.clone()).ok())
            .filter(|s| {
                s.iter().all(|v| v.is_finite())
                    && s[0] >= 592.0
                    && s[1] >= 269.0
                    && s[0] <= 16384.0
                    && s[1] <= 16384.0
            })
    });
    let initial_position = settings
        .extra
        .get("window_position")
        .and_then(|value| serde_json::from_value::<[f32; 2]>(value.clone()).ok())
        .filter(|s| s.iter().all(|v| v.is_finite() && v.abs() < 100000.0));
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_icon(icon)
        .with_app_id("observatory-core")
        .with_inner_size(initial_size.unwrap_or([800.0, 500.0]))
        .with_position(initial_position.unwrap_or([100.0, 100.0]))
        .with_min_inner_size([592.0, 269.0]);
    if let Some(position) = initial_position {
        viewport = viewport.with_position(position);
    }
    let options = eframe::NativeOptions {
        event_loop_builder: Some(Box::new(|builder| {
            #[cfg(target_os = "linux")]
            if crate::display_backend::detect() == crate::display_backend::Backend::X11 {
                use winit::platform::x11::EventLoopBuilderExtX11;
                builder.with_x11();
            }
        })),
        viewport,
        renderer: eframe::Renderer::Glow,
        persist_window: initial_size.is_none(),
        ..Default::default()
    };
    eframe::run_native(
        "Elite Observatory Core - v2.0.0",
        options,
        Box::new(move |cc| {
            let mut app =
                ui::ObservatoryApp::new(cc, settings, paths, args.smoke_frames, args.screenshot)?;
            app.set_initial_view(args.view, args.detach, args.screenshot_viewport);
            app.defer_screenshot(args.screenshot_delay_ms);
            Ok(Box::new(app))
        }),
    )
    .map_err(|error| anyhow::anyhow!("desktop startup: {error}"))
}
