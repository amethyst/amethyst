use std::collections::HashMap;

use amethyst_animation::{JointPrefab, SkinPrefab, SkinnablePrefab};
use amethyst_assets::Prefab;
use amethyst_core::{
    alga::general::SubsetOf,
    math::{try_convert, Matrix4, RealField},
};
use amethyst_error::Error;
use amethyst_rendy::{rendy::hal::Backend, skinning::JointTransformsPrefab};

use super::Buffers;
use crate::GltfPrefab;

pub fn load_skin<B, N>(
    skin: &gltf::Skin<'_>,
    buffers: &Buffers,
    skin_entity: usize,
    node_map: &HashMap<usize, usize>,
    meshes: Vec<usize>,
    prefab: &mut Prefab<GltfPrefab<B, N>>,
) -> Result<(), Error>
where
    B: Backend,
    N: RealField + SubsetOf<f32>,
{
    let joints = skin
        .joints()
        .map(|j| {
            node_map.get(&j.index()).cloned().expect(
                "Unreachable: `node_map` is initialized with the indexes from the `Gltf` object",
            )
        })
        .collect::<Vec<_>>();

    let reader = skin.reader(|buffer| buffers.buffer(&buffer));

    let inverse_bind_matrices = reader
        .read_inverse_bind_matrices()
        .map(|matrices| {
            matrices
                .map(Matrix4::from)
                .map(|m| try_convert::<_, Matrix4<N>>(m).unwrap())
                .collect()
        })
        .unwrap_or(vec![Matrix4::identity().into(); joints.len()]);

    for (_bind_index, joint_index) in joints.iter().enumerate() {
        prefab
            .data_or_default(*joint_index)
            .skinnable
            .get_or_insert_with(SkinnablePrefab::default)
            .joint
            .get_or_insert_with(JointPrefab::default)
            .skins
            .push(skin_entity);
    }
    let joint_transforms = JointTransformsPrefab::new(skin_entity, joints.len());
    for mesh_index in &meshes {
        prefab
            .data_or_default(*mesh_index)
            .skinnable
            .get_or_insert_with(SkinnablePrefab::default)
            .joint_transforms = Some(joint_transforms.clone());
    }

    let skin_prefab = SkinPrefab {
        joints,
        meshes,
        bind_shape_matrix: Matrix4::<N>::identity(),
        inverse_bind_matrices,
    };
    prefab
        .data_or_default(skin_entity)
        .skinnable
        .get_or_insert_with(SkinnablePrefab::default)
        .skin = Some(skin_prefab);

    Ok(())
}
