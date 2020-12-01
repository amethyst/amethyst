//! Demonstrates loading prefabs using the Amethyst engine.

use amethyst::{
    // assets::{PrefabLoader, PrefabLoaderSystemDesc, RonFormat},
    assets::{
        prefab::{
            register_component_type, ComponentRegistry, ComponentRegistryBuilder, Prefab, RawPrefab,
        },
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
        light::Light,
        mtl::Material,
        plugins::{RenderShaded3D, RenderToWindow},
        rendy::mesh::{Normal, Position, TexCoord},
        types::{DefaultBackend, Mesh},
        RenderingBundle,
    },
    utils::application_root_dir,
    Error,
};
use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use std::collections::HashMap;
use type_uuid::TypeUuid;

// type MyPrefabData = BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<TexCoord>)>;

// register_component_type!(Handle<Mesh>);
register_component_type!(Transform);
// register_component_type!(Handle<Material>);
// register_component_type!(Handle<Light>);

fn serialize_prefab(
    component_registry: &ComponentRegistry,
    prefab: &legion_prefab::Prefab,
) -> String {
    let prefab_serde_context = legion_prefab::PrefabSerdeContext {
        registered_components: component_registry.components_by_uuid(),
    };

    let mut ron_ser = ron::ser::Serializer::new(Some(ron::ser::PrettyConfig::default()), true);
    let prefab_ser = legion_prefab::PrefabFormatSerializer::new(prefab_serde_context, prefab);

    prefab_format::serialize(&mut ron_ser, &prefab_ser, prefab.prefab_id())
        .expect("failed to round-trip prefab");

    ron_ser.into_output_string()
}

fn generate_prefab() -> RawPrefab {
    let mut world = World::default();
    let transform = Transform::default();
    world.push((transform, Some(true)));
    RawPrefab {
        raw_prefab: legion_prefab::Prefab::new(world),
    }
}

fn write_prefab<P: AsRef<std::path::Path>>(
    component_registry: &ComponentRegistry,
    prefab: &legion_prefab::Prefab,
    path: P,
) -> std::io::Result<()> {
    let buf = serialize_prefab(component_registry, prefab);
    std::fs::write(path, buf)?;
    Ok(())
}

struct AssetsExample {
    prefab_handle: Option<Handle<Prefab>>,
}

impl SimpleState for AssetsExample {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        // let component_registry = ComponentRegistryBuilder::new()
        //     .auto_register_components()
        // .add_spawn_mapping::<TransformValues, Transform>()
        // .build();
        // let raw_prefab = generate_prefab();
        // write_prefab(&component_registry, &raw_prefab.raw_prefab, "prefab.ron");
        let StateData {
            world, resources, ..
        } = data;
        let loader = resources.get_mut::<DefaultLoader>().unwrap();
        let prefab_handle: Handle<Prefab> = loader.load("prefab/entity_with_a_transform.prefab");
        self.prefab_handle = Some(prefab_handle);
    }
    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        log::info!("update");
        let StateData {
            world, resources, ..
        } = data;

        if self.prefab_handle.is_none() {
            log::info!("No prefab");
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
            return Trans::Quit;
        };
        Trans::None
    }
}

/// Wrapper around the main, so we can return errors easily.
fn main() -> Result<(), Error> {
    {
        let mut config = amethyst::LoggerConfig::default();
        // config.log_file = Some(std::path::PathBuf::from("asset_loading.log"));
        config.level_filter = amethyst::LogLevelFilter::Info;
        config.module_levels.push((
            "amethyst_assets".to_string(),
            amethyst::LogLevelFilter::Debug,
        ));
        config.module_levels.push((
            "atelier_daemon".to_string(),
            amethyst::LogLevelFilter::Debug,
        ));
        config.module_levels.push((
            "atelier_loader".to_string(),
            amethyst::LogLevelFilter::Trace,
        ));
        amethyst::start_logger(config);
    }
    // amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    // Add our meshes directory to the asset loader.
    let assets_dir = app_root.join("examples/prefab_atelier/assets");

    let display_config_path = app_root.join("examples/prefab_atelier/config/display.ron");

    let mut dispatcher_builder = DispatcherBuilder::default();
    dispatcher_builder
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
