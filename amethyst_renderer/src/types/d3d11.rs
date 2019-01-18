//! Direct3D 11 backend types.

/// Command buffer type.
pub type CommandBuffer = gfx_device_dx11::CommandBuffer<DeferredContext>;

/// Graphics device type.
pub type Device = gfx_device_dx11::Device;

/// Graphics factory type.
pub type Factory = gfx_device_dx11::Factory;

/// Graphics resource type.
pub type Resources = gfx_device_dx11::Resources;

/// Window type.
pub type Window = gfx_window_dxgi::Window;
