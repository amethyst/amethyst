use crate::{
    shape::{FromShape, ShapePrefab},
    types::Mesh,
};
use amethyst_assets::{
    AssetPrefab, AssetStorage, Format, Handle, Loader, PrefabData, ProgressCounter, SimpleFormat,
};
use amethyst_core::ecs::{Entity, Read, ReadExpect, WriteStorage};
use amethyst_error::Error;
use rendy::{hal::Backend, mesh::MeshBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObjFormat;

impl<B: Backend> SimpleFormat<Mesh<B>> for ObjFormat {
    const NAME: &'static str = "WAVEFRONT_OBJ";
    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<MeshBuilder<'static>, Error> {
        rendy::mesh::obj::load_from_obj(&bytes).map_err(|e| e.compat().into())
    }
}

/// Internal mesh loading
///
/// ### Type parameters:
///
/// `B`: `Backend` type parameter for `Mesh<B>`
/// `V`: Vertex format to use for generated `Mesh`es, must be one of:
///     * `Vec<PosTex>`
///     * `Vec<PosNormTex>`
///     * `Vec<PosNormTangTex>`
///     * `ComboMeshCreator`
/// `M`: `Format` to use for loading `Mesh`es from file
#[derive(Deserialize, Serialize)]
#[serde(bound = "")]
pub enum MeshPrefab<B, V, F = ObjFormat>
where
    B: Backend,
    F: Format<Mesh<B>>,
{
    /// Load an asset Mesh from file
    #[serde(bound(
        serialize = "AssetPrefab<Mesh<B>, F>: Serialize",
        deserialize = "AssetPrefab<Mesh<B>, F>: Deserialize<'de>",
    ))]
    Asset(AssetPrefab<Mesh<B>, F>),
    /// Generate a Mesh from basic type
    Shape(ShapePrefab<B, V>),
}

impl<'a, B, V, F> PrefabData<'a> for MeshPrefab<B, V, F>
where
    B: Backend,
    F: Format<Mesh<B>>,
    V: FromShape + Into<MeshBuilder<'static>>,
{
    type SystemData = (
        ReadExpect<'a, Loader>,
        WriteStorage<'a, Handle<Mesh<B>>>,
        Read<'a, AssetStorage<Mesh<B>>>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
        children: &[Entity],
    ) -> Result<(), Error> {
        match self {
            MeshPrefab::Asset(m) => {
                m.add_to_entity(entity, system_data, entities, children)?;
            }
            MeshPrefab::Shape(s) => {
                s.add_to_entity(entity, system_data, entities, children)?;
            }
        }
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        Ok(match self {
            MeshPrefab::Asset(m) => m.load_sub_assets(progress, system_data)?,
            MeshPrefab::Shape(s) => s.load_sub_assets(progress, system_data)?,
        })
    }
}
