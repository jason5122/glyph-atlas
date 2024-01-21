use std::borrow::Cow;
use std::ptr;

use crossfont::RasterizedGlyph;

use crate::gl;
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

/// Size of the Atlas.
pub const ATLAS_SIZE: i32 = 1024;

/// Manages a single texture atlas.
///
/// The strategy for filling an atlas looks roughly like this:
///
/// ```text
///                           (width, height)
///   ┌─────┬─────┬─────┬─────┬─────┐
///   │ 10  │     │     │     │     │ <- Empty spaces; can be filled while
///   │     │     │     │     │     │    glyph_height < height - row_baseline
///   ├─────┼─────┼─────┼─────┼─────┤
///   │ 5   │ 6   │ 7   │ 8   │ 9   │
///   │     │     │     │     │     │
///   ├─────┼─────┼─────┼─────┴─────┤ <- Row height is tallest glyph in row; this is
///   │ 1   │ 2   │ 3   │ 4         │    used as the baseline for the following row.
///   │     │     │     │           │ <- Row considered full when next glyph doesn't
///   └─────┴─────┴─────┴───────────┘    fit in the row.
/// (0, 0)  x->
/// ```
#[derive(Debug)]
pub struct Atlas {
    id: GLuint,
    width: i32,
    height: i32,
    row_extent: i32,   // Left-most free pixel in a row.
    row_baseline: i32, // Baseline for glyphs in the current row.
    row_tallest: i32,  // Tallest glyph in current row.
}

impl Atlas {
    pub fn new(size: i32) -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            // Use RGBA texture for both normal and emoji glyphs, since it has no performance
            // impact.
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                size,
                size,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        Self { id, width: size, height: size, row_extent: 0, row_baseline: 0, row_tallest: 0 }
    }

    pub fn insert_inner(&mut self, glyph: &RasterizedGlyph) -> Glyph {
        let offset_y = self.row_baseline;
        let offset_x = self.row_extent;

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);

            let buffer = Cow::Borrowed(&glyph.buffer);

            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                offset_x,
                offset_y,
                glyph.width,
                glyph.height,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                buffer.as_ptr() as *const _,
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        // Update Atlas state.
        self.row_extent = offset_x + glyph.width;
        if glyph.height > self.row_tallest {
            self.row_tallest = glyph.height;
        }

        // Generate UV coordinates.
        let uv_bot = offset_y as f32 / self.height as f32;
        let uv_left = offset_x as f32 / self.width as f32;
        let uv_height = glyph.height as f32 / self.height as f32;
        let uv_width = glyph.width as f32 / self.width as f32;

        if glyph.character == 'E' {
            println!(
                "{} {} {} {} {} {} {} {} {}",
                self.id,
                glyph.top as i16,
                glyph.left as i16,
                glyph.width as i16,
                glyph.height as i16,
                uv_bot,
                uv_left,
                uv_width,
                uv_height
            );
        }

        Glyph {
            tex_id: self.id,
            top: glyph.top as i16,
            left: glyph.left as i16,
            width: glyph.width as i16,
            height: glyph.height as i16,
            uv_bot,
            uv_left,
            uv_width,
            uv_height,
        }
    }
}
