use atelier_assets::{importer as atelier_importer, loader::handle::Handle};
use atelier_importer::{typetag, SerdeImportable};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

use crate::asset::Asset;

/// Cooked Prefab, containing a World with Entities
#[derive(TypeUuid, Serialize, Deserialize, SerdeImportable)]
#[uuid = "5e751ea4-e63b-4192-a008-f5bf8674e45b"]
pub struct Prefab {
    /// contains Legion World and Entity Mappings
    pub prefab: legion_prefab::CookedPrefab,
}

/// Raw prefab type, used to generate the cooked prefab
#[derive(TypeUuid, Serialize, Deserialize, SerdeImportable)]
#[uuid = "c77ccda8-f2f0-4a7f-91ef-f38fabc0e6ce"]
pub struct RawPrefab {
    /// Contains World to cook and references to other prefabs
    pub raw_prefab: legion_prefab::Prefab,
    /// `None`: dependencies have not been processed yet
    /// `Some(Vec::len())` is 0: There are no dependencies
    pub(crate) dependencies: Option<Vec<Handle<RawPrefab>>>,
}

impl Asset for Prefab {
    fn name() -> &'static str {
        "PREFAB"
    }
    type Data = RawPrefab;
}
