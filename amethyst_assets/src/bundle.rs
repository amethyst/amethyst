use std::{default::Default, path::PathBuf};

use amethyst_core::ecs::{DispatcherBuilder, Resources, SystemBundle, World};
use amethyst_error::Error;
use distill::daemon::{default_importer_contexts, default_importers, AssetDaemon, ImporterMap};
use log::info;

use crate::{
    prefab::{ComponentRegistryBuilder, PrefabImporter},
    simple_importer::get_source_importers,
    DefaultLoader, Loader,
};

fn asset_loading_tick(_: &mut World, resources: &mut Resources) {
    let mut loader = resources
        .get_mut::<DefaultLoader>()
        .expect("Could not get_mut DefaultLoader");

    loader
        .process(resources)
        .expect("Error in Loader processing");
}

/// starts the asset thread with distill_daemon
pub fn start_asset_daemon(asset_dirs: Vec<PathBuf>) {
    let db_path = ".assets_db";
    let address = "127.0.0.1:9999";

    let mut importer_map = ImporterMap::default();
    let mut importers = default_importers();

    importers.push(("prefab", Box::new(PrefabImporter::default())));

    importers.extend(get_source_importers());

    for (ext, importer) in importers {
        info!("Adding importer for ext {}", ext);
        importer_map.insert(ext, importer);
    }

    AssetDaemon {
        db_dir: PathBuf::from(db_path),
        address: address.parse().unwrap(),
        importers: importer_map,
        importer_contexts: default_importer_contexts(),
        asset_dirs,
    }
    .run();
}

/// Bundle that initializes Loader as well as related processing systems and resources
pub struct LoaderBundle;

impl SystemBundle for LoaderBundle {
    fn load(
        &mut self,
        _: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        let component_registry = ComponentRegistryBuilder::default()
            .auto_register_components()
            .build();
        resources.insert(component_registry);
        let mut loader = DefaultLoader::default();
        loader.init_world(resources);
        loader.init_dispatcher(builder);
        resources.insert(loader);

        builder.add_thread_local_fn(asset_loading_tick);
        builder.add_thread_local_fn(crate::prefab::system::prefab_spawning_tick);

        Ok(())
    }
}
