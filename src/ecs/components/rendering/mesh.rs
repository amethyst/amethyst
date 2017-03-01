//! Mesh resource handling.

use std::fmt::{Debug, Formatter, Error as FormatError};

use gfx;
use gfx::traits::FactoryExt;

use asset_manager::Asset;
use gfx_device::gfx_types;

use engine::Context;
use renderer::VertexPosNormal;

/// A physical piece of geometry.
#[derive(Clone)]
pub struct Mesh {
    /// A buffer full of vertices.
    pub buffer: gfx::handle::Buffer<gfx_types::Resources, VertexPosNormal>,
    /// A read-only slice of the vertex buffer data.
    pub slice: gfx::Slice<gfx_types::Resources>,
}

impl Asset for Mesh {
    type Data = Vec<VertexPosNormal>;
    type Error = ();

    fn category() -> &'static str {
        "meshes"
    }

    fn from_data(data: Vec<VertexPosNormal>, context: &mut Context) -> Result<Mesh, ()> {
        let (buffer, slice) = context.factory.create_vertex_buffer_with_slice(&data, ());

        Ok(Mesh {
            buffer: buffer,
            slice: slice,
        })
    }
}

impl Debug for Mesh {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        write!(f, "Mesh")
    }
}
