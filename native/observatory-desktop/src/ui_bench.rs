//! Opt-in CPU profiling for the real BioInsights painter. The input is an
//! isolated runtime snapshot; no game data is included in this repository.
use super::*;

#[test]
#[ignore = "requires OBSERVATORY_UI_PROFILE_INPUT; run in release mode"]
fn profile_bioinsights_paint() {
    let input = std::env::var("OBSERVATORY_UI_PROFILE_INPUT").expect("profile input path");
    let data: Value = serde_json::from_reader(std::fs::File::open(input).unwrap()).unwrap();
    let bio = &data["bio"];
    let context = egui::Context::default();
    let mut receipts = Vec::new();
    for size in [egui::vec2(1700.0, 900.0), egui::vec2(3440.0, 1368.0)] {
        let mut frames = Vec::new();
        let mut primitives = 0;
        for frame in 0..160 {
            let started = Instant::now();
            let output = context.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                    time: Some(frame as f64 / 60.0),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let _ = crate::bioinsights_main::show(
                            ui,
                            &bio["main_view"],
                            &bio["summary"],
                            &bio["settings"],
                        );
                    });
                },
            );
            primitives = context
                .tessellate(output.shapes, output.pixels_per_point)
                .len();
            if frame >= 20 {
                frames.push(started.elapsed().as_secs_f64() * 1000.0);
            }
        }
        frames.sort_by(f64::total_cmp);
        receipts.push(json!({"width":size.x,"height":size.y,"mean_ms":frames.iter().sum::<f64>()/frames.len()as f64,"p95_ms":frames[frames.len()*95/100],"max_ms":frames.last(),"paint_batches":primitives,"samples":frames.len()}));
    }
    println!("{}", serde_json::to_string_pretty(&receipts).unwrap());
    if let Ok(path) = std::env::var("OBSERVATORY_UI_PROFILE_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&receipts).unwrap()).unwrap();
    }
}
