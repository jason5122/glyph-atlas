use std::ffi::CStr;

use bitflags::bitflags;

use crate::display::SizeInfo;
use crate::gl;
use crate::gl::types::*;
use crate::renderer::RenderableCell;

pub mod atlas;
pub mod glyph_cache;

pub use glyph_cache::GlyphCache;
pub use glyph_cache::{Glyph, LoadGlyph};

use std::mem::size_of;

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
pub enum RenderingPass {
    /// Rendering pass used to render background color in text shaders.
    Background = 0,

    /// The first pass to render text.
    SubpixelPass1 = 1,
}

static TEXT_SHADER_F: &str = include_str!("../../res/glsl3/text.f.glsl");
static TEXT_SHADER_V: &str = include_str!("../../res/glsl3/text.v.glsl");

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
    pub tex: GLuint,
    pub instances: Vec<InstanceData>,
}

impl Batch {
    pub fn add_item(&mut self, cell: &RenderableCell, glyph: &Glyph) {
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
        macro_rules! cstr {
            ($s:literal) => {
                // This can be optimized into an no-op with pre-allocated NUL-terminated bytes.
                unsafe { std::ffi::CStr::from_ptr(concat!($s, "\0").as_ptr().cast()) }
            };
        }

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

    pub fn set_rendering_pass(&self, rendering_pass: RenderingPass) {
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

/// A wrapper for a shader program id, with automatic lifetime management.
#[derive(Debug)]
pub struct ShaderProgram(GLuint);

impl ShaderProgram {
    pub fn new(
        shader_header: Option<&str>,
        vertex_shader: &'static str,
        fragment_shader: &'static str,
    ) -> Self {
        let vertex_shader = Shader::new(shader_header, gl::VERTEX_SHADER, vertex_shader);
        let fragment_shader = Shader::new(shader_header, gl::FRAGMENT_SHADER, fragment_shader);

        let program = unsafe { Self(gl::CreateProgram()) };

        let mut success: GLint = 0;
        unsafe {
            gl::AttachShader(program.id(), vertex_shader.id());
            gl::AttachShader(program.id(), fragment_shader.id());
            gl::LinkProgram(program.id());
            gl::GetProgramiv(program.id(), gl::LINK_STATUS, &mut success);
        }

        program
    }

    /// Get uniform location by name. Panic if failed.
    pub fn get_uniform_location(&self, name: &'static CStr) -> GLint {
        // This call doesn't require `UseProgram`.
        let ret = unsafe { gl::GetUniformLocation(self.id(), name.as_ptr()) };
        ret
    }

    /// Get the shader program id.
    pub fn id(&self) -> GLuint {
        self.0
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.0) }
    }
}

/// A wrapper for a shader id, with automatic lifetime management.
#[derive(Debug)]
struct Shader(GLuint);

impl Shader {
    fn new(shader_header: Option<&str>, kind: GLenum, source: &'static str) -> Self {
        let version_header = "#version 330 core\n";
        let mut sources = Vec::<*const GLchar>::with_capacity(3);
        let mut lengthes = Vec::<GLint>::with_capacity(3);
        sources.push(version_header.as_ptr().cast());
        lengthes.push(version_header.len() as GLint);

        if let Some(shader_header) = shader_header {
            sources.push(shader_header.as_ptr().cast());
            lengthes.push(shader_header.len() as GLint);
        }

        sources.push(source.as_ptr().cast());
        lengthes.push(source.len() as GLint);

        let shader = unsafe { Self(gl::CreateShader(kind)) };

        let mut success: GLint = 0;
        unsafe {
            gl::ShaderSource(
                shader.id(),
                lengthes.len() as GLint,
                sources.as_ptr().cast(),
                lengthes.as_ptr(),
            );
            gl::CompileShader(shader.id());
            gl::GetShaderiv(shader.id(), gl::COMPILE_STATUS, &mut success);
        }

        shader
    }

    fn id(&self) -> GLuint {
        self.0
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.0) }
    }
}
