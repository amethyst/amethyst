//! Provides utilities for building and describing scenes in your game.

use amethyst_assets::{PrefabData, ProgressCounter};
use amethyst_controls::ControlTagPrefab;
use amethyst_core::{ecs::prelude::Entity, Transform};
use amethyst_derive::PrefabData;
use amethyst_error::Error;
use amethyst_rendy::{
    camera::CameraPrefab, formats::GraphicsPrefab, light::LightPrefab, rendy::mesh::MeshBuilder,
    shape::FromShape,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

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
/// - `R`: The type of id used by the Removal component.
#[derive(Deserialize, Debug, Serialize, PrefabData)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct BasicScenePrefab<V, R = ()>
where
    R: PartialEq + Debug + Clone + Send + Sync + 'static,
    V: FromShape + Into<MeshBuilder<'static>>,
{
    graphics: Option<GraphicsPrefab<V>>,
    transform: Option<Transform>,
    light: Option<LightPrefab>,
    camera: Option<CameraPrefab>,
    control_tag: Option<ControlTagPrefab>,
    removal: Option<Removal<R>>,
}

impl<V, R> Default for BasicScenePrefab<V, R>
where
    R: PartialEq + Debug + Clone + Send + Sync + 'static,
    V: FromShape + Into<MeshBuilder<'static>>,
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
