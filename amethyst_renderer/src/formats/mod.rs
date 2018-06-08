//! Provides texture formats
//!

pub use self::mesh::*;
pub use self::mtl::*;
pub use self::texture::*;

use amethyst_assets::{AssetPrefab, Format, PrefabData, PrefabError, ProgressCounter};
use amethyst_core::specs::prelude::Entity;
use serde::de::DeserializeOwned;
use serde::Serialize;

use shape::InternalShape;
use {Mesh, ShapePrefab, Texture};

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
pub enum MeshPrefab<V, M>
where
    M: Format<Mesh>,
    M::Options: DeserializeOwned + Serialize,
{
    /// Load an asset Mesh from file
    Asset(AssetPrefab<Mesh, M>),
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
pub struct GraphicsPrefab<V, M = ObjFormat, T = TextureFormat>
where
    M: Format<Mesh>,
    M::Options: DeserializeOwned + Serialize,
    T: Format<Texture, Options = TextureMetadata>,
{
    mesh: MeshPrefab<V, M>,
    material: MaterialPrefab<T>,
}

impl<'a, V, M, T> PrefabData<'a> for GraphicsPrefab<V, M, T>
where
    M: Format<Mesh> + Clone,
    M::Options: Clone + DeserializeOwned + Serialize,
    T: Format<Texture, Options = TextureMetadata> + Sync + Clone,
    V: From<InternalShape> + Into<MeshData>,
{
    type SystemData = (
        <AssetPrefab<Mesh, M> as PrefabData<'a>>::SystemData,
        <MaterialPrefab<T> as PrefabData<'a>>::SystemData,
    );
    type Result = ();

    fn load_prefab(
        &self,
        entity: Entity,
        system_data: &mut <Self as PrefabData>::SystemData,
        entities: &[Entity],
    ) -> Result<(), PrefabError> {
        match self.mesh {
            MeshPrefab::Asset(ref m) => {
                m.load_prefab(entity, &mut system_data.0, entities)?;
            }
            MeshPrefab::Shape(ref s) => {
                s.load_prefab(entity, &mut system_data.0, entities)?;
            }
        }
        self.material
            .load_prefab(entity, &mut system_data.1, entities)?;
        Ok(())
    }

    fn trigger_sub_loading(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, PrefabError> {
        let mut ret = false;
        if match self.mesh {
            MeshPrefab::Asset(ref mut m) => m.trigger_sub_loading(progress, &mut system_data.0)?,
            MeshPrefab::Shape(ref mut s) => s.trigger_sub_loading(progress, &mut system_data.0)?,
        } {
            ret = true;
        }
        if self.material
            .trigger_sub_loading(progress, &mut system_data.1)?
        {
            ret = true;
        }
        Ok(ret)
    }
}
