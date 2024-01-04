//! The display subsystem including window management, font rasterization, and
//! GPU drawing.

use std::fmt::{self, Formatter};
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

use glutin::context::{NotCurrentContext, PossiblyCurrentContext};
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};

use log::{debug, info};
use winit::dpi::PhysicalSize;

use crossfont::{self, Rasterize, Rasterizer};

use crate::display::content::{RenderableCell, RenderableCursor};
use crate::display::window::Window;
use crate::point::Point;
use crate::renderer::{self, GlyphCache, Renderer};

pub mod content;
pub mod window;

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

    #[inline]
    pub fn as_tuple(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

#[derive(Debug)]
pub enum Error {
    /// Error with window management.
    Window(window::Error),

    /// Error dealing with fonts.
    Font(crossfont::Error),

    /// Error in renderer.
    Render(renderer::Error),

    /// Error during context operations.
    Context(glutin::error::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Window(err) => err.source(),
            Error::Font(err) => err.source(),
            Error::Render(err) => err.source(),
            Error::Context(err) => err.source(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Window(err) => err.fmt(f),
            Error::Font(err) => err.fmt(f),
            Error::Render(err) => err.fmt(f),
            Error::Context(err) => err.fmt(f),
        }
    }
}

impl From<window::Error> for Error {
    fn from(val: window::Error) -> Self {
        Error::Window(val)
    }
}

impl From<crossfont::Error> for Error {
    fn from(val: crossfont::Error) -> Self {
        Error::Font(val)
    }
}

impl From<renderer::Error> for Error {
    fn from(val: renderer::Error) -> Self {
        Error::Render(val)
    }
}

impl From<glutin::error::Error> for Error {
    fn from(val: glutin::error::Error) -> Self {
        Error::Context(val)
    }
}

/// Terminal size info.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SizeInfo<T = f32> {
    /// Terminal window width.
    width: T,

    /// Terminal window height.
    height: T,

    /// Width of individual cell.
    cell_width: T,

    /// Height of individual cell.
    cell_height: T,

    /// Horizontal window padding.
    padding_x: T,

    /// Vertical window padding.
    padding_y: T,

    /// Number of lines in the viewport.
    screen_lines: usize,

    /// Number of columns in the viewport.
    columns: usize,
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
            screen_lines: size_info.screen_lines,
            columns: size_info.screen_lines,
        }
    }
}

impl<T: Clone + Copy> SizeInfo<T> {
    #[inline]
    pub fn width(&self) -> T {
        self.width
    }

    #[inline]
    pub fn height(&self) -> T {
        self.height
    }

    #[inline]
    pub fn cell_width(&self) -> T {
        self.cell_width
    }

    #[inline]
    pub fn cell_height(&self) -> T {
        self.cell_height
    }

    #[inline]
    pub fn padding_x(&self) -> T {
        self.padding_x
    }

    #[inline]
    pub fn padding_y(&self) -> T {
        self.padding_y
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
        let lines = (height - 2. * padding_y) / cell_height;
        let screen_lines = lines as usize;

        let columns = (width - 2. * padding_x) / cell_width;
        let columns = columns as usize;

        SizeInfo {
            width,
            height,
            cell_width,
            cell_height,
            padding_x: padding_x.floor(),
            padding_y: padding_y.floor(),
            screen_lines,
            columns,
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct DisplayUpdate {
    pub dirty: bool,

    dimensions: Option<PhysicalSize<u32>>,
}

/// The display wraps a window, font rasterizer, and GPU renderer.
pub struct Display {
    pub window: Window,

    pub size_info: SizeInfo,

    /// Unprocessed display updates.
    pub pending_update: DisplayUpdate,

    /// The renderer update that takes place only once before the actual rendering.
    pub pending_renderer_update: Option<RendererUpdate>,

    renderer: ManuallyDrop<Renderer>,

    surface: ManuallyDrop<Surface<WindowSurface>>,

    context: ManuallyDrop<Replaceable<PossiblyCurrentContext>>,

    glyph_cache: GlyphCache,
}

impl Display {
    pub fn new(window: Window, gl_context: NotCurrentContext) -> Result<Display, Error> {
        let scale_factor = window.scale_factor as f32;
        let rasterizer = Rasterizer::new(scale_factor)?;

        let mut glyph_cache = GlyphCache::new(rasterizer)?;

        let metrics = glyph_cache.font_metrics();
        let (cell_width, cell_height) = compute_cell_size(&metrics);

        // Create the GL surface to draw into.
        let surface = renderer::platform::create_gl_surface(
            &gl_context,
            window.inner_size(),
            window.raw_window_handle(),
        )?;

        // Make the context current.
        let context = gl_context.make_current(&surface)?;

        // Create renderer.
        let mut renderer = Renderer::new(&context)?;

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

        info!("Cell size: {} x {}", cell_width, cell_height);
        info!("Padding: {} x {}", size_info.padding_x(), size_info.padding_y());
        info!("Width: {}, Height: {}", size_info.width(), size_info.height());

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

        Ok(Self {
            window,
            context: ManuallyDrop::new(Replaceable::new(context)),
            surface: ManuallyDrop::new(surface),
            renderer: ManuallyDrop::new(renderer),
            glyph_cache,
            size_info,
            pending_update: Default::default(),
            pending_renderer_update: Default::default(),
        })
    }

    pub fn make_current(&self) {
        if !self.context.get().is_current() {
            self.context.make_current(&self.surface).expect("failed to make context current")
        }
    }

    fn swap_buffers(&self) {
        #[allow(clippy::single_match)]
        let res = match (self.surface.deref(), &self.context.get()) {
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

        {
            let glyph_cache = &mut self.glyph_cache;

            let mut cells = Vec::new();

            let s = "Hello world!";
            // Red
            // let fg = Rgb::new(0xfc, 0xfd, 0xfd);
            // let bg = Rgb::new(0xec, 0x5f, 0x66);
            // Black
            let fg = Rgb::new(0x33, 0x33, 0x33);
            let bg = Rgb::new(0xfc, 0xfd, 0xfd);
            for (column, character) in s.chars().enumerate() {
                let cell = RenderableCell {
                    character,
                    line: 10,
                    column,
                    bg_alpha: 1.0,
                    fg,
                    bg,
                    underline: Rgb::new(0x33, 0x33, 0x33),
                };
                cells.push(cell);
            }

            let cursor_point = Point::new(10, 3);
            let cursor =
                RenderableCursor { point: cursor_point, color: Rgb::new(0x5f, 0xb4, 0xb4) };

            self.renderer.draw_cells(&size_info, glyph_cache, cells.into_iter());

            // Draw cursor.
            let mut rects = Vec::new();
            rects.push(cursor.rects(&size_info, 0.2));
            self.renderer.draw_rects(&size_info, rects);
        }

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
            ManuallyDrop::drop(&mut self.context);
            ManuallyDrop::drop(&mut self.surface);
        }
    }
}

/// Pending renderer updates.
///
/// All renderer updates are cached to be applied just before rendering, to avoid platform-specific
/// rendering issues.
#[derive(Debug, Default, Copy, Clone)]
pub struct RendererUpdate {}

/// Struct for safe in-place replacement.
///
/// This struct allows easily replacing struct fields that provide `self -> Self` methods in-place,
/// without having to deal with constantly unwrapping the underlying [`Option`].
struct Replaceable<T>(Option<T>);

impl<T> Replaceable<T> {
    pub fn new(inner: T) -> Self {
        Self(Some(inner))
    }

    /// Get immutable access to the wrapped value.
    pub fn get(&self) -> &T {
        self.0.as_ref().unwrap()
    }

    /// Get mutable access to the wrapped value.
    pub fn get_mut(&mut self) -> &mut T {
        self.0.as_mut().unwrap()
    }
}

impl<T> Deref for Replaceable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> DerefMut for Replaceable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
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
