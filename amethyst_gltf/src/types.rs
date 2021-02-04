use amethyst_assets::Handle;
use amethyst_rendy::Mesh;
use amethyst_assets::prefab::{Prefab, register_component_type, SerdeDiff};
use amethyst_assets::erased_serde::private::serde::ser::SerializeSeq;
use amethyst_assets::prefab::serde_diff::{DiffContext, ApplyContext};
use amethyst_assets::erased_serde::private::serde::de::SeqAccess;
use amethyst_assets::erased_serde::private::serde::de;

use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;
use amethyst_rendy::types::MeshData;

/// MeshHandle is a component that will handle the fact that we attach
/// a mesh to an entity as an asset handle that will later be loaded.
#[derive(Serialize, Deserialize, TypeUuid, Clone)]
#[uuid = "34310974-b4cf-4dc2-a81b-40627c20543a"]
pub struct MeshHandle(pub Handle<Mesh>);
impl Default for MeshHandle{
    fn default() -> Self {
        unimplemented!()
    }
}

impl SerdeDiff for MeshHandle{
    fn diff<'a, S: SerializeSeq>(&self, ctx: &mut DiffContext<'a, S>, other: &Self) -> Result<bool, <S as SerializeSeq>::Error> {
        unimplemented!()
    }

    fn apply<'de, A>(&mut self, seq: &mut A, ctx: &mut ApplyContext) -> Result<bool, <A as SeqAccess<'de>>::Error> where
        A: de::SeqAccess<'de> {
        unimplemented!()
    }
}

register_component_type!(MeshHandle);