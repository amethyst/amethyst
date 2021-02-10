use amethyst_assets::{
    erased_serde::private::serde::{de, de::SeqAccess, ser::SerializeSeq},
    prefab::{
        register_component_type,
        serde_diff::{ApplyContext, DiffContext},
        SerdeDiff,
    },
    Handle,
};
use amethyst_rendy::{Material, Mesh};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

/// MeshHandle is a component that will handle the fact that we attach
/// a mesh to an entity as an asset handle that will later be loaded.
#[derive(Serialize, Deserialize, TypeUuid, Clone)]
#[uuid = "34310974-b4cf-4dc2-a81b-40627c20543a"]
pub struct MeshHandle(pub Handle<Mesh>);
impl Default for MeshHandle {
    fn default() -> Self {
        unimplemented!()
    }
}

impl SerdeDiff for MeshHandle {
    fn diff<'a, S: SerializeSeq>(
        &self,
        _ctx: &mut DiffContext<'a, S>,
        _other: &Self,
    ) -> Result<bool, <S as SerializeSeq>::Error> {
        unimplemented!()
    }

    fn apply<'de, A>(
        &mut self,
        _seq: &mut A,
        _ctx: &mut ApplyContext,
    ) -> Result<bool, <A as SeqAccess<'de>>::Error>
    where
        A: de::SeqAccess<'de>,
    {
        unimplemented!()
    }
}

register_component_type!(MeshHandle);

/// MaterialHandle is a component that will handle the fact that we attach
/// a material handle to an entity as an asset handle that will later be loaded.
#[derive(Serialize, Deserialize, TypeUuid, Clone)]
#[uuid = "40a2d8f7-54e8-46ad-b668-66d759feb806"]
pub struct MaterialHandle(pub Handle<Material>);
impl Default for MaterialHandle {
    fn default() -> Self {
        unimplemented!()
    }
}

impl SerdeDiff for MaterialHandle {
    fn diff<'a, S: SerializeSeq>(
        &self,
        _ctx: &mut DiffContext<'a, S>,
        _other: &Self,
    ) -> Result<bool, <S as SerializeSeq>::Error> {
        unimplemented!()
    }

    fn apply<'de, A>(
        &mut self,
        _seq: &mut A,
        _ctx: &mut ApplyContext,
    ) -> Result<bool, <A as SeqAccess<'de>>::Error>
    where
        A: de::SeqAccess<'de>,
    {
        unimplemented!()
    }
}

register_component_type!(MaterialHandle);
