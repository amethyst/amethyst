//! Provides texture formats
//!

pub use self::{mesh::*, mtl::*, texture::*};

use crate::{shape::InternalShape, Mesh, ShapePrefab, Texture};
use amethyst_assets::{AssetPrefab, PrefabData, ProgressCounter};
use amethyst_core::ecs::prelude::Entity;
use amethyst_error::Error;
use serde::{Deserialize, Serialize};

mod mesh;
mod mtl;
mod texture;

/// Internal mesh loading
///
/// ### Type parameters:
///
/// `V`: Vertex format to use for generated `Mesh`es, must be one of:
///     * `Vec<PosTex>`
///     * `Vec<PosNormTex>`
///     * `Vec<PosNormTangTex>`
///     * `ComboMeshCreator`
/// `M`: `Format` to use for loading `Mesh`es from file
#[derive(Deserialize, Serialize)]
pub enum MeshPrefab<V> {
    /// Load an asset Mesh from file
    Asset(AssetPrefab<Mesh>),
    /// Generate a Mesh from basic type
    Shape(ShapePrefab<V>),
}

/// `PrefabData` for loading graphics, ie `Mesh` + `Material`
///
/// ### Type parameters:
///
/// `V`: Vertex format to use for generated `Mesh`es, must be one of:
///     * `Vec<PosTex>`
///     * `Vec<PosNormTex>`
///     * `Vec<PosNormTangTex>`
///     * `ComboMeshCreator`
/// - `M`: `Format` to use for loading `Mesh`es from file
/// - `T`: `Format` to use for loading `Texture`s from file
#[derive(Deserialize, Serialize)]
pub struct GraphicsPrefab<V> {
    mesh: MeshPrefab<V>,
    material: MaterialPrefab,
}

impl<'a, V> PrefabData<'a> for GraphicsPrefab<V>
where
    V: From<InternalShape> + Into<MeshData>,
{
    type SystemData = (
        <AssetPrefab<Mesh> as PrefabData<'a>>::SystemData,
        <MaterialPrefab as PrefabData<'a>>::SystemData,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut <Self as PrefabData<'_>>::SystemData,
        entities: &[Entity],
        children: &[Entity],
    ) -> Result<(), Error> {
        match self.mesh {
            MeshPrefab::Asset(ref m) => {
                m.add_to_entity(entity, &mut system_data.0, entities, children)?;
            }
            MeshPrefab::Shape(ref s) => {
                s.add_to_entity(entity, &mut system_data.0, entities, children)?;
            }
        }
        self.material
            .add_to_entity(entity, &mut system_data.1, entities, children)?;
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let load_mesh = match self.mesh {
            MeshPrefab::Asset(ref mut m) => m.load_sub_assets(progress, &mut system_data.0)?,
            MeshPrefab::Shape(ref mut s) => s.load_sub_assets(progress, &mut system_data.0)?,
        };

        let load_material = self
            .material
            .load_sub_assets(progress, &mut system_data.1)?;

        Ok(load_mesh || load_material)
    }
}
