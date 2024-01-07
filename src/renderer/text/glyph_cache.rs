use std::collections::hash_map::RandomState;
use std::collections::HashMap;

use crossfont::Size;
use crossfont::{FontDesc, FontKey, GlyphKey, Metrics, RasterizedGlyph, Rasterizer};
use unicode_width::UnicodeWidthChar;

use crate::gl::types::*;

pub trait LoadGlyph {
    fn load_glyph(&mut self, rasterized: &RasterizedGlyph) -> Glyph;
}

#[derive(Copy, Clone, Debug)]
pub struct Glyph {
    pub tex_id: GLuint,
    pub top: i16,
    pub left: i16,
    pub width: i16,
    pub height: i16,
    pub uv_bot: f32,
    pub uv_left: f32,
    pub uv_width: f32,
    pub uv_height: f32,
}

pub struct GlyphCache {
    cache: HashMap<GlyphKey, Glyph, RandomState>,
    rasterizer: Rasterizer,
    pub font_key: FontKey,
    pub bold_key: FontKey,
    pub italic_key: FontKey,
    pub bold_italic_key: FontKey,
    pub font_size: crossfont::Size,
    pub metrics: Metrics,
}

impl GlyphCache {
    pub fn new(mut rasterizer: Rasterizer) -> GlyphCache {
        let font_name = String::from("Source Code Pro");
        let font_size = Size::new(16.);

        let regular_desc = FontDesc::new(&font_name, &String::from("Regular"));
        let bold_desc = FontDesc::new(&font_name, &String::from("Bold"));
        let italic_desc = FontDesc::new(&font_name, &String::from("Italic"));
        let bold_italic_desc = FontDesc::new(&font_name, &String::from("Bold Italic"));

        let font_key = rasterizer.load_font(&regular_desc, font_size).unwrap();
        let bold_key = rasterizer.load_font(&bold_desc, font_size).unwrap();
        let italic_key = rasterizer.load_font(&italic_desc, font_size).unwrap();
        let bold_italic_key = rasterizer.load_font(&bold_italic_desc, font_size).unwrap();

        let metrics = rasterizer.metrics(font_key, font_size);

        Self {
            cache: Default::default(),
            rasterizer,
            font_size,
            font_key,
            bold_key,
            italic_key,
            bold_italic_key,
            metrics,
        }
    }

    pub fn get<L: ?Sized>(&mut self, glyph_key: GlyphKey, loader: &mut L) -> Glyph
    where
        L: LoadGlyph,
    {
        // Try to load glyph from cache.
        if let Some(glyph) = self.cache.get(&glyph_key) {
            return *glyph;
        };

        let rasterized = self.rasterizer.get_glyph(glyph_key).unwrap();
        let glyph = self.load_glyph(loader, rasterized);
        *self.cache.entry(glyph_key).or_insert(glyph)
    }

    pub fn load_glyph<L: ?Sized>(&self, loader: &mut L, mut glyph: RasterizedGlyph) -> Glyph
    where
        L: LoadGlyph,
    {
        glyph.top -= self.metrics.descent as i32;

        // The metrics of zero-width characters are based on rendering
        // the character after the current cell, with the anchor at the
        // right side of the preceding character. Since we render the
        // zero-width characters inside the preceding character, the
        // anchor has been moved to the right by one cell.
        if glyph.character.width() == Some(0) {
            glyph.left += self.metrics.average_advance as i32;
        }

        // Add glyph to cache.
        loader.load_glyph(&glyph)
    }
}
