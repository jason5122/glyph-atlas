//! The OpenGL platform display selection and creation.
#![allow(unreachable_patterns)]

use std::collections::HashSet;
use std::ffi::{self, CStr};
use std::fmt;

use bitflags::bitflags;
use raw_window_handle::RawDisplayHandle;

use crate::config::{Config, ConfigTemplate, GlConfig};
use crate::context::{ContextAttributes, NotCurrentContext, NotCurrentGlContext};
use crate::error::Result;
use crate::private::{gl_api_dispatch, Sealed};
use crate::surface::{
    GlSurface, PbufferSurface, PixmapSurface, Surface, SurfaceAttributes, WindowSurface,
};

#[cfg(cgl_backend)]
use crate::api::cgl::display::Display as CglDisplay;

/// A trait to group common display operations.
pub trait GlDisplay: Sealed {
    /// A window surface created by the display.
    type WindowSurface: GlSurface<WindowSurface>;
    /// A pixmap surface created by the display.
    type PixmapSurface: GlSurface<PixmapSurface>;
    /// A pbuffer surface created by the display.
    type PbufferSurface: GlSurface<PbufferSurface>;
    /// A config that is used by the display.
    type Config: GlConfig;
    /// A context that is being used by the display.
    type NotCurrentContext: NotCurrentGlContext;

    /// Find configurations matching the given `template`.
    ///
    /// # Safety
    ///
    /// Some platforms use [`RawWindowHandle`] to pick configs, so it
    /// must point to a valid object if it was passed on
    /// [`crate::config::ConfigTemplate`].
    ///
    /// [`RawWindowHandle`]: raw_window_handle::RawWindowHandle
    unsafe fn find_configs(
        &self,
        template: ConfigTemplate,
    ) -> Result<Box<dyn Iterator<Item = Self::Config> + '_>>;

    /// Create the graphics platform context.
    ///
    /// # Safety
    ///
    /// Some platforms use [`RawWindowHandle`] for context creation, so it must
    /// point to a valid object.
    ///
    /// [`RawWindowHandle`]: raw_window_handle::RawWindowHandle
    unsafe fn create_context(
        &self,
        config: &Self::Config,
        context_attributes: &ContextAttributes,
    ) -> Result<Self::NotCurrentContext>;

    /// Create the surface that can be used to render into native window.
    ///
    /// # Safety
    ///
    /// The [`RawWindowHandle`] must point to a valid object.
    ///
    /// [`RawWindowHandle`]: raw_window_handle::RawWindowHandle
    unsafe fn create_window_surface(
        &self,
        config: &Self::Config,
        surface_attributes: &SurfaceAttributes<WindowSurface>,
    ) -> Result<Self::WindowSurface>;

    /// Create the surface that can be used to render into pbuffer.
    ///
    /// # Safety
    ///
    /// The function is safe in general, but marked as not for compatibility
    /// reasons.
    unsafe fn create_pbuffer_surface(
        &self,
        config: &Self::Config,
        surface_attributes: &SurfaceAttributes<PbufferSurface>,
    ) -> Result<Self::PbufferSurface>;

    /// Create the surface that can be used to render into pixmap.
    ///
    /// # Safety
    ///
    /// The [`NativePixmap`] must represent a valid native pixmap.
    ///
    /// [`NativePixmap`]: crate::surface::NativePixmap
    unsafe fn create_pixmap_surface(
        &self,
        config: &Self::Config,
        surface_attributes: &SurfaceAttributes<PixmapSurface>,
    ) -> Result<Self::PixmapSurface>;

    /// Return the address of an OpenGL function.
    ///
    /// # Api-specific
    ///
    /// **WGL:** - To load all the functions you must have a current context on
    /// the calling thread, otherwise only limited set of functions will be
    /// loaded.
    fn get_proc_address(&self, addr: &CStr) -> *const ffi::c_void;

    /// Helper to obtain the information about the underlying display.
    ///
    /// This function is intended to be used for logging purposes to help with
    /// troubleshooting issues.
    fn version_string(&self) -> String;

    /// Get the features supported by the display.
    ///
    /// These features could be used to check that something is supported
    /// beforehand instead of doing fallback.
    fn supported_features(&self) -> DisplayFeatures;
}

/// Get the [`Display`].
pub trait GetGlDisplay: Sealed {
    /// The display used by the object.
    type Target: GlDisplay;

    /// Obtain the GL display used to create a particular GL object.
    fn display(&self) -> Self::Target;
}

/// Obtain the underlying api extensions.
pub trait GetDisplayExtensions: Sealed {
    /// Supported extensions by the display.
    ///
    /// # Api-specific
    ///
    /// **WGL:** - To have extensions loaded, `raw_window_handle` must be used
    /// when creating display.
    fn extensions(&self) -> &HashSet<&'static str>;
}

/// Get the raw handle to the [`Display`].
pub trait AsRawDisplay {
    /// A raw handle to the underlying Api display.
    fn raw_display(&self) -> RawDisplay;
}

#[derive(Debug, Clone)]
pub enum Display {
    /// The CGL display.
    #[cfg(cgl_backend)]
    Cgl(CglDisplay),
}

impl Display {
    pub unsafe fn new(display: RawDisplayHandle, preference: DisplayApiPreference) -> Result<Self> {
        match preference {
            #[cfg(cgl_backend)]
            DisplayApiPreference::Cgl => unsafe { Ok(Self::Cgl(CglDisplay::new(display)?)) },
        }
    }
}

impl GlDisplay for Display {
    type Config = Config;
    type NotCurrentContext = NotCurrentContext;
    type PbufferSurface = Surface<PbufferSurface>;
    type PixmapSurface = Surface<PixmapSurface>;
    type WindowSurface = Surface<WindowSurface>;

    unsafe fn find_configs(
        &self,
        template: ConfigTemplate,
    ) -> Result<Box<dyn Iterator<Item = Self::Config> + '_>> {
        match self {
            #[cfg(cgl_backend)]
            Self::Cgl(display) => unsafe {
                Ok(Box::new(display.find_configs(template)?.into_iter().map(Config::Cgl)))
            },
        }
    }

    unsafe fn create_context(
        &self,
        config: &Self::Config,
        context_attributes: &ContextAttributes,
    ) -> Result<Self::NotCurrentContext> {
        match (self, config) {
            #[cfg(cgl_backend)]
            (Self::Cgl(display), Config::Cgl(config)) => unsafe {
                Ok(NotCurrentContext::Cgl(display.create_context(config, context_attributes)?))
            },
            _ => unreachable!(),
        }
    }

    unsafe fn create_window_surface(
        &self,
        config: &Self::Config,
        surface_attributes: &SurfaceAttributes<WindowSurface>,
    ) -> Result<Self::WindowSurface> {
        match (self, config) {
            #[cfg(cgl_backend)]
            (Self::Cgl(display), Config::Cgl(config)) => unsafe {
                Ok(Surface::Cgl(display.create_window_surface(config, surface_attributes)?))
            },
            _ => unreachable!(),
        }
    }

    unsafe fn create_pbuffer_surface(
        &self,
        config: &Self::Config,
        surface_attributes: &SurfaceAttributes<PbufferSurface>,
    ) -> Result<Self::PbufferSurface> {
        match (self, config) {
            #[cfg(cgl_backend)]
            (Self::Cgl(display), Config::Cgl(config)) => unsafe {
                Ok(Surface::Cgl(display.create_pbuffer_surface(config, surface_attributes)?))
            },
            _ => unreachable!(),
        }
    }

    unsafe fn create_pixmap_surface(
        &self,
        config: &Self::Config,
        surface_attributes: &SurfaceAttributes<PixmapSurface>,
    ) -> Result<Self::PixmapSurface> {
        match (self, config) {
            #[cfg(cgl_backend)]
            (Self::Cgl(display), Config::Cgl(config)) => unsafe {
                Ok(Surface::Cgl(display.create_pixmap_surface(config, surface_attributes)?))
            },
            _ => unreachable!(),
        }
    }

    fn get_proc_address(&self, addr: &CStr) -> *const ffi::c_void {
        gl_api_dispatch!(self; Self(display) => display.get_proc_address(addr))
    }

    fn version_string(&self) -> String {
        gl_api_dispatch!(self; Self(display) => display.version_string())
    }

    fn supported_features(&self) -> DisplayFeatures {
        gl_api_dispatch!(self; Self(display) => display.supported_features())
    }
}

impl AsRawDisplay for Display {
    fn raw_display(&self) -> RawDisplay {
        gl_api_dispatch!(self; Self(display) => display.raw_display())
    }
}

impl Sealed for Display {}

/// Preference of the display that should be used.
pub enum DisplayApiPreference {
    /// Use only CGL.
    ///
    /// The only option on macOS for now.
    #[cfg(cgl_backend)]
    Cgl,
}

impl fmt::Debug for DisplayApiPreference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let api = match self {
            #[cfg(cgl_backend)]
            DisplayApiPreference::Cgl => "Cgl",
        };

        f.write_fmt(format_args!("DisplayApiPreference::{api}"))
    }
}

bitflags! {
    /// The features and extensions supported by the [`Display`].
    pub struct DisplayFeatures: u32 {
        /// The display supports creating [`robust`] context.
        ///
        /// [`robust`]: crate::context::Robustness
        const CONTEXT_ROBUSTNESS          = 0b0000_0001;

        /// The display supports creating [`no error`] context.
        ///
        /// [`no error`]: crate::context::Robustness::NoError
        const CONTEXT_NO_ERROR            = 0b0000_0010;

        /// The display supports [`floating`] pixel formats.
        ///
        /// [`floating`]: crate::config::ConfigTemplateBuilder::with_float_pixels
        const FLOAT_PIXEL_FORMAT          = 0b0000_0100;

        /// The display supports changing the [`swap interval`] on surfaces.
        ///
        /// [`swap interval`]: crate::surface::GlSurface::set_swap_interval
        const SWAP_CONTROL                = 0b0000_1000;

        /// The display supports creating context with explicit [`release behavior`].
        ///
        /// [`release behavior`]: crate::context::ReleaseBehavior
        const CONTEXT_RELEASE_BEHAVIOR   = 0b0001_0000;

        /// The display supports creating OpenGL ES [`context`].
        ///
        /// [`context`]: crate::context::ContextApi::Gles
        const CREATE_ES_CONTEXT           = 0b0010_0000;

        /// The display supports pixel formats with [`multisampling`].
        ///
        /// [`multisampling`]: crate::config::ConfigTemplateBuilder::with_multisampling
        const MULTISAMPLING_PIXEL_FORMATS = 0b0100_0000;

        /// The display supports creating surfaces backed by [`SRGB`] framebuffers.
        ///
        /// [`SRGB`]: crate::surface::SurfaceAttributesBuilder::with_srgb
        const SRGB_FRAMEBUFFERS           = 0b1000_0000;
    }
}

/// Raw GL platform display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawDisplay {
    /// Raw display is CGL.
    #[cfg(cgl_backend)]
    Cgl,
}
