//! Skinned mesh and bone implementation for renderer.
use amethyst_assets::{
    erased_serde::private::serde::{de, de::SeqAccess, ser::SerializeSeq},
    prefab::{
        register_component_type,
        serde_diff::{ApplyContext, DiffContext},
        SerdeDiff,
    },
};
use amethyst_core::{ecs::*, math::Matrix4};
use rendy::{
    hal::format::Format,
    mesh::{AsAttribute, AsVertex, VertexFormat},
};
use type_uuid::TypeUuid;

/// Type for joint weights attribute of vertex
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct JointWeights(pub [f32; 4]);

impl From<[f32; 4]> for JointWeights {
    fn from(from: [f32; 4]) -> Self {
        Self(from)
    }
}

impl AsAttribute for JointWeights {
    const NAME: &'static str = "joint_weights";
    const FORMAT: Format = Format::Rgba32Sfloat;
}

/// Type for joint ids attribute of vertex
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct JointIds(pub [u16; 4]);

impl From<[u16; 4]> for JointIds {
    fn from(from: [u16; 4]) -> Self {
        Self(from)
    }
}

impl AsAttribute for JointIds {
    const NAME: &'static str = "joint_ids";
    const FORMAT: Format = Format::Rgba16Uint;
}

/// A type for vertex buffer value with interleaved joint data
#[repr(C, align(8))]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct JointCombined {
    /// Joint ids influencing the vertex.
    pub joint_ids: JointIds,
    /// Joint weights influencing the vertex.
    pub joint_weights: JointWeights,
}

impl JointCombined {
    /// Create a new set of joint weights for vertex consumption.
    pub fn new<I: Into<JointIds>, W: Into<JointWeights>>(ids: I, weights: W) -> Self {
        Self {
            joint_ids: ids.into(),
            joint_weights: weights.into(),
        }
    }
}

impl AsVertex for JointCombined {
    fn vertex() -> VertexFormat {
        VertexFormat::new((JointIds::vertex(), JointWeights::vertex()))
    }
}

/// Transform storage for the skin, should be attached to all mesh entities that use a skin
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, TypeUuid)]
#[uuid = "03487599-63f9-4e3b-9e8a-ea73ee08c11d"]
pub struct JointTransforms {
    /// Skin entity
    pub skin: Entity,
    /// The current joint matrices
    pub matrices: Vec<Matrix4<f32>>,
}

impl Default for JointTransforms {
    fn default() -> Self {
        unimplemented!()
    }
}

impl SerdeDiff for JointTransforms {
    fn diff<'a, S: SerializeSeq>(
        &self,
        ctx: &mut DiffContext<'a, S>,
        other: &Self,
    ) -> Result<bool, <S as SerializeSeq>::Error> {
        unimplemented!()
    }

    fn apply<'de, A>(
        &mut self,
        seq: &mut A,
        ctx: &mut ApplyContext,
    ) -> Result<bool, <A as SeqAccess<'de>>::Error>
    where
        A: de::SeqAccess<'de>,
    {
        unimplemented!()
    }
}

register_component_type!(JointTransforms);
