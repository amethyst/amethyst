extern crate gfx;
extern crate gfx_device_gl;

use self::gfx::traits::FactoryExt;
use renderer::VertexPosNormal;
use asset_manager::{AssetLoader, Assets};
use gfx_device::GfxLoader;

#[derive(Clone)]
/// Variants of this enum hold `gfx::handle::Buffer`,`gfx::Slice` pairs.
pub enum MeshInner {
    OpenGL {
        buffer: gfx::handle::Buffer<gfx_device_gl::Resources, VertexPosNormal>,
        slice: gfx::Slice<gfx_device_gl::Resources>,
    },
    #[cfg(windows)]
    Direct3D {
        // stub
    },
    Null,
}

#[derive(Clone)]
/// This struct represents a piece of geometry. It is part of a `Renderable`
pub struct Mesh {
    pub mesh_inner: MeshInner,
}

impl AssetLoader<Mesh> for Vec<VertexPosNormal> {
    /// # Panics
    /// Panics if factory isn't registered as loader.
    fn from_data(assets: &mut Assets, data: Vec<VertexPosNormal>) -> Option<Mesh> {
        let factory_inner = assets.get_loader_mut::<GfxLoader>().expect("Unable to retrieve factory");
        let mesh_inner = match *factory_inner {
            GfxLoader::OpenGL { ref mut factory } => {
                let (buffer, slice) = factory.create_vertex_buffer_with_slice(&data, ());
                MeshInner::OpenGL {
                    buffer: buffer,
                    slice: slice,
                }
            }
            #[cfg(windows)]
            GfxLoader::Direct3D {} => unimplemented!(),
            GfxLoader::Null => MeshInner::Null,
        };
        Some(Mesh { mesh_inner: mesh_inner })
    }
}
