use crate::{types::Mesh, shape::ShapePrefab};
use amethyst_assets::{SimpleFormat, AssetPrefab, Format};
use amethyst_error::Error;
use rendy::hal::Backend;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObjFormat;

impl<B: Backend> SimpleFormat<Mesh<B>> for ObjFormat {
    const NAME: &'static str = "WAVEFRONT_OBJ";
    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<rendy::mesh::MeshBuilder<'static>, Error> {
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
