use amethyst_assets::{
    erased_serde::private::serde::{de, de::SeqAccess, ser::SerializeSeq},
    prefab::{
        register_component_type,
        serde_diff::{ApplyContext, DiffContext},
        SerdeDiff,
    },
};
use amethyst_core::{ecs::*, math::Matrix4};
use type_uuid::TypeUuid;

/// Joint, attach to an entity with a `Transform`
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, TypeUuid, Default)]
#[uuid = "74dc2d04-fa35-4979-8a26-e4f6c9ca6c4f"]
pub struct Joint {
    /// The skins attached to this joint.
    pub skins: Vec<Entity>,
}

impl SerdeDiff for Joint {
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

/// Skin, attach to the root entity in the mesh hierarchy
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, TypeUuid, Default)]
#[uuid = "2ed8ab2f-cb1f-4c64-a762-5c8b8fb95b40"]
pub struct Skin {
    /// Joint entities for the skin
    pub joints: Vec<Entity>,
    /// Mesh entities that use the skin
    pub meshes: Vec<Entity>,
    /// Bind shape matrix
    pub bind_shape_matrix: Matrix4<f32>,
    /// Bring the mesh into the joints local coordinate system
    pub inverse_bind_matrices: Vec<Matrix4<f32>>,
    /// Scratch area holding the current joint matrices
    pub joint_matrices: Vec<Matrix4<f32>>,
}

impl SerdeDiff for Skin {
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

register_component_type!(Skin);

// impl Skin {
//     /// Creates a new `Skin`
//     pub fn new(
//         joints: Vec<Entity>,
//         meshes: Vec<Entity>,
//         inverse_bind_matrices: Vec<Matrix4<f32>>,
//     ) -> Self {
//         let len = joints.len();
//         Skin {
//             joints,
//             meshes,
//             inverse_bind_matrices,
//             bind_shape_matrix: Matrix4::identity(),
//             joint_matrices: Vec::with_capacity(len),
//         }
//     }
// }
