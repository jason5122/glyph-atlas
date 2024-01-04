use bitflags::bitflags;
use crossfont::{GlyphKey, RasterizedGlyph};

use crate::display::{RenderableCell, SizeInfo};
use crate::gl::types::*;

mod atlas;
mod glsl3;
pub mod glyph_cache;

use atlas::Atlas;
pub use glsl3::{Batch, Glsl3Renderer};
pub use glyph_cache::GlyphCache;
use glyph_cache::{Glyph, LoadGlyph};

// NOTE: These flags must be in sync with their usage in the text.*.glsl shaders.
bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct RenderingGlyphFlags: u8 {
        const COLORED   = 0b0000_0001;
        const WIDE_CHAR = 0b0000_0010;
    }
}

#[repr(u8)]
enum RenderingPass {
    /// Rendering pass used to render background color in text shaders.
    Background = 0,

    /// The first pass to render text.
    SubpixelPass1 = 1,
}

pub trait TextRenderer<'a> {
    type RenderApi: TextRenderApi;

    fn with_api<'b: 'a, F, T>(&'b mut self, size_info: &'b SizeInfo, func: F) -> T
    where
        F: FnOnce(Self::RenderApi) -> T;
}

pub trait TextRenderApi: LoadGlyph {
    fn batch(&mut self) -> &mut Batch;

    fn render_batch(&mut self);

    #[inline]
    fn add_render_item(&mut self, cell: &RenderableCell, glyph: &Glyph, size_info: &SizeInfo) {
        // Flush batch if tex changing.
        if !self.batch().is_empty() && self.batch().tex() != glyph.tex_id {
            self.render_batch();
        }

        self.batch().add_item(cell, glyph, size_info);

        if self.batch().full() {
            self.render_batch();
        }
    }

    fn draw_cell(
        &mut self,
        cell: RenderableCell,
        glyph_cache: &mut GlyphCache,
        size_info: &SizeInfo,
    ) {
        let font_key = match cell.font_key {
            0 => glyph_cache.font_key,
            1 => glyph_cache.bold_key,
            2 => glyph_cache.italic_key,
            3 => glyph_cache.bold_italic_key,
            _ => glyph_cache.font_key,
        };

        let glyph_key =
            GlyphKey { font_key, size: glyph_cache.font_size, character: cell.character };

        let glyph = glyph_cache.get(glyph_key, self, true);
        self.add_render_item(&cell, &glyph, size_info);
    }
}

pub trait TextShader {
    fn id(&self) -> GLuint;
    fn projection_uniform(&self) -> GLint;
}

#[derive(Debug)]
pub struct LoaderApi<'a> {
    active_tex: &'a mut GLuint,
    atlas: &'a mut Vec<Atlas>,
    current_atlas: &'a mut usize,
}

impl<'a> LoadGlyph for LoaderApi<'a> {
    fn load_glyph(&mut self, rasterized: &RasterizedGlyph) -> Glyph {
        Atlas::load_glyph(self.active_tex, self.atlas, self.current_atlas, rasterized)
    }

    fn clear(&mut self) {
        Atlas::clear_atlas(self.atlas, self.current_atlas)
    }
}
