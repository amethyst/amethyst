//! OpenGL backend types.

use gfx_device_metal;
use gfx_window_metal;

/// Command buffer type.
pub type CommandBuffer = gfx_device_metal::CommandBuffer;

/// Graphics device type.
pub type Device = gfx_device_metal::Device;

/// Graphics factory type.
pub type Factory = gfx_device_metal::Factory;

/// Graphics resource type.
pub type Resources = gfx_device_metal::Resources;

/// Shader model version.
pub type ShaderModel = gfx_device_metal::Version;

/// Window type.
pub type Window = gfx_window_metal::MetalWindow;
