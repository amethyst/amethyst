//! Provides utilities for building and describing scenes in your game.

use std::fmt::Debug;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use amethyst_assets::{Format, PrefabData, ProgressCounter};
use amethyst_controls::ControlTagPrefab;
use amethyst_core::{ecs::prelude::Entity, Transform};
use amethyst_derive::PrefabData;
use amethyst_error::Error;
use amethyst_rendy::{
    camera::CameraPrefab,
    //GraphicsPrefab,
    shape::InternalShape, light::LightPrefab,
    types::Mesh,
    formats::{
        texture::ImageFormat,
        mesh::ObjFormat,
    },
    rendy::{
        hal::Backend,
        mesh::MeshBuilder,
    },
};

include!("placeholder.rs"); // GraphicsPrefab placeholder

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
/// - `M`: `Format` to use for loading `Mesh`es from file
#[derive(Deserialize, Serialize, PrefabData)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct BasicScenePrefab<B, V, R = (), M = ObjFormat>
where
    B: Backend,
    M: Format<Mesh<B>> + Clone,
    M::Options: DeserializeOwned + Serialize + Clone,
    R: PartialEq + Debug + Clone + Send + Sync + 'static,
    V: From<InternalShape> + Into<MeshBuilder<'static>>,
{
    #[serde(bound(deserialize = "GraphicsPrefab<B, V, M, ImageFormat>: Deserialize<'de>"))]
    graphics: Option<GraphicsPrefab<B, V, M, ImageFormat>>,
    transform: Option<Transform>,
    light: Option<LightPrefab>,
    camera: Option<CameraPrefab>,
    control_tag: Option<ControlTagPrefab>,
    removal: Option<Removal<R>>,
}

impl<B, V, R, M> Default for BasicScenePrefab<B, V, R, M>
where
    B: Backend,
    M: Format<Mesh<B>> + Clone,
    M::Options: DeserializeOwned + Serialize + Clone,
    R: PartialEq + Debug + Clone + Send + Sync + 'static,
    V: From<InternalShape> + Into<MeshBuilder<'static>>,
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
