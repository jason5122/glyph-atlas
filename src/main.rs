use std::mem::ManuallyDrop;
use std::ops::Deref;

use winit::event::Event as WinitEvent;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder};

use glutin::context::{
    NotCurrentGlContextSurfaceAccessor, PossiblyCurrentContext,
    PossiblyCurrentContextGlSurfaceAccessor, PossiblyCurrentGlContext,
};
use glutin::prelude::*;
use glutin::surface::{Surface, WindowSurface};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

mod platform;

include!(concat!(env!("OUT_DIR"), "/cpp_bindings.rs"));

fn main() {
    let window_event_loop = EventLoopBuilder::<Event>::with_user_event().build();
    let processor = Processor::new(window_event_loop);
    processor.run();
}

pub struct Event {}

pub struct Processor {
    // display: Display,
    event_loop: EventLoop<Event>,
    pub window: Window,
    surface: ManuallyDrop<Surface<WindowSurface>>,
    context: PossiblyCurrentContext,
}

impl Processor {
    /// Create a new event processor.
    ///
    /// Takes a writer which is expected to be hooked up to the write end of a PTY.
    pub fn new(event_loop: EventLoop<Event>) -> Processor {
        let raw_display_handle = event_loop.raw_display_handle();

        #[cfg(not(windows))]
        let raw_window_handle = None;

        let gl_display =
            platform::create_gl_display(raw_display_handle, raw_window_handle).unwrap();
        let gl_config = platform::pick_gl_config(&gl_display, raw_window_handle).unwrap();

        let window_builder = WindowBuilder::new();
        let window = window_builder
            .with_title("GlyphAtlas")
            .with_theme(None)
            .with_visible(false)
            .with_transparent(false)
            .with_maximized(true)
            .with_fullscreen(None)
            .build(&event_loop)
            .unwrap();
        window.set_transparent(false);

        let gl_context =
            platform::create_gl_context(&gl_display, &gl_config, raw_window_handle).unwrap();

        let viewport_size = window.inner_size();
        let surface =
            platform::create_gl_surface(&gl_context, viewport_size, window.raw_window_handle());
        let context = gl_context.make_current(&surface).unwrap();

        window.set_visible(true);

        Processor { event_loop, window, context, surface: ManuallyDrop::new(surface) }
    }

    pub fn run(mut self) {
        self.event_loop.run_return(move |event, _, control_flow| match event {
            WinitEvent::Resumed => {
                if !self.context.is_current() {
                    self.context
                        .make_current(&self.surface)
                        .expect("failed to make context current")
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

                *control_flow = ControlFlow::Wait;
            },
            _ => (),
        });
    }
}
