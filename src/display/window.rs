use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window as WinitWindow, WindowBuilder, WindowId};

/// A window which can be used for displaying the terminal.
///
/// Wraps the underlying windowing library to provide a stable API in Alacritty.
pub struct Window {
    /// Cached scale factor for quickly scaling pixel sizes.
    pub scale_factor: f64,

    window: WinitWindow,
}

impl Window {
    /// Create a new window.
    ///
    /// This creates a window and fully initializes a window.
    pub fn new<E>(event_loop: &EventLoopWindowTarget<E>) -> Window {
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

        // Set initial transparency hint.
        window.set_transparent(false);

        let scale_factor = window.scale_factor();

        Self { window, scale_factor }
    }

    #[inline]
    pub fn raw_window_handle(&self) -> RawWindowHandle {
        self.window.raw_window_handle()
    }

    #[inline]
    pub fn inner_size(&self) -> PhysicalSize<u32> {
        self.window.inner_size()
    }

    #[inline]
    pub fn set_visible(&self, visibility: bool) {
        self.window.set_visible(visibility);
    }

    pub fn id(&self) -> WindowId {
        self.window.id()
    }
}
