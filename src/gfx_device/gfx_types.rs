//! Graphics API resources that select their implementations at compile time.

#[cfg(feature="opengl")]
mod types {
    extern crate gfx_device_gl;
    extern crate glutin;

    /// An application window.
    pub type Window = glutin::Window;
    /// A handle to a GPU resource, e.g. a buffer, shader, texture, etc.
    pub type Resources = gfx_device_gl::Resources;
    /// A sequence of GPU commands.
    pub type CommandBuffer = gfx_device_gl::CommandBuffer;
    /// Creates new GPU resources.
    pub type Factory = gfx_device_gl::Factory;
    /// Handles drawing output.
    pub type Device = gfx_device_gl::Device;
}
#[cfg(all(windows, feature="direct3d"))]
mod types {
    extern crate gfx_device_dx11;
    extern crate gfx_window_dxgi;

    /// An application window.
    pub type Window = gfx_window_dxgi::Window;
    /// A handle to a GPU resource, e.g. a buffer, shader, texture, etc.
    pub type Resources = gfx_device_dx11::Resources;
    /// A sequence of GPU commands.
    pub type CommandBuffer = gfx_device_dx11::CommandBuffer<gfx_device_dx11::DeferredContext>;
    /// Creates new GPU resources.
    pub type Factory = gfx_device_dx11::Factory;
    /// Handles drawing output.
    pub type Device = gfx_device_dx11::Device;
}

pub use self::types::*;
