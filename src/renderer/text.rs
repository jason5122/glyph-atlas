use std::mem::size_of;

use crate::gl;
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
        if self.len() == 0 {
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

    #[inline]
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.len() * size_of::<InstanceData>()
    }
}

#[derive(Debug)]
pub struct Shader(pub GLuint);

impl Shader {
    pub fn new(kind: GLenum, source: &'static str) -> Self {
        let mut sources = Vec::<*const GLchar>::with_capacity(3);
        let mut lengthes = Vec::<GLint>::with_capacity(3);

        sources.push(source.as_ptr().cast());
        lengthes.push(source.len() as GLint);

        let shader = unsafe { Self(gl::CreateShader(kind)) };

        unsafe {
            gl::ShaderSource(
                shader.0,
                lengthes.len() as GLint,
                sources.as_ptr().cast(),
                lengthes.as_ptr(),
            );
            gl::CompileShader(shader.0);
        }

        shader
    }
}
