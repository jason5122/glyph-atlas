use bitflags::bitflags;
use crossfont::{GlyphKey, RasterizedGlyph};

use crate::display::{RenderableCell, SizeInfo};
use crate::gl;
use crate::gl::types::*;

mod atlas;
mod glsl3;
pub mod glyph_cache;

use atlas::Atlas;
pub use glsl3::Glsl3Renderer;
pub use glyph_cache::GlyphCache;
use glyph_cache::{Glyph, LoadGlyph};

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
enum RenderingPass {
    /// Rendering pass used to render background color in text shaders.
    Background = 0,

    /// The first pass to render text.
    SubpixelPass1 = 1,
}

pub trait TextRenderer<'a> {
    type Shader: TextShader;
    type RenderBatch: TextRenderBatch;
    type RenderApi: TextRenderApi<Self::RenderBatch>;

    /// Get loader API for the renderer.
    fn loader_api(&mut self) -> LoaderApi<'_>;

    /// Draw cells.
    fn draw_cells<'b: 'a, I: Iterator<Item = RenderableCell>>(
        &'b mut self,
        size_info: &'b SizeInfo,
        glyph_cache: &'a mut GlyphCache,
        cells: I,
    ) {
        self.with_api(size_info, |mut api| {
            for cell in cells {
                api.draw_cell(cell, glyph_cache, size_info);
            }
        })
    }

    fn with_api<'b: 'a, F, T>(&'b mut self, size_info: &'b SizeInfo, func: F) -> T
    where
        F: FnOnce(Self::RenderApi) -> T;

    fn program(&self) -> &Self::Shader;

    /// Resize the text rendering.
    fn resize(&self, size: &SizeInfo) {
        unsafe {
            let program = self.program();
            gl::UseProgram(program.id());
            update_projection(program.projection_uniform(), size);
            gl::UseProgram(0);
        }
    }

    /// Invoke renderer with the loader.
    fn with_loader<F: FnOnce(LoaderApi<'_>) -> T, T>(&mut self, func: F) -> T {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
        }

        func(self.loader_api())
    }
}

pub trait TextRenderBatch {
    fn is_empty(&self) -> bool;
    fn full(&self) -> bool;
    fn tex(&self) -> GLuint;
    fn add_item(&mut self, cell: &RenderableCell, glyph: &Glyph, size_info: &SizeInfo);
}

pub trait TextRenderApi<T: TextRenderBatch>: LoadGlyph {
    fn batch(&mut self) -> &mut T;

    fn render_batch(&mut self);

    #[inline]
    fn add_render_item(&mut self, cell: &RenderableCell, glyph: &Glyph, size_info: &SizeInfo) {
        // Flush batch if tex changing.
        if !self.batch().is_empty() && self.batch().tex() != glyph.tex_id {
            self.render_batch();
        }

        self.batch().add_item(cell, glyph, size_info);

        if self.batch().full() {
            self.render_batch();
        }
    }

    fn draw_cell(
        &mut self,
        cell: RenderableCell,
        glyph_cache: &mut GlyphCache,
        size_info: &SizeInfo,
    ) {
        let font_key = match cell.font_key {
            0 => glyph_cache.font_key,
            1 => glyph_cache.bold_key,
            2 => glyph_cache.italic_key,
            3 => glyph_cache.bold_italic_key,
            _ => glyph_cache.font_key,
        };

        let glyph_key =
            GlyphKey { font_key, size: glyph_cache.font_size, character: cell.character };

        let glyph = glyph_cache.get(glyph_key, self, true);
        self.add_render_item(&cell, &glyph, size_info);
    }
}

pub trait TextShader {
    fn id(&self) -> GLuint;
    fn projection_uniform(&self) -> GLint;
}

#[derive(Debug)]
pub struct LoaderApi<'a> {
    active_tex: &'a mut GLuint,
    atlas: &'a mut Vec<Atlas>,
    current_atlas: &'a mut usize,
}

impl<'a> LoadGlyph for LoaderApi<'a> {
    fn load_glyph(&mut self, rasterized: &RasterizedGlyph) -> Glyph {
        Atlas::load_glyph(self.active_tex, self.atlas, self.current_atlas, rasterized)
    }

    fn clear(&mut self) {
        Atlas::clear_atlas(self.atlas, self.current_atlas)
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
