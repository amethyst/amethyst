use amethyst_assets::{Format, PrefabData, ProgressCounter};
use amethyst_controls::ControlTagPrefab;
use amethyst_core::specs::error::Error;
use amethyst_core::specs::prelude::Entity;
use amethyst_core::Transform;
use amethyst_renderer::{
    CameraPrefab, GraphicsPrefab, InternalShape, LightPrefab, Mesh, MeshData, ObjFormat,
    TextureFormat,
};
use serde::de::DeserializeOwned;
use serde::Serialize;

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
/// - `M`: `Format` to use for loading `Mesh`es from file
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct BasicScenePrefab<V, M = ObjFormat>
where
    M: Format<Mesh>,
    M::Options: DeserializeOwned + Serialize,
{
    graphics: Option<GraphicsPrefab<V, M, TextureFormat>>,
    transform: Option<Transform>,
    light: Option<LightPrefab>,
    camera: Option<CameraPrefab>,
    control_tag: Option<ControlTagPrefab>,
}

impl<V, M> Default for BasicScenePrefab<V, M>
where
    M: Format<Mesh>,
    M::Options: DeserializeOwned + Serialize,
{
    fn default() -> Self {
        BasicScenePrefab {
            graphics: None,
            transform: None,
            light: None,
            camera: None,
            control_tag: None,
        }
    }
}

impl<'a, V, M> PrefabData<'a> for BasicScenePrefab<V, M>
where
    M: Format<Mesh> + Clone,
    M::Options: DeserializeOwned + Serialize + Clone,
    V: From<InternalShape> + Into<MeshData>,
{
    type SystemData = (
        <GraphicsPrefab<V, M, TextureFormat> as PrefabData<'a>>::SystemData,
        <Transform as PrefabData<'a>>::SystemData,
        <LightPrefab as PrefabData<'a>>::SystemData,
        <CameraPrefab as PrefabData<'a>>::SystemData,
        <ControlTagPrefab as PrefabData<'a>>::SystemData,
    );

    type Result = ();

    fn load_prefab(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), Error> {
        let (ref mut graphics, ref mut transforms, ref mut lights, ref mut cameras, ref mut tags) =
            system_data;
        self.graphics.load_prefab(entity, graphics, entities)?;
        self.transform.load_prefab(entity, transforms, entities)?;
        self.light.load_prefab(entity, lights, entities)?;
        self.camera.load_prefab(entity, cameras, entities)?;
        self.control_tag.load_prefab(entity, tags, entities)?;
        Ok(())
    }

    fn trigger_sub_loading(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let mut ret = false;
        let (ref mut graphics, ref mut transforms, ref mut lights, ref mut cameras, ref mut tags) =
            system_data;
        if self.graphics.trigger_sub_loading(progress, graphics)? {
            ret = true;
        }
        if self.transform.trigger_sub_loading(progress, transforms)? {
            ret = true;
        }
        if self.light.trigger_sub_loading(progress, lights)? {
            ret = true;
        }
        if self.camera.trigger_sub_loading(progress, cameras)? {
            ret = true;
        }
        if self.control_tag.trigger_sub_loading(progress, tags)? {
            ret = true;
        }
        Ok(ret)
    }
}
