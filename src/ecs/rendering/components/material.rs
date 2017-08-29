//! Graphical texture resource.

use renderer::Material;

use ecs::{Component, VecStorage};

/// Wraps `Material` into component
#[derive(Clone, Debug)]
pub struct MaterialComponent(pub Material);

impl Component for MaterialComponent {
    type Storage = VecStorage<Self>;
}
