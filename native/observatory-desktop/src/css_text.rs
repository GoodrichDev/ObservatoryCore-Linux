//! Native CSS text metrics and glyph rasterization using Linux's Pango/Cairo.
//! This avoids egui's integer font-height quantization changing HTML column
//! widths. Textures belong to the current egui context and are bounded/cached.
use eframe::egui::{self, Align2, Color32, FontFamily, FontId, Pos2, Rect, Vec2};
use libloading::Library;
use std::{
    collections::BTreeMap,
    ffi::{c_char, c_int, c_void, CStr, CString},
    sync::{Arc, Mutex, OnceLock},
};
type Ptr = *mut c_void;
#[repr(C)]
#[derive(Default)]
struct CairoMatrix {
    xx: f64,
    yx: f64,
    xy: f64,
    yy: f64,
    x0: f64,
    y0: f64,
}
#[repr(C)]
#[derive(Default)]
struct CairoFontExtents {
    ascent: f64,
    descent: f64,
    height: f64,
    max_x_advance: f64,
    max_y_advance: f64,
}
#[repr(C)]
struct CairoGlyph {
    index: std::ffi::c_ulong,
    x: f64,
    y: f64,
}
#[repr(C)]
#[derive(Default)]
struct CairoTextExtents {
    x_bearing: f64,
    y_bearing: f64,
    width: f64,
    height: f64,
    x_advance: f64,
    y_advance: f64,
}

struct Api {
    _libraries: [Library; 5],
    font_options: unsafe extern "C" fn() -> Ptr,
    font_options_destroy: unsafe extern "C" fn(Ptr),
    font_get_options: unsafe extern "C" fn(Ptr, Ptr),
    font_hint_metrics: unsafe extern "C" fn(Ptr, c_int),
    font_matrix: unsafe extern "C" fn(Ptr, *mut CairoMatrix),
    font_ctm: unsafe extern "C" fn(Ptr, *mut CairoMatrix),
    font_face: unsafe extern "C" fn(Ptr) -> Ptr,
    font_create: unsafe extern "C" fn(Ptr, *const CairoMatrix, *const CairoMatrix, Ptr) -> Ptr,
    font_extents: unsafe extern "C" fn(Ptr, *mut CairoFontExtents),
    font_destroy: unsafe extern "C" fn(Ptr),
    lock_face: unsafe extern "C" fn(Ptr) -> Ptr,
    unlock_face: unsafe extern "C" fn(Ptr),
    sfnt_table: unsafe extern "C" fn(
        Ptr,
        std::ffi::c_ulong,
        std::ffi::c_long,
        *mut u8,
        *mut std::ffi::c_ulong,
    ) -> c_int,
    map: unsafe extern "C" fn() -> Ptr,
    context: unsafe extern "C" fn(Ptr) -> Ptr,
    context_round: unsafe extern "C" fn(Ptr, c_int),
    layout_units: unsafe extern "C" fn(Ptr, *mut c_int, *mut c_int),
    layout_attributes: unsafe extern "C" fn(Ptr, Ptr),
    layout_context_changed: unsafe extern "C" fn(Ptr),
    attributes_new: unsafe extern "C" fn() -> Ptr,
    attributes_unref: unsafe extern "C" fn(Ptr),
    attributes_insert: unsafe extern "C" fn(Ptr, Ptr),
    font_features: unsafe extern "C" fn(*const c_char) -> Ptr,
    layout: unsafe extern "C" fn(Ptr) -> Ptr,
    description: unsafe extern "C" fn(*const c_char) -> Ptr,
    description_size: unsafe extern "C" fn(Ptr, f64),
    description_weight: unsafe extern "C" fn(Ptr, c_int),
    description_style: unsafe extern "C" fn(Ptr, c_int),
    description_free: unsafe extern "C" fn(Ptr),
    describe_font: unsafe extern "C" fn(Ptr) -> Ptr,
    description_family: unsafe extern "C" fn(Ptr) -> *const c_char,
    layout_font: unsafe extern "C" fn(Ptr, Ptr),
    layout_text: unsafe extern "C" fn(Ptr, *const c_char, c_int),
    layout_size: unsafe extern "C" fn(Ptr, *mut c_int, *mut c_int),
    layout_unknown: unsafe extern "C" fn(Ptr) -> c_int,
    layout_baseline: unsafe extern "C" fn(Ptr) -> c_int,
    load_font: unsafe extern "C" fn(Ptr, Ptr, Ptr) -> Ptr,
    scaled_font: unsafe extern "C" fn(Ptr) -> Ptr,
    glyph_extents: unsafe extern "C" fn(Ptr, *const CairoGlyph, c_int, *mut CairoTextExtents),
    set_scaled_font: unsafe extern "C" fn(Ptr, Ptr),
    show_glyphs: unsafe extern "C" fn(Ptr, *const CairoGlyph, c_int),
    show: unsafe extern "C" fn(Ptr, Ptr),
    unref: unsafe extern "C" fn(Ptr),
    surface: unsafe extern "C" fn(c_int, c_int, c_int) -> Ptr,
    surface_data: unsafe extern "C" fn(Ptr) -> *mut u8,
    surface_stride: unsafe extern "C" fn(Ptr) -> c_int,
    surface_flush: unsafe extern "C" fn(Ptr),
    surface_destroy: unsafe extern "C" fn(Ptr),
    create: unsafe extern "C" fn(Ptr) -> Ptr,
    color: unsafe extern "C" fn(Ptr, f64, f64, f64, f64),
    destroy: unsafe extern "C" fn(Ptr),
}
impl Api {
    unsafe fn load() -> Option<Self> {
        unsafe {
            let pango = Library::new("libpango-1.0.so.0").ok()?;
            let pc = Library::new("libpangocairo-1.0.so.0").ok()?;
            let cairo = Library::new("libcairo.so.2").ok()?;
            let object = Library::new("libgobject-2.0.so.0").ok()?;
            let freetype = Library::new("libfreetype.so.6").ok()?;
            macro_rules! sym {
                ($l:ident,$s:literal) => {
                    *$l.get(concat!($s, "\0").as_bytes()).ok()?
                };
            }
            Some(Self {
                font_options: sym!(cairo, "cairo_font_options_create"),
                font_options_destroy: sym!(cairo, "cairo_font_options_destroy"),
                font_get_options: sym!(cairo, "cairo_scaled_font_get_font_options"),
                font_hint_metrics: sym!(cairo, "cairo_font_options_set_hint_metrics"),
                font_matrix: sym!(cairo, "cairo_scaled_font_get_font_matrix"),
                font_ctm: sym!(cairo, "cairo_scaled_font_get_ctm"),
                font_face: sym!(cairo, "cairo_scaled_font_get_font_face"),
                font_create: sym!(cairo, "cairo_scaled_font_create"),
                font_extents: sym!(cairo, "cairo_scaled_font_extents"),
                font_destroy: sym!(cairo, "cairo_scaled_font_destroy"),
                lock_face: sym!(cairo, "cairo_ft_scaled_font_lock_face"),
                unlock_face: sym!(cairo, "cairo_ft_scaled_font_unlock_face"),
                sfnt_table: sym!(freetype, "FT_Load_Sfnt_Table"),
                map: sym!(pc, "pango_cairo_font_map_get_default"),
                context: sym!(pango, "pango_font_map_create_context"),
                context_round: sym!(pango, "pango_context_set_round_glyph_positions"),
                layout_units: sym!(pango, "pango_layout_get_size"),
                layout_attributes: sym!(pango, "pango_layout_set_attributes"),
                layout_context_changed: sym!(pango, "pango_layout_context_changed"),
                attributes_new: sym!(pango, "pango_attr_list_new"),
                attributes_unref: sym!(pango, "pango_attr_list_unref"),
                attributes_insert: sym!(pango, "pango_attr_list_insert"),
                font_features: sym!(pango, "pango_attr_font_features_new"),
                layout: sym!(pango, "pango_layout_new"),
                description: sym!(pango, "pango_font_description_from_string"),
                description_size: sym!(pango, "pango_font_description_set_absolute_size"),
                description_weight: sym!(pango, "pango_font_description_set_weight"),
                description_style: sym!(pango, "pango_font_description_set_style"),
                description_free: sym!(pango, "pango_font_description_free"),
                describe_font: sym!(pango, "pango_font_describe"),
                description_family: sym!(pango, "pango_font_description_get_family"),
                layout_font: sym!(pango, "pango_layout_set_font_description"),
                layout_text: sym!(pango, "pango_layout_set_text"),
                layout_size: sym!(pango, "pango_layout_get_pixel_size"),
                layout_unknown: sym!(pango, "pango_layout_get_unknown_glyphs_count"),
                layout_baseline: sym!(pango, "pango_layout_get_baseline"),
                load_font: sym!(pango, "pango_font_map_load_font"),
                scaled_font: sym!(pc, "pango_cairo_font_get_scaled_font"),
                glyph_extents: sym!(cairo, "cairo_scaled_font_glyph_extents"),
                set_scaled_font: sym!(cairo, "cairo_set_scaled_font"),
                show_glyphs: sym!(cairo, "cairo_show_glyphs"),
                show: sym!(pc, "pango_cairo_show_layout"),
                unref: sym!(object, "g_object_unref"),
                surface: sym!(cairo, "cairo_image_surface_create"),
                surface_data: sym!(cairo, "cairo_image_surface_get_data"),
                surface_stride: sym!(cairo, "cairo_image_surface_get_stride"),
                surface_flush: sym!(cairo, "cairo_surface_flush"),
                surface_destroy: sym!(cairo, "cairo_surface_destroy"),
                create: sym!(cairo, "cairo_create"),
                color: sym!(cairo, "cairo_set_source_rgba"),
                destroy: sym!(cairo, "cairo_destroy"),
                _libraries: [pango, pc, cairo, object, freetype],
            })
        }
    }
    fn substituted_color_family(&self, family: &str) -> Option<bool> {
        let family = CString::new(family).ok()?;
        unsafe {
            let context = (self.context)((self.map)());
            let desc = (self.description)(family.as_ptr());
            if context.is_null() || desc.is_null() {
                if !context.is_null() {
                    (self.unref)(context);
                }
                if !desc.is_null() {
                    (self.description_free)(desc);
                }
                return None;
            }
            (self.description_size)(desc, 15.0 * 1024.0);
            let font = (self.load_font)((self.map)(), context, desc);
            let mut substituted_color = false;
            if !font.is_null() {
                let actual = (self.describe_font)(font);
                if !actual.is_null() {
                    let name = (self.description_family)(actual);
                    if !name.is_null()
                        && !CStr::from_ptr(name)
                            .to_bytes()
                            .eq_ignore_ascii_case(family.as_bytes())
                    {
                        let scaled = (self.scaled_font)(font);
                        if !scaled.is_null() {
                            let face = (self.lock_face)(scaled);
                            if !face.is_null() {
                                // Color outline/bitmap tables. A replacement
                                // emoji face must not supply ordinary digits.
                                substituted_color =
                                    [*b"COLR", *b"CBDT", *b"sbix"].iter().any(|tag| {
                                        let mut length = 0;
                                        (self.sfnt_table)(
                                            face,
                                            u32::from_be_bytes(*tag).into(),
                                            0,
                                            std::ptr::null_mut(),
                                            &mut length,
                                        ) == 0
                                            && length > 0
                                    });
                                (self.unlock_face)(scaled);
                            }
                        }
                    }
                    (self.description_free)(actual);
                }
                (self.unref)(font);
            }
            (self.description_free)(desc);
            (self.unref)(context);
            Some(substituted_color)
        }
    }
    fn advance(
        &self,
        family: &str,
        bold: bool,
        italic: bool,
        size: f32,
        text: &str,
    ) -> Option<f32> {
        let family = CString::new(family).ok()?;
        let text = CString::new(text).ok()?;
        if !size.is_finite() || size <= 0.0 || size > 4096.0 {
            return None;
        }
        unsafe {
            let map = (self.map)();
            if map.is_null() {
                return None;
            }
            let context = (self.context)(map);
            if context.is_null() {
                return None;
            }
            let desc = (self.description)(family.as_ptr());
            let layout = (self.layout)(context);
            if desc.is_null() || layout.is_null() {
                if !desc.is_null() {
                    (self.description_free)(desc);
                }
                if !layout.is_null() {
                    (self.unref)(layout);
                }
                (self.unref)(context);
                return None;
            }
            (self.description_size)(desc, f64::from(size) * 1024.0);
            (self.description_weight)(desc, if bold { 700 } else { 400 });
            (self.description_style)(desc, if italic { 2 } else { 0 });
            (self.layout_font)(layout, desc);
            (self.layout_text)(layout, text.as_ptr(), -1);
            let attributes = (self.attributes_new)();
            let kern_off = (self.font_features)(c"kern=0".as_ptr());
            (self.attributes_insert)(attributes, kern_off);
            let width = |rounded, attrs| {
                (self.context_round)(context, rounded);
                (self.layout_attributes)(layout, attrs);
                (self.layout_context_changed)(layout);
                let (mut width, mut height) = (0, 0);
                (self.layout_units)(layout, &mut width, &mut height);
                width
            };
            // WebKit combines hinted per-glyph Cairo advances with fractional
            // HarfBuzz kerning. Pango's default rounds both; disabling rounding
            // also un-hints advances. Isolate kerning to preserve both parts.
            let hinted_without_kerning = width(1, attributes);
            let unhinted_with_kerning = width(0, std::ptr::null_mut());
            let unhinted_without_kerning = width(0, attributes);
            (self.attributes_unref)(attributes);
            (self.description_free)(desc);
            (self.unref)(layout);
            (self.unref)(context);
            Some(
                (hinted_without_kerning + unhinted_with_kerning - unhinted_without_kerning) as f32
                    / 1024.0,
            )
        }
    }
    fn font_extents(
        &self,
        family: &str,
        bold: bool,
        italic: bool,
        size: f32,
    ) -> Option<(f32, f32, f32)> {
        let family = CString::new(family).ok()?;
        if !size.is_finite() || size <= 0.0 || size > 4096.0 {
            return None;
        }
        unsafe {
            let map = (self.map)();
            if map.is_null() {
                return None;
            }
            let context = (self.context)(map);
            if context.is_null() {
                return None;
            }
            let desc = (self.description)(family.as_ptr());
            if desc.is_null() {
                (self.unref)(context);
                return None;
            }
            (self.description_size)(desc, f64::from(size) * 1024.0);
            (self.description_weight)(desc, if bold { 700 } else { 400 });
            (self.description_style)(desc, if italic { 2 } else { 0 });
            let font = (self.load_font)(map, context, desc);
            let result = if font.is_null() {
                None
            } else {
                let scaled = (self.scaled_font)(font);
                if scaled.is_null() {
                    None
                } else {
                    // CSS normal line spacing uses unhinted font metrics,
                    // rounding ascent/descent/gap separately (WebKit FT).
                    let options = (self.font_options)();
                    (self.font_get_options)(scaled, options);
                    (self.font_hint_metrics)(options, 1);
                    let mut matrix = CairoMatrix::default();
                    let mut ctm = CairoMatrix::default();
                    (self.font_matrix)(scaled, &mut matrix);
                    (self.font_ctm)(scaled, &mut ctm);
                    let unhinted =
                        (self.font_create)((self.font_face)(scaled), &matrix, &ctm, options);
                    let mut extents = CairoFontExtents::default();
                    (self.font_extents)(unhinted, &mut extents);
                    (self.font_destroy)(unhinted);
                    (self.font_options_destroy)(options);
                    let (mut ascent, mut descent, mut gap) = (
                        extents.ascent,
                        extents.descent,
                        extents.height - extents.ascent - extents.descent,
                    );
                    let face = (self.lock_face)(scaled);
                    if !face.is_null() {
                        let mut os2 = [0u8; 74];
                        let mut len = os2.len() as std::ffi::c_ulong;
                        if (self.sfnt_table)(
                            face,
                            u32::from_be_bytes(*b"OS/2") as _,
                            0,
                            os2.as_mut_ptr(),
                            &mut len,
                        ) == 0
                            && len >= 74
                            && u16::from_be_bytes([os2[62], os2[63]]) & (1 << 7) != 0
                        {
                            let mut head = [0u8; 20];
                            let mut head_len = 20;
                            if (self.sfnt_table)(
                                face,
                                u32::from_be_bytes(*b"head") as _,
                                0,
                                head.as_mut_ptr(),
                                &mut head_len,
                            ) == 0
                                && head_len >= 20
                            {
                                let units = u16::from_be_bytes([head[18], head[19]]);
                                if units > 0 {
                                    let ratio = f64::from(size) / f64::from(units);
                                    ascent =
                                        f64::from(i16::from_be_bytes([os2[68], os2[69]])) * ratio;
                                    descent =
                                        -f64::from(i16::from_be_bytes([os2[70], os2[71]])) * ratio;
                                    gap = f64::from(i16::from_be_bytes([os2[72], os2[73]])) * ratio;
                                }
                            }
                        }
                        (self.unlock_face)(scaled);
                    }
                    Some((
                        ascent.round() as f32,
                        descent.round() as f32,
                        gap.round() as f32,
                    ))
                }
            };
            if !font.is_null() {
                (self.unref)(font);
            }
            (self.description_free)(desc);
            (self.unref)(context);
            result
        }
    }
    fn raster(
        &self,
        family: &str,
        bold: bool,
        italic: bool,
        size: f32,
        text: &str,
    ) -> Option<egui::ColorImage> {
        if !size.is_finite() || size <= 0.0 || size > 4096.0 || text.len() > 64 * 1024 {
            return None;
        }
        let family = CString::new(family).ok()?;
        let scalar_count = text.chars().count();
        let text = CString::new(text).ok()?;
        unsafe {
            let map = (self.map)();
            if map.is_null() {
                return None;
            }
            let context = (self.context)(map);
            if context.is_null() {
                return None;
            }
            let layout = (self.layout)(context);
            let desc = (self.description)(family.as_ptr());
            if layout.is_null() || desc.is_null() {
                if !layout.is_null() {
                    (self.unref)(layout);
                }
                if !desc.is_null() {
                    (self.description_free)(desc);
                }
                (self.unref)(context);
                return None;
            }
            (self.description_size)(desc, f64::from(size) * 1024.0);
            (self.description_weight)(desc, if bold { 700 } else { 400 });
            (self.description_style)(desc, if italic { 2 } else { 0 });
            (self.layout_font)(layout, desc);
            (self.layout_text)(layout, text.as_ptr(), -1);
            let (mut width, mut height) = (0, 0);
            (self.layout_size)(layout, &mut width, &mut height);
            // WebKit uses the selected face's .notdef glyph for an unavailable
            // scalar. Pango's diagnostic hex box has a different width/shape.
            // Use that face's real glyph0 rather than inventing a replacement.
            let missing = if scalar_count == 1 && (self.layout_unknown)(layout) > 0 {
                let font = (self.load_font)((self.map)(), context, desc);
                if font.is_null() {
                    None
                } else {
                    let scaled = (self.scaled_font)(font);
                    if scaled.is_null() {
                        (self.unref)(font);
                        None
                    } else {
                        let glyph = CairoGlyph {
                            index: 0,
                            x: 0.0,
                            y: f64::from((self.layout_baseline)(layout)) / 1024.0,
                        };
                        let mut extents = CairoTextExtents::default();
                        (self.glyph_extents)(scaled, &glyph, 1, &mut extents);
                        width = extents.x_advance.ceil() as c_int;
                        Some((font, scaled, glyph))
                    }
                }
            } else {
                None
            };
            let result = if width > 0
                && height > 0
                && width <= 16384
                && height <= 4096
                && i64::from(width) * i64::from(height) <= 4_000_000
            {
                let surface = (self.surface)(0, width, height);
                let cr = (self.create)(surface);
                (self.color)(cr, 1.0, 1.0, 1.0, 1.0);
                if let Some((_, scaled, glyph)) = &missing {
                    (self.set_scaled_font)(cr, *scaled);
                    (self.show_glyphs)(cr, glyph, 1);
                } else {
                    (self.show)(cr, layout);
                }
                (self.surface_flush)(surface);
                let data = (self.surface_data)(surface);
                let stride = (self.surface_stride)(surface) as usize;
                let image = if !data.is_null() && stride >= width as usize * 4 {
                    let data = std::slice::from_raw_parts(data, stride * height as usize);
                    // Cairo's coverage is already premultiplied in the same
                    // gamma space used by WebKit. Reapplying egui's linear
                    // alpha conversion would make every glyph visibly bolder.
                    let mut pixels = Vec::with_capacity(width as usize * height as usize);
                    for y in 0..height as usize {
                        for x in 0..width as usize {
                            let offset = y * stride + x * 4;
                            // CAIRO_FORMAT_ARGB32 is a native-endian 0xAARRGGBB
                            // word. Keep its premultiplied color channels: color
                            // emoji must not become a white coverage mask.
                            let argb = u32::from_ne_bytes([
                                data[offset],
                                data[offset + 1],
                                data[offset + 2],
                                data[offset + 3],
                            ]);
                            pixels.push(Color32::from_rgba_premultiplied(
                                (argb >> 16) as u8,
                                (argb >> 8) as u8,
                                argb as u8,
                                (argb >> 24) as u8,
                            ));
                        }
                    }
                    Some(egui::ColorImage {
                        size: [width as usize, height as usize],
                        pixels,
                    })
                } else {
                    None
                };
                (self.destroy)(cr);
                (self.surface_destroy)(surface);
                image
            } else {
                None
            };
            if let Some((font, _, _)) = missing {
                (self.unref)(font);
            }
            (self.description_free)(desc);
            (self.unref)(layout);
            (self.unref)(context);
            result
        }
    }
}
fn api() -> Option<&'static Api> {
    static API: OnceLock<Option<Api>> = OnceLock::new();
    API.get_or_init(|| unsafe { Api::load() }).as_ref()
}
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
struct Key {
    family: String,
    bold: bool,
    italic: bool,
    size: u32,
    text: String,
}
struct Entry {
    image: egui::ColorImage,
    texture: Option<egui::TextureHandle>,
    colored: bool,
}
type FontKey = (String, bool, bool, u32);
type FontMetrics = (f32, f32, f32);
#[derive(Default)]
struct Cache {
    entries: BTreeMap<Key, Entry>,
    bytes: usize,
    font_extents: BTreeMap<FontKey, FontMetrics>,
    advances: BTreeMap<Key, f32>,
}
fn cache(ctx: &egui::Context) -> Arc<Mutex<Cache>> {
    let id = egui::Id::new("Source CSS Pango text cache");
    if let Some(cache) = ctx.data(|d| d.get_temp::<Arc<Mutex<Cache>>>(id)) {
        return cache;
    }
    let cache = Arc::new(Mutex::new(Cache::default()));
    ctx.data_mut(|d| d.insert_temp(id, cache.clone()));
    cache
}
pub(crate) fn css_family(family: &str) -> String {
    static FAMILIES: OnceLock<Mutex<BTreeMap<String, String>>> = OnceLock::new();
    let mut families = FAMILIES.get_or_init(Mutex::default).lock().unwrap();
    families
        .entry(family.to_owned())
        .or_insert_with(|| {
            // WebKit's non-colored cluster fallback uses the normal sans face
            // here (independently measured for the absent Segoe/Noto profile).
            // Pango alone instead uses the replacement face's digit emoji advances.
            // Keep explicitly selected/installed color families unchanged;
            // per-character fallback still supplies actual colored emoji.
            if api().and_then(|api| api.substituted_color_family(family)) == Some(true) {
                "sans-serif".to_owned()
            } else {
                family.to_owned()
            }
        })
        .clone()
}
fn key(font: &FontId, text: &str, ppp: f32) -> Option<Key> {
    let FontFamily::Name(name) = &font.family else {
        return None;
    };
    let (role, family) = name.split_once(':')?;
    if !role.starts_with("Bio") && !role.starts_with("SourceCss") {
        return None;
    }
    Some(Key {
        family: css_family(family),
        bold: role.contains("Bold"),
        italic: role.contains("Italic"),
        size: (font.size * ppp).to_bits(),
        text: text.to_owned(),
    })
}
fn entry<'a>(cache: &'a mut Cache, key: &Key) -> Option<&'a mut Entry> {
    if !cache.entries.contains_key(key) {
        let image = api()?.raster(
            &key.family,
            key.bold,
            key.italic,
            f32::from_bits(key.size),
            &key.text,
        )?;
        if cache.entries.len() >= 1024 || cache.bytes + image.pixels.len() * 4 > 32 * 1024 * 1024 {
            cache.entries.clear();
            cache.bytes = 0;
        }
        cache.bytes += image.pixels.len() * 4;
        cache.entries.insert(
            key.clone(),
            Entry {
                colored: image
                    .pixels
                    .iter()
                    .any(|pixel| pixel.r() != pixel.g() || pixel.g() != pixel.b()),
                image,
                texture: None,
            },
        );
    }
    cache.entries.get_mut(key)
}
pub(crate) fn font(family: &str, size: f32, bold: bool, italic: bool) -> FontId {
    if api().is_none() {
        return FontId::proportional(size);
    }
    FontId::new(
        size,
        FontFamily::Name(
            format!(
                "SourceCss{}{}:{family}",
                if bold { "Bold" } else { "" },
                if italic { "Italic" } else { "" }
            )
            .into(),
        ),
    )
}
pub(crate) fn measure_advance(ui: &egui::Ui, text: &str, font: &FontId) -> Option<f32> {
    let ppp = ui.ctx().pixels_per_point();
    let key = key(font, text, ppp)?;
    let cache = cache(ui.ctx());
    let mut cache = cache.lock().unwrap();
    if let Some(width) = cache.advances.get(&key) {
        return Some(*width / ppp);
    }
    let width = api()?.advance(
        &key.family,
        key.bold,
        key.italic,
        f32::from_bits(key.size),
        text,
    )?;
    if cache.advances.len() >= 1024 {
        cache.advances.clear();
    }
    cache.advances.insert(key, width);
    Some(width / ppp)
}
pub(crate) fn font_extents(ui: &egui::Ui, font: &FontId) -> Option<(f32, f32, f32)> {
    let ppp = ui.ctx().pixels_per_point();
    let key = key(font, "", ppp)?;
    let font_key = (key.family.clone(), key.bold, key.italic, key.size);
    let cache = cache(ui.ctx());
    let mut cache = cache.lock().unwrap();
    let metrics = if let Some(metrics) = cache.font_extents.get(&font_key) {
        *metrics
    } else {
        let metrics =
            api()?.font_extents(&key.family, key.bold, key.italic, f32::from_bits(key.size))?;
        cache.font_extents.insert(font_key, metrics);
        metrics
    };
    Some((metrics.0 / ppp, metrics.1 / ppp, metrics.2 / ppp))
}
pub(crate) fn line_height(ui: &egui::Ui, font: &FontId) -> Option<f32> {
    font_extents(ui, font).map(|(ascent, descent, gap)| ascent + descent + gap)
}
pub(crate) fn measure(ui: &egui::Ui, text: &str, font: &FontId) -> Option<Vec2> {
    if text.is_empty() {
        return measure(ui, " ", font).map(|size| Vec2::new(0.0, size.y));
    }
    let ppp = ui.ctx().pixels_per_point();
    let key = key(font, text, ppp)?;
    let cache = cache(ui.ctx());
    let mut cache = cache.lock().unwrap();
    let entry = entry(&mut cache, &key)?;
    Some(Vec2::new(
        entry.image.size[0] as f32 / ppp,
        entry.image.size[1] as f32 / ppp,
    ))
}
pub(crate) fn paint(
    ui: &egui::Ui,
    text: &str,
    font: &FontId,
    position: Pos2,
    align: Align2,
    color: Color32,
    clip: Rect,
) -> bool {
    if text.is_empty() {
        return true;
    }
    let Some(key) = key(font, text, ui.ctx().pixels_per_point()) else {
        return false;
    };
    let cache = cache(ui.ctx());
    let mut cache = cache.lock().unwrap();
    let Some(entry) = entry(&mut cache, &key) else {
        return false;
    };
    let texture = entry.texture.get_or_insert_with(|| {
        ui.ctx().load_texture(
            "Source CSS text",
            entry.image.clone(),
            egui::TextureOptions::NEAREST,
        )
    });
    let size = Vec2::new(entry.image.size[0] as f32, entry.image.size[1] as f32)
        / ui.ctx().pixels_per_point();
    let raw = align.anchor_size(position, size);
    let ppp = ui.ctx().pixels_per_point();
    let rect = Rect::from_min_size(
        egui::pos2(
            (raw.min.x * ppp).round() / ppp,
            (raw.min.y * ppp).round() / ppp,
        ),
        size,
    );
    ui.painter().with_clip_rect(clip).image(
        texture.id(),
        rect,
        Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
        if entry.colored { Color32::WHITE } else { color },
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires the private Noto-only Fontconfig profile"]
    fn missing_segoe_uses_css_text_fallback_with_colored_emoji() {
        assert_eq!(css_family("Segoe UI Emoji"), "sans-serif");
        let api = api().expect("actual Pango/Cairo runtime");
        let image = api
            .raster(&css_family("Segoe UI Emoji"), false, false, 32.0, "😀")
            .unwrap();
        assert!(image
            .pixels
            .iter()
            .any(|pixel| pixel.r() != pixel.g() || pixel.g() != pixel.b()));
        assert_eq!(css_family("Noto Color Emoji"), "Noto Color Emoji");
        // Real WebKit canvas in the private Noto-only profile, CSS15px Segoe.
        for (text, width) in [
            ("Aurasus", 61),
            ("Bacterium", 77),
            ("20.01M", 58),
            ("1M", 23),
            ("500m", 45),
        ] {
            let image = api
                .raster(&css_family("Segoe UI Emoji"), false, false, 15.0, text)
                .unwrap();
            assert_eq!(image.size[0], width, "original CSS text width {text}");
        }
    }
    #[test]
    #[ignore = "requires an explicitly selected color-font Fontconfig profile"]
    fn real_color_font_retains_cairo_palette_and_untinted_texture() {
        let family = std::env::var("OBSERVATORY_QA_COLOR_FONT")
            .expect("set OBSERVATORY_QA_COLOR_FONT in a private Fontconfig profile");
        let mut cache = Cache::default();
        let entry = entry(
            &mut cache,
            &Key {
                family,
                bold: false,
                italic: false,
                size: 32.0_f32.to_bits(),
                text: "😀".into(),
            },
        )
        .expect("actual Pango/Cairo color-font raster");
        assert!(
            entry.colored,
            "real color glyph must select the untinted painting branch"
        );
        let palette: std::collections::BTreeSet<_> = entry
            .image
            .pixels
            .iter()
            .filter(|pixel| pixel.a() > 128 && pixel.r() != pixel.b())
            .map(|pixel| [pixel.r(), pixel.g(), pixel.b()])
            .collect();
        assert!(
            palette.len() > 10,
            "preserve the real glyph's multiple colors: {palette:?}"
        );
    }
    #[test]
    fn real_css_font_metrics_and_gamma_alpha_match_webkit_source_probe() {
        let Some(api) = api() else { return };
        assert_eq!(css_family("DejaVu Sans"), "DejaVu Sans");
        // Source DOM probe: Segoe's system fallback is DejaVu Sans here.
        // WebKit canvas measures these strings at77/82/61 CSS pixels.
        for (text, width) in [("Bacterium", 77), ("Metallicum", 82), ("Aurasus", 61)] {
            let image = api.raster("DejaVu Sans", false, false, 15.0, text).unwrap();
            assert_eq!(image.size, [width, 18], "{text}");
            assert!(image
                .pixels
                .iter()
                .any(|pixel| pixel.a() > 0 && pixel.a() < 255));
            assert!(image.pixels.iter().all(|pixel| pixel.r() == pixel.a()
                && pixel.g() == pixel.a()
                && pixel.b() == pixel.a()));
        }
        let description="A bacterial species found in ammonia-based atmospheres that lives in sunlight. A colony's appearance resembles an intricate maze.";
        let width = api
            .advance("Verdana", false, false, 16.0, description)
            .unwrap();
        eprintln!("source description advance={width}");
        assert!(
            (width - 1_056.296_9).abs() < 0.001,
            "real WebKit source maxcontent1056.296875, native{width}"
        );
        for (size, height) in [(13.0, 15.0), (15.0, 18.0), (18.72, 21.0)] {
            assert_eq!(
                api.font_extents("DejaVu Sans", false, false, size)
                    .map(|(a, d, g)| a + d + g),
                Some(height)
            );
        }
        assert!(api
            .raster("DejaVu Sans", false, false, f32::NAN, "x")
            .is_none());
        assert!(api
            .raster("DejaVu Sans", false, false, 15.0, "nul\0text")
            .is_none());
        assert!(api
            .raster(
                "DejaVu Sans",
                false,
                false,
                15.0,
                "Valid after rejected input"
            )
            .is_some());
    }
}
