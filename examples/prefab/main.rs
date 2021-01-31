//! Demonstrates loading prefabs using the Amethyst engine.
use amethyst::{
    assets::{
        prefab::{legion_prefab, register_component_type, serde_diff, Prefab, SerdeDiff},
        DefaultLoader, Handle, Loader, LoaderBundle,
    },
    core::{transform::TransformBundle, Time},
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
use type_uuid::TypeUuid;

#[derive(TypeUuid, Serialize, Deserialize, SerdeDiff, Clone, Default, Debug)]
#[uuid = "f5780013-bae4-49f0-ac0e-a108ff52fec0"]
struct Position2D {
    position: Vec<f32>,
}

register_component_type!(Position2D);

struct AssetsExample {
    prefab_handle: Option<Handle<Prefab>>,
}

impl SimpleState for AssetsExample {
    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        if self.prefab_handle.is_none() {
            log::info!("No prefab loaded, loading now...");

            let loader = data.resources.get_mut::<DefaultLoader>().unwrap();
            let prefab_handle: Handle<Prefab> = loader.load("prefab/test.prefab");
            self.prefab_handle = Some(prefab_handle.clone());
            data.world.push((prefab_handle,));
        }

        let time = data.resources.get::<Time>().unwrap();

        if time.frame_number() % 60 == 0 {
            let mut query = <(Entity,)>::query();
            let entities: Vec<Entity> = query.iter(data.world).map(|(ent,)| *ent).collect();
            for entity in entities {
                if let Some(entry) = data.world.entry(entity) {
                    log::info!("{:?}: {:?}", entity, entry.archetype());
                    if let Ok(pos) = entry.get_component::<Position2D>() {
                        log::info!("{:?}", pos);
                    }
                }
            }
        }

        Trans::None
    }
}

/// Wrapper around the main, so we can return errors easily.
fn main() -> Result<(), Error> {
    let config = amethyst::LoggerConfig {
        level_filter: amethyst::LogLevelFilter::Debug,
        module_levels: vec![
            (
                "amethyst_assets".to_string(),
                amethyst::LogLevelFilter::Trace,
            ),
            ("distill_daemon".to_string(), amethyst::LogLevelFilter::Warn),
            ("distill_loader".to_string(), amethyst::LogLevelFilter::Warn),
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
