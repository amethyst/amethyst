//! Mesh resource handling.

use ecs::{Component, VecStorage};
use renderer::prelude::{Mesh, MeshBuilder, VertexFormat};
use renderer::{Renderer, Result};
use super::unfinished::{ComponentBuilder, IntoUnfinished, Unfinished};

/// Wraps `Mesh` into component
pub struct MeshComponent(pub Mesh);

impl Component for MeshComponent {
    type Storage = VecStorage<Self>;
}

impl<D, V> ComponentBuilder for MeshBuilder<D, V>
    where D: AsRef<[V]>,
          V: VertexFormat,
{
    type Output = MeshComponent;
    fn build(self: Box<Self>, renderer: &mut Renderer) -> Result<MeshComponent> {
        renderer.create_mesh(*self).map(MeshComponent)
    }
}

impl<D, V> IntoUnfinished for MeshBuilder<D, V>
   where D: AsRef<[V]> + Send + Sync + 'static,
          V: VertexFormat + 'static,
{
    type Output = MeshComponent;
    fn unfinished(self) -> Unfinished<MeshComponent> {
        Unfinished::new(self)
    }
}