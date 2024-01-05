use std::ffi::CString;
use std::mem::size_of;
use std::ptr;

use glutin::context::PossiblyCurrentContext;
use glutin::display::{GetGlDisplay, GlDisplay};

use crate::display::{RenderableCell, Rgb, SizeInfo};
use crate::gl;
use crate::gl::types::*;

use text::atlas::{Atlas, ATLAS_SIZE};

pub mod platform;
mod shader;
pub mod text;

pub use text::{Batch, GlyphCache, InstanceData, LoaderApi, RenderApi, TextShaderProgram};

macro_rules! cstr {
    ($s:literal) => {
        // This can be optimized into an no-op with pre-allocated NUL-terminated bytes.
        unsafe { std::ffi::CStr::from_ptr(concat!($s, "\0").as_ptr().cast()) }
    };
}
pub(crate) use cstr;

/// Maximum items to be drawn in a batch.
const BATCH_MAX: usize = 0x1_0000;

#[derive(Debug)]
pub struct Glsl3Renderer {
    program: TextShaderProgram,
    vao: GLuint,
    ebo: GLuint,
    vbo_instance: GLuint,
    atlas: Vec<Atlas>,
    current_atlas: usize,
    active_tex: GLuint,
    batch: Batch,
}

impl Glsl3Renderer {
    pub fn new(context: &PossiblyCurrentContext) -> Self {
        let gl_display = context.display();
        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        });

        let program = TextShaderProgram::new();
        let mut vao: GLuint = 0;
        let mut ebo: GLuint = 0;
        let mut vbo_instance: GLuint = 0;

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC1_COLOR, gl::ONE_MINUS_SRC1_COLOR);

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

            macro_rules! add_attr {
                ($count:expr, $gl_type:expr, $type:ty) => {
                    gl::VertexAttribPointer(
                        index,
                        $count,
                        $gl_type,
                        gl::FALSE,
                        size_of::<InstanceData>() as i32,
                        size as *const _,
                    );
                    gl::EnableVertexAttribArray(index);
                    gl::VertexAttribDivisor(index, 1);

                    #[allow(unused_assignments)]
                    {
                        size += $count * size_of::<$type>();
                        index += 1;
                    }
                };
            }

            // Coords.
            add_attr!(2, gl::UNSIGNED_SHORT, u16);

            // Glyph offset and size.
            add_attr!(4, gl::SHORT, i16);

            // UV offset.
            add_attr!(4, gl::FLOAT, f32);

            // Color and cell flags.
            //
            // These are packed together because of an OpenGL driver issue on macOS, which caused a
            // `vec3(u8)` text color and a `u8` cell flags to increase the rendering time by a
            // huge margin.
            add_attr!(4, gl::UNSIGNED_BYTE, u8);

            // Background color.
            add_attr!(4, gl::UNSIGNED_BYTE, u8);

            // Cleanup.
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        Self {
            program,
            vao,
            ebo,
            vbo_instance,
            atlas: vec![Atlas::new(ATLAS_SIZE)],
            current_atlas: 0,
            active_tex: 0,
            batch: Batch::new(),
        }
    }

    pub fn draw_cells(&mut self, size_info: &SizeInfo, glyph_cache: &mut GlyphCache) {
        let mut cells = Vec::new();

        let strs = vec![
            "Hello world!",
            "let x = &[1, 2, 4];",
            "let mut iterator = x.iter();",
            "assert_eq!(iterator.next(), Some(&1));",
            "assert_eq!(iterator.next(), Some(&2));",
            "assert_eq!(iterator.next(), Some(&4));",
            "assert_eq!(iterator.next(), None);",
            "huh 🤨 🤨 🤨",
        ];
        // Red
        // let fg = Rgb::new(0xfc, 0xfd, 0xfd);
        // let bg = Rgb::new(0xec, 0x5f, 0x66);
        // Black
        let fg = Rgb::new(0x33, 0x33, 0x33);
        let bg = Rgb::new(0xfc, 0xfd, 0xfd);
        for (i, s) in strs.iter().enumerate() {
            for (column, character) in s.chars().enumerate() {
                let cell = RenderableCell {
                    character,
                    line: 10 + i,
                    column,
                    bg_alpha: 1.0,
                    fg,
                    bg,
                    font_key: 0,
                };
                cells.push(cell);
            }
        }

        unsafe {
            gl::UseProgram(self.program.id());
            self.program.set_term_uniforms(size_info);

            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_instance);
            gl::ActiveTexture(gl::TEXTURE0);
        }

        self.with_api(cells, size_info, glyph_cache);

        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            gl::UseProgram(0);
        }
    }

    pub fn with_api(
        &mut self,
        cells: Vec<RenderableCell>,
        size_info: &SizeInfo,
        glyph_cache: &mut GlyphCache,
    ) {
        let mut api = RenderApi {
            active_tex: &mut self.active_tex,
            batch: &mut self.batch,
            atlas: &mut self.atlas,
            current_atlas: &mut self.current_atlas,
            program: &mut self.program,
        };

        for cell in cells {
            api.draw_cell(cell, glyph_cache, size_info);
        }
    }

    /// Fill the window with `color` and `alpha`.
    pub fn clear(&self, color: Rgb, alpha: f32) {
        unsafe {
            gl::ClearColor(
                (f32::from(color.r) / 255.0).min(1.0) * alpha,
                (f32::from(color.g) / 255.0).min(1.0) * alpha,
                (f32::from(color.b) / 255.0).min(1.0) * alpha,
                alpha,
            );
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    /// Set the viewport for cell rendering.
    #[inline]
    pub fn set_viewport(&self, size: &SizeInfo) {
        unsafe {
            gl::Viewport(
                size.padding_x as i32,
                size.padding_y as i32,
                size.width as i32 - 2 * size.padding_x as i32,
                size.height as i32 - 2 * size.padding_y as i32,
            );
        }
    }

    pub fn resize(&self, size: &SizeInfo) {
        self.set_viewport(size);
        unsafe {
            let program = &self.program;
            gl::UseProgram(program.id());
            update_projection(program.projection_uniform(), size);
            gl::UseProgram(0);
        }
    }

    pub fn with_loader<F: FnOnce(LoaderApi) -> T, T>(&mut self, func: F) -> T {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
        }

        let loader_api = LoaderApi {
            active_tex: &mut self.active_tex,
            atlas: &mut self.atlas,
            current_atlas: &mut self.current_atlas,
        };
        func(loader_api)
    }
}

impl Drop for Glsl3Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo_instance);
            gl::DeleteBuffers(1, &self.ebo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

fn update_projection(u_projection: GLint, size: &SizeInfo) {
    let width = size.width;
    let height = size.height;
    let padding_x = size.padding_x;
    let padding_y = size.padding_y;

    // Bounds check.
    if (width as u32) < (2 * padding_x as u32) || (height as u32) < (2 * padding_y as u32) {
        return;
    }

    // Compute scale and offset factors, from pixel to ndc space. Y is inverted.
    //   [0, width - 2 * padding_x] to [-1, 1]
    //   [height - 2 * padding_y, 0] to [-1, 1]
    let scale_x = 2. / (width - 2. * padding_x);
    let scale_y = -2. / (height - 2. * padding_y);
    let offset_x = -1.;
    let offset_y = 1.;

    unsafe {
        gl::Uniform4f(u_projection, offset_x, offset_y, scale_x, scale_y);
    }
}
