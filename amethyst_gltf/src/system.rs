

use amethyst_assets::{AssetStorage};
use amethyst_core::ecs::{Entity, IntoQuery, Resources, World};
use amethyst_rendy::{
    loaders::load_from_srgba, palette::Srgba, types::TextureData, Camera, Material,
    MaterialDefaults, Mesh, Texture,
};

use crate::types::{MeshHandle, MaterialHandle};

/// This will attach a Handle<Mesh> to any Entity with a MeshHandle, and remove the Meshhandle
pub(crate) fn mesh_handle_loading(world: &mut World, _resources: &mut Resources) {
    let mut entity_mesh = Vec::new();

    <(Entity, &MeshHandle)>::query().for_each(world, |(entity, mesh_handle)| {
        entity_mesh.push((*entity, mesh_handle.0.clone()));
    });

    while let Some((entity, mesh)) = entity_mesh.pop() {
        world
            .entry(entity)
            .expect("This can't exist because we just register this entity from the world")
            .remove_component::<MeshHandle>();
        world.entry(entity).expect("This can't exist because we just register this entity from the world").add_component(mesh);
    }
}

/// This will attach a Handle<Material> to any Entity with a MaterialHandle, and remove the MaterialHandle
pub(crate) fn material_handle_loading(world: &mut World, _resources: &mut Resources) {
    let mut entity_material = Vec::new();

    <(Entity, &MaterialHandle)>::query().for_each(world, |(entity, material_handle)| {
        entity_material.push((*entity, material_handle.0.clone()));
    });

    while let Some((entity, material)) = entity_material.pop() {
        world
            .entry(entity)
            .expect("This can't exist because we just register this entity from the world")
            .remove_component::<MaterialHandle>();
        world.entry(entity).expect("This can't exist because we just register this entity from the world").add_component(material);
    }
}
