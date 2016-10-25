//! This module contains `VideoContext` enum which holds all the resources related to video subsystem.

extern crate amethyst_renderer;
extern crate glutin;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate gfx;

use amethyst_config::Element;
use std::path::Path;
use self::amethyst_renderer::{Renderer, Frame};

config!(
/// Contains display config,
/// it is required to create a `VideoContext`
    struct DisplayConfig {
        pub title: String = "Amethyst game".to_string(),
        pub fullscreen: bool = false,
        pub dimensions: Option<(u32, u32)> = None,
        pub min_dimensions: Option<(u32, u32)> = None,
        pub max_dimensions: Option<(u32, u32)> = None,
        pub vsync: bool = true,
        pub multisampling: u16 = 1,
        pub visibility: bool = true,
        pub backend: String = "Null".to_string(),
    }
);

/// Contains all resources related to video subsystem,
/// variants of this enum represent available backends
pub enum VideoContext {
    /// Context for a video backend that uses glutin and OpenGL
    OpenGL {
        window: glutin::Window,
        device: gfx_device_gl::Device,
        renderer: Renderer<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
        frame: Frame<gfx_device_gl::Resources>,
    },

    #[cfg(windows)]
    /// Context for a video backend that uses dxgi and Direct3D (not implemented)
    Direct3D {
        // stub
    },
    Null,
}
