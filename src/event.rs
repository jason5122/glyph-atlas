//! Process window events.

use std::collections::HashMap;
use std::error::Error;

use ahash::RandomState;
use winit::event::Event as WinitEvent;
use winit::event_loop::{ControlFlow, DeviceEvents, EventLoop, EventLoopWindowTarget};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::WindowId;

use crate::window_context::WindowContext;

/// Alacritty events.
#[derive(Clone)]
pub struct Event {
    /// Limit event to a specific window.
    window_id: Option<WindowId>,

    /// Event payload.
    payload: EventType,
}

impl Event {
    pub fn new<I: Into<Option<WindowId>>>(payload: EventType, window_id: I) -> Self {
        Self { window_id: window_id.into(), payload }
    }
}

impl From<Event> for WinitEvent<'_, Event> {
    fn from(event: Event) -> Self {
        WinitEvent::UserEvent(event)
    }
}

/// Alacritty events.
#[derive(Clone)]
pub enum EventType {
    CreateWindow,
    CloseWindow,
    RedrawEditor,
    Frame,
}

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
        let window_context = WindowContext::initial(event_loop)?;

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
                // Check for shutdown.
                WinitEvent::UserEvent(Event {
                    window_id: Some(window_id),
                    payload: EventType::CloseWindow,
                }) => {
                    // Remove the closed window.
                    match self.windows.remove(&window_id) {
                        Some(window_context) => window_context,
                        None => return,
                    };

                    // Shutdown if no more terminals are open.
                    if self.windows.is_empty() {
                        *control_flow = ControlFlow::Exit;
                    }
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
