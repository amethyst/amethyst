#[cfg(feature="opengl")]
mod types {
    extern crate gfx_device_gl;
    extern crate glutin;
    pub type Window = glutin::Window;
    pub type Resources = gfx_device_gl::Resources;
    pub type CommandBuffer = gfx_device_gl::CommandBuffer;
    pub type Factory = gfx_device_gl::Factory;
    pub type Device = gfx_device_gl::Device;
}
#[cfg(all(windows, feature="direct3d"))]
mod types {
    extern crate gfx_device_dx11;
    extern crate gfx_window_dxgi;
    pub type Window = gfx_window_dxgi;
    pub type Resources = gfx_device_dx11::Resources;
    pub type CommandBuffer = gfx_device_dx11::CommandBuffer;
    pub type Factory = gfx_device_dx11::Factory;
    pub type Device = gfx_device_dx11::Device;
}

pub use self::types::*;
