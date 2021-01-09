use std::collections::HashMap;

use amethyst_core::ecs::{query, Entity, IntoQuery, Resources, World};

use crate::{
    prefab::{ComponentRegistry, Prefab},
    AssetStorage, Handle,
};

/// Attaches prefabs to entities that have Handle<Prefab>
/// FIXME: Add a check so that the prefab is only applied once.
pub fn prefab_spawning_tick(world: &mut World, resources: &mut Resources) {
    let component_registry = resources
        .get::<ComponentRegistry>()
        .expect("ComponentRegistry can not be retrieved from ECS Resources");
    let prefab_storage = resources
        .get::<AssetStorage<Prefab>>()
        .expect("AssetStorage<Prefab> can not be retrieved from ECS Resources");

    let mut clone_impl_result = HashMap::default();
    let mut prefab_handle_query = <(Entity, &Handle<Prefab>)>::query();

    let mut prefabs: Vec<(Entity, &legion_prefab::CookedPrefab)> = Vec::new();

    prefab_handle_query.for_each(world, |(entity, handle)| {
        if let Some(Prefab { prefab }) = prefab_storage.get(handle) {
            prefabs.push((*entity, prefab));
        }
    });

    let mut entity_query = <(Entity,)>::query();
    for (entity, prefab) in prefabs.iter() {
        // Spawn the prefab in a new world.
        clone_impl_result.clear();
        if let Some((root_entity,)) = entity_query.iter(&prefab.world).next() {
            clone_impl_result.insert(*root_entity, *entity);
        };
        let mut spawn_impl = component_registry.spawn_clone_impl(&resources, &clone_impl_result);
        world.clone_from(&prefab.world, &query::any(), &mut spawn_impl);
        log::debug!("Spawn {:?}", entity);
    }
}
