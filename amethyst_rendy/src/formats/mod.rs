//! Pre-defined graphical formats and data provided by amethyst_rendy
pub mod mesh;
pub mod mtl;
pub mod texture;

use self::{mesh::MeshPrefab, mtl::MaterialPrefab};
use crate::shape::FromShape;
use amethyst_assets::{PrefabData, ProgressCounter};
use amethyst_core::ecs::prelude::Entity;
use amethyst_error::Error;
use rendy::mesh::MeshBuilder;
use serde::{Deserialize, Serialize};

/// `PrefabData` for loading graphics, ie `Mesh` + `Material`
///
/// ### Type parameters:
///
/// `V`: Vertex format to use for generated `Mesh`es, for example:
///     * `Vec<PosTex>`
///     * `Vec<PosNormTex>`
///     * `(Vec<Position>, Vec<Normal>)`
#[derive(Debug, Deserialize, Serialize)]
pub struct GraphicsPrefab<V> {
    mesh: MeshPrefab<V>,
    material: MaterialPrefab,
}

impl<'a, V> PrefabData<'a> for GraphicsPrefab<V>
where
    V: FromShape + Into<MeshBuilder<'static>>,
{
    type SystemData = (
        <MeshPrefab<V> as PrefabData<'a>>::SystemData,
        <MaterialPrefab as PrefabData<'a>>::SystemData,
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
