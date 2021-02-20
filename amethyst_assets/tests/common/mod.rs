use std::{panic, path::PathBuf, sync::Once};

use amethyst_assets::{AssetDaemon, LoaderBundle};
use amethyst_core::{
    dispatcher::{Dispatcher, DispatcherBuilder},
    ecs::{Resources, World},
    Logger, LoggerConfig,
};

pub static INIT: Once = Once::new();

pub(crate) fn run_test<T>(test: T)
where
    T: FnOnce(&mut Dispatcher, &mut World, &mut Resources) + panic::UnwindSafe,
{
    INIT.call_once(|| {
        Logger::from_config(LoggerConfig {
            level_filter: log::LevelFilter::Trace,
            ..Default::default()
        })
        .level_for("mio", log::LevelFilter::Error)
        .start();
        let mut asset_daemon = AssetDaemon::new(vec![PathBuf::from("tests/assets")]);
        asset_daemon.start_on_new_thread();
    });

    let result = panic::catch_unwind(|| {
        let mut world = World::default();
        let mut resources = Resources::default();

        let mut dispatcher = DispatcherBuilder::default()
            .add_bundle(LoaderBundle)
            .build(&mut world, &mut resources)
            .expect("Failed to create dispatcher in test setup");

        test(&mut dispatcher, &mut world, &mut resources);

        dispatcher.unload(&mut world, &mut resources).unwrap();
    });

    assert!(result.is_ok())
}
