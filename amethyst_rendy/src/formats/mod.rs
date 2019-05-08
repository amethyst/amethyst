pub mod mesh;
pub mod mtl;
pub mod texture;

use self::{mesh::MeshPrefab, mtl::MaterialPrefab};
use crate::shape::FromShape;
use amethyst_assets::{PrefabData, ProgressCounter};
use amethyst_core::ecs::prelude::Entity;
use amethyst_error::Error;
use rendy::{hal::Backend, mesh::MeshBuilder};
use serde::{Deserialize, Serialize};

/// `PrefabData` for loading graphics, ie `Mesh` + `Material`
///
/// ### Type parameters:
///
/// `B`: `Backend` type parameter for `Mesh<B>` and `Texture<B>`
/// `V`: Vertex format to use for generated `Mesh`es, must be one of:
///     * `Vec<PosTex>`
///     * `Vec<PosNormTex>`
///     * `Vec<PosNormTangTex>`
///     * `ComboMeshCreator`
/// - `M`: `Format` to use for loading `Mesh`es from file
/// - `T`: `Format` to use for loading `Texture`s from file
#[derive(Deserialize, Serialize)]
pub struct GraphicsPrefab<B: Backend, V> {
    mesh: MeshPrefab<B, V>,
    material: MaterialPrefab<B>,
}

impl<'a, B, V> PrefabData<'a> for GraphicsPrefab<B, V>
where
    B: Backend,
    V: FromShape + Into<MeshBuilder<'static>>,
{
    type SystemData = (
        <MeshPrefab<B, V> as PrefabData<'a>>::SystemData,
        <MaterialPrefab<B> as PrefabData<'a>>::SystemData,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        (ref mut mesh_data, ref mut mat_data): &mut Self::SystemData,
        ent: &[Entity],
        ch: &[Entity],
    ) -> Result<(), Error> {
        self.mesh.add_to_entity(entity, mesh_data, ent, ch)?;
        self.material.add_to_entity(entity, mat_data, ent, ch)?;
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        (ref mut mesh_data, ref mut mat_data): &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let load_mesh = self.mesh.load_sub_assets(progress, mesh_data)?;
        let load_material = self.material.load_sub_assets(progress, mat_data)?;
        Ok(load_mesh || load_material)
    }
}
