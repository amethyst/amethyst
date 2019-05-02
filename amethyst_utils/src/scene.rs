//! Provides utilities for building and describing scenes in your game.

use std::fmt::Debug;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use amethyst_assets::{Format, PrefabData, ProgressCounter};
use amethyst_controls::ControlTagPrefab;
use amethyst_core::{ecs::prelude::Entity, math::RealField, Transform};
use amethyst_derive::PrefabData;
use amethyst_error::Error;
use amethyst_renderer::{
    CameraPrefab, GraphicsPrefab, InternalShape, LightPrefab, Mesh, MeshData, ObjFormat,
    TextureFormat,
};

use crate::removal::Removal;

/// Basic `Prefab` scene node, meant to be used for fast prototyping, and most likely replaced
/// for more complex scenarios.
///
/// ### Type parameters:
///
/// - `V`: Vertex format to use for generated `Mesh`es, must to be one of:
///     * `Vec<PosTex>`
///     * `Vec<PosNormTex>`
///     * `Vec<PosNormTangTex>`
///     * `ComboMeshCreator`
/// - `N`: RealField bound (f32 or f64).
/// - `R`: The type of id used by the Removal component.
/// - `M`: `Format` to use for loading `Mesh`es from file
#[derive(Deserialize, Serialize, PrefabData)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct BasicScenePrefab<V, N, R = (), M = ObjFormat>
where
    N: RealField,
    M: Format<Mesh> + Clone,
    M::Options: DeserializeOwned + Serialize + Clone,
    R: PartialEq + Debug + Clone + Send + Sync + 'static,
    V: From<InternalShape> + Into<MeshData>,
{
    graphics: Option<GraphicsPrefab<V, M, TextureFormat>>,
    transform: Option<Transform<N>>,
    light: Option<LightPrefab>,
    camera: Option<CameraPrefab>,
    control_tag: Option<ControlTagPrefab<N>>,
    removal: Option<Removal<R>>,
}

impl<V, N, R, M> Default for BasicScenePrefab<V, N, R, M>
where
    M: Format<Mesh> + Clone,
    M::Options: DeserializeOwned + Serialize + Clone,
    R: PartialEq + Debug + Clone + Send + Sync + 'static,
    V: From<InternalShape> + Into<MeshData>,
    N: RealField,
{
    fn default() -> Self {
        BasicScenePrefab {
            graphics: None,
            transform: None,
            light: None,
            camera: None,
            control_tag: None,
            removal: None,
        }
    }
}
