use std::collections::HashMap;

use amethyst_animation::{Joint, Skin};
use amethyst_core::{
    ecs::{Entity, World},
    math::{convert, Matrix4},
};
use amethyst_rendy::skinning::JointTransforms;
use gltf::buffer::Data;

use crate::importer::SkinInfo;

pub fn load_skin(
    skin: &gltf::Skin<'_>,
    buffers: &Vec<Data>,
    entity: Entity,
    skin_infos: &SkinInfo,
    node_map: &HashMap<usize, Entity>,
    world: &mut World,
) {
    let joint_entities = skin
        .joints()
        .map(|j| {
            node_map.get(&j.index()).cloned().expect(
                "Unreachable: `node_map` is initialized with the indexes from the `Gltf` object",
            )
        })
        .collect::<Vec<_>>();

    let reader = skin.reader(|buffer| {
        Some(
            buffers
                .get(buffer.index())
                .expect("Error while reading skin buffer")
                .0
                .as_slice(),
        )
    });

    let inverse_bind_matrices = reader
        .read_inverse_bind_matrices()
        .map(|matrices| {
            matrices
                .map(Matrix4::from)
                .map(convert::<_, Matrix4<f32>>)
                .collect()
        })
        .unwrap_or_else(|| vec![Matrix4::identity(); joint_entities.len()]);

    let mut aggregator = HashMap::<Entity, Vec<Entity>>::new();

    for (_bind_index, joint_entity) in joint_entities.iter().enumerate() {
        aggregator
            .entry(*joint_entity)
            .or_insert_with(Vec::new)
            .push(entity);
    }

    for (entity, skins) in aggregator.iter() {
        let joint = Joint {
            skins: skins.clone(),
        };
        world
            .entry(*entity)
            .expect("This can't be reached because the entity comes from this world")
            .add_component(joint);
    }

    let joint_transforms = JointTransforms {
        skin: entity,
        matrices: vec![Matrix4::identity(); joint_entities.len()],
    };
    for mesh_entity in &skin_infos.mesh_indices {
        world
            .entry(*mesh_entity)
            .expect("This can't be reached because the entity comes from this world")
            .add_component(joint_transforms.clone());
    }
    let len = joint_entities.len();
    let skin = Skin {
        joints: joint_entities,
        meshes: skin_infos.mesh_indices.clone(),
        bind_shape_matrix: Matrix4::identity(),
        inverse_bind_matrices,
        joint_matrices: Vec::with_capacity(len),
    };

    world
        .entry(entity)
        .expect("This can't be reached because the entity comes from this world")
        .add_component(skin);
}
