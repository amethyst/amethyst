pub mod mesh;
pub mod mtl;
pub mod texture;

use self::{
    mesh::{MeshPrefab, ObjFormat},
    mtl::MaterialPrefab,
    texture::ImageFormat,
};
use crate::{
    shape::InternalShape,
    types::{Mesh, Texture},
};
use amethyst_assets::{Format, PrefabData, ProgressCounter};
use amethyst_core::ecs::prelude::Entity;
use amethyst_error::Error;
use rendy::{hal::Backend, mesh::MeshBuilder, texture::image::ImageTextureConfig};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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
pub struct GraphicsPrefab<B, V, M = ObjFormat, T = ImageFormat>
where
    B: Backend,
    M: Format<Mesh<B>>,
    M::Options: DeserializeOwned + Serialize,
    T: Format<Texture<B>, Options = ImageTextureConfig>,
{
    #[serde(bound(deserialize = "MeshPrefab<B, V, M>: for<'d> Deserialize<'d>"))]
    mesh: MeshPrefab<B, V, M>,
    #[serde(bound(deserialize = "MaterialPrefab<B, T>: for<'d> Deserialize<'d>"))]
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
        <MeshPrefab<B, V, M> as PrefabData<'a>>::SystemData,
        <MaterialPrefab<B, T> as PrefabData<'a>>::SystemData,
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
