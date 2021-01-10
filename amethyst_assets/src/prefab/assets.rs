use atelier_assets::importer as atelier_importer;
use atelier_importer::{typetag, SerdeImportable};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

use crate::{asset::Asset, Handle};

/// Cooked Prefab, containing a World with Entities
#[derive(TypeUuid, Serialize, Deserialize, SerdeImportable)]
#[uuid = "5e751ea4-e63b-4192-a008-f5bf8674e45b"]
pub struct Prefab {
    /// contains Legion World and Entity Mappings
    pub(crate) prefab: Option<legion_prefab::CookedPrefab>,
    /// Contains World to cook and references to other prefabs
    pub(crate) raw_prefab: legion_prefab::Prefab,
    /// `None`: dependencies have not been processed yet
    /// `Some(Vec::len())` is 0: There are no dependencies
    pub(crate) dependencies: Option<Vec<Handle<Prefab>>>,
}

impl Asset for Prefab {
    fn name() -> &'static str {
        "PREFAB"
    }
    type Data = Self;
}
