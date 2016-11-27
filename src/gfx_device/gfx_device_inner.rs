extern crate amethyst_renderer;
extern crate glutin;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate gfx;

use self::amethyst_renderer::{Renderer, Pipeline};
// use self::amethyst_renderer::target::{ColorFormat, DepthFormat, ColorBuffer};
// use gfx_device::DisplayConfig;

pub enum GfxDeviceInner {
    OpenGL {
        window: glutin::Window,
        device: gfx_device_gl::Device,
        renderer: Renderer<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
        pipeline: Pipeline,
    },
    #[cfg(windows)]
    Direct3D {
        // stub
    },
    Null,
}
