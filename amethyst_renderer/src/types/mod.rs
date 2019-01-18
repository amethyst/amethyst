//! Compile-time graphics API types.

#[cfg(all(feature = "d3d11", target_os = "windows"))]
pub use self::d3d11::{CommandBuffer, Device, Factory, Resources, Window};
#[cfg(all(feature = "metal", target_os = "macos"))]
pub use self::metal::{CommandBuffer, Device, Factory, Resources, Window};
#[cfg(feature = "opengl")]
pub use self::opengl::{CommandBuffer, Device, Factory, Resources, Window};
#[cfg(feature = "vulkan")]
pub use self::vulkan::{CommandBuffer, Device, Factory, Resources, Window};

#[cfg(all(feature = "d3d11", target_os = "windows"))]
mod d3d11;
#[cfg(all(feature = "metal", target_os = "macos"))]
mod metal;
#[cfg(feature = "opengl")]
mod opengl;
#[cfg(feature = "vulkan")]
mod vulkan;

// /// Handle to a typed GPU buffer.
// pub type Buffer<V> = gfx::handle::Buffer<Resources, V>;

/// Color buffer format.
pub type SurfaceFormat = gfx::format::R8_G8_B8_A8;
pub type ChannelFormat = gfx::format::Unorm;
pub type ColorFormat = (SurfaceFormat, ChannelFormat);

/// Depth buffer format.
#[cfg(feature = "metal")]
pub type DepthFormat = gfx::format::Depth32F;
/// Depth buffer format.
#[cfg(not(feature = "metal"))]
pub type DepthFormat = gfx::format::DepthStencil;

/// Depth-stencil view type.
pub type DepthStencilView = gfx::handle::DepthStencilView<Resources, DepthFormat>;

/// Command buffer encoder type.
///
/// Created by calling `CommandBuffer::into()`.
pub type Encoder = gfx::Encoder<Resources, CommandBuffer>;

/// Statically-typed pipeline state object (PSO).
pub type PipelineState<M> = gfx::PipelineState<Resources, M>;

/// Handle to a chunk of GPU memory.
///
/// This handle can represent a vertex buffer, index buffer, constant buffer,
/// or staging buffer.
pub type RawBuffer = gfx::handle::RawBuffer<Resources>;

/// Dynamically typed shader resource view.
pub type RawShaderResourceView = gfx::handle::RawShaderResourceView<Resources>;

/// Dynamically typed texture resource.
pub type RawTexture = gfx::handle::RawTexture<Resources>;

/// Render target view type.
pub type RenderTargetView = gfx::handle::RenderTargetView<Resources, ColorFormat>;

/// Texture sampler type.
pub type Sampler = gfx::handle::Sampler<Resources>;

/// Shader resource view type.
pub type ShaderResourceView<T> = gfx::handle::ShaderResourceView<Resources, T>;

/// Slice associated with a vertex buffer.
pub type Slice = gfx::Slice<Resources>;
