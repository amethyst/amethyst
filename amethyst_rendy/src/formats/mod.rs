pub mod mesh;
pub mod mtl;
pub mod texture;

use self::{
    mesh::{ObjFormat, MeshPrefab},
    mtl::MaterialPrefab,
    texture::ImageFormat,
};
use crate::{
    types::{Mesh, Texture},
    shape::InternalShape,
};
use rendy::{
    hal::Backend,
    texture::image::ImageTextureConfig,
    mesh::MeshBuilder,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use amethyst_core::ecs::prelude::Entity;
use amethyst_assets::{AssetPrefab, Format, PrefabData, ProgressCounter};
use amethyst_error::Error;

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
pub struct GraphicsPrefab<B, V, M = ObjFormat, T = ImageFormat>
    where
        B: Backend,
        M: Format<Mesh<B>>,
        M::Options: DeserializeOwned + Serialize,
        T: Format<Texture<B>, Options = ImageTextureConfig>,
{
    #[serde(bound(deserialize = "MeshPrefab<B, V, M>: Deserialize<'de>"))]
    mesh: MeshPrefab<B, V, M>,
    #[serde(bound(deserialize = "MaterialPrefab<B, T>: Deserialize<'de>"))]
    material: MaterialPrefab<B, T>,
}

impl<'a, B, V, M, T> PrefabData<'a> for GraphicsPrefab<B, V, M, T>
    where
        B: Backend,
        M: Format<Mesh<B>> + Clone,
        M::Options: Clone + DeserializeOwned + Serialize,
        T: Format<Texture<B>, Options = ImageTextureConfig> + Sync + Clone,
        V: From<InternalShape> + Into<MeshBuilder<'static>>,
{
    type SystemData = (
        <AssetPrefab<Mesh<B>, M> as PrefabData<'a>>::SystemData,
        <MaterialPrefab<B, T> as PrefabData<'a>>::SystemData,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut <Self as PrefabData<'_>>::SystemData,
        entities: &[Entity],
    ) -> Result<(), Error> {
        match self.mesh {
            MeshPrefab::Asset(ref m) => {
                m.add_to_entity(entity, &mut system_data.0, entities)?;
            }
            MeshPrefab::Shape(ref s) => {
                s.add_to_entity(entity, &mut system_data.0, entities)?;
            }
        }
        self.material
            .add_to_entity(entity, &mut system_data.1, entities)?;
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
