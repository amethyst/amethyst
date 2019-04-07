use amethyst_assets::PrefabData;
use amethyst_core::{
    ecs::prelude::{Component, DenseVecStorage, Entity, FlaggedStorage, WriteStorage},
    math::Matrix4,
};
use amethyst_error::Error;
use rendy::{
    hal::format::Format,
    mesh::{
        AsAttribute, AsVertex, Attribute, Normal, Position, Tangent, TexCoord, VertexFormat,
        WithAttribute,
    },
};
use std::{borrow::Cow, result::Result as StdResult};

/// Type for joint weights attribute of vertex
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct JointWeights(pub [f32; 4]);

impl From<[f32; 4]> for JointWeights {
    fn from(from: [f32; 4]) -> Self {
        Self(from)
    }
}

impl AsAttribute for JointWeights {
    const NAME: &'static str = "joint_weights";
    const SIZE: u32 = 16;
    const FORMAT: Format = Format::Rgba32Float;
}

/// Type for joint ids attribute of vertex
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
    const SIZE: u32 = 8;
}

/// Vertex format with position, normal, tangent, and UV texture coordinate attributes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PosNormTangTexJoint {
    /// Position of the vertex in 3D space.
    pub position: Position,
    /// Normal vector of the vertex.
    pub normal: Normal,
    /// Tangent vector of the vertex.
    pub tangent: Tangent,
    /// UV texture coordinates used by the vertex.
    pub tex_coord: TexCoord,
    /// Joint ids influencing the vertex.
    pub joint_ids: JointIds,
    /// Joint weights influencing the vertex.
    pub joint_weights: JointWeights,
}

impl WithAttribute<Position> for PosNormTangTexJoint {
    const ATTRIBUTE: Attribute = Attribute {
        offset: 0,
        format: Position::FORMAT,
    };
}

impl WithAttribute<Normal> for PosNormTangTexJoint {
    const ATTRIBUTE: Attribute = Attribute {
        offset: Position::SIZE,
        format: Normal::FORMAT,
    };
}

impl WithAttribute<Tangent> for PosNormTangTexJoint {
    const ATTRIBUTE: Attribute = Attribute {
        offset: Position::SIZE + Normal::SIZE,
        format: Tangent::FORMAT,
    };
}

impl WithAttribute<TexCoord> for PosNormTangTexJoint {
    const ATTRIBUTE: Attribute = Attribute {
        offset: Position::SIZE + Normal::SIZE + Tangent::SIZE,
        format: TexCoord::FORMAT,
    };
}

impl WithAttribute<JointIds> for PosNormTangTexJoint {
    const ATTRIBUTE: Attribute = Attribute {
        offset: Position::SIZE + Normal::SIZE + Tangent::SIZE + TexCoord::SIZE,
        format: JointIds::FORMAT,
    };
}

impl WithAttribute<JointWeights> for PosNormTangTexJoint {
    const ATTRIBUTE: Attribute = Attribute {
        offset: Position::SIZE + Normal::SIZE + Tangent::SIZE + TexCoord::SIZE + JointIds::SIZE,
        format: JointWeights::FORMAT,
    };
}

impl AsVertex for PosNormTangTexJoint {
    const VERTEX: VertexFormat<'static> = VertexFormat {
        attributes: Cow::Borrowed(&[
            <Self as WithAttribute<Position>>::ATTRIBUTE,
            <Self as WithAttribute<Normal>>::ATTRIBUTE,
            <Self as WithAttribute<Tangent>>::ATTRIBUTE,
            <Self as WithAttribute<TexCoord>>::ATTRIBUTE,
            <Self as WithAttribute<JointIds>>::ATTRIBUTE,
            <Self as WithAttribute<JointWeights>>::ATTRIBUTE,
        ]),
        stride: Position::SIZE
            + Normal::SIZE
            + Tangent::SIZE
            + TexCoord::SIZE
            + JointIds::SIZE
            + JointWeights::SIZE,
    };
}

/// Transform storage for the skin, should be attached to all mesh entities that use a skin
#[derive(Debug, Clone)]
pub struct JointTransforms {
    /// Skin entity
    pub skin: Entity,
    /// The current joint matrices
    pub matrices: Vec<Matrix4<f32>>,
}

impl Component for JointTransforms {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

/// Prefab for `JointTransforms`
#[derive(Default, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct JointTransformsPrefab {
    /// Index of skin `Entity`
    pub skin: usize,
    /// Number of joints in the skin
    pub size: usize,
}

impl<'a> PrefabData<'a> for JointTransformsPrefab {
    type SystemData = WriteStorage<'a, JointTransforms>;
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        storage: &mut Self::SystemData,
        entities: &[Entity],
        _: &[Entity],
    ) -> StdResult<(), Error> {
        storage.insert(
            entity,
            JointTransforms {
                skin: entities[self.skin],
                matrices: vec![Matrix4::identity(); self.size],
            },
        )?;

        Ok(())
    }
}
