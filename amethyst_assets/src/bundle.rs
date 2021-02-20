use std::default::Default;

use amethyst_core::ecs::{DispatcherBuilder, Resources, SystemBundle, World};
use amethyst_error::Error;

use crate::{prefab::ComponentRegistryBuilder, DefaultLoader, Loader};

fn asset_loading_tick(_: &mut World, resources: &mut Resources) {
    let mut loader = resources
        .get_mut::<DefaultLoader>()
        .expect("Could not get_mut DefaultLoader");
    loader
        .process(resources)
        .expect("Error in Loader processing");
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
