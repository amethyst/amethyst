use super::{Buffers, GltfError};
use animation::{JointPrefab, SkinPrefab, SkinnablePrefab};
use assets::Prefab;
use core::cgmath::{Matrix4, SquareMatrix};
use gltf;
use renderer::JointTransformsPrefab;
use std::collections::HashMap;
use GltfPrefab;

pub fn load_skin(
    skin: &gltf::Skin,
    buffers: &Buffers,
    skin_entity: usize,
    node_map: &HashMap<usize, usize>,
    meshes: Vec<usize>,
    prefab: &mut Prefab<GltfPrefab>,
) -> Result<(), GltfError> {
    let joints = skin
        .joints()
        .map(|j| node_map.get(&j.index()).cloned().unwrap())
        .collect::<Vec<_>>();

    let reader = skin.reader(|buffer| buffers.buffer(&buffer));

    let inverse_bind_matrices = reader
        .read_inverse_bind_matrices()
        .map(|matrices| matrices.map(|m| m.into()).collect())
        .unwrap_or(vec![Matrix4::identity().into(); joints.len()]);

    for (bind_index, joint_index) in joints.iter().enumerate() {
        prefab
            .data_or_default(*joint_index)
            .skinnable
            .get_or_insert_with(SkinnablePrefab::default)
            .joint
            .get_or_insert_with(JointPrefab::default)
            .skins
            .push(skin_entity);
    }
    let joint_transforms = JointTransformsPrefab {
        skin: skin_entity,
        size: joints.len(),
    };
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
        bind_shape_matrix: Matrix4::identity(),
        inverse_bind_matrices,
    };
    prefab
        .data_or_default(skin_entity)
        .skinnable
        .get_or_insert_with(SkinnablePrefab::default)
        .skin = Some(skin_prefab);

    Ok(())
}
