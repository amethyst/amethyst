//! OpenGL backend types.

/// Command buffer type.
pub type CommandBuffer = gfx_device_gl::CommandBuffer;

/// Graphics device type.
pub type Device = gfx_device_gl::Device;

/// Graphics factory type.
pub type Factory = gfx_device_gl::Factory;

/// Graphics resource type.
pub type Resources = gfx_device_gl::Resources;

/// Window type.
pub type Window = glutin::GlWindow;
