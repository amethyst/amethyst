//! Vulkan backend types.

use super::ColorFormat;

/// Command buffer type.
pub type CommandBuffer = gfx_device_vulkan::CommandBuffer;

/// Graphics device type.
pub type Device = gfx_device_vulkan::GraphicsQueue;

/// Graphics factory type.
pub type Factory = gfx_device_vulkan::Factory;

/// Graphics resource type.
pub type Resources = gfx_device_vulkan::Resources;

/// Window type.
pub type Window = gfx_window_vulkan::Window<ColorFormat>;
