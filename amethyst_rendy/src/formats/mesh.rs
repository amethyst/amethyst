//! Module for mesh support.
use crate::{
    shape::{FromShape, ShapePrefab},
    types::{Mesh, MeshData},
};
use amethyst_assets::{
    AssetPrefab, AssetStorage, Format, Handle, Loader, PrefabData, ProgressCounter,
};
use amethyst_core::ecs::{Entity, Read, ReadExpect, WriteStorage};
use amethyst_error::Error;
use rendy::mesh::MeshBuilder;
use serde::{Deserialize, Serialize};

/// 'Obj' mesh format `Format` implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObjFormat;

amethyst_assets::register_format_type!(MeshData);

amethyst_assets::register_format!("OBJ", ObjFormat as MeshData);
impl Format<MeshData> for ObjFormat {
    fn name(&self) -> &'static str {
        "OBJ"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<MeshData, Error> {
        rendy::mesh::obj::load_from_obj(&bytes)
            .map(|mut builders| {
                let mut iter = builders.drain(..);
                let builder = iter.next().unwrap();
                if iter.next().is_some() {
                    log::warn!("OBJ file contains more than one object, only loading the first");
                }
                builder.0.into()
            })
            .map_err(Error::new)
    }
}

/// Internal mesh loading
///
/// ### Type parameters:
///
/// `V`: Vertex format to use for generated `Mesh`es, for example:
///     * `Vec<PosTex>`
///     * `Vec<PosNormTex>`
///     * `(Vec<Position>, Vec<Normal>)`
#[derive(Debug, Deserialize, Serialize)]
#[serde(bound = "")]
pub enum MeshPrefab<V> {
    /// Load an asset Mesh from file
    Asset(AssetPrefab<Mesh>),
    /// Generate a Mesh from basic type
    Shape(ShapePrefab<V>),
}

impl<'a, V> PrefabData<'a> for MeshPrefab<V>
where
    V: FromShape + Into<MeshBuilder<'static>>,
{
    type SystemData = (
        ReadExpect<'a, Loader>,
        WriteStorage<'a, Handle<Mesh>>,
        Read<'a, AssetStorage<Mesh>>,
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
