//! Demonstrates loading prefabs using the Amethyst engine.

use amethyst::{
    // assets::{PrefabLoader, PrefabLoaderSystemDesc, RonFormat},
    assets::{
        prefab::{ComponentRegistry, Prefab},
        AssetStorage, DefaultLoader, Format as AssetFormat, Handle, Loader, LoaderBundle,
        ProcessingQueue,
    },
    core::{
        math::Vector3,
        transform::{Transform, TransformBundle},
    },
    ecs::{query, Resources, World},
    prelude::*,
    renderer::{
        plugins::{RenderShaded3D, RenderToWindow},
        rendy::mesh::{Normal, Position, TexCoord},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
    Error,
};
use std::collections::HashMap;

// type MyPrefabData = BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<TexCoord>)>;

struct AssetsExample {
    prefab_handle: Option<Handle<Prefab>>,
}

impl SimpleState for AssetsExample {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData {
            world, resources, ..
        } = data;
        // let prefab_handle = data.world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
        //     loader.load("prefab/example.prefab", RonFormat, ())
        // });
        // data.world.create_entity().with(prefab_handle).build();
        let loader = resources.get_mut::<DefaultLoader>().unwrap();
        let prefab_handle: Handle<Prefab> = loader.load("prefab/example.prefab");
        self.prefab_handle = Some(prefab_handle);
    }
    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        let StateData {
            world, resources, ..
        } = data;

        if self.prefab_handle.is_none() {
            return Trans::None;
        }

        let mut component_registry = resources.get_mut::<ComponentRegistry>().unwrap();
        let mut prefab_storage = resources.get_mut::<AssetStorage<Prefab>>().unwrap();
        if let Some(opened_prefab) = prefab_storage.get(self.prefab_handle.as_ref().unwrap()) {
            let mut clone_impl_result = HashMap::default();
            let mut spawn_impl =
                component_registry.spawn_clone_impl(&resources, &mut clone_impl_result);
            let mappings = world.clone_from(
                &opened_prefab.prefab.world,
                &query::any(),
                &mut spawn_impl,
                // &mut component_registry, // .spawn_clone_impl(resources, &opened_prefab.prefab_to_world_mappings),
            );
            log::info!("{:?}", mappings);
        };
        Trans::None
    }
}

/// Wrapper around the main, so we can return errors easily.
fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    // Add our meshes directory to the asset loader.
    let assets_dir = app_root.join("examples/prefab/assets");

    let display_config_path = app_root.join("examples/prefab/config/display.ron");

    let mut dispatcher_builder = DispatcherBuilder::default();
    dispatcher_builder
        // with_system_desc(PrefabLoaderSystemDesc::<MyPrefabData>::default(), "", &[])
        .add_bundle(LoaderBundle)
        .add_bundle(TransformBundle)
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderShaded3D::default()),
        );

    let mut game = Application::new(
        assets_dir,
        AssetsExample {
            prefab_handle: None,
        },
        dispatcher_builder,
    )?;
    game.run();
    Ok(())
}
