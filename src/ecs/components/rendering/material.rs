//! Graphical texture resource.

use ecs::{Component, VecStorage};
use renderer::Material;

/// Wraps `Material` into component
#[derive(Clone, Debug)]
pub struct MaterialComponent(pub Material);

impl Component for MaterialComponent {
    type Storage = VecStorage<Self>;
}
