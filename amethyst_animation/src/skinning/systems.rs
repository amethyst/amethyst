use amethyst_core::{
    specs::prelude::{
        BitSet, ComponentEvent, Join, ReadStorage, ReaderId, Resources, System, WriteExpect,
        WriteStorage,
    },
    GlobalTransform,
};
use amethyst_renderer::JointTransforms;

use super::resources::*;

/// System for performing vertex skinning.
///
/// Needs to run after global transforms have been updated for the current frame.
pub struct VertexSkinningSystem;

/// A resource for `VertexSkinningSystem`.  Automatically created and managed by `VertexSkinningSystem`.
pub struct VertexSkinningSystemData {
    /// Also scratch space, used while determining which skins need to be updated.
    updated: BitSet,
    updated_skins: BitSet,
    /// Used for tracking modifications to global transforms
    updated_id: ReaderId<ComponentEvent>,
}

impl VertexSkinningSystemData {
    pub fn populate_updated_skins(&mut self, joints: &ReadStorage<'_, Joint>) {
        self.updated_skins.clear();
        for (_, joint) in (&self.updated, joints).join() {
            for skin in &joint.skins {
                self.updated_skins.add(skin.id());
            }
        }
    }
}

impl<'a> System<'a> for VertexSkinningSystem {
    type SystemData = (
        ReadStorage<'a, Joint>,
        ReadStorage<'a, GlobalTransform>,
        WriteStorage<'a, Skin>,
        WriteStorage<'a, JointTransforms>,
        WriteExpect<'a, VertexSkinningSystemData>,
    );

    fn run(
        &mut self,
        (joints, global_transforms, mut skins, mut matrices, mut data): Self::SystemData,
    ) {
        data.updated.clear();

        global_transforms
            .channel()
            .read(&mut data.updated_id)
            .for_each(|event| match event {
                ComponentEvent::Inserted(id) | ComponentEvent::Modified(id) => {
                    data.updated.add(*id);
                }
                ComponentEvent::Removed(_id) => {}
            });

        data.populate_updated_skins(&joints);

        for (_id, skin) in (&data.updated_skins, &mut skins).join() {
            // Compute the joint global_transforms
            skin.joint_matrices.clear();
            let bind_shape = skin.bind_shape_matrix;
            skin.joint_matrices.extend(
                skin.joints
                    .iter()
                    .zip(skin.inverse_bind_matrices.iter())
                    .map(|(joint_entity, inverse_bind_matrix)| {
                        if let Some(transform) = global_transforms.get(*joint_entity) {
                            Some((transform, inverse_bind_matrix))
                        } else {
                            error!(
                                "Missing `Transform` Component for join entity {:?}",
                                joint_entity
                            );
                            None
                        }
                    }).flatten()
                    .map(|(global, inverse_bind_matrix)| {
                        (global.0 * inverse_bind_matrix * bind_shape)
                    }),
            );

            // update the joint matrices in all referenced mesh entities
            for (_, mesh_global, matrix) in (&skin.meshes, &global_transforms, &mut matrices).join()
            {
                if let Some(global_inverse) = mesh_global.0.try_inverse() {
                    matrix.matrices.clear();
                    matrix
                        .matrices
                        .extend(skin.joint_matrices.iter().map(|joint_matrix| {
                            Into::<[[f32; 4]; 4]>::into(global_inverse * joint_matrix)
                        }));
                }
            }
        }

        for (_, mesh_global, mut joint_transform) in
            (&data.updated, &global_transforms, &mut matrices).join()
        {
            if let Some(global_inverse) = mesh_global.0.try_inverse() {
                if let Some(skin) = skins.get(joint_transform.skin) {
                    joint_transform.matrices.clear();
                    joint_transform
                        .matrices
                        .extend(skin.joint_matrices.iter().map(|joint_matrix| {
                            Into::<[[f32; 4]; 4]>::into(global_inverse * joint_matrix)
                        }));
                } else {
                    error!(
                        "Missing `Skin` Component for join transform entity {:?}",
                        joint_transform.skin
                    );
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::specs::prelude::SystemData;
        Self::SystemData::setup(res);
        let updated_id = WriteStorage::<GlobalTransform>::fetch(res).register_reader();
        res.insert(VertexSkinningSystemData {
            updated_id,
            updated: BitSet::new(),
            updated_skins: BitSet::new(),
        });
    }
}
