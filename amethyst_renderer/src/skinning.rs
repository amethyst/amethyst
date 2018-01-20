use gfx::format::{ChannelType, Format, SurfaceType};
use specs::{Component, DenseVecStorage, Entity, FlaggedStorage};

use error::Result;
use formats::MeshCreator;
use mesh::{Mesh, MeshBuilder};
use renderer::Renderer;
use vertex::{Attribute, Color, Normal, Position, Separate, Tangent, TexCoord};

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
#[derive(Debug)]
pub struct AnimatedComboMeshCreator {
    combo: AnimatedVertexBufferCombination,
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
}

impl From<AnimatedVertexBufferCombination> for AnimatedComboMeshCreator {
    fn from(combo: AnimatedVertexBufferCombination) -> Self {
        Self::new(combo)
    }
}
