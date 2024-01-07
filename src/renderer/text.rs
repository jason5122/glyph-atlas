use crate::gl::types::*;
use crate::renderer::RenderableCell;

pub mod atlas;
pub mod glyph_cache;

pub use glyph_cache::{Glyph, GlyphCache, LoadGlyph};

#[derive(Debug)]
#[repr(C)]
pub struct InstanceData {
    // Coords.
    col: u16,
    row: u16,

    // Glyph offset and size.
    left: i16,
    top: i16,
    width: i16,
    height: i16,

    // UV offset and scale.
    uv_left: f32,
    uv_bot: f32,
    uv_width: f32,
    uv_height: f32,
}

#[derive(Debug, Default)]
pub struct Batch {
    pub tex: GLuint,
    pub instances: Vec<InstanceData>,
}

impl Batch {
    pub fn add_item(&mut self, cell: &RenderableCell, glyph: &Glyph) {
        if self.instances.len() == 0 {
            self.tex = glyph.tex_id;
        }

        self.instances.push(InstanceData {
            col: cell.column as u16,
            row: cell.line as u16,

            top: glyph.top,
            left: glyph.left,
            width: glyph.width,
            height: glyph.height,

            uv_bot: glyph.uv_bot,
            uv_left: glyph.uv_left,
            uv_width: glyph.uv_width,
            uv_height: glyph.uv_height,
        });
    }

    #[inline]
    pub fn new() -> Self {
        Self { tex: 0, instances: Vec::new() }
    }
}
