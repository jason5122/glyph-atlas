use std::collections::HashMap;
use std::iter;
use std::ops::{Add, Mul};
use std::path::PathBuf;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

use core_foundation::array::{CFArray, CFIndex};
use core_foundation::base::{CFType, ItemRef, TCFType};
use core_foundation::number::{CFNumber, CFNumberRef};
use core_foundation::string::CFString;
use core_graphics::base::kCGImageAlphaPremultipliedFirst;
use core_graphics::color_space::CGColorSpace;
use core_graphics::context::CGContext;
use core_graphics::font::CGGlyph;
use core_graphics::geometry::{CGPoint, CGRect, CGSize};
use core_text::font::{
    cascade_list_for_languages as ct_cascade_list_for_languages, new_from_descriptor,
    new_from_name, CTFont,
};
use core_text::font_collection::create_for_family;
use core_text::font_descriptor::{
    self, kCTFontColorGlyphsTrait, kCTFontDefaultOrientation, kCTFontEnabledAttribute,
    CTFontDescriptor, SymbolicTraitAccessors,
};

use log::{trace, warn};

/// According to the documentation, the index of 0 must be a missing glyph character:
/// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM07/appendixB.html
const MISSING_GLYPH_INDEX: u32 = 0;

pub mod darwin;
use darwin::kCGBitmapByteOrder32Host;

pub struct Rasterizer {
    fonts: HashMap<FontKey, Font>,
    keys: HashMap<(FontDesc, Size), FontKey>,
    device_pixel_ratio: f32,
}

impl Rasterize for Rasterizer {
    fn new(device_pixel_ratio: f32) -> Rasterizer {
        Rasterizer { fonts: HashMap::new(), keys: HashMap::new(), device_pixel_ratio }
    }

    /// Get metrics for font specified by FontKey.
    fn metrics(&self, key: FontKey, _size: Size) -> Metrics {
        let font = self.fonts.get(&key).ok_or(Error::UnknownFontKey).unwrap();
        font.metrics()
    }

    fn load_font(&mut self, desc: &FontDesc, size: Size) -> Result<FontKey, Error> {
        let scaled_size = Size::new(size.as_f32_pts() * self.device_pixel_ratio);
        self.keys.get(&(desc.to_owned(), scaled_size)).map(|k| Ok(*k)).unwrap_or_else(|| {
            let font = self.get_font(desc, size)?;
            let key = FontKey::next();

            self.fonts.insert(key, font);
            self.keys.insert((desc.clone(), scaled_size), key);

            Ok(key)
        })
    }

    /// Get rasterized glyph for given glyph key.
    fn get_glyph(&mut self, glyph: GlyphKey) -> Result<RasterizedGlyph, Error> {
        // Get loaded font.
        let font = self.fonts.get(&glyph.font_key).ok_or(Error::UnknownFontKey)?;

        // Find a font where the given character is present.
        let (font, glyph_index) = iter::once(font)
            .chain(font.fallbacks.iter())
            .find_map(|font| match font.glyph_index(glyph.character) {
                MISSING_GLYPH_INDEX => None,
                glyph_index => Some((font, glyph_index)),
            })
            .unwrap_or((font, MISSING_GLYPH_INDEX));

        let glyph = font.get_glyph(glyph.character, glyph_index);

        if glyph_index == MISSING_GLYPH_INDEX {
            Err(Error::MissingGlyph(glyph))
        } else {
            Ok(glyph)
        }
    }

    fn update_dpr(&mut self, device_pixel_ratio: f32) {
        self.device_pixel_ratio = device_pixel_ratio;
    }
}

impl Rasterizer {
    fn get_specific_face(
        &mut self,
        desc: &FontDesc,
        style: &str,
        size: Size,
    ) -> Result<Font, Error> {
        let descriptors = descriptors_for_family(&desc.name[..]);
        for descriptor in descriptors {
            if descriptor.style_name == style {
                // Found the font we want.
                let scaled_size = f64::from(size.as_f32_pts()) * f64::from(self.device_pixel_ratio);
                let font = descriptor.to_font(scaled_size, true);
                return Ok(font);
            }
        }

        Err(Error::FontNotFound(desc.to_owned()))
    }

    fn get_matching_face(
        &mut self,
        desc: &FontDesc,
        slant: Slant,
        weight: Weight,
        size: Size,
    ) -> Result<Font, Error> {
        let bold = weight == Weight::Bold;
        let italic = slant != Slant::Normal;
        let scaled_size = f64::from(size.as_f32_pts()) * f64::from(self.device_pixel_ratio);

        let descriptors = descriptors_for_family(&desc.name[..]);
        for descriptor in descriptors {
            let font = descriptor.to_font(scaled_size, true);
            if font.is_bold() == bold && font.is_italic() == italic {
                // Found the font we want.
                return Ok(font);
            }
        }

        Err(Error::FontNotFound(desc.to_owned()))
    }

    fn get_font(&mut self, desc: &FontDesc, size: Size) -> Result<Font, Error> {
        match desc.style {
            Style::Specific(ref style) => self.get_specific_face(desc, style, size),
            Style::Description { slant, weight } => {
                self.get_matching_face(desc, slant, weight, size)
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontDesc {
    name: String,
    style: Style,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slant {
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weight {
    Normal,
    Bold,
}

/// Style of font.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Style {
    Specific(String),
    Description { slant: Slant, weight: Weight },
}

impl FontDesc {
    pub fn new<S>(name: S, style: Style) -> FontDesc
    where
        S: Into<String>,
    {
        FontDesc { name: name.into(), style }
    }
}

/// Identifier for a Font for use in maps/etc.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct FontKey {
    token: u32,
}

impl FontKey {
    /// Get next font key for given size.
    ///
    /// The generated key will be globally unique.
    pub fn next() -> FontKey {
        static TOKEN: AtomicUsize = AtomicUsize::new(0);

        FontKey { token: TOKEN.fetch_add(1, Ordering::SeqCst) as _ }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct GlyphKey {
    pub character: char,
    pub font_key: FontKey,
    pub size: Size,
}

/// Font size stored as integer.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Size(i16);

impl Size {
    /// Create a new `Size` from a f32 size in points.
    pub fn new(size: f32) -> Size {
        Size((size * Size::factor()) as i16)
    }

    /// Scale factor between font "Size" type and point size.
    #[inline]
    pub fn factor() -> f32 {
        2.0
    }

    /// Get the f32 size in points.
    pub fn as_f32_pts(self) -> f32 {
        f32::from(self.0) / Size::factor()
    }
}

impl<T: Into<Size>> Add<T> for Size {
    type Output = Size;

    fn add(self, other: T) -> Size {
        Size(self.0.saturating_add(other.into().0))
    }
}

impl<T: Into<Size>> Mul<T> for Size {
    type Output = Size;

    fn mul(self, other: T) -> Size {
        Size(self.0 * other.into().0)
    }
}

impl From<f32> for Size {
    fn from(float: f32) -> Size {
        Size::new(float)
    }
}

#[derive(Debug, Clone)]
pub struct RasterizedGlyph {
    pub character: char,
    pub width: i32,
    pub height: i32,
    pub top: i32,
    pub left: i32,
    pub advance: (i32, i32),
    pub buffer: BitmapBuffer,
}

#[derive(Clone, Debug)]
pub enum BitmapBuffer {
    /// RGB alphamask.
    Rgb(Vec<u8>),

    /// RGBA pixels with premultiplied alpha.
    Rgba(Vec<u8>),
}

impl Default for RasterizedGlyph {
    fn default() -> RasterizedGlyph {
        RasterizedGlyph {
            character: ' ',
            width: 0,
            height: 0,
            top: 0,
            left: 0,
            advance: (0, 0),
            buffer: BitmapBuffer::Rgb(Vec::new()),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Metrics {
    pub average_advance: f64,
    pub line_height: f64,
    pub descent: f32,
    pub underline_position: f32,
    pub underline_thickness: f32,
    pub strikeout_position: f32,
    pub strikeout_thickness: f32,
}

/// Errors occuring when using the rasterizer.
#[derive(Debug)]
pub enum Error {
    /// Unable to find a font matching the description.
    FontNotFound(FontDesc),

    /// Unable to find metrics for a font face.
    MetricsNotFound,

    /// The glyph could not be found in any font.
    MissingGlyph(RasterizedGlyph),

    /// Requested an operation with a FontKey that isn't known to the rasterizer.
    UnknownFontKey,

    /// Error from platfrom's font system.
    PlatformError(String),
}

pub trait Rasterize {
    /// Create a new Rasterizer.
    fn new(device_pixel_ratio: f32) -> Self
    where
        Self: Sized;

    /// Get `Metrics` for the given `FontKey`.
    fn metrics(&self, _: FontKey, _: Size) -> Metrics;

    /// Load the font described by `FontDesc` and `Size`.
    fn load_font(&mut self, _: &FontDesc, _: Size) -> Result<FontKey, Error>;

    /// Rasterize the glyph described by `GlyphKey`..
    fn get_glyph(&mut self, _: GlyphKey) -> Result<RasterizedGlyph, Error>;

    /// Update the Rasterizer's DPI factor.
    fn update_dpr(&mut self, device_pixel_ratio: f32);
}

/// A font.
#[derive(Clone)]
struct Font {
    ct_font: CTFont,
    fallbacks: Vec<Font>,
}

unsafe impl Send for Font {}

impl Font {
    fn metrics(&self) -> Metrics {
        let average_advance = self.glyph_advance('0');

        let ascent = self.ct_font.ascent().round() as f64;
        let descent = self.ct_font.descent().round() as f64;
        let leading = self.ct_font.leading().round() as f64;
        let line_height = ascent + descent + leading;

        // Strikeout and underline metrics.
        // CoreText doesn't provide strikeout so we provide our own.
        let underline_position = self.ct_font.underline_position() as f32;
        let underline_thickness = self.ct_font.underline_thickness() as f32;
        let strikeout_position = (line_height / 2. - descent) as f32;
        let strikeout_thickness = underline_thickness;

        Metrics {
            average_advance,
            line_height,
            descent: -(descent as f32),
            underline_position,
            underline_thickness,
            strikeout_position,
            strikeout_thickness,
        }
    }

    fn is_bold(&self) -> bool {
        self.ct_font.symbolic_traits().is_bold()
    }

    fn is_italic(&self) -> bool {
        self.ct_font.symbolic_traits().is_italic()
    }

    fn is_colored(&self) -> bool {
        (self.ct_font.symbolic_traits() & kCTFontColorGlyphsTrait) != 0
    }

    fn glyph_advance(&self, character: char) -> f64 {
        let index = self.glyph_index(character);

        let indices = [index as CGGlyph];

        unsafe {
            self.ct_font.get_advances_for_glyphs(
                kCTFontDefaultOrientation,
                &indices[0],
                ptr::null_mut(),
                1,
            )
        }
    }

    fn get_glyph(&self, character: char, glyph_index: u32) -> RasterizedGlyph {
        let bounds = self
            .ct_font
            .get_bounding_rects_for_glyphs(kCTFontDefaultOrientation, &[glyph_index as CGGlyph]);

        let rasterized_left = bounds.origin.x.floor() as i32;
        let rasterized_width =
            (bounds.origin.x - f64::from(rasterized_left) + bounds.size.width).ceil() as u32;
        let rasterized_descent = (-bounds.origin.y).ceil() as i32;
        let rasterized_ascent = (bounds.size.height + bounds.origin.y).ceil() as i32;
        let rasterized_height = (rasterized_descent + rasterized_ascent) as u32;

        if rasterized_width == 0 || rasterized_height == 0 {
            return RasterizedGlyph {
                character: ' ',
                width: 0,
                height: 0,
                top: 0,
                left: 0,
                advance: (0, 0),
                buffer: BitmapBuffer::Rgb(Vec::new()),
            };
        }

        let mut cg_context = CGContext::create_bitmap_context(
            None,
            rasterized_width as usize,
            rasterized_height as usize,
            8, // bits per component
            rasterized_width as usize * 4,
            &CGColorSpace::create_device_rgb(),
            kCGImageAlphaPremultipliedFirst | kCGBitmapByteOrder32Host,
        );

        let is_colored = self.is_colored();

        // Set background color for graphics context.
        let bg_a = if is_colored { 0.0 } else { 1.0 };
        cg_context.set_rgb_fill_color(0.0, 0.0, 0.0, bg_a);

        let context_rect = CGRect::new(
            &CGPoint::new(0.0, 0.0),
            &CGSize::new(f64::from(rasterized_width), f64::from(rasterized_height)),
        );

        cg_context.fill_rect(context_rect);

        cg_context.set_allows_font_smoothing(true);
        cg_context.set_should_smooth_fonts(false);
        cg_context.set_allows_font_subpixel_quantization(true);
        cg_context.set_should_subpixel_quantize_fonts(true);
        cg_context.set_allows_font_subpixel_positioning(true);
        cg_context.set_should_subpixel_position_fonts(true);
        cg_context.set_allows_antialiasing(true);
        cg_context.set_should_antialias(true);

        // Set fill color to white for drawing the glyph.
        cg_context.set_rgb_fill_color(1.0, 1.0, 1.0, 1.0);
        let rasterization_origin =
            CGPoint { x: f64::from(-rasterized_left), y: f64::from(rasterized_descent) };

        self.ct_font.draw_glyphs(
            &[glyph_index as CGGlyph],
            &[rasterization_origin],
            cg_context.clone(),
        );

        let rasterized_pixels = cg_context.data().to_vec();

        let buffer = if is_colored {
            BitmapBuffer::Rgba(darwin::extract_rgba(&rasterized_pixels))
        } else {
            BitmapBuffer::Rgb(darwin::extract_rgb(&rasterized_pixels))
        };

        RasterizedGlyph {
            character,
            left: rasterized_left,
            top: (bounds.size.height + bounds.origin.y).ceil() as i32,
            width: rasterized_width as i32,
            height: rasterized_height as i32,
            advance: (0, 0),
            buffer,
        }
    }

    fn glyph_index(&self, character: char) -> u32 {
        // Encode this char as utf-16.
        let mut buffer = [0; 2];
        let encoded: &[u16] = character.encode_utf16(&mut buffer);
        // And use the utf-16 buffer to get the index.
        self.glyph_index_utf16(encoded)
    }

    fn glyph_index_utf16(&self, encoded: &[u16]) -> u32 {
        // Output buffer for the glyph. for non-BMP glyphs, like
        // emojis, this will be filled with two chars the second
        // always being a 0.
        let mut glyphs: [CGGlyph; 2] = [0; 2];

        let res = unsafe {
            self.ct_font.get_glyphs_for_characters(
                encoded.as_ptr(),
                glyphs.as_mut_ptr(),
                encoded.len() as CFIndex,
            )
        };

        if res {
            u32::from(glyphs[0])
        } else {
            MISSING_GLYPH_INDEX
        }
    }
}

/// Font descriptor.
///
/// The descriptor provides data about a font and supports creating a font.
#[derive(Debug)]
struct Descriptor {
    style_name: String,
    font_path: PathBuf,

    ct_descriptor: CTFontDescriptor,
}

impl Descriptor {
    fn new(desc: CTFontDescriptor) -> Descriptor {
        Descriptor {
            style_name: desc.style_name(),
            font_path: desc.font_path().unwrap_or_else(PathBuf::new),
            ct_descriptor: desc,
        }
    }

    /// Create a Font from this descriptor.
    fn to_font(&self, size: f64, load_fallbacks: bool) -> Font {
        let ct_font = new_from_descriptor(&self.ct_descriptor, size);

        let fallbacks = if load_fallbacks {
            // TODO fixme, hardcoded en for english.
            let mut fallbacks = cascade_list_for_languages(&ct_font, &["en".to_owned()])
                .into_iter()
                .filter(|desc| !desc.font_path.as_os_str().is_empty())
                .map(|desc| desc.to_font(size, false))
                .collect::<Vec<_>>();

            // TODO, we can't use apple's proposed
            // .Apple Symbol Fallback (filtered out below),
            // but not having these makes us not able to render
            // many chars. We add the symbols back in.
            // Investigate if we can actually use the .-prefixed
            // fallbacks somehow.
            if let Ok(apple_symbols) = new_from_name("Apple Symbols", size) {
                fallbacks.push(Font { ct_font: apple_symbols, fallbacks: Vec::new() })
            };

            fallbacks
        } else {
            Vec::new()
        };

        Font { ct_font, fallbacks }
    }
}

/// Return fallback descriptors for font/language list.
fn cascade_list_for_languages(ct_font: &CTFont, languages: &[String]) -> Vec<Descriptor> {
    // Convert language type &Vec<String> -> CFArray.
    let langarr: CFArray<CFString> = {
        let tmp: Vec<CFString> = languages.iter().map(|language| CFString::new(language)).collect();
        CFArray::from_CFTypes(&tmp)
    };

    // CFArray of CTFontDescriptorRef (again).
    let list = ct_cascade_list_for_languages(ct_font, &langarr);

    // Convert CFArray to Vec<Descriptor>.
    list.into_iter().filter(is_enabled).map(|fontdesc| Descriptor::new(fontdesc.clone())).collect()
}

/// Check if a font is enabled.
fn is_enabled(fontdesc: &ItemRef<'_, CTFontDescriptor>) -> bool {
    unsafe {
        let descriptor = fontdesc.as_concrete_TypeRef();
        let attr_val =
            font_descriptor::CTFontDescriptorCopyAttribute(descriptor, kCTFontEnabledAttribute);

        if attr_val.is_null() {
            return false;
        }

        let attr_val = CFType::wrap_under_create_rule(attr_val);
        let attr_val = CFNumber::wrap_under_get_rule(attr_val.as_CFTypeRef() as CFNumberRef);

        attr_val.to_i32().unwrap_or(0) != 0
    }
}

/// Get descriptors for family name.
fn descriptors_for_family(family: &str) -> Vec<Descriptor> {
    let mut out = Vec::new();

    trace!("Family: {}", family);
    let ct_collection = create_for_family(family).unwrap_or_else(|| {
        // Fallback to Menlo if we can't find the config specified font family.
        warn!("Unable to load specified font {}, falling back to Menlo", &family);
        create_for_family("Menlo").expect("Menlo exists")
    });

    // CFArray of CTFontDescriptorRef (i think).
    let descriptors = ct_collection.get_descriptors();
    if let Some(descriptors) = descriptors {
        for descriptor in descriptors.iter() {
            out.push(Descriptor::new(descriptor.clone()));
        }
    }

    out
}
