use amethyst_core::cgmath::{Matrix4, SquareMatrix};
use amethyst_core::specs::prelude::{
    BitSet, InsertedFlag, Join, ModifiedFlag, ReadStorage, ReaderId, Resources, System,
    WriteStorage,
};
use amethyst_core::GlobalTransform;
use amethyst_renderer::JointTransforms;

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
    updated_skins: BitSet,
    /// Used for tracking modifications to global transforms
    updated_id: Option<ReaderId<ModifiedFlag>>,
    inserted_id: Option<ReaderId<InsertedFlag>>,
}

impl VertexSkinningSystem {
    pub fn new() -> Self {
        Self {
            joint_matrices: Vec::new(),
            updated: BitSet::new(),
            updated_skins: BitSet::new(),
            inserted_id: None,
            updated_id: None,
        }
    }
}

impl<'a> System<'a> for VertexSkinningSystem {
    type SystemData = (
        ReadStorage<'a, Joint>,
        ReadStorage<'a, GlobalTransform>,
        ReadStorage<'a, Skin>,
        WriteStorage<'a, JointTransforms>,
    );

    fn run(&mut self, (joints, transforms, skins, mut matrices): Self::SystemData) {
        self.updated.clear();
        transforms.populate_modified(&mut self.updated_id.as_mut().unwrap(), &mut self.updated);
        transforms.populate_inserted(&mut self.inserted_id.as_mut().unwrap(), &mut self.updated);
        self.updated_skins.clear();
        for (_, joint) in (&self.updated, &joints).join() {
            self.updated_skins.add(joint.skin.id());
        }

        for (_id, skin) in (&self.updated_skins, &skins).join() {
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

        for (_, mesh_global, mut joint_transform) in
            (&self.updated, &transforms, &mut matrices).join()
        {
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

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::specs::prelude::SystemData;
        Self::SystemData::setup(res);
        let mut transform = WriteStorage::<GlobalTransform>::fetch(res);
        self.updated_id = Some(transform.track_modified());
        self.inserted_id = Some(transform.track_inserted());
    }
}
