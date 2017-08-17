//! Mesh resource handling.

use ecs::{Component, VecStorage};
use renderer::Mesh;

/// Wraps `Mesh` into component
pub struct MeshComponent(pub Mesh);

impl Component for MeshComponent {
    type Storage = VecStorage<Self>;
}
