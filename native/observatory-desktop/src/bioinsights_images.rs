//! Source Codex Companion images. Two bounded workers perform all network,
//! image decoding and source region-map flood fills away from the UI thread.
use eframe::egui::{self, Color32, TextureHandle, Vec2};
use serde_json::Value;
use std::{
    collections::{BTreeMap, VecDeque},
    hash::{Hash, Hasher},
    io::Read,
    sync::{mpsc, Arc, Mutex, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const PREFIX: &str = "https://elitetools.github.io/";
const MANIFEST: &str = "https://elitetools.github.io/codexcompanion/manifest.json";
const MAX_BYTES: u64 = 4 * 1024 * 1024;

enum Work {
    Image {
        key: String,
        source: String,
        ctx: egui::Context,
    },
    Map {
        key: String,
        possible: u64,
        seen: u64,
        ctx: egui::Context,
    },
    Manifest(egui::Context),
}
#[derive(Default)]
struct Cache {
    // None means queued or failed. Failed downloads are not retried per frame.
    textures: BTreeMap<String, Option<TextureHandle>>,
    order: VecDeque<String>,
    manifest: Option<Value>,
    manifest_requested: bool,
}
struct Service {
    queue: mpsc::SyncSender<Work>,
    cache: Arc<Mutex<Cache>>,
}
fn service() -> &'static Service {
    static SERVICE: OnceLock<Service> = OnceLock::new();
    SERVICE.get_or_init(|| {
        let (queue, input) = mpsc::sync_channel::<Work>(16);
        let input = Arc::new(Mutex::new(input));
        let cache = Arc::new(Mutex::new(Cache::default()));
        for index in 0..2 {
            let input = input.clone();
            let cache = cache.clone();
            let _ = std::thread::Builder::new()
                .name(format!("bioinsights-images-{index}"))
                .spawn(move || {
                    let client = reqwest::blocking::Client::builder()
                        .connect_timeout(Duration::from_secs(5))
                        .timeout(Duration::from_secs(8))
                        .redirect(reqwest::redirect::Policy::custom(|attempt| {
                            if attempt.previous().len() < 3
                                && attempt.url().host_str() == Some("elitetools.github.io")
                            {
                                attempt.follow()
                            } else {
                                attempt.stop()
                            }
                        }))
                        .build()
                        .ok();
                    loop {
                        let work = input.lock().ok().and_then(|input| input.recv().ok());
                        let Some(work) = work else { break };
                        let (key, image, ctx) = match work {
                            Work::Manifest(ctx) => {
                                // Cached source metadata is usable offline, while a
                                // fresh manifest can add newly contributed images.
                                if let Some(value) = disk_read(MANIFEST)
                                    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
                                {
                                    if let Ok(mut cache) = cache.lock() {
                                        cache.manifest = Some(value);
                                    }
                                    ctx.request_repaint();
                                }
                                let value = download(client.as_ref(), MANIFEST)
                                    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
                                if value.is_some() {
                                    if let Ok(mut cache) = cache.lock() {
                                        cache.manifest = value;
                                    }
                                }
                                ctx.request_repaint();
                                continue;
                            }
                            Work::Image { key, source, ctx } => {
                                let bytes = if source == "source:Unknown" {
                                    Some(
                                        include_bytes!("../assets/bioinsights/species.Unknown.jpg")
                                            .to_vec(),
                                    )
                                } else {
                                    disk_read(&source)
                                        .or_else(|| download(client.as_ref(), &source))
                                };
                                let decoded = bytes.and_then(|bytes| {
                                    // Limits apply before decode and all decoding runs here.
                                    let mut reader =
                                        image::ImageReader::new(std::io::Cursor::new(bytes))
                                            .with_guessed_format()
                                            .ok()?;
                                    let mut limits = image::Limits::default();
                                    limits.max_image_width = Some(4096);
                                    limits.max_image_height = Some(4096);
                                    limits.max_alloc = Some(96 * 1024 * 1024);
                                    reader.limits(limits);
                                    let image =
                                        reader.decode().ok()?.thumbnail(960, 540).to_rgba8();
                                    Some(egui::ColorImage::from_rgba_unmultiplied(
                                        [image.width() as usize, image.height() as usize],
                                        image.as_raw(),
                                    ))
                                });
                                (key, decoded, ctx)
                            }
                            Work::Map {
                                key,
                                possible,
                                seen,
                                ctx,
                            } => (key, region_map(possible, seen), ctx),
                        };
                        let texture = image.map(|image| {
                            ctx.load_texture(&key, image, egui::TextureOptions::LINEAR)
                        });
                        if let Ok(mut cache) = cache.lock() {
                            // A request can finish after its cache entry was evicted.
                            // Do not reinsert it and grow beyond the 64-entry bound.
                            if let Some(entry) = cache.textures.get_mut(&key) {
                                *entry = texture;
                            }
                        }
                        ctx.request_repaint();
                    }
                });
        }
        Service { queue, cache }
    })
}

fn download(client: Option<&reqwest::blocking::Client>, url: &str) -> Option<Vec<u8>> {
    if !url.starts_with(PREFIX) {
        return None;
    }
    let mut response = client?.get(url).send().ok()?.error_for_status().ok()?;
    if response
        .content_length()
        .is_some_and(|length| length > MAX_BYTES)
    {
        return None;
    }
    let mut bytes = Vec::new();
    response
        .by_ref()
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() as u64 > MAX_BYTES {
        return None;
    }
    if let Some(path) = cache_path(url) {
        if let Some(parent) = path.parent() {
            if std::fs::create_dir_all(parent).is_ok() {
                let _ = std::fs::write(path, &bytes);
            }
        }
    }
    Some(bytes)
}

fn cache_path(url: &str) -> Option<std::path::PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".cache"))
        })?;
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    url.hash(&mut hash);
    Some(
        base.join("observatory-core")
            .join("bioinsights")
            .join(format!("{:016x}.bin", hash.finish())),
    )
}

fn disk_read(url: &str) -> Option<Vec<u8>> {
    let path = cache_path(url)?;
    if std::fs::metadata(&path).ok()?.len() > MAX_BYTES {
        return None;
    }
    std::fs::read(path).ok()
}

fn cached(ctx: &egui::Context, key: &str, work: impl FnOnce() -> Work) -> Option<TextureHandle> {
    let service = service();
    let mut cache = service.cache.lock().ok()?;
    if let Some(texture) = cache.textures.get(key) {
        return texture.clone();
    }
    if cache.order.len() >= 64 {
        if let Some(old) = cache.order.pop_front() {
            cache.textures.remove(&old);
        }
    }
    if service.queue.try_send(work()).is_ok() {
        cache.order.push_back(key.to_owned());
        cache.textures.insert(key.to_owned(), None);
        ctx.request_repaint();
    }
    None
}

pub(super) fn show(ui: &mut egui::Ui, source: &str, size: Vec2) {
    let texture = cached(ui.ctx(), source, || Work::Image {
        key: source.to_owned(),
        source: source.to_owned(),
        ctx: ui.ctx().clone(),
    });
    let fallback = if texture.is_none() && source != "source:Unknown" {
        cached(ui.ctx(), "source:Unknown", || Work::Image {
            key: "source:Unknown".into(),
            source: "source:Unknown".into(),
            ctx: ui.ctx().clone(),
        })
    } else {
        None
    };
    if let Some(texture) = texture.or(fallback) {
        ui.add(egui::Image::new((texture.id(), size)));
    } else {
        ui.allocate_space(size);
    }
}

pub(super) struct Gallery {
    pub url: String,
    pub name: String,
    pub cmdr: String,
}
fn field<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .as_object()
        .and_then(|object| {
            object
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(key))
        })
        .and_then(|(_, value)| value.as_str())
        .unwrap_or("")
}
pub(super) fn gallery(ctx: &egui::Context, species: &str) -> Option<Gallery> {
    let service = service();
    let mut cache = service.cache.lock().ok()?;
    if !cache.manifest_requested && service.queue.try_send(Work::Manifest(ctx.clone())).is_ok() {
        cache.manifest_requested = true;
    }
    let entries = cache.manifest.as_ref()?.get(species)?.as_array()?;
    if entries.is_empty() {
        return None;
    }
    let id = egui::Id::new(("BioInsights source gallery selection", species));
    let index = ctx.data_mut(|data| {
        if let Some(index) = data.get_temp::<usize>(id) {
            index
        } else {
            let index = (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                % 60) as usize;
            data.insert_temp(id, index);
            index
        }
    });
    let image = &entries[index % entries.len()];
    let file = field(image, "FileName");
    if file.is_empty() {
        return None;
    }
    Some(Gallery {
        url: format!("{PREFIX}codexcompanion/{file}"),
        name: field(image, "Name").into(),
        cmdr: field(image, "Cmdr").into(),
    })
}

pub(super) fn map(ui: &mut egui::Ui, possible: u64, seen: u64, color: Color32, size: f32) {
    let key = format!("BioInsights source region map:{possible}:{seen}");
    let texture = cached(ui.ctx(), &key, || Work::Map {
        key: key.clone(),
        possible,
        seen,
        ctx: ui.ctx().clone(),
    });
    let response = if let Some(texture) = texture {
        ui.add(egui::Image::new((texture.id(), Vec2::splat(size))))
    } else {
        ui.allocate_response(Vec2::splat(size), egui::Sense::hover())
    };
    let scale = size / 400.0;
    for (text, y) in [("Possible", 368.0), ("Seen", 383.0)] {
        let font =
            crate::bioinsights_main::css_text::font("Century Gothic", 16.0 * scale, true, false);
        let at = response.rect.min + Vec2::new(352.0, y) * scale;
        if !crate::bioinsights_main::css_text::paint(
            ui,
            text,
            &font,
            at,
            egui::Align2::LEFT_TOP,
            color,
            response.rect,
        ) {
            ui.painter().text(
                at,
                egui::Align2::LEFT_TOP,
                text,
                egui::FontId::proportional(16.0 * scale),
                color,
            );
        }
    }
}

fn region_map(possible: u64, seen: u64) -> Option<egui::ColorImage> {
    let mut image = image::load_from_memory(include_bytes!("../assets/bioinsights/regionmap.png"))
        .ok()?
        .to_rgba8();
    let coords: Vec<[u32; 3]> = serde_json::from_str(include_str!(
        "../assets/bioinsights/region-coordinates.json"
    ))
    .ok()?;
    let possible = if possible == 0 {
        (1u64 << 43) - 2
    } else {
        possible
    };
    let brown = image::Rgba([136, 78, 8, 255]);
    let mint = image::Rgba([164, 242, 203, 255]);
    for [region, x, y] in &coords {
        if possible & (1u64 << region) != 0 {
            flood_fill(&mut image, x * 400 / 720, y * 400 / 720, brown);
        }
    }
    for [region, x, y] in coords {
        if seen & (1u64 << region) != 0 {
            flood_fill(&mut image, x * 400 / 720, y * 400 / 720, mint);
        }
    }
    flood_fill(&mut image, 345, 375, brown);
    flood_fill(&mut image, 345, 390, mint);
    Some(egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        image.as_raw(),
    ))
}

fn flood_fill(image: &mut image::RgbaImage, x: u32, y: u32, replacement: image::Rgba<u8>) {
    if x >= image.width() || y >= image.height() {
        return;
    }
    let target = *image.get_pixel(x, y);
    if target == replacement {
        return;
    }
    let mut stack = vec![(x, y)];
    while let Some((x, y)) = stack.pop() {
        if x >= image.width() || y >= image.height() || *image.get_pixel(x, y) != target {
            continue;
        }
        image.put_pixel(x, y, replacement);
        if x > 0 {
            stack.push((x - 1, y));
        }
        if y > 0 {
            stack.push((x, y - 1));
        }
        stack.push((x + 1, y));
        stack.push((x, y + 1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_region_map_colours_change_only_selected_regions() {
        let map = region_map(1 << 18, 1 << 1).unwrap();
        assert_eq!(map.size, [400, 400]);
        assert_eq!(
            map[(345 * 400 / 720, 530 * 400 / 720)],
            Color32::from_rgb(136, 78, 8)
        );
        assert_eq!(
            map[(358 * 400 / 720, 358 * 400 / 720)],
            Color32::from_rgb(164, 242, 203)
        );
    }
}
