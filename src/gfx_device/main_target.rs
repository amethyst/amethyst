extern crate gfx;
extern crate gfx_device_gl;
use renderer;
use gfx_device::gfx_types;

/// This struct represents the screen render target.
pub struct MainTarget {
    pub main_color: gfx::handle::RenderTargetView<gfx_types::Resources, renderer::target::ColorFormat>,
    pub main_depth: gfx::handle::DepthStencilView<gfx_types::Resources, renderer::target::DepthFormat>,
}
