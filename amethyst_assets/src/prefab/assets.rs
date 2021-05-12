use amethyst_core::ecs::World;
use distill::importer as distill_importer;
use distill_importer::{typetag, SerdeImportable};
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

    #[serde(skip)]
    pub(crate) dependencies: Vec<Handle<Prefab>>,

    #[serde(skip)]
    pub(crate) dependers: FnvHashSet<WeakHandle>,

    /// Incremented everytime the prefab is cooked.
    #[serde(skip)]
    pub(crate) version: u32,
}

impl Prefab {
    /// Create a new Amethyst Prefab giving a legion Prefab object
    pub fn new(raw: legion_prefab::Prefab) -> Self {
        Self {
            raw,
            ..Default::default()
        }
    }
}

impl Default for Prefab {
    fn default() -> Self {
        Prefab {
            raw: legion_prefab::Prefab::new(World::default()),
            dependencies: Vec::new(),
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
