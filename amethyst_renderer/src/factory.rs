use failure::Error;
use gfx_hal::Backend;
use mesh::{Mesh, MeshBuilder};
use texture::{Texture, TextureBuilder};

pub trait Factory<B: Backend> {
    fn create_mesh(&mut self, mesh: MeshBuilder) -> Result<Mesh<B>, Error>;
    fn create_texture(&mut self, texture: TextureBuilder) -> Result<Texture<B>, Error>;
}
