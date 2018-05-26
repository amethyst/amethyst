//! Provides texture formats
//!

pub use self::mesh::*;
pub use self::mtl::*;
pub use self::texture::*;

use amethyst_assets::{AssetPrefab, Format, PrefabData};
use amethyst_core::specs::error::Error;
use amethyst_core::specs::prelude::Entity;
use serde::{Deserialize, Serialize};

use {Mesh, Texture};

mod mesh;
mod mtl;
mod texture;

/// `PrefabData` for loading graphics, ie `Mesh` + `Material`
///
/// ### Type parameters:
///
/// - `M`: `Format` to use for loading `Mesh`es from file
/// - `T`: `Format` to use for loading `Texture`s from file
#[derive(Deserialize, Serialize)]
pub struct GraphicsPrefab<M, T>
where
    M: Format<Mesh>,
    M::Options: for<'a> Deserialize<'a> + Serialize,
    T: Format<Texture, Options = TextureMetadata>,
{
    mesh: AssetPrefab<Mesh, M>,
    material: MaterialPrefab<T>,
}

impl<'a, M, T> PrefabData<'a> for GraphicsPrefab<M, T>
where
    M: Format<Mesh> + Clone,
    M::Options: Clone + for<'b> Deserialize<'b> + Serialize,
    T: Format<Texture, Options = TextureMetadata> + Sync + Clone,
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
    ) -> Result<(), Error> {
        self.mesh.load_prefab(entity, &mut system_data.0, entities)?;
        self.material
            .load_prefab(entity, &mut system_data.1, entities)?;
        Ok(())
    }
}
