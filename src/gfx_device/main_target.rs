//! Primary render target used by the renderer.

extern crate gfx;
extern crate gfx_device_gl;

use gfx_device::gfx_types;
use renderer::target;

/// Main render target that gets drawn on the screen.
pub struct MainTarget {
    /// Primary color render target.
    pub color: gfx::handle::RenderTargetView<gfx_types::Resources, target::ColorFormat>,
    /// Primary depth-stencil render target.
    pub depth: gfx::handle::DepthStencilView<gfx_types::Resources, target::DepthFormat>,
}
