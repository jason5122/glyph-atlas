use std::mem::ManuallyDrop;
use std::ops::Deref;

use glutin::context::{NotCurrentContext, PossiblyCurrentContext};
use glutin::prelude::*;
use glutin::surface::{Surface, WindowSurface};

use raw_window_handle::HasRawWindowHandle;

use winit::window::Window;

use crate::platform;

include!(concat!(env!("OUT_DIR"), "/cpp_bindings.rs"));

/// The display wraps a window, font rasterizer, and GPU renderer.
pub struct Display {
    pub window: Window,

    surface: ManuallyDrop<Surface<WindowSurface>>,

    context: PossiblyCurrentContext,
}

impl Display {
    pub fn new(window: Window, gl_context: NotCurrentContext) -> Display {
        let viewport_size = window.inner_size();
        let surface =
            platform::create_gl_surface(&gl_context, viewport_size, window.raw_window_handle());

        let context = gl_context.make_current(&surface).unwrap();

        window.set_visible(true);

        Self { window, context, surface: ManuallyDrop::new(surface) }
    }

    pub fn draw(&mut self) {
        if !self.context.is_current() {
            self.context.make_current(&self.surface).expect("failed to make context current")
        }

        let mut vao: GLuint = 0;
        let mut ebo: GLuint = 0;
        let mut vbo_instance: GLuint = 0;
        let mut tex_id: GLuint = 0;

        unsafe {
            renderer_setup(&mut vao, &mut ebo, &mut vbo_instance, &mut tex_id);
            draw(vao, ebo, vbo_instance, tex_id);
        }

        let _ = match (self.surface.deref(), &self.context) {
            (surface, context) => surface.swap_buffers(context),
        };
    }
}
