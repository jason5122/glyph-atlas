use std::num::NonZeroU32;

use glutin::config::{Config, ConfigTemplateBuilder, GetGlConfig};
use glutin::context::NotCurrentContext;
use glutin::display::{Display, GetGlDisplay};
use glutin::prelude::*;
use glutin::surface::{Surface, SurfaceAttributesBuilder, WindowSurface};

use raw_window_handle::RawWindowHandle;
use winit::dpi::PhysicalSize;

pub fn pick_gl_config(
    gl_display: &Display,
    raw_window_handle: Option<RawWindowHandle>,
) -> Result<Config, String> {
    let mut default_config = ConfigTemplateBuilder::new().with_transparency(true);

    if let Some(raw_window_handle) = raw_window_handle {
        default_config = default_config.compatible_with_native_window(raw_window_handle);
    }

    let configs = [default_config.clone()];

    for config in configs {
        let gl_config = unsafe {
            gl_display.find_configs(config.build()).ok().and_then(|mut configs| configs.next())
        };

        if let Some(gl_config) = gl_config {
            return Ok(gl_config);
        }
    }

    Err(String::from("failed to find suitable GL configuration."))
}

pub fn create_gl_surface(
    gl_context: &NotCurrentContext,
    size: PhysicalSize<u32>,
    raw_window_handle: RawWindowHandle,
) -> Surface<WindowSurface> {
    let gl_display = gl_context.display();
    let gl_config = gl_context.config();

    let surface_attributes =
        SurfaceAttributesBuilder::<WindowSurface>::new().with_srgb(Some(false)).build(
            raw_window_handle,
            NonZeroU32::new(size.width).unwrap(),
            NonZeroU32::new(size.height).unwrap(),
        );

    // Create the GL surface to draw into.
    unsafe { gl_display.create_window_surface(&gl_config, &surface_attributes).unwrap() }
}
