use amethyst_core::ecs::World;
use atelier_assets::importer as atelier_importer;
use atelier_importer::{typetag, SerdeImportable};
use fnv::FnvHashSet;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

use crate::{asset::Asset, Handle, WeakHandle};

/// Prefab Asset, containing a cooked world.
#[derive(TypeUuid, Serialize, Deserialize, SerdeImportable)]
#[uuid = "5e751ea4-e63b-4192-a008-f5bf8674e45b"]
pub struct Prefab {
    /// contains Legion World and Entity Mappings
    pub(crate) cooked: Option<legion_prefab::CookedPrefab>,

    /// Contains World to cook and references to other prefabs
    pub(crate) raw: legion_prefab::Prefab,

    /// `None`: dependencies have not been processed yet
    /// `Some(Vec::len())` is 0: There are no dependencies
    #[serde(skip)]
    pub(crate) dependencies: Option<Vec<Handle<Prefab>>>,

    #[serde(skip)]
    pub(crate) dependers: FnvHashSet<WeakHandle>,

    /// Incremented everytime the prefab is cooked.
    #[serde(skip)]
    pub(crate) version: u32,
}

impl Default for Prefab {
    fn default() -> Self {
        Prefab {
            raw: legion_prefab::Prefab::new(World::default()),
            dependencies: None,
            dependers: FnvHashSet::default(),
            cooked: None,
            version: 0,
        }
    }
}

impl Asset for Prefab {
    fn name() -> &'static str {
        "PREFAB"
    }
    type Data = Self;
}
