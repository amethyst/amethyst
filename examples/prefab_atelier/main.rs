//! Demonstrates loading prefabs using the Amethyst engine.

use std::{collections::HashMap, io::Cursor};

use amethyst::{
    assets::{
        prefab::{
            register_component_type, ComponentRegistry, ComponentRegistryBuilder, Prefab, RawPrefab,
        },
        AssetStorage, DefaultLoader, Handle, Loader, LoaderBundle,
    },
    core::transform::{Transform, TransformBundle},
    ecs::{query, World},
    prelude::*,
    renderer::{
        plugins::{RenderShaded3D, RenderToWindow},
        rendy::hal::command::ClearColor,
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
    Error,
};

// type MyPrefabData = BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<TexCoord>)>;

// register_component_type!(Handle<Mesh>);
register_component_type!(Transform);
// register_component_type!(Handle<Material>);
// register_component_type!(Handle<Light>);

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
    let prefab_serde_context = legion_prefab::PrefabSerdeContext {
        registered_components: component_registry.components_by_uuid(),
    };

    let mut buf = Cursor::new(Vec::new());
    let mut ron_ser =
        ron::ser::Serializer::new(buf.get_mut(), Some(ron::ser::PrettyConfig::default()), true)
            .expect("created ron serializer");
    let prefab_ser = legion_prefab::PrefabFormatSerializer::new(prefab_serde_context, prefab);

    prefab_format::serialize(&mut ron_ser, &prefab_ser, prefab.prefab_id())
        .expect("failed to round-trip prefab");

    let buf = String::from_utf8(buf.into_inner()).expect("String from prefab.");

    std::fs::write(path, buf)?;
    Ok(())
}

struct AssetsExample {
    prefab_handle: Option<Handle<Prefab>>,
}

impl SimpleState for AssetsExample {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let component_registry = ComponentRegistryBuilder::default()
            .auto_register_components()
            //.add_spawn_mapping::<TransformValues, Transform>()
            .build();
        let raw_prefab = generate_prefab();
        write_prefab(&component_registry, &raw_prefab.raw_prefab, "prefab.ron")
            .expect("Could not serialize Prefab");
        let StateData { resources, .. } = data;
        let loader = resources.get_mut::<DefaultLoader>().unwrap();
        let prefab_handle: Handle<Prefab> = loader.load("prefab/example.prefab");
        self.prefab_handle = Some(prefab_handle);
    }
    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        let StateData {
            world, resources, ..
        } = data;

        if self.prefab_handle.is_none() {
            log::info!("No prefab");
            return Trans::None;
        }

        let component_registry = resources.get_mut::<ComponentRegistry>().unwrap();
        let prefab_storage = resources.get_mut::<AssetStorage<Prefab>>().unwrap();
        if let Some(opened_prefab) = prefab_storage.get(self.prefab_handle.as_ref().unwrap()) {
            let clone_impl_result = HashMap::default();
            let mut spawn_impl =
                component_registry.spawn_clone_impl(&resources, &clone_impl_result);
            let mappings =
                world.clone_from(&opened_prefab.prefab.world, &query::any(), &mut spawn_impl);
            log::info!("{:?}", mappings);
            return Trans::Quit;
        };
        Trans::None
    }
}

/// Wrapper around the main, so we can return errors easily.
fn main() -> Result<(), Error> {
    let config = amethyst::LoggerConfig {
        level_filter: amethyst::LogLevelFilter::Info,
        module_levels: vec![
            (
                "amethyst_assets".to_string(),
                amethyst::LogLevelFilter::Debug,
            ),
            (
                "atelier_daemon".to_string(),
                amethyst::LogLevelFilter::Debug,
            ),
            (
                "atelier_loader".to_string(),
                amethyst::LogLevelFilter::Trace,
            ),
        ],
        ..Default::default()
    };

    amethyst::start_logger(config);

    let app_root = application_root_dir()?;

    // Add our meshes directory to the asset loader.
    let assets_dir = app_root.join("assets");

    let display_config_path = app_root.join("config/display.ron");

    let mut dispatcher_builder = DispatcherBuilder::default();
    dispatcher_builder
        .add_bundle(LoaderBundle)
        .add_bundle(TransformBundle)
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                        float32: [0.34, 0.36, 0.52, 1.0],
                    }),
                )
                .with_plugin(RenderShaded3D::default()),
        );

    let game = Application::new(
        assets_dir,
        AssetsExample {
            prefab_handle: None,
        },
        dispatcher_builder,
    )?;
    game.run();
    Ok(())
}
