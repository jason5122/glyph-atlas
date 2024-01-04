use std::collections::HashMap;
use std::error::Error;

use ahash::RandomState;
use winit::event::Event as WinitEvent;
use winit::event_loop::{ControlFlow, DeviceEvents, EventLoop, EventLoopWindowTarget};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::WindowId;

use glutin::context::NotCurrentContext;
use raw_window_handle::HasRawDisplayHandle;

use crate::display::{Display, Window};
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

    /// Create initial window and load GL platform.
    ///
    /// This will initialize the OpenGL Api and pick a config that
    /// will be used for the rest of the windows.
    pub fn create_initial_window(
        &mut self,
        event_loop: &EventLoopWindowTarget<Event>,
    ) -> Result<(), Box<dyn Error>> {
        let window_context = WindowContext::initial(event_loop);

        self.windows.insert(window_context.id(), window_context);

        Ok(())
    }

    /// Run the event loop.
    ///
    /// The result is exit code generate from the loop.
    pub fn run(&mut self, mut event_loop: EventLoop<Event>) -> Result<(), Box<dyn Error>> {
        // Disable all device events, since we don't care about them.
        event_loop.listen_device_events(DeviceEvents::Never);

        let exit_code = event_loop.run_return(move |event, event_loop, control_flow| {
            match event {
                // The event loop just got initialized. Create a window.
                WinitEvent::Resumed => {
                    if let Err(err) = self.create_initial_window(event_loop) {
                        // Log the error right away since we can't return it.
                        eprintln!("Error: {}", err);
                        *control_flow = ControlFlow::ExitWithCode(1);
                        return;
                    }

                    println!("Initialization complete");

                    for window_context in self.windows.values_mut() {
                        // window_context.handle_event(WinitEvent::RedrawEventsCleared);
                        window_context.display.draw();
                    }

                    *control_flow = ControlFlow::Wait;
                },
                _ => (),
            }
        });

        if exit_code == 0 {
            Ok(())
        } else {
            Err(format!("Event loop terminated with code: {}", exit_code).into())
        }
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

        #[cfg(not(windows))]
        let window = Window::new(event_loop);

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

    /// ID of this terminal context.
    pub fn id(&self) -> WindowId {
        self.display.window.id()
    }
}
