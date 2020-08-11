//! Skinned mesh and bone implementation for renderer.
// use amethyst_assets::PrefabData;
use amethyst_core::{ecs::*, math::Matrix4};
// use amethyst_error::Error;
use rendy::{
    hal::format::Format,
    mesh::{AsAttribute, AsVertex, VertexFormat},
};
// use std::result::Result as StdResult;

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
#[derive(Debug, Clone)]
pub struct JointTransforms {
    /// Skin entity
    pub skin: Entity,
    /// The current joint matrices
    pub matrices: Vec<Matrix4<f32>>,
}

/// Prefab for `JointTransforms`
#[derive(Default, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct JointTransformsPrefab {
    /// Index of skin `Entity`
    pub skin: usize,
    /// Number of joints in the skin
    pub size: usize,
}

impl JointTransformsPrefab {
    /// Creates a new `JointTransformsPrefab`.
    pub fn new(skin: usize, size: usize) -> Self {
        JointTransformsPrefab { skin, size }
    }
}

// impl<'a> PrefabData<'a> for JointTransformsPrefab {
//     type SystemData = WriteStorage<'a, JointTransforms>;
//     type Result = ();

//     fn add_to_entity(
//         &self,
//         entity: Entity,
//         storage: &mut Self::SystemData,
//         entities: &[Entity],
//         _: &[Entity],
//     ) -> StdResult<(), Error> {
//         storage.insert(
//             entity,
//             JointTransforms {
//                 skin: entities[self.skin],
//                 matrices: vec![Matrix4::identity(); self.size],
//             },
//         )?;

//         Ok(())
//     }
// }
