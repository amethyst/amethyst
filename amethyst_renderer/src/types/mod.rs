//! Compile-time graphics API types.

#[cfg(feature = "opengl")]
pub use self::opengl::{CommandBuffer, Device, Factory, Resources, Window};

#[cfg(feature = "opengl")]
mod opengl;

// /// Handle to a typed GPU buffer.
// pub type Buffer<V> = gfx::handle::Buffer<Resources, V>;

/// Color buffer format.
pub type SurfaceFormat = gfx::format::R8_G8_B8_A8;
pub type ChannelFormat = gfx::format::Unorm;
pub type ColorFormat = (SurfaceFormat, ChannelFormat);
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
