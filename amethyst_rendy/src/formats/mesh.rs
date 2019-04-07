use crate::{shape::ShapePrefab, types::Mesh};
use amethyst_assets::{AssetPrefab, Format, SimpleFormat};
use amethyst_error::Error;
use rendy::{hal::Backend, mesh::MeshBuilder};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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
pub enum MeshPrefab<B, V, M>
where
    B: Backend,
    M: Format<Mesh<B>>,
    M::Options: DeserializeOwned + Serialize,
{
    /// Load an asset Mesh from file
    #[serde(bound(deserialize = "AssetPrefab<Mesh<B>, M>: Deserialize<'de>"))]
    Asset(AssetPrefab<Mesh<B>, M>),
    /// Generate a Mesh from basic type
    #[serde(bound(deserialize = "ShapePrefab<B, V>: Deserialize<'de>"))]
    Shape(ShapePrefab<B, V>),
}

// pub struct PrefabMeshBuilder<B: Backend> {
//     inner: MeshBuilder<'static>,
//     marker: PhantomData<B>,
// }

// impl<B: Backend> From<MeshBuilder<'static>> for PrefabMeshBuilder<B> {
//     fn from(from: MeshBuilder<'static>) -> Self {
//         Self {
//             inner: from,
//             marker: PhantomData,
//         }
//     }
// }

// impl<'a, B: Backend> PrefabData<'a> for PrefabMeshBuilder<B> {
//     type SystemData = (
//         ReadExpect<'a, Loader>,
//         WriteStorage<'a, Handle<Mesh<B>>>,
//         Read<'a, AssetStorage<Mesh<B>>>,
//     );
//     type Result = ();

//     fn add_to_entity(
//         &self,
//         entity: Entity,
//         system_data: &mut Self::SystemData,
//         _: &[Entity],
//     ) -> Result<(), Error> {
//         let handle = system_data
//             .0
//             .load_from_data(self.inner.clone(), (), &system_data.2);
//         system_data.1.insert(entity, handle).map(|_| ())?;
//         Ok(())
//     }
// }
