//! Process window events.

use std::collections::HashMap;
use std::error::Error;

use ahash::RandomState;
use log::error;
use winit::event::{Event as WinitEvent, StartCause, WindowEvent};
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

    /// Create a new terminal window.
    pub fn create_window(
        &mut self,
        event_loop: &EventLoopWindowTarget<Event>,
    ) -> Result<(), Box<dyn Error>> {
        let window = self.windows.iter().next().as_ref().unwrap().1;

        #[allow(unused_mut)]
        let mut window_context = window.additional(event_loop)?;

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
            // Ignore all events we do not care about.
            if Self::skip_event(&event) {
                return;
            }

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
                // Process all pending events.
                WinitEvent::RedrawEventsCleared => {
                    // Dispatch event to all windows.
                    for window_context in self.windows.values_mut() {
                        window_context.handle_event(WinitEvent::RedrawEventsCleared);
                    }

                    *control_flow = ControlFlow::Wait;
                },
                // Create a new terminal window.
                WinitEvent::UserEvent(Event { payload: EventType::CreateWindow, .. }) => {
                    // XXX Ensure that no context is current when creating a new window, otherwise
                    // it may lock the backing buffer of the surface of current context when asking
                    // e.g. EGL on Wayland to create a new context.
                    for window_context in self.windows.values_mut() {
                        window_context.display.make_not_current();
                    }

                    if let Err(err) = self.create_window(event_loop) {
                        error!("Could not open window: {:?}", err);
                    }
                },
                // Process events affecting all windows.
                WinitEvent::UserEvent(event @ Event { window_id: None, .. }) => {
                    for window_context in self.windows.values_mut() {
                        window_context.handle_event(event.clone().into());
                    }
                },
                // Process window-specific events.
                WinitEvent::WindowEvent { window_id, .. }
                | WinitEvent::UserEvent(Event { window_id: Some(window_id), .. })
                | WinitEvent::RedrawRequested(window_id) => {
                    if let Some(window_context) = self.windows.get_mut(&window_id) {
                        window_context.handle_event(event);
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

    /// Check if an event is irrelevant and can be skipped.
    fn skip_event(event: &WinitEvent<'_, Event>) -> bool {
        match event {
            WinitEvent::NewEvents(StartCause::Init) => false,
            WinitEvent::WindowEvent { event, .. } => matches!(
                event,
                WindowEvent::KeyboardInput { is_synthetic: true, .. }
                    | WindowEvent::TouchpadPressure { .. }
                    | WindowEvent::CursorEntered { .. }
                    | WindowEvent::AxisMotion { .. }
                    | WindowEvent::HoveredFileCancelled
                    | WindowEvent::Destroyed
                    | WindowEvent::HoveredFile(_)
                    | WindowEvent::Moved(_)
            ),
            WinitEvent::Suspended { .. }
            | WinitEvent::NewEvents { .. }
            | WinitEvent::MainEventsCleared
            | WinitEvent::LoopDestroyed => true,
            _ => false,
        }
    }
}
