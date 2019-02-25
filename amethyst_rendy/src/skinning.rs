use {
    amethyst_assets::PrefabData,
    amethyst_core::{
        nalgebra::Matrix4,
        specs::prelude::{Component, DenseVecStorage, Entity, FlaggedStorage, WriteStorage},
    },
    amethyst_error::Error,
    rendy::{hal::format::Format, mesh::AsAttribute},
    std::result::Result as StdResult,
};

/// Type for joint weights attribute of vertex
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct JointWeights(pub [f32; 4]);

impl AsAttribute for JointWeights {
    const NAME: &'static str = "joint_weights";
    const SIZE: u32 = 16;
    const FORMAT: Format = Format::Rgba32Float;
}

/// Type for joint ids attribute of vertex
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct JointIds(pub [u16; 4]);

impl AsAttribute for JointIds {
    const NAME: &'static str = "joint_ids";
    const FORMAT: Format = Format::Rgba16Uint;
    const SIZE: u32 = 8;
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
