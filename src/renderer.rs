use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};

use glutin::context::PossiblyCurrentContext;
use glutin::display::{GetGlDisplay, GlDisplay};

use crate::display::{RenderableCell, Rgb, SizeInfo};
use crate::gl;

pub mod platform;
mod shader;
mod text;

pub use text::{GlyphCache, LoaderApi};

use text::Glsl3Renderer;

macro_rules! cstr {
    ($s:literal) => {
        // This can be optimized into an no-op with pre-allocated NUL-terminated bytes.
        unsafe { std::ffi::CStr::from_ptr(concat!($s, "\0").as_ptr().cast()) }
    };
}
pub(crate) use cstr;

pub static GL_FUNS_LOADED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Delta<T: Default> {
    pub x: T,
    pub y: T,
}

#[derive(Debug)]
pub struct Renderer {
    text_renderer: Glsl3Renderer,
}

impl Renderer {
    pub fn new(context: &PossiblyCurrentContext) -> Self {
        // We need to load OpenGL functions once per instance, but only after we make our context
        // current due to WGL limitations.
        if !GL_FUNS_LOADED.swap(true, Ordering::Relaxed) {
            let gl_display = context.display();
            gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });
        }

        let text_renderer = Glsl3Renderer::new();

        Self { text_renderer }
    }

    pub fn draw_cells<I: Iterator<Item = RenderableCell>>(
        &mut self,
        size_info: &SizeInfo,
        glyph_cache: &mut GlyphCache,
        cells: I,
    ) {
        self.text_renderer.draw_cells(size_info, glyph_cache, cells)
    }

    pub fn with_loader<F, T>(&mut self, func: F) -> T
    where
        F: FnOnce(LoaderApi<'_>) -> T,
    {
        self.text_renderer.with_loader(func)
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

    /// Resize the renderer.
    pub fn resize(&self, size_info: &SizeInfo) {
        self.set_viewport(size_info);
        self.text_renderer.resize(size_info)
    }
}
