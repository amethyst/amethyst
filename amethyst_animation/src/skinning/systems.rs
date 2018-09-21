use super::resources::*;
use amethyst_core::cgmath::SquareMatrix;
use amethyst_core::specs::prelude::{
    BitSet, InsertedFlag, Join, ModifiedFlag, ReadStorage, ReaderId, Resources, System,
    WriteStorage,
};
use amethyst_core::GlobalTransform;
use amethyst_renderer::JointTransforms;

/// System for performing vertex skinning.
///
/// Needs to run after global transforms have been updated for the current frame.
pub struct VertexSkinningSystem {
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
        WriteStorage<'a, Skin>,
        WriteStorage<'a, JointTransforms>,
    );

    fn run(&mut self, (joints, global_transforms, mut skins, mut matrices): Self::SystemData) {
        self.updated.clear();
        global_transforms.populate_modified(&mut self.updated_id.as_mut().unwrap(), &mut self.updated);
        global_transforms.populate_inserted(&mut self.inserted_id.as_mut().unwrap(), &mut self.updated);
        self.updated_skins.clear();
        for (_, joint) in (&self.updated, &joints).join() {
            for skin in &joint.skins {
                self.updated_skins.add(skin.id());
            }
        }

        for (_id, skin) in (&self.updated_skins, &mut skins).join() {
            // Compute the joint global_transforms
            skin.joint_matrices.clear();
            let bind_shape = skin.bind_shape_matrix;
            skin.joint_matrices.extend(
                skin.joints
                    .iter()
                    .zip(skin.inverse_bind_matrices.iter())
                    .map(|(joint_entity, inverse_bind_matrix)| {
                        if let Some(transform) = global_transforms.get(*joint_entity){
                            Some((transform, inverse_bind_matrix))
                        } else {
                            error!("Missing `Transform` Component for join entity {:?}", joint_entity);
                            None
                        }
                    })
                    .flatten()
                    .map(|(global, inverse_bind_matrix)| {
                        (global.0 * inverse_bind_matrix * bind_shape)
                    }),
            );

            // update the joint matrices in all referenced mesh entities
            for (_, mesh_global, matrix) in (&skin.meshes, &global_transforms, &mut matrices).join() {
                if let Some(global_inverse) = mesh_global.0.invert() {
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
            (&self.updated, &global_transforms, &mut matrices).join()
        {
            if let Some(global_inverse) = mesh_global.0.invert() {
                let skin = skins.get(joint_transform.skin).unwrap();
                joint_transform.matrices.clear();
                joint_transform
                    .matrices
                    .extend(skin.joint_matrices.iter().map(|joint_matrix| {
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
