

use amethyst_assets::{AssetStorage};
use amethyst_core::ecs::{Entity, IntoQuery, Resources, World};
use amethyst_rendy::{
    loaders::load_from_srgba, palette::Srgba, types::TextureData, Camera, Material,
    MaterialDefaults, Mesh, Texture,
};

use crate::types::MeshHandle;

/// This will attach a Mesh to any Entity with a MeshHandle, and remove the Meshhandle
pub(crate) fn mesh_handle_loading(world: &mut World, resources: &mut Resources) {
    let mut mesh_storage = resources
        .get_mut::<AssetStorage<Mesh>>()
        .expect("AssetStorage<Mesh> can not be retrieved from ECS Resources");

    let mut entity_mesh = Vec::new();

    <(Entity, &MeshHandle)>::query().for_each(world, |(entity, mesh_handle)| {
        if let Some(mesh) = mesh_storage.pop(&mesh_handle.0) {
            entity_mesh.push((*entity, mesh));
        }
    });

    while let Some((entity, mesh)) = entity_mesh.pop() {
        world
            .entry(entity)
            .expect("This can't exist because we just register this entity from the world")
            .remove_component::<MeshHandle>();
        world.entry(entity).expect("This can't exist because we just register this entity from the world").add_component(mesh);
    }
}
