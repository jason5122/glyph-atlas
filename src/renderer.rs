use std::ffi::CString;
use std::mem::size_of;
use std::ptr;

use crossfont::{FontKey, GlyphKey, Rasterizer, Size};

use glutin::context::PossiblyCurrentContext;
use glutin::display::{GetGlDisplay, GlDisplay};

use crate::display::SizeInfo;
use crate::gl;
use crate::gl::types::*;

mod atlas;
pub mod platform;

use atlas::{Atlas, ATLAS_SIZE};

/// Maximum items to be drawn in a batch.
const BATCH_MAX: usize = 0x1_0000;

#[derive(Debug)]
pub struct Glsl3Renderer {
    shader_program: GLuint,
    u_projection: GLint,
    u_cell_dim: GLint,
    vao: GLuint,
    ebo: GLuint,
    vbo_instance: GLuint,
    atlas: Atlas,
    active_tex: GLuint,
    tex: GLuint,
    instances: Vec<InstanceData>,
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

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC1_COLOR, gl::ONE_MINUS_SRC1_COLOR);

            gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

            // Disable depth mask, as the renderer never uses depth tests.
            gl::DepthMask(gl::FALSE);

            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut ebo);
            gl::GenBuffers(1, &mut vbo_instance);
            gl::BindVertexArray(vao);

            // ---------------------
            // Set up element buffer
            // ---------------------
            let indices: [u32; 6] = [0, 1, 3, 1, 2, 3];

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (6 * size_of::<u32>()) as isize,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            // ----------------------------
            // Setup vertex instance buffer
            // ----------------------------
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo_instance);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (BATCH_MAX * size_of::<InstanceData>()) as isize,
                ptr::null(),
                gl::STREAM_DRAW,
            );

            let mut index = 0;
            let mut size = 0;

            // Coords.
            gl::VertexAttribPointer(
                index,
                2,
                gl::UNSIGNED_SHORT,
                gl::FALSE,
                size_of::<InstanceData>() as i32,
                size as *const _,
            );
            gl::EnableVertexAttribArray(index);
            gl::VertexAttribDivisor(index, 1);

            // Cleanup.
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
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

            let u_projection = gl::GetUniformLocation(shader_program, cstr!("projection").as_ptr());
            let u_cell_dim = gl::GetUniformLocation(shader_program, cstr!("cellDim").as_ptr());

            Self {
                shader_program,
                u_projection,
                u_cell_dim,
                vao,
                ebo,
                vbo_instance,
                atlas: Atlas::new(ATLAS_SIZE),
                active_tex: 0,
                tex: 0,
                instances: Vec::new(),
            }
        }
    }

    pub fn draw_cells(
        &mut self,
        size_info: &SizeInfo,
        rasterizer: &mut Rasterizer,
        font_key: FontKey,
        font_size: Size,
    ) {
        unsafe {
            gl::UseProgram(self.shader_program);
            gl::Uniform2f(self.u_cell_dim, size_info.cell_width, size_info.cell_height);

            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_instance);
            gl::ActiveTexture(gl::TEXTURE0);
        }

        let strs = vec![
            "E",
            "Hello world!",
            "Hello world!",
            "Hello world!",
            "Hello world!",
            "Hello world!",
            "Hello world!",
            "Hello world!",
            "Hello world!",
            "Hello world!",
            "Hello world!",
            "Hello world!",
            "Hello world!",
            "Hello world!",
            "Hello world!",
            "Hello world!",
            "Hello world!",
            "Hello world!",
            "Hello world!",
            "Hello world!",
        ];

        for (i, s) in strs.iter().enumerate() {
            for (column, character) in s.chars().enumerate() {
                let line = 3 + i;

                let glyph_key = GlyphKey { font_key, size: font_size, character };
                let rasterized = rasterizer.get_glyph(glyph_key).unwrap();
                let glyph = self.atlas.insert_inner(&rasterized);

                if self.instances.len() == 0 {
                    self.tex = glyph.tex_id;
                }

                self.instances.push(InstanceData { col: (column + 5) as u16, row: line as u16 });
            }
        }

        unsafe {
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (self.instances.len() * size_of::<InstanceData>()) as isize,
                self.instances.as_ptr() as *const _,
            );

            // Bind texture if necessary.
            // if self.active_tex != self.tex {
            gl::BindTexture(gl::TEXTURE_2D, self.tex);
            //     self.active_tex = self.tex;
            // }

            gl::DrawElementsInstanced(
                gl::TRIANGLES,
                6,
                gl::UNSIGNED_INT,
                ptr::null(),
                self.instances.len() as GLsizei,
            );
        }
    }

    pub fn resize(&self, size: &SizeInfo) {
        unsafe {
            gl::Viewport(
                size.padding_x as i32,
                size.padding_y as i32,
                size.width as i32 - 2 * size.padding_x as i32,
                size.height as i32 - 2 * size.padding_y as i32,
            );

            gl::UseProgram(self.shader_program);

            let u_projection = self.u_projection;
            let width = size.width;
            let height = size.height;
            let padding_x = size.padding_x;
            let padding_y = size.padding_y;

            // Bounds check.
            if (width as u32) < (2 * padding_x as u32) || (height as u32) < (2 * padding_y as u32) {
                return;
            }

            // Compute scale and offset factors, from pixel to ndc space. Y is inverted.
            //   [0, width] to [-1, 1]
            //   [height, 0] to [-1, 1]
            gl::Uniform2f(u_projection, 2. / width, -2. / height);

            gl::UseProgram(0);
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

#[derive(Debug)]
#[repr(C)]
pub struct InstanceData {
    // Coords.
    pub col: u16,
    pub row: u16,
}
