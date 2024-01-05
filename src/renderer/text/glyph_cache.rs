use std::collections::hash_map::RandomState;
use std::collections::HashMap;

use crossfont::Size as FontSize;
use crossfont::{
    Error as RasterizerError, FontDesc, FontKey, GlyphKey, Metrics, Rasterize, RasterizedGlyph,
    Rasterizer, Style,
};
use unicode_width::UnicodeWidthChar;

use crate::gl::types::*;

/// `LoadGlyph` allows for copying a rasterized glyph into graphics memory.
pub trait LoadGlyph {
    /// Load the rasterized glyph into GPU memory.
    fn load_glyph(&mut self, rasterized: &RasterizedGlyph) -> Glyph;
}

#[derive(Copy, Clone, Debug)]
pub struct Glyph {
    pub tex_id: GLuint,
    pub multicolor: bool,
    pub top: i16,
    pub left: i16,
    pub width: i16,
    pub height: i16,
    pub uv_bot: f32,
    pub uv_left: f32,
    pub uv_width: f32,
    pub uv_height: f32,
}

/// Naïve glyph cache.
///
/// Currently only keyed by `char`, and thus not possible to hold different
/// representations of the same code point.
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
        let font_size = FontSize::new(16.);
        let (regular, bold, italic, bold_italic) =
            Self::compute_font_keys(&mut rasterizer, font_size).unwrap();

        let metrics = rasterizer.metrics(regular, font_size).unwrap();

        Self {
            cache: Default::default(),
            rasterizer,
            font_size,
            font_key: regular,
            bold_key: bold,
            italic_key: italic,
            bold_italic_key: bold_italic,
            metrics,
        }
    }

    fn load_glyphs_for_font<L: LoadGlyph>(&mut self, font: FontKey, loader: &mut L) {
        let size = self.font_size;

        // Cache all ascii characters.
        for i in 32u8..=126u8 {
            self.get(GlyphKey { font_key: font, character: i as char, size }, loader, true);
        }
    }

    /// Computes font keys for (Regular, Bold, Italic, Bold Italic).
    fn compute_font_keys(
        rasterizer: &mut Rasterizer,
        font_size: crossfont::Size,
    ) -> Result<(FontKey, FontKey, FontKey, FontKey), crossfont::Error> {
        let font_name = String::from("Source Code Pro");

        let regular_desc = FontDesc::new(&font_name, Style::Specific(String::from("Regular")));
        let bold_desc = FontDesc::new(&font_name, Style::Specific(String::from("Bold")));
        let italic_desc = FontDesc::new(&font_name, Style::Specific(String::from("Italic")));
        let bold_italic_desc =
            FontDesc::new(&font_name, Style::Specific(String::from("Bold Italic")));

        let regular = rasterizer.load_font(&regular_desc, font_size).unwrap();
        let bold = rasterizer.load_font(&bold_desc, font_size).unwrap();
        let italic = rasterizer.load_font(&italic_desc, font_size).unwrap();
        let bold_italic = rasterizer.load_font(&bold_italic_desc, font_size).unwrap();

        Ok((regular, bold, italic, bold_italic))
    }

    /// Get a glyph from the font.
    ///
    /// If the glyph has never been loaded before, it will be rasterized and inserted into the
    /// cache.
    ///
    /// # Errors
    ///
    /// This will fail when the glyph could not be rasterized. Usually this is due to the glyph
    /// not being present in any font.
    pub fn get<L: ?Sized>(
        &mut self,
        glyph_key: GlyphKey,
        loader: &mut L,
        show_missing: bool,
    ) -> Glyph
    where
        L: LoadGlyph,
    {
        // Try to load glyph from cache.
        if let Some(glyph) = self.cache.get(&glyph_key) {
            return *glyph;
        };

        // Rasterize the glyph using the built-in font for special characters or the user's font
        // for everything else.
        let rasterized = self.rasterizer.get_glyph(glyph_key);

        let glyph = match rasterized {
            Ok(rasterized) => self.load_glyph(loader, rasterized),
            // Load fallback glyph.
            Err(RasterizerError::MissingGlyph(rasterized)) if show_missing => {
                // Use `\0` as "missing" glyph to cache it only once.
                let missing_key = GlyphKey { character: '\0', ..glyph_key };
                if let Some(glyph) = self.cache.get(&missing_key) {
                    *glyph
                } else {
                    // If no missing glyph was loaded yet, insert it as `\0`.
                    let glyph = self.load_glyph(loader, rasterized);
                    self.cache.insert(missing_key, glyph);

                    glyph
                }
            },
            Err(_) => self.load_glyph(loader, Default::default()),
        };

        // Cache rasterized glyph.
        *self.cache.entry(glyph_key).or_insert(glyph)
    }

    /// Load glyph into the atlas.
    ///
    /// This will apply all transforms defined for the glyph cache to the rasterized glyph before
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

    /// Prefetch glyphs that are almost guaranteed to be loaded anyways.
    pub fn load_common_glyphs<L: LoadGlyph>(&mut self, loader: &mut L) {
        self.load_glyphs_for_font(self.font_key, loader);
        self.load_glyphs_for_font(self.bold_key, loader);
        self.load_glyphs_for_font(self.italic_key, loader);
        self.load_glyphs_for_font(self.bold_italic_key, loader);
    }
}
