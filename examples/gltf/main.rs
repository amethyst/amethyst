//! Displays a 2D GLTF scene

extern crate amethyst;
extern crate amethyst_gltf;
#[macro_use]
extern crate serde;

use amethyst::animation::{
    get_animation_set, AnimationBundle, AnimationCommand, AnimationControlSet, AnimationSet,
    EndControl, VertexSkinningBundle,
};
use amethyst::assets::{
    AssetPrefab, Completion, Handle, Prefab, PrefabData, PrefabLoader, PrefabLoaderSystem,
    ProgressCounter, RonFormat,
};
use amethyst::controls::{ControlTagPrefab, FlyControlBundle};
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::ecs::error::Error;
use amethyst::ecs::prelude::{Entity, ReadStorage, Write, WriteStorage};
use amethyst::input::{is_close_requested, is_key_down};
use amethyst::prelude::*;
use amethyst::renderer::*;
use amethyst::utils::application_root_dir;
use amethyst::utils::tag::{Tag, TagFinder};
use amethyst_gltf::{GltfSceneAsset, GltfSceneFormat, GltfSceneLoaderSystem};

#[derive(Default)]
struct Example {
    entity: Option<Entity>,
    initialised: bool,
    progress: Option<ProgressCounter>,
}

#[derive(Clone, Serialize, Deserialize)]
struct AnimationMarker;

#[derive(Default)]
struct Scene {
    handle: Option<Handle<Prefab<ScenePrefabData>>>,
    animation_index: usize,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct ScenePrefabData {
    transform: Option<Transform>,
    gltf: Option<AssetPrefab<GltfSceneAsset, GltfSceneFormat>>,
    camera: Option<CameraPrefab>,
    light: Option<LightPrefab>,
    tag: Option<Tag<AnimationMarker>>,
    fly_tag: Option<ControlTagPrefab>,
}

impl<'a> PrefabData<'a> for ScenePrefabData {
    type SystemData = (
        <Option<Transform> as PrefabData<'a>>::SystemData,
        <Option<AssetPrefab<GltfSceneAsset, GltfSceneFormat>> as PrefabData<'a>>::SystemData,
        <Option<CameraPrefab> as PrefabData<'a>>::SystemData,
        <Option<LightPrefab> as PrefabData<'a>>::SystemData,
        <Option<Tag<AnimationMarker>> as PrefabData<'a>>::SystemData,
        <Option<ControlTagPrefab> as PrefabData<'a>>::SystemData,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), Error> {
        let (
            ref mut transforms,
            ref mut gltfs,
            ref mut cameras,
            ref mut lights,
            ref mut tags,
            ref mut control_tags,
        ) = system_data;
        self.transform.add_to_entity(entity, transforms, entities)?;
        self.gltf.add_to_entity(entity, gltfs, entities)?;
        self.camera.add_to_entity(entity, cameras, entities)?;
        self.light.add_to_entity(entity, lights, entities)?;
        self.tag.add_to_entity(entity, tags, entities)?;
        self.fly_tag.add_to_entity(entity, control_tags, entities)?;
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let (_, ref mut gltfs, _, _, _, _) = system_data;
        self.gltf.load_sub_assets(progress, gltfs)
    }
}

impl<'a, 'b> SimpleState<'a, 'b> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;

        self.progress = Some(ProgressCounter::default());

        world.exec(
            |(loader, mut scene): (PrefabLoader<ScenePrefabData>, Write<Scene>)| {
                scene.handle = Some(loader.load(
                    "prefab/puffy_scene.ron",
                    RonFormat,
                    (),
                    self.progress.as_mut().unwrap(),
                ));
            },
        );
    }

    fn handle_event(
        &mut self,
        data: StateData<GameData>,
        event: StateEvent<()>,
    ) -> SimpleTrans<'a, 'b> {
        let StateData { world, .. } = data;
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else if is_key_down(&event, VirtualKeyCode::Space) {
                toggle_or_cycle_animation(
                    self.entity,
                    &mut world.write_resource(),
                    &world.read_storage(),
                    &mut world.write_storage(),
                );
                Trans::None
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans<'a, 'b> {
        if !self.initialised {
            let remove = match self.progress.as_ref().map(|p| p.complete()) {
                None | Some(Completion::Loading) => false,

                Some(Completion::Complete) => {
                    let scene_handle = data
                        .world
                        .read_resource::<Scene>()
                        .handle
                        .as_ref()
                        .unwrap()
                        .clone();

                    data.world
                        .create_entity()
                        .with(scene_handle)
                        .with(GlobalTransform::default())
                        .build();

                    true
                }

                Some(Completion::Failed) => {
                    println!("Error: {:?}", self.progress.as_ref().unwrap().errors());
                    return Trans::Quit;
                }
            };
            if remove {
                self.progress = None;
            }
            if self.entity.is_none() {
                if let Some(entity) = data
                    .world
                    .exec(|finder: TagFinder<AnimationMarker>| finder.find())
                {
                    self.entity = Some(entity);
                    self.initialised = true;
                }
            }
        }
        Trans::None
    }
}

fn toggle_or_cycle_animation(
    entity: Option<Entity>,
    scene: &mut Scene,
    sets: &ReadStorage<AnimationSet<usize, Transform>>,
    controls: &mut WriteStorage<AnimationControlSet<usize, Transform>>,
) {
    if let Some((entity, Some(animations))) = entity.map(|entity| (entity, sets.get(entity))) {
        if animations.animations.len() > scene.animation_index {
            let animation = animations.animations.get(&scene.animation_index).unwrap();
            let mut set = get_animation_set::<usize, Transform>(controls, entity).unwrap();
            if set.has_animation(scene.animation_index) {
                set.toggle(scene.animation_index);
            } else {
                println!("Running animation {}", scene.animation_index);
                set.add_animation(
                    scene.animation_index,
                    animation,
                    EndControl::Normal,
                    1.0,
                    AnimationCommand::Start,
                );
            }
            scene.animation_index += 1;
            if scene.animation_index >= animations.animations.len() {
                scene.animation_index = 0;
            }
        }
    }
}

fn main() -> Result<(), amethyst::Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir();

    let path = format!("{}/examples/gltf/resources/display_config.ron", app_root);

    let resources_directory = format!("{}/examples/assets/", app_root);

    let game_data = GameDataBuilder::default()
        .with(
            PrefabLoaderSystem::<ScenePrefabData>::default(),
            "scene_loader",
            &[],
        ).with(
            GltfSceneLoaderSystem::default(),
            "gltf_loader",
            &["scene_loader"], // This is important so that entity instantiation is performed in a single frame.
        ).with_basic_renderer(
            path,
            DrawPbmSeparate::new()
                .with_vertex_skinning()
                .with_transparency(ColorMask::all(), ALPHA, Some(DepthMode::LessEqualWrite)),
            false,
        )?.with_bundle(
            AnimationBundle::<usize, Transform>::new("animation_control", "sampler_interpolation")
                .with_dep(&["gltf_loader"]),
        )?.with_bundle(
            FlyControlBundle::<String, String>::new(None, None, None)
                .with_sensitivity(0.1, 0.1)
                .with_speed(5.),
        )?.with_bundle(TransformBundle::new().with_dep(&[
            "animation_control",
            "sampler_interpolation",
            "fly_movement",
        ]))?.with_bundle(VertexSkinningBundle::new().with_dep(&[
            "transform_system",
            "animation_control",
            "sampler_interpolation",
        ]))?;

    let mut game = Application::build(resources_directory, Example::default())?.build(game_data)?;
    game.run();
    Ok(())
}
