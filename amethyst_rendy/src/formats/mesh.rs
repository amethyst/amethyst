//! Module for mesh support.
use amethyst_assets::Format;
use amethyst_error::Error;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

use crate::types::MeshData;

/// 'Obj' mesh format `Format` implementation.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    TypeUuid,
)]
#[uuid = "7994868a-3ca1-4498-a6e5-4849598a6b22"]
pub struct ObjFormat;

amethyst_assets::register_importer!(".obj", ObjFormat);
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
            .map_err(|e| e.into())
    }
}
