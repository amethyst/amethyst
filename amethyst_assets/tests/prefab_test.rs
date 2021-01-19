use std::time::{Duration, Instant};

use amethyst_assets::{
    prefab::{register_component_type, Prefab},
    AssetHandle, AssetStorage, DefaultLoader, Handle, LoadStatus, Loader,
};
use amethyst_core::ecs::{world::ComponentError, Dispatcher, Entity, IntoQuery, Resources, World};
use log::warn;
use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use serial_test::serial;
use type_uuid::TypeUuid;

mod common;

#[test]
#[serial]
fn a_prefab_can_be_loaded() {
    common::run_test(|dispatcher, world, resources| {
        let prefab_handle: Handle<Prefab> = {
            let loader = resources
                .get_mut::<DefaultLoader>()
                .expect("Missing loader");
            loader.load("single_entity.prefab")
        };

        execute_dispatcher_until_loaded(dispatcher, world, resources, prefab_handle.clone());

        let storage = {
            resources
                .get_mut::<AssetStorage<Prefab>>()
                .expect("Could not get prefab storage from ECS resources")
        };

        let prefab = storage.get(&prefab_handle);
        assert!(prefab.is_some());

        drop(storage);
    })
}

// Components require TypeUuid + Serialize + Deserialize + SerdeDiff + Send + Sync
#[derive(TypeUuid, Serialize, Debug, Deserialize, PartialEq, SerdeDiff, Clone, Default)]
#[uuid = "f5780013-bae4-49f0-ac0e-a108ff52fec0"]
struct Position2D {
    x: i32,
    y: i32,
}
register_component_type!(Position2D);

#[derive(TypeUuid, Serialize, Debug, Deserialize, PartialEq, SerdeDiff, Clone, Default)]
#[uuid = "9652ae70-84d6-48f9-94ab-9d3fef661350"]
struct SpotLight2D {
    color: [u8; 3],
    y: u8,
}
register_component_type!(SpotLight2D);

#[test]
#[serial]
fn a_prefab_is_applied_to_an_entity() {
    common::run_test(|dispatcher, world, resources| {
        let prefab_handle = {
            let loader = resources.get::<DefaultLoader>().expect("Missing loader");
            loader.load("test_provided_component.prefab")
        };

        execute_dispatcher_until_loaded(dispatcher, world, resources, prefab_handle.clone());

        let entity = world.push((prefab_handle.clone(),));

        execute_dispatcher_until_prefab_is_applied(dispatcher, world, resources, entity);

        <(Entity, &Position2D)>::query().for_each(world, |(entity, position)| {
            println!("Entity: {:?}, Position: {:?}", entity, position);
        });

        let entry = world
            .entry(entity)
            .expect("Could not retrieve entity from world");

        let component = entry
            .get_component::<Position2D>()
            .expect("Could not retrive compont from entry");

        let expected = Position2D { x: 100, y: 100 };

        assert_eq!(
            *component, expected,
            "Position2D component value does not match",
        );

        drop(prefab_handle);
    })
}

#[test]
#[serial]
fn a_prefab_with_dependencies_is_applied_to_an_entity() {
    common::run_test(|dispatcher, world, resources| {
        let prefab_handle: Handle<Prefab> = {
            let loader = resources
                .get_mut::<DefaultLoader>()
                .expect("Missing loader");
            loader.load("entity_with_dependencies.prefab")
        };

        execute_dispatcher_until_loaded(dispatcher, world, resources, prefab_handle.clone());

        let entity = world.push((prefab_handle,));

        execute_dispatcher_until_prefab_is_applied(dispatcher, world, resources, entity);

        let mut query = <(Entity, &Position2D)>::query();
        query.for_each(world, |(entity, position)| {
            println!("Entity: {:?}, Position: {:?}", entity, position);
        });

        let entry = world
            .entry(entity)
            .expect("Could not retrieve entity from world");

        let component = entry
            .get_component::<Position2D>()
            .expect("Could not retrive compont from entry");

        let expected = Position2D { x: 100, y: 0 };

        assert_eq!(
            *component, expected,
            "Position2D component value does not match",
        );
    });
}

fn execute_dispatcher_until_prefab_is_applied(
    dispatcher: &mut Dispatcher,
    world: &mut World,
    resources: &mut Resources,
    entity: Entity,
) {
    let timeout = Instant::now() + Duration::from_secs(5);
    loop {
        assert!(
            Instant::now() < timeout,
            "Timed out waiting for prefab to be applied"
        );

        if let Some(entry) = world.entry(entity) {
            log::warn!("{:?}", entry.archetype());
            match entry.get_component::<Position2D>() {
                Ok(_) => break,
                Err(ComponentError::NotFound { .. }) => warn!("Component not found."),
                Err(ComponentError::Denied { .. }) => panic!("Access to component was denied"),
            }
        }

        dispatcher.execute(world, resources);
    }
}

fn execute_dispatcher_until_loaded(
    dispatcher: &mut Dispatcher,
    world: &mut World,
    resources: &mut Resources,
    prefab_handle: Handle<Prefab>,
) {
    let timeout = Instant::now() + Duration::from_secs(20);
    loop {
        assert!(
            Instant::now() < timeout,
            "Timed out waiting for prefab to load"
        );
        {
            let loader = resources
                .get_mut::<DefaultLoader>()
                .expect("Missing loader");

            match loader.get_load_status_handle(prefab_handle.load_handle()) {
                LoadStatus::Unresolved => (),
                LoadStatus::Loading => (),
                LoadStatus::Loaded => break,
                LoadStatus::DoesNotExist => unreachable!("Prefab does not exist"),
                LoadStatus::Error(_) => unreachable!("Error"),
                LoadStatus::NotRequested => unreachable!("NotRequested"),
                LoadStatus::Unloading => unreachable!("Unloading"),
            }
        }
        dispatcher.execute(world, resources);
    }
}
