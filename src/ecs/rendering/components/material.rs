//! Graphical texture resource.

use renderer::Material;

use ecs::{Component, VecStorage};

/// Wraps `Material` into component
#[derive(Clone, Debug)]
pub struct MaterialComponent(pub Material);

impl Component for MaterialComponent {
    type Storage = VecStorage<Self>;
}

impl AsRef<Material> for MaterialComponent {
    fn as_ref(&self) -> &Material {
        &self.0
    }
}

impl AsMut<Material> for MaterialComponent {
    fn as_mut(&mut self) -> &mut Material {
        &mut self.0
    }
}
