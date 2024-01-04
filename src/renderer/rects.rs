use std::mem;

use crate::display::{Rgb, SizeInfo};
use crate::gl;
use crate::gl::types::*;
use crate::renderer::shader::{ShaderError, ShaderProgram};
use crate::renderer::{self, cstr};

#[derive(Debug, Copy, Clone)]
pub struct RenderRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: Rgb,
    pub alpha: f32,
    pub kind: RectKind,
}

impl RenderRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32, color: Rgb, alpha: f32) -> Self {
        RenderRect { kind: RectKind::Normal, x, y, width, height, color, alpha }
    }
}

// NOTE: These flags must be in sync with their usage in the rect.*.glsl shaders.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RectKind {
    Normal = 0,
    NumKinds = 1,
}

/// Shader sources for rect rendering program.
static RECT_SHADER_F: &str = include_str!("../../res/rect.f.glsl");
static RECT_SHADER_V: &str = include_str!("../../res/rect.v.glsl");

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Vertex {
    // Normalized screen coordinates.
    x: f32,
    y: f32,

    // Color.
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[derive(Debug)]
pub struct RectRenderer {
    // GL buffer objects.
    vao: GLuint,
    vbo: GLuint,

    programs: [RectShaderProgram; 1],
    vertices: [Vec<Vertex>; 4],
}

impl RectRenderer {
    pub fn new() -> Result<Self, renderer::Error> {
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;

        let rect_program = RectShaderProgram::new()?;

        unsafe {
            // Allocate buffers.
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);

            // VBO binding is not part of VAO itself, but VBO binding is stored in attributes.
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            let mut attribute_offset = 0;

            // Position.
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as i32,
                attribute_offset as *const _,
            );
            gl::EnableVertexAttribArray(0);
            attribute_offset += mem::size_of::<f32>() * 2;

            // Color.
            gl::VertexAttribPointer(
                1,
                4,
                gl::UNSIGNED_BYTE,
                gl::TRUE,
                mem::size_of::<Vertex>() as i32,
                attribute_offset as *const _,
            );
            gl::EnableVertexAttribArray(1);

            // Reset buffer bindings.
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        let programs = [rect_program];
        Ok(Self { vao, vbo, programs, vertices: Default::default() })
    }

    pub fn draw(&mut self, size_info: &SizeInfo, rects: Vec<RenderRect>) {
        unsafe {
            // Bind VAO to enable vertex attribute slots.
            gl::BindVertexArray(self.vao);

            // Bind VBO only once for buffer data upload only.
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        }

        let half_width = size_info.width() / 2.;
        let half_height = size_info.height() / 2.;

        // Build rect vertices vector.
        self.vertices.iter_mut().for_each(|vertices| vertices.clear());
        for rect in &rects {
            Self::add_rect(&mut self.vertices[rect.kind as usize], half_width, half_height, rect);
        }

        unsafe {
            // We iterate in reverse order to draw plain rects at the end, since we want visual
            // bell or damage rects be above the lines.
            for rect_kind in (RectKind::Normal as u8..RectKind::NumKinds as u8).rev() {
                let vertices = &mut self.vertices[rect_kind as usize];
                if vertices.is_empty() {
                    continue;
                }

                let program = &self.programs[rect_kind as usize];
                gl::UseProgram(program.id());
                program.update_uniforms(size_info);

                // Upload accumulated undercurl vertices.
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (vertices.len() * mem::size_of::<Vertex>()) as isize,
                    vertices.as_ptr() as *const _,
                    gl::STREAM_DRAW,
                );

                // Draw all vertices as list of triangles.
                gl::DrawArrays(gl::TRIANGLES, 0, vertices.len() as i32);
            }

            // Disable program.
            gl::UseProgram(0);

            // Reset buffer bindings to nothing.
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
    }

    fn add_rect(vertices: &mut Vec<Vertex>, half_width: f32, half_height: f32, rect: &RenderRect) {
        // Calculate rectangle vertices positions in normalized device coordinates.
        // NDC range from -1 to +1, with Y pointing up.
        let x = rect.x / half_width - 1.0;
        let y = -rect.y / half_height + 1.0;
        let width = rect.width / half_width;
        let height = rect.height / half_height;
        let (r, g, b) = rect.color.as_tuple();
        let a = (rect.alpha * 255.) as u8;

        // Make quad vertices.
        let quad = [
            Vertex { x, y, r, g, b, a },
            Vertex { x, y: y - height, r, g, b, a },
            Vertex { x: x + width, y, r, g, b, a },
            Vertex { x: x + width, y: y - height, r, g, b, a },
        ];

        // Append the vertices to form two triangles.
        vertices.push(quad[0]);
        vertices.push(quad[1]);
        vertices.push(quad[2]);
        vertices.push(quad[2]);
        vertices.push(quad[3]);
        vertices.push(quad[1]);
    }
}

impl Drop for RectRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

/// Rectangle drawing program.
#[derive(Debug)]
pub struct RectShaderProgram {
    /// Shader program.
    program: ShaderProgram,

    /// Cell width.
    u_cell_width: Option<GLint>,

    /// Cell height.
    u_cell_height: Option<GLint>,

    /// Terminal padding.
    u_padding_x: Option<GLint>,

    /// A padding from the bottom of the screen to viewport.
    u_padding_y: Option<GLint>,
}

impl RectShaderProgram {
    pub fn new() -> Result<Self, ShaderError> {
        let program = ShaderProgram::new(None, RECT_SHADER_V, RECT_SHADER_F)?;

        Ok(Self {
            u_cell_width: program.get_uniform_location(cstr!("cellWidth")).ok(),
            u_cell_height: program.get_uniform_location(cstr!("cellHeight")).ok(),
            u_padding_x: program.get_uniform_location(cstr!("paddingX")).ok(),
            u_padding_y: program.get_uniform_location(cstr!("paddingY")).ok(),
            program,
        })
    }

    fn id(&self) -> GLuint {
        self.program.id()
    }

    pub fn update_uniforms(&self, size_info: &SizeInfo) {
        let viewport_height = size_info.height() - size_info.padding_y();
        let padding_y = viewport_height
            - (viewport_height / size_info.cell_height()).floor() * size_info.cell_height();

        unsafe {
            if let Some(u_cell_width) = self.u_cell_width {
                gl::Uniform1f(u_cell_width, size_info.cell_width());
            }
            if let Some(u_cell_height) = self.u_cell_height {
                gl::Uniform1f(u_cell_height, size_info.cell_height());
            }
            if let Some(u_padding_y) = self.u_padding_y {
                gl::Uniform1f(u_padding_y, padding_y);
            }
            if let Some(u_padding_x) = self.u_padding_x {
                gl::Uniform1f(u_padding_x, size_info.padding_x());
            }
        }
    }
}
