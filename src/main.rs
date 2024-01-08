use winit::event::Event as WinitEvent;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::WindowBuilder;

use raw_window_handle::HasRawDisplayHandle;

use crate::display::Display;

mod display;
mod platform;

fn main() {
    let window_event_loop = EventLoopBuilder::<Event>::with_user_event().build();
    let processor = Processor::new(window_event_loop);
    processor.run();
}

pub struct Event {}

pub struct Processor {
    display: Display,
    event_loop: EventLoop<Event>,
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

        let display = Display::new(window, gl_context);

        Processor { display, event_loop }
    }

    pub fn run(mut self) {
        self.event_loop.run_return(move |event, _, control_flow| match event {
            WinitEvent::Resumed => {
                self.display.draw();

                *control_flow = ControlFlow::Wait;
            },
            _ => (),
        });
    }
}
