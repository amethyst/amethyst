use amethyst_animation::AnimationHierarchy;
use amethyst_core::{
    ecs::{component, Entity, IntoQuery, Resources, World},
    Transform,
};
use log::debug;

use crate::{
    importer::{NodeEntityIdentifier, UniqueAnimationHierarchyId},
    types::{MaterialHandle, MeshHandle},
};

/// This will attach a Handle<Mesh> to any Entity with a MeshHandle, and remove the Meshhandle
pub(crate) fn mesh_handle_loading(world: &mut World, _resources: &mut Resources) {
    let mut entity_mesh = Vec::new();

    <(Entity, &MeshHandle)>::query().for_each(world, |(entity, mesh_handle)| {
        entity_mesh.push((*entity, mesh_handle.0.clone()));
    });

    while let Some((entity, mesh)) = entity_mesh.pop() {
        world
            .entry(entity)
            .expect("This can't exist because we just register this entity from the world")
            .remove_component::<MeshHandle>();
        world
            .entry(entity)
            .expect("This can't exist because we just register this entity from the world")
            .add_component(mesh);
    }
}

/// This will attach a Handle<Material> to any Entity with a MaterialHandle, and remove the MaterialHandle
pub(crate) fn material_handle_loading(world: &mut World, _resources: &mut Resources) {
    let mut entity_material = Vec::new();

    <(Entity, &MaterialHandle)>::query().for_each(world, |(entity, material_handle)| {
        entity_material.push((*entity, material_handle.0.clone()));
    });

    while let Some((entity, material)) = entity_material.pop() {
        world
            .entry(entity)
            .expect("This can't exist because we just register this entity from the world")
            .remove_component::<MaterialHandle>();
        world
            .entry(entity)
            .expect("This can't exist because we just register this entity from the world")
            .add_component(material);
    }
}

/// This will attach a new AnimationHierarchy on any entity with a UniqueAnimationHierarchyId component
pub(crate) fn animation_hierarchy_loading(world: &mut World, _resources: &mut Resources) {
    let mut accumulator = Vec::new();
    let mut query_hierarchy_id = <(Entity, &UniqueAnimationHierarchyId)>::query()
        .filter(!component::<AnimationHierarchy<Transform>>());
    let (hierarchy_w, else_w) = world.split_for_query(&query_hierarchy_id);
    query_hierarchy_id.for_each(&hierarchy_w, |(entity, anim_hierachy_id)| {
        let mut node_ids = Vec::new();
        <(Entity, &NodeEntityIdentifier)>::query().for_each(&else_w, |(e, node_entity_id)| {
            if node_entity_id.id == anim_hierachy_id.id {
                node_ids.push((node_entity_id.node, *e));
            }
        });
        accumulator.push((*entity, node_ids));
    });

    accumulator.iter().for_each(|(entity, nodes)| {
        world.entry(*entity).expect("Unreachable").add_component(
            AnimationHierarchy::<Transform>::new_many(
                nodes
                    .iter()
                    .map(|(node, entity)| {
                        debug!(" index: {:?} entity {:?}", node, entity);
                        (*node, *entity)
                    })
                    .collect(),
            ),
        )
    });
}
