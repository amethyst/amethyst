use num_traits::NumCast;
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, fmt::Debug};

use amethyst_animation::{JointPrefab, SkinPrefab, SkinnablePrefab};
use amethyst_assets::Prefab;
use amethyst_core::math::{Matrix4, RealField};
use amethyst_error::Error;
use amethyst_renderer::JointTransformsPrefab;

use super::Buffers;
use crate::GltfPrefab;

pub fn load_skin<
    N: Clone + Debug + Default + DeserializeOwned + Serialize + NumCast + RealField + From<f32>,
>(
    skin: &gltf::Skin<'_>,
    buffers: &Buffers,
    skin_entity: usize,
    node_map: &HashMap<usize, usize>,
    meshes: Vec<usize>,
    prefab: &mut Prefab<GltfPrefab<N>>,
) -> Result<(), Error> {
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
                .map(|m| {
                    [
                        [
                            m[0][0].into(),
                            m[0][1].into(),
                            m[0][2].into(),
                            m[0][3].into(),
                        ],
                        [
                            m[1][0].into(),
                            m[1][1].into(),
                            m[1][2].into(),
                            m[1][3].into(),
                        ],
                        [
                            m[2][0].into(),
                            m[2][1].into(),
                            m[2][2].into(),
                            m[2][3].into(),
                        ],
                        [
                            m[3][0].into(),
                            m[3][1].into(),
                            m[3][2].into(),
                            m[3][3].into(),
                        ],
                    ]
                    .into()
                })
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
