use std::collections::hash_map::RandomState;
use std::collections::HashMap;

use winit::event::Event as WinitEvent;
use winit::event_loop::{ControlFlow, DeviceEvents, EventLoop, EventLoopWindowTarget};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder, WindowId};

use glutin::context::NotCurrentContext;
use raw_window_handle::HasRawDisplayHandle;

use crate::display::Display;
use crate::renderer;

pub struct Event {}

/// The event processor.
///
/// Stores some state from received events and dispatches actions when they are
/// triggered.
pub struct Processor {
    windows: HashMap<WindowId, WindowContext, RandomState>,
}

impl Processor {
    /// Create a new event processor.
    ///
    /// Takes a writer which is expected to be hooked up to the write end of a PTY.
    pub fn new() -> Processor {
        Processor { windows: Default::default() }
    }

    /// Run the event loop.
    ///
    /// The result is exit code generate from the loop.
    pub fn run(&mut self, mut event_loop: EventLoop<Event>) {
        // Disable all device events, since we don't care about them.
        event_loop.listen_device_events(DeviceEvents::Never);

        event_loop.run_return(move |event, event_loop, control_flow| {
            match event {
                // The event loop just got initialized. Create a window.
                WinitEvent::Resumed => {
                    let window_context = WindowContext::initial(event_loop);
                    self.windows.insert(window_context.display.window.id(), window_context);

                    for window_context in self.windows.values_mut() {
                        // window_context.handle_event(WinitEvent::RedrawEventsCleared);
                        window_context.display.draw();
                    }

                    *control_flow = ControlFlow::Wait;
                },
                _ => (),
            }
        });
    }
}

/// Event context for one individual Alacritty window.
pub struct WindowContext {
    pub display: Display,
}

impl WindowContext {
    /// Create initial window context that dous bootstrapping the graphics Api we're going to use.
    pub fn initial(event_loop: &EventLoopWindowTarget<Event>) -> Self {
        let raw_display_handle = event_loop.raw_display_handle();

        #[cfg(not(windows))]
        let raw_window_handle = None;

        let gl_display =
            renderer::platform::create_gl_display(raw_display_handle, raw_window_handle).unwrap();
        let gl_config = renderer::platform::pick_gl_config(&gl_display, raw_window_handle).unwrap();

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
        window.set_transparent(false);

        // Create context.
        let gl_context =
            renderer::platform::create_gl_context(&gl_display, &gl_config, raw_window_handle)
                .unwrap();

        Self::new(window, gl_context)
    }

    /// Create a new terminal window context.
    fn new(window: Window, context: NotCurrentContext) -> Self {
        let display = Display::new(window, context);
        WindowContext { display }
    }
}
