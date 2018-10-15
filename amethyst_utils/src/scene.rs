//! Provides utilities for building and describing scenes in your game.

use amethyst_assets::{Format, PrefabData, ProgressCounter};
use amethyst_controls::ControlTagPrefab;
use amethyst_core::specs::error::Error;
use amethyst_core::specs::prelude::Entity;
use amethyst_core::Transform;
use amethyst_renderer::{
    CameraPrefab, GraphicsPrefab, InternalShape, LightPrefab, Mesh, MeshData, ObjFormat,
    TextureFormat,
};
use removal::RemovalPrefab;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

/// Basic `Prefab` scene node, meant to be used for fast prototyping, and most likely replaced
/// for more complex scenarios.
///
/// ### Type parameters:
///
/// `V`: Vertex format to use for generated `Mesh`es, must to be one of:
///     * `Vec<PosTex>`
///     * `Vec<PosNormTex>`
///     * `Vec<PosNormTangTex>`
///     * `ComboMeshCreator`
/// `R`: The type of id used by the Removal component.
/// - `M`: `Format` to use for loading `Mesh`es from file
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct BasicScenePrefab<V, R = (), M = ObjFormat>
where
    M: Format<Mesh>,
    M::Options: DeserializeOwned + Serialize,
    R: PartialEq + Debug + Clone + Send + Sync + 'static,
{
    graphics: Option<GraphicsPrefab<V, M, TextureFormat>>,
    transform: Option<Transform>,
    light: Option<LightPrefab>,
    camera: Option<CameraPrefab>,
    control_tag: Option<ControlTagPrefab>,
    removal: Option<RemovalPrefab<R>>,
}

impl<V, R, M> Default for BasicScenePrefab<V, R, M>
where
    M: Format<Mesh>,
    M::Options: DeserializeOwned + Serialize,
    R: PartialEq + Debug + Clone + Send + Sync + 'static,
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

impl<'a, V, R, M> PrefabData<'a> for BasicScenePrefab<V, R, M>
where
    M: Format<Mesh> + Clone,
    M::Options: DeserializeOwned + Serialize + Clone,
    V: From<InternalShape> + Into<MeshData>,
    R: PartialEq + Debug + Clone + Send + Sync + 'static,
{
    type SystemData = (
        <GraphicsPrefab<V, M, TextureFormat> as PrefabData<'a>>::SystemData,
        <Transform as PrefabData<'a>>::SystemData,
        <LightPrefab as PrefabData<'a>>::SystemData,
        <CameraPrefab as PrefabData<'a>>::SystemData,
        <ControlTagPrefab as PrefabData<'a>>::SystemData,
        <RemovalPrefab<R> as PrefabData<'a>>::SystemData,
    );

    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), Error> {
        let (
            ref mut graphics,
            ref mut transforms,
            ref mut lights,
            ref mut cameras,
            ref mut tags,
            ref mut removals,
        ) = system_data;
        self.graphics.add_to_entity(entity, graphics, entities)?;
        self.transform.add_to_entity(entity, transforms, entities)?;
        self.light.add_to_entity(entity, lights, entities)?;
        self.camera.add_to_entity(entity, cameras, entities)?;
        self.control_tag.add_to_entity(entity, tags, entities)?;
        self.removal.add_to_entity(entity, removals, entities)?;
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let mut ret = false;
        let (
            ref mut graphics,
            ref mut transforms,
            ref mut lights,
            ref mut cameras,
            ref mut tags,
            ref mut removals,
        ) = system_data;
        if self.graphics.load_sub_assets(progress, graphics)? {
            ret = true;
        }
        if self.transform.load_sub_assets(progress, transforms)? {
            ret = true;
        }
        if self.light.load_sub_assets(progress, lights)? {
            ret = true;
        }
        if self.camera.load_sub_assets(progress, cameras)? {
            ret = true;
        }
        if self.control_tag.load_sub_assets(progress, tags)? {
            ret = true;
        }
        if self.removal.load_sub_assets(progress, removals)? {
            ret = true;
        }
        Ok(ret)
    }
}
