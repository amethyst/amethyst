use amethyst_core::ecs::{World, Resources, Entity, IntoQuery};
use crate::types::MeshHandle;
use amethyst_rendy::Camera;
use amethyst_assets::Handle;

/// This will attach a Mesh to any Entity with a MeshHandle
pub(crate) fn mesh_handle_loading(world: &mut World, _resources: &mut Resources) {
    //<(Entity, &Handle<GltfScene>)>::query().for_each(world, |(e)| println!("Heullkjhkjhkhjkhkjhjkooooh"));
}