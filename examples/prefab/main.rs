//! Demonstrates loading prefabs using the Amethyst engine.

use amethyst::{
    assets::{
        prefab::{register_component_type, Prefab},
        DefaultLoader, Handle, Loader, LoaderBundle,
    },
    core::transform::TransformBundle,
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
use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use type_uuid::TypeUuid;

#[derive(TypeUuid, Serialize, Deserialize, SerdeDiff, Clone, Default)]
#[uuid = "f5780013-bae4-49f0-ac0e-a108ff52fec0"]
struct Position2D {
    position: Vec<f32>,
}

register_component_type!(Position2D);

struct AssetsExample {
    prefab_handle: Option<Handle<Prefab>>,
    root_entity: Option<Entity>,
}

impl SimpleState for AssetsExample {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData {
            world, resources, ..
        } = data;
        let loader = resources.get_mut::<DefaultLoader>().unwrap();
        let prefab_handle: Handle<Prefab> = loader.load("prefab/test.prefab");
        self.prefab_handle = Some(prefab_handle.clone());
        self.root_entity = Some(world.push((prefab_handle,)));
    }

    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        let StateData { world, .. } = data;

        if self.prefab_handle.is_none() {
            log::info!("No prefab");
            return Trans::None;
        }

        let mut query = <(Entity, &Position2D)>::query();
        query.for_each(*world, |(entity, _position)| {
            log::info!("Entity: {:?}", entity,);
        });
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
            root_entity: None,
        },
        dispatcher_builder,
    )?;
    game.run();
    Ok(())
}
