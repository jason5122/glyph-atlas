use bitflags::bitflags;
use crossfont::{GlyphKey, RasterizedGlyph};

use crate::display::{RenderableCell, SizeInfo};
use crate::gl;
use crate::gl::types::*;
use crate::renderer::cstr;
use crate::renderer::shader::ShaderProgram;

pub mod atlas;
pub mod glyph_cache;

use atlas::Atlas;
pub use glyph_cache::GlyphCache;
use glyph_cache::{Glyph, LoadGlyph};

use std::mem::size_of;
use std::ptr;

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

// Shader source.
pub static TEXT_SHADER_F: &str = include_str!("../../res/glsl3/text.f.glsl");
static TEXT_SHADER_V: &str = include_str!("../../res/glsl3/text.v.glsl");

/// Maximum items to be drawn in a batch.
const BATCH_MAX: usize = 0x1_0000;

#[derive(Debug)]
pub struct RenderApi<'a> {
    pub active_tex: &'a mut GLuint,
    pub batch: &'a mut Batch,
    pub atlas: &'a mut Vec<Atlas>,
    pub current_atlas: &'a mut usize,
    pub program: &'a mut TextShaderProgram,
}

impl RenderApi<'_> {
    pub fn draw_cell(
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
        self.batch().add_item(&cell, &glyph, size_info);
    }

    fn batch(&mut self) -> &mut Batch {
        self.batch
    }

    fn render_batch(&mut self) {
        unsafe {
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                self.batch.size() as isize,
                self.batch.instances.as_ptr() as *const _,
            );
        }

        // Bind texture if necessary.
        if *self.active_tex != self.batch.tex {
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, self.batch.tex);
            }
            *self.active_tex = self.batch.tex;
        }

        unsafe {
            self.program.set_rendering_pass(RenderingPass::Background);
            gl::DrawElementsInstanced(
                gl::TRIANGLES,
                6,
                gl::UNSIGNED_INT,
                ptr::null(),
                self.batch.len() as GLsizei,
            );
            self.program.set_rendering_pass(RenderingPass::SubpixelPass1);
            gl::DrawElementsInstanced(
                gl::TRIANGLES,
                6,
                gl::UNSIGNED_INT,
                ptr::null(),
                self.batch.len() as GLsizei,
            );
        }
    }
}

impl<'a> LoadGlyph for RenderApi<'a> {
    fn load_glyph(&mut self, rasterized: &RasterizedGlyph) -> Glyph {
        Atlas::load_glyph(self.active_tex, self.atlas, self.current_atlas, rasterized)
    }
}

impl<'a> Drop for RenderApi<'a> {
    fn drop(&mut self) {
        self.render_batch();
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct InstanceData {
    // Coords.
    col: u16,
    row: u16,

    // Glyph offset.
    left: i16,
    top: i16,

    // Glyph size.
    width: i16,
    height: i16,

    // UV offset.
    uv_left: f32,
    uv_bot: f32,

    // uv scale.
    uv_width: f32,
    uv_height: f32,

    // Color.
    r: u8,
    g: u8,
    b: u8,

    // Cell flags like multicolor or fullwidth character.
    cell_flags: RenderingGlyphFlags,

    // Background color.
    bg_r: u8,
    bg_g: u8,
    bg_b: u8,
    bg_a: u8,
}

#[derive(Debug, Default)]
pub struct Batch {
    tex: GLuint,
    instances: Vec<InstanceData>,
}

impl Batch {
    pub fn add_item(&mut self, cell: &RenderableCell, glyph: &Glyph, _: &SizeInfo) {
        if self.len() == 0 {
            self.tex = glyph.tex_id;
        }

        let mut cell_flags = RenderingGlyphFlags::empty();
        cell_flags.set(RenderingGlyphFlags::COLORED, glyph.multicolor);

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

            r: cell.fg.r,
            g: cell.fg.g,
            b: cell.fg.b,
            cell_flags,

            bg_r: cell.bg.r,
            bg_g: cell.bg.g,
            bg_b: cell.bg.b,
            bg_a: (cell.bg_alpha * 255.0) as u8,
        });
    }

    #[inline]
    pub fn new() -> Self {
        Self { tex: 0, instances: Vec::with_capacity(BATCH_MAX) }
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

/// Text drawing program.
///
/// Uniforms are prefixed with "u", and vertex attributes are prefixed with "a".
#[derive(Debug)]
pub struct TextShaderProgram {
    /// Shader program.
    program: ShaderProgram,

    /// Projection scale and offset uniform.
    u_projection: GLint,

    /// Cell dimensions (pixels).
    u_cell_dim: GLint,

    /// Background pass flag.
    ///
    /// Rendering is split into two passes; one for backgrounds, and one for text.
    u_rendering_pass: GLint,
}

impl TextShaderProgram {
    pub fn new() -> TextShaderProgram {
        let program = ShaderProgram::new(None, TEXT_SHADER_V, TEXT_SHADER_F);
        Self {
            u_projection: program.get_uniform_location(cstr!("projection")),
            u_cell_dim: program.get_uniform_location(cstr!("cellDim")),
            u_rendering_pass: program.get_uniform_location(cstr!("renderingPass")),
            program,
        }
    }

    pub fn set_term_uniforms(&self, props: &SizeInfo) {
        unsafe {
            gl::Uniform2f(self.u_cell_dim, props.cell_width, props.cell_height);
        }
    }

    fn set_rendering_pass(&self, rendering_pass: RenderingPass) {
        let value = match rendering_pass {
            RenderingPass::Background | RenderingPass::SubpixelPass1 => rendering_pass as i32,
        };

        unsafe {
            gl::Uniform1i(self.u_rendering_pass, value);
        }
    }

    pub fn id(&self) -> GLuint {
        self.program.id()
    }

    pub fn projection_uniform(&self) -> GLint {
        self.u_projection
    }
}

#[derive(Debug)]
pub struct LoaderApi<'a> {
    pub active_tex: &'a mut GLuint,
    pub atlas: &'a mut Vec<Atlas>,
    pub current_atlas: &'a mut usize,
}

impl<'a> LoadGlyph for LoaderApi<'a> {
    fn load_glyph(&mut self, rasterized: &RasterizedGlyph) -> Glyph {
        Atlas::load_glyph(self.active_tex, self.atlas, self.current_atlas, rasterized)
    }
}
