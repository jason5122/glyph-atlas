use bitflags::bitflags;
use crossfont::RasterizedGlyph;

use crate::gl::types::*;

mod atlas;
mod glsl3;
pub mod glyph_cache;

use atlas::Atlas;
pub use glsl3::{Batch, Glsl3Renderer, RenderApi};
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
