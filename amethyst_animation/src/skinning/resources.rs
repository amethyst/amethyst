use amethyst_core::{ecs::*, math::Matrix4};

/// Joint, attach to an entity with a `Transform`
#[derive(Debug, Clone)]
pub struct Joint {
    /// The skins attached to this joint.
    pub skins: Vec<Entity>,
}

/// Skin, attach to the root entity in the mesh hierarchy
#[derive(Debug)]
pub struct Skin {
    /// Joint entities for the skin
    pub joints: Vec<Entity>,
    /// Mesh entities that use the skin
    pub meshes: Vec<Entity>,
    /// Bind shape matrix
    pub bind_shape_matrix: Matrix4<f32>,
    /// Bring the mesh into the joints local coordinate system
    pub inverse_bind_matrices: Vec<Matrix4<f32>>,
    /// Scratch area holding the current joint matrices
    pub joint_matrices: Vec<Matrix4<f32>>,
}

// impl Skin {
//     /// Creates a new `Skin`
//     pub fn new(
//         joints: Vec<Entity>,
//         meshes: Vec<Entity>,
//         inverse_bind_matrices: Vec<Matrix4<f32>>,
//     ) -> Self {
//         let len = joints.len();
//         Skin {
//             joints,
//             meshes,
//             inverse_bind_matrices,
//             bind_shape_matrix: Matrix4::identity(),
//             joint_matrices: Vec::with_capacity(len),
//         }
//     }
// }
