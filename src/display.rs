use std::mem::ManuallyDrop;
use std::ops::Deref;

use glutin::context::{NotCurrentContext, PossiblyCurrentContext};
use glutin::prelude::*;
use glutin::surface::{Surface, WindowSurface};

use crossfont::{FontDesc, FontKey, Rasterizer, Size};

use raw_window_handle::HasRawWindowHandle;

use winit::window::Window;

use crate::renderer::{self, Glsl3Renderer};

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

    rasterizer: Rasterizer,

    font_key: FontKey,

    font_size: Size,
}

impl Display {
    pub fn new(window: Window, gl_context: NotCurrentContext) -> Display {
        let mut rasterizer = Rasterizer::new(window.scale_factor() as f32);

        let font_name = String::from("Source Code Pro");
        let font_size = Size::new(16.);
        let regular_desc = FontDesc::new(&font_name, &String::from("Regular"));
        let font_key = rasterizer.load_font(&regular_desc, font_size).unwrap();

        let offset_x = 1 as f64;
        let offset_y = 2 as f64;
        let metrics = rasterizer.metrics(font_key, font_size);
        println!(
            "average_advance = {}, line_height = {}",
            metrics.average_advance, metrics.line_height
        );
        let cell_width = (metrics.average_advance + offset_x).floor().max(1.) as f32;
        let cell_height = (metrics.line_height + offset_y).floor().max(1.) as f32;
        println!("cell_width = {}, cell_height = {}", cell_width, cell_height);

        // Create the GL surface to draw into.
        let viewport_size = window.inner_size();
        let surface = renderer::platform::create_gl_surface(
            &gl_context,
            viewport_size,
            window.raw_window_handle(),
        );

        let context = gl_context.make_current(&surface).unwrap();

        let renderer = Glsl3Renderer::new(&context);

        // Create new size with at least one column and row.
        let size_info = SizeInfo::new(
            viewport_size.width as f32,
            viewport_size.height as f32,
            cell_width,
            cell_height,
            5. * (window.scale_factor() as f32),
            5. * (window.scale_factor() as f32),
        );

        window.set_visible(true);

        Self {
            window,
            context,
            surface: ManuallyDrop::new(surface),
            renderer: ManuallyDrop::new(renderer),
            size_info,
            rasterizer,
            font_key,
            font_size,
        }
    }

    pub fn make_current(&self) {
        if !self.context.is_current() {
            self.context.make_current(&self.surface).expect("failed to make context current")
        }
    }

    pub fn draw(&mut self) {
        self.make_current();

        self.renderer.draw_cells(&mut self.rasterizer, self.font_key, self.font_size);

        // Clearing debug highlights from the previous frame requires full redraw.
        let _ = match (self.surface.deref(), &self.context) {
            (surface, context) => surface.swap_buffers(context),
        };
    }
}
