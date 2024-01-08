use std::ffi::c_void;

use winit::event::Event as WinitEvent;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder};

use glutin::context::{
    ContextApi, ContextAttributesBuilder, NotCurrentGlContextSurfaceAccessor,
    PossiblyCurrentContext, PossiblyCurrentContextGlSurfaceAccessor, Version,
};
use glutin::display::{Display, DisplayApiPreference};
use glutin::prelude::*;
use glutin::surface::{AsRawSurface, RawSurface, Surface, WindowSurface};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use objc2::{msg_send, msg_send_id, ClassType};

use glutin::api::cgl::appkit::*;

mod platform;

include!(concat!(env!("OUT_DIR"), "/objcpp_bindings.rs"));

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
            .with_visible(false)
            .with_maximized(true)
            .build(&event_loop)
            .unwrap();

        let profile = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .build(raw_window_handle);
        let gl_context = unsafe { gl_display.create_context(&gl_config, &profile).unwrap() };

        let viewport_size = window.inner_size();
        let surface =
            platform::create_gl_surface(&gl_context, viewport_size, window.raw_window_handle());
        let context = gl_context.make_current(&surface).unwrap();

        window.set_visible(true);

        let mut attrs = Vec::<NSOpenGLPixelFormatAttribute>::with_capacity(32);
        attrs.push(NSOpenGLPFAMinimumPolicy);
        attrs.push(NSOpenGLPFAAllowOfflineRenderers);
        attrs.push(NSOpenGLPFAOpenGLProfile);
        attrs.push(NSOpenGLProfileVersion4_1Core);
        attrs.push(0);
        let pixel_format = unsafe { NSOpenGLPixelFormat::newWithAttributes(&attrs) };

        Processor { event_loop, window, context, surface }
    }

    pub fn run(mut self) {
        self.event_loop.run_return(move |event, _, control_flow| match event {
            WinitEvent::Resumed => {
                let raw_surface: *const c_void = match self.surface.raw_surface() {
                    RawSurface::Cgl(hi) => hi,
                };

                unsafe {
                    let raw_context = raw_surface.cast::<NSOpenGLContext>().as_ref().unwrap();
                    // raw_context.makeCurrentContext();
                }

                let _ = self.context.make_current(&self.surface);

                unsafe {
                    // cgl_context();
                    draw();
                }

                let _ = &self.surface.swap_buffers(&self.context);

                *control_flow = ControlFlow::Wait;
            },
            _ => (),
        });
    }
}
