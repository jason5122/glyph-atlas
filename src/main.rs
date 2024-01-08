use winit::event::Event as WinitEvent;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder};

use glutin::context::{
    NotCurrentGlContextSurfaceAccessor, PossiblyCurrentContext,
    PossiblyCurrentContextGlSurfaceAccessor,
};
use glutin::display::{Display, DisplayApiPreference};
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
    event_loop: EventLoop<Event>,
    pub window: Window,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
}

impl Processor {
    pub fn new(event_loop: EventLoop<Event>) -> Processor {
        let raw_display_handle = event_loop.raw_display_handle();

        #[cfg(not(windows))]
        let raw_window_handle = None;

        let gl_display =
            unsafe { Display::new(raw_display_handle, DisplayApiPreference::Cgl).unwrap() };
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

        Processor { event_loop, window, context, surface }
    }

    pub fn run(mut self) {
        self.event_loop.run_return(move |event, _, control_flow| match event {
            WinitEvent::Resumed => {
                let _ = self.context.make_current(&self.surface);

                unsafe {
                    draw();
                }

                let _ = &self.surface.swap_buffers(&self.context);

                *control_flow = ControlFlow::Wait;
            },
            _ => (),
        });
    }
}
