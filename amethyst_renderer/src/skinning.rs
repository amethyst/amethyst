use std::result::Result as StdResult;

use gfx::format::{ChannelType, Format, SurfaceType};
use serde::{Deserialize, Serialize};

use amethyst_assets::{PrefabData, PrefabError};
use amethyst_core::specs::prelude::{
    Component, DenseVecStorage, Entity, FlaggedStorage, WriteStorage,
};

use crate::{
    error::Result,
    formats::MeshCreator,
    mesh::{Mesh, MeshBuilder},
    renderer::Renderer,
    vertex::{Attribute, Color, Normal, Position, Separate, Tangent, TexCoord},
};

/// Type for joint weights attribute of vertex
#[derive(Clone, Debug)]
pub enum JointWeights {}
impl Attribute for JointWeights {
    const NAME: &'static str = "joint_weights";
    const FORMAT: Format = Format(SurfaceType::R32_G32_B32_A32, ChannelType::Unorm);
    const SIZE: u32 = 16;
    type Repr = [f32; 4];
}

/// Type for joint ids attribute of vertex
#[derive(Clone, Debug)]
pub enum JointIds {}
impl Attribute for JointIds {
    const NAME: &'static str = "joint_ids";
    const FORMAT: Format = Format(SurfaceType::R16_G16_B16_A16, ChannelType::Uint);
    const SIZE: u32 = 8;
    type Repr = [u16; 4];
}

/// Transform storage for the skin, should be attached to all mesh entities that use a skin
#[derive(Debug, Clone)]
pub struct JointTransforms {
    /// Skin entity
    pub skin: Entity,
    /// The current joint matrices
    pub matrices: Vec<[[f32; 4]; 4]>,
}

impl Component for JointTransforms {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

/// Animation vertex combo
pub type AnimatedVertexBufferCombination = (
    Vec<Separate<Position>>,
    Option<Vec<Separate<Color>>>,
    Option<Vec<Separate<TexCoord>>>,
    Option<Vec<Separate<Normal>>>,
    Option<Vec<Separate<Tangent>>>,
    Option<Vec<Separate<JointIds>>>,
    Option<Vec<Separate<JointWeights>>>,
);

/// Build Mesh with vertex buffer combination
fn build_mesh_with_combo(
    combo: AnimatedVertexBufferCombination,
    renderer: &mut Renderer,
) -> Result<Mesh> {
    build_mesh_with_some!(
        MeshBuilder::new(combo.0),
        renderer,
        combo.1,
        combo.2,
        combo.3,
        combo.4,
        combo.5,
        combo.6
    )
}

/// Mesh creator for `VertexBufferCombination`.
#[derive(Debug, Clone)]
pub struct AnimatedComboMeshCreator {
    /// The internal mesh combo data.
    pub combo: AnimatedVertexBufferCombination,
}

impl AnimatedComboMeshCreator {
    /// Create a new combo mesh creator with the given combo
    pub fn new(combo: AnimatedVertexBufferCombination) -> Self {
        AnimatedComboMeshCreator { combo }
    }
}

impl MeshCreator for AnimatedComboMeshCreator {
    fn build(self: Box<Self>, renderer: &mut Renderer) -> Result<Mesh> {
        build_mesh_with_combo(self.combo, renderer)
    }

    fn vertices(&self) -> &Vec<Separate<Position>> {
        &self.combo.0
    }

    fn box_clone(&self) -> Box<dyn MeshCreator> {
        Box::new((*self).clone())
    }
}

impl From<AnimatedVertexBufferCombination> for AnimatedComboMeshCreator {
    fn from(combo: AnimatedVertexBufferCombination) -> Self {
        Self::new(combo)
    }
}

/// Prefab for `JointTransforms`
#[derive(Default, Clone, Debug, Deserialize, Serialize)]
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
    ) -> StdResult<(), PrefabError> {
        storage
            .insert(
                entity,
                JointTransforms {
                    skin: entities[self.skin],
                    matrices: vec![[[0.; 4]; 4]; self.size],
                },
            )
            .map(|_| ())
    }
}
