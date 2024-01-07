use crossfont::Size;
use crossfont::{FontDesc, FontKey, Metrics, Rasterizer};

use crate::gl::types::*;

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
    pub rasterizer: Rasterizer,
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

        Self { rasterizer, font_size, font_key, bold_key, italic_key, bold_italic_key, metrics }
    }
}
