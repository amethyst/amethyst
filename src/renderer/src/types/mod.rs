//! Compile-time graphics API types.

#[cfg(all(feature = "d3d11", target_os = "windows"))]
pub use self::d3d11::{CommandBuffer, Device, Factory, Resources, Window};
#[cfg(feature = "opengl")]
pub use self::opengl::{CommandBuffer, Device, Factory, Resources, Window};
#[cfg(feature = "vulkan")]
pub use self::vulkan::{CommandBuffer, Device, Factory, Resources, Window};

use gfx;
use gfx::format::Format;
use gfx::pso::buffer::Structure;
use gfx::traits::Pod;
use std::fmt::Debug;

#[cfg(all(feature = "d3d11", target_os = "windows"))]
mod d3d11;
#[cfg(feature = "opengl")]
mod opengl;
#[cfg(feature = "vulkan")]
mod vulkan;

/// Handle to a GPU buffer.
pub type Buffer<V: VertexFormat> = gfx::handle::Buffer<Resources, V>;

/// Color buffer format.
pub type ColorFormat = gfx::format::Srgba8;

/// Depth buffer format.
#[cfg(feature = "metal")]
pub type DepthFormat = gfx::format::Depth32F;
/// Depth buffer format.
#[cfg(not(feature = "metal"))]
pub type DepthFormat = gfx::format::DepthStencil;

/// Command buffer encoder type.
///
/// Created by calling `CommandBuffer::into()`.
pub type Encoder = gfx::Encoder<Resources, CommandBuffer>;

/// Slice of a vertex buffer.
pub type Slice = gfx::Slice<Resources>;

/// Trait implemented by all valid vertex formats.
pub trait VertexFormat: Debug + Pod + Structure<Format> {}

impl<T> VertexFormat for T where T: Debug + Pod + Structure<Format> {}
