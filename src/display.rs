use std::mem::ManuallyDrop;
use std::ops::Deref;

use glutin::context::{NotCurrentContext, PossiblyCurrentContext};
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};

use log::{debug, info};

use crossfont::{self, Rasterize, Rasterizer};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window as WinitWindow, WindowBuilder, WindowId};

use crate::renderer::text::Glsl3Renderer;
use crate::renderer::{self, GlyphCache};

#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    #[inline]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

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
        let scale_factor = window.scale_factor as f32;
        let rasterizer = Rasterizer::new(scale_factor).unwrap();

        let mut glyph_cache = GlyphCache::new(rasterizer);

        let metrics = glyph_cache.font_metrics();
        let (cell_width, cell_height) = compute_cell_size(&metrics);

        // Create the GL surface to draw into.
        let surface = renderer::platform::create_gl_surface(
            &gl_context,
            window.inner_size(),
            window.raw_window_handle(),
        );

        let context = gl_context.make_current(&surface).unwrap();

        let mut renderer = Glsl3Renderer::new(&context);

        // Load font common glyphs to accelerate rendering.
        debug!("Filling glyph cache with common glyphs");
        renderer.with_loader(|mut api| {
            glyph_cache.reset_glyph_cache(&mut api);
        });

        let padding = (5. * (window.scale_factor as f32), 5. * (window.scale_factor as f32));
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

        // Clear screen.
        let background_color = Rgb::new(0xfc, 0xfd, 0xfd);
        renderer.clear(background_color, 1.);

        window.set_visible(true);

        // Disable vsync.
        if let Err(err) = surface.set_swap_interval(&context, SwapInterval::DontWait) {
            info!("Failed to disable vsync: {}", err);
        }

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
        #[allow(clippy::single_match)]
        let res = match (self.surface.deref(), &self.context) {
            (surface, context) => surface.swap_buffers(context),
        };
        if let Err(err) = res {
            debug!("error calling swap_buffers: {}", err);
        }
    }

    pub fn draw(&mut self) {
        let size_info = self.size_info;

        // Make sure this window's OpenGL context is active.
        self.make_current();

        let background_color = Rgb::new(0xfc, 0xfd, 0xfd);
        self.renderer.clear(background_color, 1.);

        let glyph_cache = &mut self.glyph_cache;
        self.renderer.draw_cells(&size_info, glyph_cache);

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

/// Calculate the cell dimensions based on font metrics.
///
/// This will return a tuple of the cell width and height.
#[inline]
fn compute_cell_size(metrics: &crossfont::Metrics) -> (f32, f32) {
    let offset_x = f64::from(1);
    let offset_y = f64::from(2);
    (
        (metrics.average_advance + offset_x).floor().max(1.) as f32,
        (metrics.line_height + offset_y).floor().max(1.) as f32,
    )
}

/// Cell ready for rendering.
#[derive(Clone, Debug)]
pub struct RenderableCell {
    pub character: char,
    pub line: usize,
    pub column: usize,
    pub fg: Rgb,
    pub bg: Rgb,
    pub bg_alpha: f32,
    pub font_key: usize,
}

/// A window which can be used for displaying the terminal.
///
/// Wraps the underlying windowing library to provide a stable API in Alacritty.
pub struct Window {
    /// Cached scale factor for quickly scaling pixel sizes.
    pub scale_factor: f64,

    window: WinitWindow,
}

impl Window {
    /// Create a new window.
    ///
    /// This creates a window and fully initializes a window.
    pub fn new<E>(event_loop: &EventLoopWindowTarget<E>) -> Window {
        let window_builder = WindowBuilder::new();

        let window = window_builder
            .with_title("GlyphAtlas")
            .with_theme(None)
            .with_visible(false)
            .with_transparent(false)
            .with_maximized(true)
            .with_fullscreen(None)
            .build(event_loop)
            .unwrap();

        // Set initial transparency hint.
        window.set_transparent(false);

        let scale_factor = window.scale_factor();

        Self { window, scale_factor }
    }

    #[inline]
    pub fn raw_window_handle(&self) -> RawWindowHandle {
        self.window.raw_window_handle()
    }

    #[inline]
    pub fn inner_size(&self) -> PhysicalSize<u32> {
        self.window.inner_size()
    }

    #[inline]
    pub fn set_visible(&self, visibility: bool) {
        self.window.set_visible(visibility);
    }

    pub fn id(&self) -> WindowId {
        self.window.id()
    }
}
