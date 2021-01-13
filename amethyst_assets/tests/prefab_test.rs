use std::time::{Duration, Instant};

use amethyst_assets::{
    prefab::{register_component_type, Prefab},
    AssetHandle, AssetStorage, DefaultLoader, Handle, LoadStatus, Loader, LoaderBundle,
};
use amethyst_core::ecs::{
    world::ComponentError, Dispatcher, DispatcherBuilder, Entity, IntoQuery, Resources, World,
};
use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use type_uuid::TypeUuid;
mod common;
use serial_test::serial;

fn setup() -> (Dispatcher, World, Resources) {
    let mut dispatcher_builder = DispatcherBuilder::default();
    let mut world = World::default();
    let mut resources = Resources::default();

    let dispatcher = dispatcher_builder
        .add_bundle(LoaderBundle)
        .build(&mut world, &mut resources)
        .expect("Failed to create dispatcher in test setup");
    (dispatcher, world, resources)
}

#[test]
#[serial]
fn a_prefab_can_be_loaded() {
    common::run_test(|| {
        let (mut dispatcher, mut world, mut resources) = setup();

        let prefab_handle: Handle<Prefab> = {
            let loader = resources
                .get_mut::<DefaultLoader>()
                .expect("Missing loader");
            loader.load("single_entity.prefab")
        };

        execute_dispatcher_until_loaded(
            &mut dispatcher,
            &mut world,
            &mut resources,
            prefab_handle.clone(),
        );

        let storage = {
            resources
                .get_mut::<AssetStorage<Prefab>>()
                .expect("Could not get prefab storage from ECS resources")
        };

        let prefab = storage.get(&prefab_handle);
        assert!(prefab.is_some());

        drop(storage);
        dispatcher.unload(&mut world, &mut resources).unwrap();
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

#[test]
#[serial]
fn a_prefab_is_applied_to_an_entity() {
    common::run_test(|| {
        let (mut dispatcher, mut world, mut resources) = setup();

        let prefab_handle: Handle<Prefab> = {
            let loader = resources
                .get_mut::<DefaultLoader>()
                .expect("Missing loader");
            loader.load("test_provided_component.prefab")
        };

        execute_dispatcher_until_loaded(
            &mut dispatcher,
            &mut world,
            &mut resources,
            prefab_handle.clone(),
        );

        let entity = world.push((prefab_handle,));

        execute_dispatcher_until_prefab_is_applied(
            &mut dispatcher,
            &mut world,
            &mut resources,
            entity,
        );

        let mut query = <(Entity, &Position2D)>::query();
        query.for_each(&world, |(entity, position)| {
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

        dispatcher.unload(&mut world, &mut resources).unwrap();
    })
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
        {
            if let Some(entry) = world.entry(entity) {
                match entry.get_component::<Position2D>() {
                    Ok(_position) => break,
                    Err(ComponentError::NotFound { .. }) => (),
                    Err(ComponentError::Denied { .. }) => panic!("Access to component was denied"),
                }
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
