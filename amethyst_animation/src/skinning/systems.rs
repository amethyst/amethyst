use amethyst_core::Transform;
use amethyst_core::cgmath::{Matrix4, SquareMatrix};
use amethyst_renderer::JointTransforms;
use hibitset::BitSet;
use specs::{Join, ReadStorage, System, WriteStorage};

use super::resources::*;

/// System for performing vertex skinning.
///
/// Needs to run after global transforms have been updated for the current frame.
pub struct VertexSkinningSystem {
    /// Scratch space vector used while calculating joint matrices.
    /// This exists only to allow the system to re-use the same allocation
    /// for animation calculations.
    joint_matrices: Vec<Matrix4<f32>>,
    /// Also scratch space, used while determining which skins need to be updated.
    updated: BitSet,
}

impl VertexSkinningSystem {
    pub fn new() -> Self {
        Self {
            joint_matrices: Vec::new(),
            updated: BitSet::new(),
        }
    }
}

impl<'a> System<'a> for VertexSkinningSystem {
    type SystemData = (
        ReadStorage<'a, Joint>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Skin>,
        WriteStorage<'a, JointTransforms>,
    );

    fn run(&mut self, (joints, transforms, skins, mut matrices): Self::SystemData) {
        // for flagged joint transforms, calculate a new set of joint matrices for the related skin
        let updated_iter = (&joints, transforms.open().1)
            .join()
            .map(|(joint, _)| joint.skin.id());
        self.updated.clear();
        for updated in updated_iter {
            self.updated.add(updated);
        }

        for (_id, skin) in (&self.updated, &skins).join() {
            // Compute the joint transforms
            self.joint_matrices.clear();
            self.joint_matrices.extend(
                skin.joints
                    .iter()
                    .map(|joint_entity| {
                        (
                            joints.get(*joint_entity).unwrap(),
                            transforms.get(*joint_entity).unwrap(),
                        )
                    })
                    .map(|(joint, global)| {
                        (global.0 * joint.inverse_bind_matrix * skin.bind_shape_matrix)
                    }),
            );

            // update the joint matrices in all referenced mesh entities
            for (_, mesh_global, matrix) in (&skin.meshes, &transforms, &mut matrices).join() {
                if let Some(global_inverse) = mesh_global.0.invert() {
                    matrix.matrices.clear();
                    matrix
                        .matrices
                        .extend(self.joint_matrices.iter().map(|joint_matrix| {
                            Into::<[[f32; 4]; 4]>::into(global_inverse * joint_matrix)
                        }));
                }
            }
        }

        for (mesh_global, mut joint_transform) in (transforms.open().1, &mut matrices).join() {
            if let Some(global_inverse) = mesh_global.0.invert() {
                let skin = skins.get(joint_transform.skin).unwrap();
                let joint_matrices = skin.joints
                    .iter()
                    .map(|joint_entity| {
                        (
                            joints.get(*joint_entity).unwrap(),
                            transforms.get(*joint_entity).unwrap(),
                        )
                    })
                    .map(|(joint, global)| {
                        (global.0 * joint.inverse_bind_matrix * skin.bind_shape_matrix)
                    });

                joint_transform.matrices.clear();
                joint_transform
                    .matrices
                    .extend(joint_matrices.map(|joint_matrix| {
                        Into::<[[f32; 4]; 4]>::into(global_inverse * joint_matrix)
                    }));
            }
        }
    }
}
