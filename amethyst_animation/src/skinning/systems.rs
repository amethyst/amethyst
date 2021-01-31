use std::collections::HashSet;

use amethyst_core::{
    ecs::*,
    math::{convert, Matrix4},
    transform::Transform,
};
use amethyst_rendy::skinning::JointTransforms;
use log::error;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use super::resources::*;

/// System for performing vertex skinning.
///
/// Needs to run after global transforms have been updated for the current frame.
#[derive(Debug, Default)]
pub struct VertexSkinningSystem;

impl System for VertexSkinningSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        let mut updated = HashSet::new();
        let mut updated_skins = HashSet::new();

        Box::new(
            SystemBuilder::new("VertexSkinningSystem")
                .read_component::<Joint>()
                .read_component::<Transform>()
                .write_component::<Skin>()
                .write_component::<JointTransforms>()
                .with_query(
                    <(Entity, Read<Transform>, Read<Joint>)>::query()
                        .filter(maybe_changed::<Transform>()),
                )
                .build(move |_, world, _, global_transforms| {
                    #[cfg(feature = "profiler")]
                    profile_scope!("vertex_skinning_system");

                    updated.clear();
                    updated_skins.clear();

                    global_transforms.for_each(world, |(entity, _, joint)| {
                        updated.insert(*entity);
                        for skin in &joint.skins {
                            updated_skins.insert(*skin);
                        }
                    });

                    let mut q = <(Entity, &Transform, &mut JointTransforms)>::query();
                    let (mut left, mut right) = world.split_for_query(&q);

                    for entity in updated_skins.iter() {
                        if let Ok(mut entry) = right.entry_mut(*entity) {
                            if let Ok(skin) = entry.get_component_mut::<Skin>() {
                                // Compute the joint global_transforms
                                skin.joint_matrices.clear();
                                let bind_shape = skin.bind_shape_matrix;
                                skin.joint_matrices.extend(
                                    skin.joints
                                        .iter()
                                        .zip(skin.inverse_bind_matrices.iter())
                                        .map(|(joint_entity, inverse_bind_matrix)| {
                                            if let Ok(transform) =
                                                global_transforms.get(&left, *joint_entity)
                                            {
                                                Some((transform, inverse_bind_matrix))
                                            } else {
                                                error!("Missing `Transform` Component for join entity {:?}",joint_entity );
                                                None
                                            }
                                        })
                                        .flatten()
                                        .map(|(global, inverse_bind_matrix)| {
                                            global.1.global_matrix()
                                                * inverse_bind_matrix
                                                * bind_shape
                                        }),
                                );

                                // update the joint matrices in all referenced mesh entities
                                for (entity, mesh_global, matrix) in q.iter_mut(&mut left) {
                                    if skin.meshes.contains(entity) {
                                        if let Some(global_inverse) =
                                            mesh_global.global_matrix().try_inverse()
                                        {
                                            matrix.matrices.clear();
                                            matrix.matrices.extend(skin.joint_matrices.iter().map(
                                                |joint_matrix| {
                                                    convert::<_, Matrix4<f32>>(
                                                        global_inverse * joint_matrix,
                                                    )
                                                },
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let mut q = <(Entity, &Transform, &mut JointTransforms)>::query();
                    let (mut left, right) = world.split_for_query(&q);

                    for (entity, mesh_global, joint_transform) in q.iter_mut(&mut left) {
                        if updated.contains(&entity) {
                            if let Some(global_inverse) = mesh_global.global_matrix().try_inverse()
                            {
                                if let Ok(skin) = <&Skin>::query().get(&right, joint_transform.skin)
                                {
                                    joint_transform.matrices.clear();
                                    joint_transform.matrices.extend(
                                        skin.joint_matrices.iter().map(|joint_matrix| {
                                            convert::<_, Matrix4<f32>>(
                                                global_inverse * joint_matrix,
                                            )
                                        }),
                                    );
                                } else {
                                    error!(
                                        "Missing `Skin` Component for join transform entity {:?}",
                                        joint_transform.skin
                                    );
                                }
                            }
                        }
                    }
                }),
        )
    }
}
