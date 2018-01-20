use amethyst_core::cgmath::Matrix4;
use hibitset::BitSet;
use specs::{Component, DenseVecStorage, Entity};

/// Joint, attach to an entity with a `LocalTransform`
#[derive(Debug, Clone)]
pub struct Joint {
    /// Bring the mesh into the joints local coordinate system
    pub inverse_bind_matrix: Matrix4<f32>,
    pub skin: Entity,
}

impl Component for Joint {
    type Storage = DenseVecStorage<Self>;
}

/// Skin, attach to the root entity in the mesh hierarchy
#[derive(Debug)]
pub struct Skin {
    /// Joint entities for the skin
    pub joints: Vec<Entity>,
    /// Mesh entities that use the skin
    pub meshes: BitSet,
    /// Bind shape matrix
    pub bind_shape_matrix: Matrix4<f32>,
}

impl Component for Skin {
    type Storage = DenseVecStorage<Self>;
}
