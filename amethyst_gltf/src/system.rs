use amethyst_core::ecs::{World, Resources, Entity, IntoQuery};
use crate::types::MeshHandle;
use amethyst_rendy::{Camera, Mesh, MaterialDefaults};
use amethyst_assets::{Handle, AssetStorage};
use std::ops::Deref;

/// This will attach a Mesh to any Entity with a MeshHandle
pub(crate) fn mesh_handle_loading(world: &mut World, resources: &mut Resources) {
    let mut mesh_storage = resources
        .get_mut::<AssetStorage<Mesh>>()
        .expect("AssetStorage<Mesh> can not be retrieved from ECS Resources");

    let mut default_mat = resources
        .get_mut::<MaterialDefaults>()
        .expect("AssetStorage<Mesh> can not be retrieved from ECS Resources");

    let mut entity_mesh = Vec::new();

    <(Entity, &MeshHandle)>::query().for_each(world, |(entity, mesh_handle)| {
        if let Some(mesh) = mesh_storage.pop(&mesh_handle.0) {
            entity_mesh.push((*entity, mesh));
        }
    });

    while let Some((e, m)) = entity_mesh.pop(){
        world.entry(e).expect("Can't not be").remove_component::<MeshHandle>();
        world.entry(e).expect("Can't not be").add_component(m);
        world.entry(e).expect("Can't not be").add_component(default_mat.0.clone());
    }

}