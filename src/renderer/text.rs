use std::mem::size_of;

use bitflags::bitflags;

use crate::gl;
use crate::gl::types::*;
use crate::renderer::RenderableCell;

pub mod atlas;
pub mod glyph_cache;

pub use glyph_cache::{Glyph, GlyphCache, LoadGlyph};

// NOTE: These flags must be in sync with their usage in the text.*.glsl shaders.
bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct RenderingGlyphFlags: u8 {
        const COLORED   = 1;
    }
}

static TEXT_SHADER_F: &str = include_str!("../../res/text.f.glsl");
static TEXT_SHADER_V: &str = include_str!("../../res/text.v.glsl");

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
    cell_flags: u8,

    // Background color.
    bg_r: u8,
    bg_g: u8,
    bg_b: u8,
    bg_a: u8,
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

            r: cell.fg.r,
            g: cell.fg.g,
            b: cell.fg.b,
            cell_flags: glyph.multicolor as u8,

            bg_r: cell.bg.r,
            bg_g: cell.bg.g,
            bg_b: cell.bg.b,
            bg_a: (cell.bg_alpha * 255.0) as u8,
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

/// Text drawing program.
///
/// Uniforms are prefixed with "u", and vertex attributes are prefixed with "a".
#[derive(Debug)]
pub struct TextShaderProgram {
    /// Shader program.
    pub program: ShaderProgram,

    /// Projection scale and offset uniform.
    pub u_projection: GLint,

    /// Cell dimensions (pixels).
    pub u_cell_dim: GLint,
}

impl TextShaderProgram {
    pub fn new() -> TextShaderProgram {
        macro_rules! cstr {
            ($s:literal) => {
                // This can be optimized into an no-op with pre-allocated NUL-terminated bytes.
                std::ffi::CStr::from_ptr(concat!($s, "\0").as_ptr().cast())
            };
        }

        let program = ShaderProgram::new(None, TEXT_SHADER_V, TEXT_SHADER_F);
        unsafe {
            Self {
                u_projection: gl::GetUniformLocation(program.0, cstr!("projection").as_ptr()),
                u_cell_dim: gl::GetUniformLocation(program.0, cstr!("cellDim").as_ptr()),
                program,
            }
        }
    }
}

/// A wrapper for a shader program id, with automatic lifetime management.
#[derive(Debug)]
pub struct ShaderProgram(pub GLuint);

impl ShaderProgram {
    pub fn new(
        shader_header: Option<&str>,
        vertex_shader: &'static str,
        fragment_shader: &'static str,
    ) -> Self {
        let vertex_shader = Shader::new(shader_header, gl::VERTEX_SHADER, vertex_shader);
        let fragment_shader = Shader::new(shader_header, gl::FRAGMENT_SHADER, fragment_shader);

        let program = unsafe { Self(gl::CreateProgram()) };

        unsafe {
            gl::AttachShader(program.0, vertex_shader.0);
            gl::AttachShader(program.0, fragment_shader.0);
            gl::LinkProgram(program.0);
        }

        program
    }
}

/// A wrapper for a shader id, with automatic lifetime management.
#[derive(Debug)]
struct Shader(GLuint);

impl Shader {
    fn new(shader_header: Option<&str>, kind: GLenum, source: &'static str) -> Self {
        let mut sources = Vec::<*const GLchar>::with_capacity(3);
        let mut lengthes = Vec::<GLint>::with_capacity(3);

        if let Some(shader_header) = shader_header {
            sources.push(shader_header.as_ptr().cast());
            lengthes.push(shader_header.len() as GLint);
        }

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
