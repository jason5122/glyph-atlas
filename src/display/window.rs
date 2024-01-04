use std::fmt::{self, Display, Formatter};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[cfg(target_os = "macos")]
use winit::platform::macos::{OptionAsAlt, WindowBuilderExtMacOS};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window as WinitWindow, WindowBuilder, WindowId};

/// Window errors.
#[derive(Debug)]
pub enum Error {
    /// Error creating the window.
    WindowCreation(winit::error::OsError),

    /// Error dealing with fonts.
    Font(crossfont::Error),
}

/// Result of fallible operations concerning a Window.
type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::WindowCreation(err) => err.source(),
            Error::Font(err) => err.source(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::WindowCreation(err) => write!(f, "Error creating GL context; {}", err),
            Error::Font(err) => err.fmt(f),
        }
    }
}

impl From<winit::error::OsError> for Error {
    fn from(val: winit::error::OsError) -> Self {
        Error::WindowCreation(val)
    }
}

impl From<crossfont::Error> for Error {
    fn from(val: crossfont::Error) -> Self {
        Error::Font(val)
    }
}

/// A window which can be used for displaying the terminal.
///
/// Wraps the underlying windowing library to provide a stable API in Alacritty.
pub struct Window {
    /// Flag tracking that we have a frame we can draw.
    pub has_frame: Arc<AtomicBool>,

    /// Cached scale factor for quickly scaling pixel sizes.
    pub scale_factor: f64,

    window: WinitWindow,
}

impl Window {
    /// Create a new window.
    ///
    /// This creates a window and fully initializes a window.
    pub fn new<E>(event_loop: &EventLoopWindowTarget<E>) -> Result<Window> {
        let window_builder = Window::get_platform_window();

        let window = window_builder
            .with_title("GlyphAtlas")
            .with_theme(None)
            .with_visible(false)
            .with_transparent(false)
            .with_maximized(true)
            .with_fullscreen(None)
            .build(event_loop)?;

        // Set initial transparency hint.
        window.set_transparent(false);

        let scale_factor = window.scale_factor();

        Ok(Self { window, has_frame: Arc::new(AtomicBool::new(true)), scale_factor })
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

    #[cfg(target_os = "macos")]
    pub fn get_platform_window() -> WindowBuilder {
        let window = WindowBuilder::new().with_option_as_alt(OptionAsAlt::Both);
        window
    }

    pub fn id(&self) -> WindowId {
        self.window.id()
    }
}
