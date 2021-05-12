use amethyst_core::{
    dispatcher::DispatcherBuilder,
    ecs::{Resources, SystemBundle, World},
};
use amethyst_error::Error;

use crate::system::{animation_hierarchy_loading, material_handle_loading, mesh_handle_loading};

/// Bundle that initializes needed resources to use GLTF
#[derive(Debug)]
pub struct GltfBundle;

impl SystemBundle for GltfBundle {
    fn load(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        builder.add_thread_local_fn(mesh_handle_loading);
        builder.add_thread_local_fn(material_handle_loading);
        builder.add_thread_local_fn(animation_hierarchy_loading);
        Ok(())
    }

    fn unload(&mut self, _world: &mut World, _resources: &mut Resources) -> Result<(), Error> {
        Ok(())
    }
}
