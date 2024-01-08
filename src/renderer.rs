use std::borrow::Cow;
use std::ffi::CString;

use crossfont::{FontKey, GlyphKey, Rasterizer, Size};

use glutin::context::PossiblyCurrentContext;
use glutin::display::{GetGlDisplay, GlDisplay};

use crate::gl;
use crate::gl::types::*;

pub mod platform;

include!(concat!(env!("OUT_DIR"), "/cpp_bindings.rs"));

#[derive(Debug)]
#[repr(C)]
pub struct InstanceData {
    // Coords.
    pub col: u16,
    pub row: u16,

    // Glyph offset and size.
    pub left: i16,
    pub top: i16,
    pub width: i16,
    pub height: i16,

    // UV offset and scale.
    pub uv_left: f32,
    pub uv_bot: f32,
    pub uv_width: f32,
    pub uv_height: f32,
}

#[derive(Debug)]
pub struct Glsl3Renderer {
    shader_program: GLuint,
    vao: GLuint,
    ebo: GLuint,
    vbo_instance: GLuint,
    tex_id: GLuint,
}

impl Glsl3Renderer {
    pub fn new(context: &PossiblyCurrentContext) -> Self {
        let gl_display = context.display();
        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        });

        let mut vao: GLuint = 0;
        let mut ebo: GLuint = 0;
        let mut vbo_instance: GLuint = 0;
        let mut tex_id: GLuint = 0;

        unsafe {
            renderer_setup(&mut vao, &mut ebo, &mut vbo_instance, &mut tex_id);
        }

        macro_rules! cstr {
            ($s:literal) => {
                // This can be optimized into an no-op with pre-allocated NUL-terminated bytes.
                std::ffi::CStr::from_ptr(concat!($s, "\0").as_ptr().cast())
            };
        }

        unsafe {
            let shader_program = gl::CreateProgram();
            let vertex_shader = Shader::new(gl::VERTEX_SHADER, include_str!("../res/text.v.glsl"));
            let fragment_shader =
                Shader::new(gl::FRAGMENT_SHADER, include_str!("../res/text.f.glsl"));

            gl::AttachShader(shader_program, vertex_shader.0);
            gl::AttachShader(shader_program, fragment_shader.0);
            gl::LinkProgram(shader_program);

            Self { shader_program, vao, ebo, vbo_instance, tex_id }
        }
    }

    pub fn draw_cells(&mut self, rasterizer: &mut Rasterizer, font_key: FontKey, font_size: Size) {
        unsafe {
            draw(self.vao, self.ebo, self.vbo_instance, self.tex_id, self.shader_program);
        }
    }
}

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
