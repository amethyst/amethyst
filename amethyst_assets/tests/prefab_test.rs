use std::time::{Duration, Instant};

use amethyst_assets::{
    prefab::Prefab, AssetHandle, AssetStorage, DefaultLoader, Handle, LoadStatus, Loader,
    LoaderBundle,
};
use amethyst_core::ecs::{Dispatcher, DispatcherBuilder, Resources, World};
use legion_prefab::register_component_type;
use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use type_uuid::TypeUuid;
mod common;

fn setup() -> (Dispatcher, World, Resources) {
    common::setup();
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
fn a_prefab_can_be_loaded() {
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
}

#[derive(TypeUuid, Serialize, Deserialize, SerdeDiff, Clone, Default)]
#[uuid = "f5780013-bae4-49f0-ac0e-a108ff52fec0"]
struct Position2D {
    position: Vec<f32>,
}

register_component_type!(Position2D);

#[test]
fn a_prefab_is_applied_to_an_entity() {
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

    let storage = {
        resources
            .get_mut::<AssetStorage<Prefab>>()
            .expect("Could not get prefab storage from ECS resources")
    };

    let prefab = storage.get(&prefab_handle);
    assert!(prefab.is_some());
}

fn execute_dispatcher_until_loaded(
    dispatcher: &mut Dispatcher,
    world: &mut World,
    resources: &mut Resources,
    prefab_handle: Handle<Prefab>,
) {
    let timeout = Instant::now() + Duration::from_secs(15);
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
