use std::mem::ManuallyDrop;
use std::ops::Deref;

use glutin::context::{NotCurrentContext, PossiblyCurrentContext};
use glutin::prelude::*;
use glutin::surface::{Surface, WindowSurface};

use crossfont::{self, Rasterize, Rasterizer};

use raw_window_handle::HasRawWindowHandle;

use winit::window::Window;

use crate::renderer::{self, Glsl3Renderer, GlyphCache};

/// Terminal size info.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SizeInfo<T = f32> {
    pub width: T,
    pub height: T,
    pub cell_width: T,
    pub cell_height: T,
    pub padding_x: T,
    pub padding_y: T,
}

impl From<SizeInfo<f32>> for SizeInfo<u32> {
    fn from(size_info: SizeInfo<f32>) -> Self {
        Self {
            width: size_info.width as u32,
            height: size_info.height as u32,
            cell_width: size_info.cell_width as u32,
            cell_height: size_info.cell_height as u32,
            padding_x: size_info.padding_x as u32,
            padding_y: size_info.padding_y as u32,
        }
    }
}

impl SizeInfo<f32> {
    pub fn new(
        width: f32,
        height: f32,
        cell_width: f32,
        cell_height: f32,
        padding_x: f32,
        padding_y: f32,
    ) -> SizeInfo {
        SizeInfo {
            width,
            height,
            cell_width,
            cell_height,
            padding_x: padding_x.floor(),
            padding_y: padding_y.floor(),
        }
    }
}

/// The display wraps a window, font rasterizer, and GPU renderer.
pub struct Display {
    pub window: Window,

    pub size_info: SizeInfo,

    renderer: ManuallyDrop<Glsl3Renderer>,

    surface: ManuallyDrop<Surface<WindowSurface>>,

    context: PossiblyCurrentContext,

    glyph_cache: GlyphCache,
}

impl Display {
    pub fn new(window: Window, gl_context: NotCurrentContext) -> Display {
        let rasterizer = Rasterizer::new(window.scale_factor() as f32).unwrap();

        let mut glyph_cache = GlyphCache::new(rasterizer);

        let offset_x = f64::from(1);
        let offset_y = f64::from(2);
        let metrics = glyph_cache.metrics;
        let cell_width = (metrics.average_advance + offset_x).floor().max(1.) as f32;
        let cell_height = (metrics.line_height + offset_y).floor().max(1.) as f32;

        // Create the GL surface to draw into.
        let surface = renderer::platform::create_gl_surface(
            &gl_context,
            window.inner_size(),
            window.raw_window_handle(),
        );

        let context = gl_context.make_current(&surface).unwrap();

        let mut renderer = Glsl3Renderer::new(&context);

        // Load font common glyphs to accelerate rendering.
        glyph_cache.load_common_glyphs(&mut renderer);

        let padding = (5. * (window.scale_factor() as f32), 5. * (window.scale_factor() as f32));
        let viewport_size = window.inner_size();

        // Create new size with at least one column and row.
        let size_info = SizeInfo::new(
            viewport_size.width as f32,
            viewport_size.height as f32,
            cell_width,
            cell_height,
            padding.0,
            padding.1,
        );

        // Update OpenGL projection.
        renderer.resize(&size_info);

        window.set_visible(true);

        Self {
            window,
            context,
            surface: ManuallyDrop::new(surface),
            renderer: ManuallyDrop::new(renderer),
            glyph_cache,
            size_info,
        }
    }

    pub fn make_current(&self) {
        if !self.context.is_current() {
            self.context.make_current(&self.surface).expect("failed to make context current")
        }
    }

    fn swap_buffers(&self) {
        let _ = match (self.surface.deref(), &self.context) {
            (surface, context) => surface.swap_buffers(context),
        };
    }

    pub fn draw(&mut self) {
        let size_info = self.size_info;

        // Make sure this window's OpenGL context is active.
        self.make_current();

        self.renderer.draw_cells(&size_info, &mut self.glyph_cache);
        self.renderer.render_batch();

        // Clearing debug highlights from the previous frame requires full redraw.
        self.swap_buffers();
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        // Switch OpenGL context before dropping, otherwise objects (like programs) from other
        // contexts might be deleted during droping renderer.
        self.make_current();
        unsafe {
            ManuallyDrop::drop(&mut self.renderer);
            ManuallyDrop::drop(&mut self.surface);
        }
    }
}
