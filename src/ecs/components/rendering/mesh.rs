//! Mesh resource handling.

use asset_manager::{AssetLoader, Assets};
use gfx;
use gfx::traits::FactoryExt;
use gfx_device::gfx_types;
use renderer::vertex::PosNormTex;

/// A physical piece of geometry.
#[derive(Clone)]
pub struct Mesh {
    /// A buffer full of vertices.
    pub buffer: gfx::handle::Buffer<gfx_types::Resources, VertexPosNormal>,
    /// A read-only slice of the vertex buffer data.
    pub slice: gfx::Slice<gfx_types::Resources>,
}

impl AssetLoader<Mesh> for Vec<VertexPosNormal> {
    /// # Panics
    ///
    /// Panics if factory isn't registered as loader.
    fn from_data(assets: &mut Assets, data: Vec<VertexPosNormal>) -> Option<Mesh> {
        let factory = assets
            .get_loader_mut::<gfx_types::Factory>()
            .expect("Couldn't retrieve factory.");
        let (buffer, slice) = factory.create_vertex_buffer_with_slice(&data, ());
        Some(Mesh {
                 buffer: buffer,
                 slice: slice,
             })
    }
}
