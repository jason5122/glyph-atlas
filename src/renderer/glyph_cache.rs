use crate::gl::types::*;

#[derive(Copy, Clone, Debug)]
pub struct Glyph {
    pub tex_id: GLuint,
    pub top: i16,
    pub left: i16,
    pub width: i16,
    pub height: i16,
    pub uv_bot: f32,
    pub uv_left: f32,
    pub uv_width: f32,
    pub uv_height: f32,
}
