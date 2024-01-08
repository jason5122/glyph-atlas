use std::ffi::CString;

use glutin::context::PossiblyCurrentContext;
use glutin::display::{GetGlDisplay, GlDisplay};

use crate::gl;

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

        Self { vao, ebo, vbo_instance, tex_id }
    }

    pub fn draw_cells(&mut self) {
        unsafe {
            draw(self.vao, self.ebo, self.vbo_instance, self.tex_id);
        }
    }
}
