use std::collections::{HashMap, HashSet};

use amethyst_core::ecs::{
    query, world::EntityHasher, Entity, IntoQuery, Resources, TryWrite, World,
};

use crate::{
    prefab::{ComponentRegistry, Prefab},
    AssetStorage, Handle,
};

struct PrefabInstance {
    version: u32,
    entity_map: HashMap<Entity, Entity, EntityHasher>,
}

/// Attaches prefabs to entities that have Handle<Prefab>
pub fn prefab_spawning_tick(world: &mut World, resources: &mut Resources) {
    let component_registry = resources
        .get::<ComponentRegistry>()
        .expect("ComponentRegistry can not be retrieved from ECS Resources");
    let prefab_storage = resources
        .get::<AssetStorage<Prefab>>()
        .expect("AssetStorage<Prefab> can not be retrieved from ECS Resources");
    let mut prefab_handle_query = <(Entity, &Handle<Prefab>, TryWrite<PrefabInstance>)>::query();

    let mut prefabs: Vec<(
        Entity,
        &legion_prefab::CookedPrefab,
        u32,
        HashMap<Entity, Entity, EntityHasher>,
    )> = Vec::new();

    let mut entity_query = <(Entity,)>::query();

    prefab_handle_query.for_each_mut(world, |(entity, handle, instance)| {
        if let Some(prefab) = prefab_storage.get(handle) {
            if let Some(instance) = instance {
                if instance.version < prefab.version {
                    log::debug!("Updating existing prefab.");
                    if let Some(cooked_prefab) = prefab.cooked.as_ref() {
                        if instance.entity_map.is_empty() {
                            if let Some((root_entity,)) =
                                entity_query.iter(&cooked_prefab.world).next()
                            {
                                instance.entity_map.insert(*root_entity, *entity);
                            }
                        }
                        prefabs.push((
                            *entity,
                            cooked_prefab,
                            prefab.version,
                            instance.entity_map.clone(),
                        ));
                    }
                }
            } else if let Some(cooked_prefab) = prefab.cooked.as_ref() {
                log::debug!("Spawning new prefab.");
                let mut map = HashMap::<Entity, Entity, EntityHasher>::default();
                if let Some((root_entity,)) = entity_query.iter(&cooked_prefab.world).next() {
                    map.insert(*root_entity, *entity);
                };
                prefabs.push((*entity, cooked_prefab, prefab.version, map));
            }
        }
    });

    for (entity, prefab, version, prev_entity_map) in prefabs.into_iter() {
        let entity_map = world.clone_from(
            &prefab.world,
            &query::any(),
            &mut component_registry.spawn_clone_impl(&resources, &prev_entity_map),
        );

        let live_entities: HashSet<Entity, EntityHasher> = entity_map.values().copied().collect();
        let prev_entities: HashSet<_, _> = prev_entity_map.values().copied().collect();

        for value in prev_entities.difference(&live_entities).copied() {
            if world.remove(value) {
                log::debug!("Removed entity {:?}", value)
            }
        }

        log::debug!("Spawn for {:?}", entity);

        if let Some(mut entry) = world.entry(entity) {
            entry.add_component(PrefabInstance {
                version,
                entity_map,
            });
        } else {
            log::error!("Could not update entity");
        }
    }
}
