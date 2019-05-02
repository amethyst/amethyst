use hibitset::BitSet;
use serde::{Deserialize, Serialize};

use amethyst_assets::{PrefabData, ProgressCounter};
use amethyst_core::{
    ecs::prelude::{Component, DenseVecStorage, Entity, WriteStorage},
    math::{Matrix4, RealField},
};
use amethyst_derive::PrefabData;
use amethyst_error::Error;
use amethyst_renderer::JointTransformsPrefab;

/// Joint, attach to an entity with a `Transform`
#[derive(Debug, Clone)]
pub struct Joint {
    /// The skins attached to this joint.
    pub skins: Vec<Entity>,
}

impl Component for Joint {
    type Storage = DenseVecStorage<Self>;
}

/// Skin, attach to the root entity in the mesh hierarchy
#[derive(Debug)]
pub struct Skin<N: RealField> {
    /// Joint entities for the skin
    pub joints: Vec<Entity>,
    /// Mesh entities that use the skin
    pub meshes: BitSet,
    /// Bind shape matrix
    pub bind_shape_matrix: Matrix4<N>,
    /// Bring the mesh into the joints local coordinate system
    pub inverse_bind_matrices: Vec<Matrix4<N>>,
    /// Scratch area holding the current joint matrices
    pub joint_matrices: Vec<Matrix4<N>>,
}

impl<N: RealField> Skin<N> {
    /// Creates a new `Skin`
    pub fn new(
        joints: Vec<Entity>,
        meshes: BitSet,
        inverse_bind_matrices: Vec<Matrix4<N>>,
    ) -> Self {
        let len = joints.len();
        Skin {
            joints,
            meshes,
            inverse_bind_matrices,
            bind_shape_matrix: Matrix4::identity(),
            joint_matrices: Vec::with_capacity(len),
        }
    }
}

impl<N: RealField> Component for Skin<N> {
    type Storage = DenseVecStorage<Self>;
}

/// `PrefabData` for loading `Joint`s
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JointPrefab {
    /// Index of the `Prefab` `Entity` where the `Skin` is placed.
    pub skins: Vec<usize>,
}

impl<'a> PrefabData<'a> for JointPrefab {
    type SystemData = WriteStorage<'a, Joint>;
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        storage: &mut Self::SystemData,
        entities: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        storage
            .insert(
                entity,
                Joint {
                    skins: self.skins.iter().map(|i| entities[*i]).collect(),
                },
            )
            .map(|_| ())?;

        Ok(())
    }
}

/// `PrefabData` for loading `Skin`s
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinPrefab<N: RealField> {
    /// Indices of `Entity`s in the `Prefab` which have `Joint`s belonging to this `Skin`
    pub joints: Vec<usize>,
    /// The bind shape matrix of the `Skin`
    pub bind_shape_matrix: Matrix4<N>,
    /// Indices of the `Entity`s in the `Prefab` which have `Mesh`s using this `Skin`
    pub meshes: Vec<usize>,
    /// Inverse bind matrices of the `Joint`s
    pub inverse_bind_matrices: Vec<Matrix4<N>>,
}

impl<'a, N: RealField> PrefabData<'a> for SkinPrefab<N> {
    type SystemData = WriteStorage<'a, Skin<N>>;
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        storage: &mut Self::SystemData,
        entities: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        storage
            .insert(
                entity,
                Skin {
                    joints: self.joints.iter().map(|index| entities[*index]).collect(),
                    meshes: self
                        .meshes
                        .iter()
                        .map(|index| entities[*index].id())
                        .collect(),
                    bind_shape_matrix: self.bind_shape_matrix,
                    inverse_bind_matrices: self.inverse_bind_matrices.clone(),
                    joint_matrices: Vec::with_capacity(self.joints.len()),
                },
            )
            .map(|_| ())?;

        Ok(())
    }
}

/// `PrefabData` for full skinning support
#[derive(Clone, Default, Debug, Serialize, Deserialize, PrefabData)]
#[serde(default)]
pub struct SkinnablePrefab<N: RealField> {
    /// Place `Skin` on the `Entity`
    pub skin: Option<SkinPrefab<N>>,
    /// Place `Joint` on the `Entity`
    pub joint: Option<JointPrefab>,
    /// Place `JointTransforms` on the `Entity`
    pub joint_transforms: Option<JointTransformsPrefab<N>>,
}
