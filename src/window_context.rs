use std::error::Error;

use glutin::context::NotCurrentContext;
use raw_window_handle::HasRawDisplayHandle;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::WindowId;

use crate::display::window::Window;
use crate::display::Display;
use crate::event::Event;
use crate::renderer;

/// Event context for one individual Alacritty window.
pub struct WindowContext {
    pub display: Display,
}

impl WindowContext {
    /// Create initial window context that dous bootstrapping the graphics Api we're going to use.
    pub fn initial(event_loop: &EventLoopWindowTarget<Event>) -> Result<Self, Box<dyn Error>> {
        let raw_display_handle = event_loop.raw_display_handle();

        #[cfg(not(windows))]
        let raw_window_handle = None;

        let gl_display =
            renderer::platform::create_gl_display(raw_display_handle, raw_window_handle)?;
        let gl_config = renderer::platform::pick_gl_config(&gl_display, raw_window_handle)?;

        #[cfg(not(windows))]
        let window = Window::new(event_loop)?;

        // Create context.
        let gl_context =
            renderer::platform::create_gl_context(&gl_display, &gl_config, raw_window_handle)?;

        Self::new(window, gl_context)
    }

    /// Create a new terminal window context.
    fn new(window: Window, context: NotCurrentContext) -> Result<Self, Box<dyn Error>> {
        // Create a display.
        //
        // The display manages a window and can draw the terminal.
        let display = Display::new(window, context)?;

        // Create context for the Alacritty window.
        Ok(WindowContext { display })
    }

    /// ID of this terminal context.
    pub fn id(&self) -> WindowId {
        self.display.window.id()
    }
}
