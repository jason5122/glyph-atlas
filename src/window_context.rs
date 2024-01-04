//! Terminal window context.

use std::error::Error;
use std::mem;
use std::sync::atomic::Ordering;

use glutin::config::GetGlConfig;
use glutin::context::NotCurrentContext;
use glutin::display::GetGlDisplay;
use raw_window_handle::HasRawDisplayHandle;
use winit::event::Event as WinitEvent;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::WindowId;

use crate::display::window::Window;
use crate::display::Display;
use crate::event::Event;
use crate::renderer;

/// Event context for one individual Alacritty window.
pub struct WindowContext {
    pub display: Display,
    event_queue: Vec<WinitEvent<'static, Event>>,
    dirty: bool,
    occluded: bool,
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

    /// Create additional context with the graphics platform other windows are using.
    pub fn additional(
        &self,
        event_loop: &EventLoopWindowTarget<Event>,
    ) -> Result<Self, Box<dyn Error>> {
        // Get any window and take its GL config and display to build a new context.
        let (gl_display, gl_config) = {
            let gl_context = self.display.gl_context();
            (gl_context.display(), gl_context.config())
        };

        let window = Window::new(event_loop)?;

        // Create context.
        let raw_window_handle = window.raw_window_handle();
        let gl_context = renderer::platform::create_gl_context(
            &gl_display,
            &gl_config,
            Some(raw_window_handle),
        )?;

        Self::new(window, gl_context)
    }

    /// Create a new terminal window context.
    fn new(window: Window, context: NotCurrentContext) -> Result<Self, Box<dyn Error>> {
        // Create a display.
        //
        // The display manages a window and can draw the terminal.
        let display = Display::new(window, context)?;

        // Create context for the Alacritty window.
        Ok(WindowContext {
            display,
            event_queue: Default::default(),
            dirty: true,
            occluded: Default::default(),
        })
    }

    /// Process events for this terminal window.
    pub fn handle_event(&mut self, event: WinitEvent<'_, Event>) {
        match event {
            // Skip further event handling with no staged updates.
            WinitEvent::RedrawEventsCleared if self.event_queue.is_empty() && !self.dirty => {
                return;
            },
            // Continue to process all pending events.
            WinitEvent::RedrawEventsCleared => (),
            // Transmute to extend lifetime, which exists only for `ScaleFactorChanged` event.
            // Since we remap that event to remove the lifetime, this is safe.
            event => unsafe {
                self.event_queue.push(mem::transmute(event));
                return;
            },
        }

        // Process DisplayUpdate events.
        if self.display.pending_update.dirty {
            self.display.handle_update();
            self.dirty = true;
        }

        // Skip rendering until we get a new frame.
        if !self.display.window.has_frame.load(Ordering::Relaxed) {
            return;
        }

        if self.dirty && !self.occluded {
            // Force the display to process any pending display update.
            self.display.process_renderer_update();

            self.dirty = false;

            // Redraw the window.
            self.display.draw();
        }
    }

    /// ID of this terminal context.
    pub fn id(&self) -> WindowId {
        self.display.window.id()
    }
}
