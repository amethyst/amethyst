use amethyst_assets::{
    prefab::Prefab, start_asset_daemon, AssetHandle, AssetStorage, DefaultLoader, Handle,
    LoadStatus, Loader, LoaderBundle,
};
use amethyst_core::ecs::{Dispatcher, DispatcherBuilder, Resources, World};
use std::{path::PathBuf, thread, time::Duration};

mod common;

fn setup() -> (Dispatcher, World, Resources) {
    common::setup_logger();
    start_asset_daemon(vec![PathBuf::from("tests/assets")]);
    thread::sleep(Duration::from_secs(5));
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

    loop {
        {
            let loader = resources
                .get_mut::<DefaultLoader>()
                .expect("Missing loader");

            match loader.get_load_status_handle(prefab_handle.load_handle()) {
                LoadStatus::Loading => (),
                LoadStatus::Loaded => break,
                LoadStatus::DoesNotExist => assert!(false, "Prefab does not exist"),
                LoadStatus::Error(_) => assert!(false, "Error"),
                LoadStatus::NotRequested => assert!(false, "NotRequested"),
                LoadStatus::Unloading => assert!(false, "Unloading"),
                LoadStatus::Unresolved => assert!(false, "Unresolved"),
            }
        }
        dispatcher.execute(&mut world, &mut resources);
    }

    let storage = {
        resources
            .get_mut::<AssetStorage<Prefab>>()
            .expect("Could not get prefab storage from ECS resources")
    };

    let prefab = storage.get(&prefab_handle);
    assert!(prefab.is_some());
}
