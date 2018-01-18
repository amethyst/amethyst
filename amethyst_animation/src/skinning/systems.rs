use std::collections::HashSet;

use amethyst_core::Transform;
use amethyst_core::cgmath::SquareMatrix;
use amethyst_renderer::JointTransforms;
use specs::{Entity, Join, ReadStorage, System, WriteStorage};

use super::resources::*;

/// System for performing vertex skinning.
///
/// Needs to run after global transforms have been updated for the current frame.
pub struct VertexSkinningSystem;

impl VertexSkinningSystem {
    pub fn new() -> Self {
        Self {}
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
        let mut updated: HashSet<Entity> = HashSet::default();

        // for flagged joint transforms, calculate a new set of joint matrices for the related skin
        for (joint, _) in (&joints, transforms.open().1).join() {
            updated.insert(joint.skin);
        }

        for skin_entity in updated {
            match skins.get(skin_entity) {
                Some(skin) => {
                    // Compute the joint transforms
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
                        })
                        .collect::<Vec<_>>();

                    // update the joint matrices in all referenced mesh entities
                    for mesh_entity in &skin.meshes {
                        let mesh_global = transforms.get(*mesh_entity).unwrap();
                        match mesh_global.0.invert() {
                            Some(global_inverse) => {
                                let mut matrix = matrices.get_mut(*mesh_entity).unwrap();
                                matrix.matrices = joint_matrices
                                    .iter()
                                    .map(|joint_matrix| (global_inverse * joint_matrix).into())
                                    .collect();
                            }

                            None => (),
                        }
                    }
                }
                None => (),
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
                    })
                    .collect::<Vec<_>>();

                joint_transform.matrices = joint_matrices
                    .iter()
                    .map(|joint_matrix| (global_inverse * joint_matrix).into())
                    .collect();
            }
        }
    }
}
