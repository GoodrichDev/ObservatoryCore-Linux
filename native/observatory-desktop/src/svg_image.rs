//! Original SVG bytes rendered by librsvg/Cairo, including substituted theme
//! colors. API declarations: https://gnome.pages.gitlab.gnome.org/librsvg/Rsvg-2.0/
//! Cairo ARGB32 is native-endian and premultiplied; egui receives premultiplied
//! RGBA. Libraries stay loaded for the process, matching GTK's global type life.
use anyhow::{anyhow, ensure, Context, Result};
use eframe::egui::{self, Color32, ColorImage, Rect, TextureHandle, TextureOptions, Vec2};
use libloading::Library;
use std::{
    collections::HashMap,
    ffi::{c_char, c_int, c_void, CStr},
    hash::{Hash, Hasher},
    sync::{Arc, Mutex, OnceLock},
};
type Ptr = *mut c_void;
#[repr(C)]
#[derive(Default)]
struct RsvgRectangle {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}
#[repr(C)]
#[derive(Default)]
struct RsvgLength {
    length: f64,
    unit: c_int,
}
#[repr(C)]
struct GError {
    domain: u32,
    code: c_int,
    message: *const c_char,
}
struct Api {
    _libraries: [Library; 4],
    load: unsafe extern "C" fn(*const u8, usize, *mut *mut GError) -> Ptr,
    dpi: unsafe extern "C" fn(Ptr, f64),
    intrinsic: unsafe extern "C" fn(Ptr, *mut f64, *mut f64) -> c_int,
    dimensions: unsafe extern "C" fn(
        Ptr,
        *mut c_int,
        *mut RsvgLength,
        *mut c_int,
        *mut RsvgLength,
        *mut c_int,
        *mut RsvgRectangle,
    ),
    render: unsafe extern "C" fn(Ptr, Ptr, *const RsvgRectangle, *mut *mut GError) -> c_int,
    unref: unsafe extern "C" fn(Ptr),
    error_free: unsafe extern "C" fn(*mut GError),
    surface: unsafe extern "C" fn(c_int, c_int, c_int) -> Ptr,
    surface_status: unsafe extern "C" fn(Ptr) -> c_int,
    surface_flush: unsafe extern "C" fn(Ptr),
    surface_destroy: unsafe extern "C" fn(Ptr),
    cairo: unsafe extern "C" fn(Ptr) -> Ptr,
    cairo_status: unsafe extern "C" fn(Ptr) -> c_int,
    cairo_destroy: unsafe extern "C" fn(Ptr),
    pixels: unsafe extern "C" fn(Ptr) -> *mut u8,
    stride: unsafe extern "C" fn(Ptr) -> c_int,
}
impl Api {
    fn load() -> Result<Self> {
        unsafe {
            let rsvg = Library::new("librsvg-2.so.2").context("load native SVG renderer")?;
            let cairo = Library::new("libcairo.so.2")?;
            let object = Library::new("libgobject-2.0.so.0")?;
            let glib = Library::new("libglib-2.0.so.0")?;
            macro_rules! sym {
                ($lib:expr,$name:literal) => {
                    *$lib.get(concat!($name, "\0").as_bytes())?
                };
            }
            Ok(Self {
                load: sym!(rsvg, "rsvg_handle_new_from_data"),
                dpi: sym!(rsvg, "rsvg_handle_set_dpi"),
                intrinsic: sym!(rsvg, "rsvg_handle_get_intrinsic_size_in_pixels"),
                dimensions: sym!(rsvg, "rsvg_handle_get_intrinsic_dimensions"),
                render: sym!(rsvg, "rsvg_handle_render_document"),
                unref: sym!(object, "g_object_unref"),
                error_free: sym!(glib, "g_error_free"),
                surface: sym!(cairo, "cairo_image_surface_create"),
                surface_status: sym!(cairo, "cairo_surface_status"),
                surface_flush: sym!(cairo, "cairo_surface_flush"),
                surface_destroy: sym!(cairo, "cairo_surface_destroy"),
                cairo: sym!(cairo, "cairo_create"),
                cairo_status: sym!(cairo, "cairo_status"),
                cairo_destroy: sym!(cairo, "cairo_destroy"),
                pixels: sym!(cairo, "cairo_image_surface_get_data"),
                stride: sym!(cairo, "cairo_image_surface_get_stride"),
                _libraries: [rsvg, cairo, object, glib],
            })
        }
    }
    fn error(&self, error: *mut GError, fallback: &str) -> anyhow::Error {
        if error.is_null() {
            return anyhow!(fallback.to_owned());
        }
        unsafe {
            let message = if (*error).message.is_null() {
                fallback.to_owned()
            } else {
                CStr::from_ptr((*error).message)
                    .to_string_lossy()
                    .into_owned()
            };
            (self.error_free)(error);
            anyhow!(message)
        }
    }
}
fn api() -> Result<Arc<Api>> {
    static API: OnceLock<std::result::Result<Arc<Api>, String>> = OnceLock::new();
    API.get_or_init(|| Api::load().map(Arc::new).map_err(|e| format!("{e:#}")))
        .clone()
        .map_err(anyhow::Error::msg)
}
struct Document {
    ptr: Ptr,
    api: Arc<Api>,
}
impl Document {
    fn new(bytes: &[u8]) -> Result<Self> {
        let api = api()?;
        let mut error = std::ptr::null_mut();
        let ptr = unsafe { (api.load)(bytes.as_ptr(), bytes.len(), &mut error) };
        if ptr.is_null() {
            return Err(api.error(error, "Invalid SVG document"));
        }
        unsafe {
            (api.dpi)(ptr, 96.0);
        }
        Ok(Self { ptr, api })
    }
    fn size(&self) -> Result<Vec2> {
        let (mut width, mut height) = (0.0, 0.0);
        let pixels = unsafe { (self.api.intrinsic)(self.ptr, &mut width, &mut height) };
        if pixels == 0 {
            let (mut has_width, mut has_height, mut has_viewbox) = (0, 0, 0);
            let (mut w, mut h) = (RsvgLength::default(), RsvgLength::default());
            let mut viewbox = RsvgRectangle::default();
            unsafe {
                (self.api.dimensions)(
                    self.ptr,
                    &mut has_width,
                    &mut w,
                    &mut has_height,
                    &mut h,
                    &mut has_viewbox,
                    &mut viewbox,
                );
            }
            ensure!(
                has_viewbox != 0,
                "SVG has no intrinsic pixel size or viewBox"
            );
            width = viewbox.width;
            height = viewbox.height;
        }
        ensure!(
            width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0,
            "Invalid SVG dimensions"
        );
        Ok(egui::vec2(width as f32, height as f32))
    }
}
impl Drop for Document {
    fn drop(&mut self) {
        unsafe {
            (self.api.unref)(self.ptr);
        }
    }
}
struct Surface {
    ptr: Ptr,
    api: Arc<Api>,
}
impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            (self.api.surface_destroy)(self.ptr);
        }
    }
}
struct Cairo {
    ptr: Ptr,
    api: Arc<Api>,
}
impl Drop for Cairo {
    fn drop(&mut self) {
        unsafe {
            (self.api.cairo_destroy)(self.ptr);
        }
    }
}

pub fn intrinsic_size(bytes: &[u8]) -> Result<Vec2> {
    Document::new(bytes)?.size()
}

pub fn rasterize(bytes: &[u8], width: usize, height: usize) -> Result<ColorImage> {
    ensure!(
        (1..=4096).contains(&width) && (1..=4096).contains(&height),
        "SVG raster dimensions exceed4096pixels"
    );
    let document = Document::new(bytes)?;
    let api = document.api.clone();
    let surface = Surface {
        ptr: unsafe { (api.surface)(0, width as c_int, height as c_int) },
        api: api.clone(),
    };
    ensure!(
        !surface.ptr.is_null() && unsafe { (api.surface_status)(surface.ptr) } == 0,
        "Cairo image allocation failed"
    );
    let cairo = Cairo {
        ptr: unsafe { (api.cairo)(surface.ptr) },
        api: api.clone(),
    };
    ensure!(
        !cairo.ptr.is_null() && unsafe { (api.cairo_status)(cairo.ptr) } == 0,
        "Cairo context allocation failed"
    );
    let viewport = RsvgRectangle {
        x: 0.0,
        y: 0.0,
        width: width as f64,
        height: height as f64,
    };
    let mut error = std::ptr::null_mut();
    if unsafe { (api.render)(document.ptr, cairo.ptr, &viewport, &mut error) } == 0 {
        return Err(api.error(error, "SVG render failed"));
    }
    unsafe {
        (api.surface_flush)(surface.ptr);
    }
    let stride = unsafe { (api.stride)(surface.ptr) };
    let data = unsafe { (api.pixels)(surface.ptr) };
    ensure!(
        stride >= width as c_int * 4 && !data.is_null(),
        "Cairo image has invalid stride or data"
    );
    let data = unsafe { std::slice::from_raw_parts(data, stride as usize * height) };
    let mut rgba = Vec::with_capacity(width * height * 4);
    for row in data.chunks(stride as usize).take(height) {
        for pixel in row[..width * 4].as_chunks::<4>().0 {
            if cfg!(target_endian = "little") {
                rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
            } else {
                rgba.extend_from_slice(&[pixel[1], pixel[2], pixel[3], pixel[0]]);
            }
        }
    }
    Ok(ColorImage::from_rgba_premultiplied([width, height], &rgba))
}

type Cache = Arc<Mutex<HashMap<(u64, usize, usize), TextureHandle>>>;
pub fn paint_svg(ui: &egui::Ui, bytes: &[u8], key: &str, rect: Rect) -> Result<()> {
    if !rect.is_positive() {
        return Ok(());
    }
    let scale = ui.ctx().pixels_per_point();
    let width = (rect.width() * scale).round().clamp(1.0, 4096.0) as usize;
    let height = (rect.height() * scale).round().clamp(1.0, 4096.0) as usize;
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hash);
    let cache_key = (hash.finish(), width, height);
    let id = egui::Id::new("source SVG textures");
    // Release Context's data lock before loading a texture (which also locks it).
    let cache: Cache = ui
        .ctx()
        .data_mut(|data| data.get_temp_mut_or_default::<Cache>(id).clone());
    let existing = cache.lock().unwrap().get(&cache_key).cloned();
    let texture = if let Some(texture) = existing {
        texture
    } else {
        let image = rasterize(bytes, width, height)?;
        let texture = ui.ctx().load_texture(
            format!("{key}:{}:{width}x{height}", cache_key.0),
            image,
            TextureOptions::LINEAR,
        );
        let mut textures = cache.lock().unwrap();
        if textures.len() >= 512 {
            textures.clear();
        }
        textures.insert(cache_key, texture.clone());
        texture
    };
    ui.painter().image(
        texture.id(),
        rect,
        Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
        Color32::WHITE,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_svg_renderer_preserves_substituted_colors_and_premultiplied_alpha() {
        let source = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="20"><rect x="0" y="0" width="10" height="20" fill="$$COLOR$$" opacity="0.5"/></svg>"#;
        let red = source.replace("$$COLOR$$", "#ff0000");
        let green = source.replace("$$COLOR$$", "#00ff00");
        assert_eq!(
            intrinsic_size(red.as_bytes()).unwrap(),
            egui::vec2(10.0, 20.0)
        );
        let red = rasterize(red.as_bytes(), 10, 20).unwrap();
        let green = rasterize(green.as_bytes(), 10, 20).unwrap();
        assert_eq!(red.pixels[55].to_array(), [128, 0, 0, 128]);
        assert_eq!(green.pixels[55].to_array(), [0, 128, 0, 128]);
    }
    #[test]
    fn original_viewbox_aspect_and_css_dpi_are_retained() {
        let source=br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 50"><rect width="100" height="50" fill="#ff0000"/></svg>"##;
        assert_eq!(intrinsic_size(source).unwrap(), egui::vec2(100.0, 50.0));
        let image = rasterize(source, 40, 40).unwrap();
        assert_eq!(image.pixels[5 * 40 + 20].a(), 0);
        assert_eq!(image.pixels[20 * 40 + 20].to_array(), [255, 0, 0, 255]);
        assert_eq!(
            intrinsic_size(
                br#"<svg xmlns="http://www.w3.org/2000/svg" width="1in" height="0.5in"/>"#
            )
            .unwrap(),
            egui::vec2(96.0, 48.0)
        );
        assert!(rasterize(b"<invalid", 10, 10).is_err());
        assert!(rasterize(source, 10, 5).is_ok());
    }
}
