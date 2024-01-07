use std::mem::size_of;

use crate::gl;
use crate::gl::types::*;
use crate::renderer::RenderableCell;

pub mod atlas;
pub mod glyph_cache;

pub use glyph_cache::{Glyph, GlyphCache, LoadGlyph};

static TEXT_SHADER_F: &str = include_str!("../../res/text.f.glsl");
static TEXT_SHADER_V: &str = include_str!("../../res/text.v.glsl");

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
pub struct TextShaderProgram {
    pub id: GLuint,
    pub u_projection: GLint,
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

        unsafe {
            let id = gl::CreateProgram();
            let vertex_shader = Shader::new(gl::VERTEX_SHADER, TEXT_SHADER_V);
            let fragment_shader = Shader::new(gl::FRAGMENT_SHADER, TEXT_SHADER_F);

            gl::AttachShader(id, vertex_shader.0);
            gl::AttachShader(id, fragment_shader.0);
            gl::LinkProgram(id);

            Self {
                id,
                u_projection: gl::GetUniformLocation(id, cstr!("projection").as_ptr()),
                u_cell_dim: gl::GetUniformLocation(id, cstr!("cellDim").as_ptr()),
            }
        }
    }
}

#[derive(Debug)]
struct Shader(GLuint);

impl Shader {
    fn new(kind: GLenum, source: &'static str) -> Self {
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
