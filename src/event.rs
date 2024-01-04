//! Process window events.

use std::collections::HashMap;
use std::error::Error;
use std::sync::atomic::Ordering;

use ahash::RandomState;
use log::error;
use winit::event::{Event as WinitEvent, Modifiers, StartCause, WindowEvent};
use winit::event_loop::{
    ControlFlow, DeviceEvents, EventLoop, EventLoopProxy, EventLoopWindowTarget,
};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::WindowId;

use crossfont::{self, Size};

use crate::display::window::Window;
use crate::display::{Display, SizeInfo};
use crate::editor::Editor;
use crate::input::{self, ActionContext as _};
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

pub struct ActionContext<'a> {
    pub editor: &'a mut Editor,
    pub modifiers: &'a mut Modifiers,
    pub display: &'a mut Display,
    pub event_proxy: &'a EventLoopProxy<Event>,
    pub font_size: &'a mut Size,
    pub dirty: &'a mut bool,
    pub occluded: &'a mut bool,
}

impl<'a> input::ActionContext for ActionContext<'a> {
    /// Request a redraw.
    #[inline]
    fn mark_dirty(&mut self) {
        *self.dirty = true;
    }

    #[inline]
    fn size_info(&self) -> SizeInfo {
        self.display.size_info
    }

    #[inline]
    fn modifiers(&mut self) -> &mut Modifiers {
        self.modifiers
    }

    #[inline]
    fn window(&mut self) -> &mut Window {
        &mut self.display.window
    }

    #[inline]
    fn display(&mut self) -> &mut Display {
        self.display
    }

    #[inline]
    fn editor(&self) -> &Editor {
        self.editor
    }

    #[inline]
    fn editor_mut(&mut self) -> &mut Editor {
        self.editor
    }

    #[cfg(not(windows))]
    fn create_new_window(&mut self) {
        let _ = self.event_proxy.send_event(Event::new(EventType::CreateWindow, None));
    }

    fn close_window(&mut self, window_id: WindowId) {
        let _ = self.event_proxy.send_event(Event::new(EventType::CloseWindow, window_id));
    }

    fn redraw_editor(&mut self, window_id: WindowId) {
        let _ = self.event_proxy.send_event(Event::new(EventType::RedrawEditor, window_id));
    }
}

impl input::Processor<ActionContext<'_>> {
    /// Handle events from winit.
    pub fn handle_event(&mut self, event: WinitEvent<'_, Event>) {
        match event {
            WinitEvent::UserEvent(Event { payload, .. }) => match payload {
                EventType::Frame => {
                    self.ctx.display.window.has_frame.store(true, Ordering::Relaxed);
                },
                EventType::CreateWindow => {},
                EventType::CloseWindow => {},
                EventType::RedrawEditor => {},
            },
            WinitEvent::RedrawRequested(_) => *self.ctx.dirty = true,
            WinitEvent::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        let window_id = self.ctx.window().id();
                        self.ctx.close_window(window_id);
                    },
                    WindowEvent::Resized(size) => {
                        // Ignore resize events to zero in any dimension, to avoid issues with Winit
                        // and the ConPTY. A 0x0 resize will also occur when the window is minimized
                        // on Windows.
                        if size.width == 0 || size.height == 0 {
                            return;
                        }

                        self.ctx.display.pending_update.set_dimensions(size);
                    },
                    WindowEvent::KeyboardInput { event, is_synthetic: false, .. } => {
                        self.key_input(event);
                    },
                    WindowEvent::ModifiersChanged(modifiers) => self.modifiers_input(modifiers),
                    WindowEvent::Occluded(occluded) => {
                        *self.ctx.occluded = occluded;
                    },
                    WindowEvent::KeyboardInput { is_synthetic: true, .. }
                    | WindowEvent::TouchpadPressure { .. }
                    | WindowEvent::TouchpadMagnify { .. }
                    | WindowEvent::TouchpadRotate { .. }
                    | WindowEvent::SmartMagnify { .. }
                    | WindowEvent::ScaleFactorChanged { .. }
                    | WindowEvent::CursorEntered { .. }
                    | WindowEvent::CursorMoved { .. }
                    | WindowEvent::CursorLeft { .. }
                    | WindowEvent::AxisMotion { .. }
                    | WindowEvent::MouseInput { .. }
                    | WindowEvent::MouseWheel { .. }
                    | WindowEvent::HoveredFileCancelled
                    | WindowEvent::Destroyed
                    | WindowEvent::ThemeChanged(_)
                    | WindowEvent::HoveredFile(_)
                    | WindowEvent::Moved(_)
                    | WindowEvent::Touch(_)
                    | WindowEvent::Ime(_)
                    | WindowEvent::DroppedFile(_)
                    | WindowEvent::Focused(_) => (),
                }
            },
            WinitEvent::Suspended { .. }
            | WinitEvent::NewEvents { .. }
            | WinitEvent::DeviceEvent { .. }
            | WinitEvent::MainEventsCleared
            | WinitEvent::RedrawEventsCleared
            | WinitEvent::Resumed
            | WinitEvent::LoopDestroyed => (),
        }
    }
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
        let proxy = event_loop.create_proxy();

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
                        window_context.handle_event(&proxy, WinitEvent::RedrawEventsCleared);
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
                        window_context.handle_event(&proxy, event.clone().into());
                    }
                },
                // Process window-specific events.
                WinitEvent::WindowEvent { window_id, .. }
                | WinitEvent::UserEvent(Event { window_id: Some(window_id), .. })
                | WinitEvent::RedrawRequested(window_id) => {
                    if let Some(window_context) = self.windows.get_mut(&window_id) {
                        window_context.handle_event(&proxy, event);
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
