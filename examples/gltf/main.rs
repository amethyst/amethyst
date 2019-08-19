//! Displays a 2D GLTF scene

use amethyst::{
    animation::{
        get_animation_set, AnimationBundle, AnimationCommand, AnimationControlSet, AnimationSet,
        EndControl, VertexSkinningBundle,
    },
    assets::{
        AssetPrefab, Completion, Handle, Prefab, PrefabData, PrefabLoader, PrefabLoaderSystemDesc,
        ProgressCounter, RonFormat,
    },
    controls::{ControlTagPrefab, FlyControlBundle},
    core::transform::{Transform, TransformBundle},
    derive::PrefabData,
    ecs::{Entity, ReadStorage, Write, WriteStorage},
    input::{is_close_requested, is_key_down, StringBindings, VirtualKeyCode},
    prelude::*,
    renderer::{
        camera::CameraPrefab,
        light::LightPrefab,
        plugins::{RenderPbr3D, RenderSkybox, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::{
        application_root_dir,
        auto_fov::{AutoFov, AutoFovSystem},
        tag::{Tag, TagFinder},
    },
    Error,
};
use amethyst_gltf::{GltfSceneAsset, GltfSceneFormat, GltfSceneLoaderSystemDesc};

use serde::{Deserialize, Serialize};

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

#[derive(Default, Deserialize, Serialize, PrefabData)]
#[serde(default)]
struct ScenePrefabData {
    transform: Option<Transform>,
    gltf: Option<AssetPrefab<GltfSceneAsset, GltfSceneFormat>>,
    camera: Option<CameraPrefab>,
    auto_fov: Option<AutoFov>,
    light: Option<LightPrefab>,
    tag: Option<Tag<AnimationMarker>>,
    fly_tag: Option<ControlTagPrefab>,
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        self.progress = Some(ProgressCounter::default());

        world.exec(
            |(loader, mut scene): (PrefabLoader<'_, ScenePrefabData>, Write<'_, Scene>)| {
                scene.handle = Some(loader.load(
                    "prefab/puffy_scene.ron",
                    RonFormat,
                    self.progress.as_mut().unwrap(),
                ));
            },
        );
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
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

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
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

                    data.world.create_entity().with(scene_handle).build();

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
                    .exec(|finder: TagFinder<'_, AnimationMarker>| finder.find())
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
    sets: &ReadStorage<'_, AnimationSet<usize, Transform>>,
    controls: &mut WriteStorage<'_, AnimationControlSet<usize, Transform>>,
) {
    if let Some((entity, Some(animations))) = entity.map(|entity| (entity, sets.get(entity))) {
        if animations.animations.len() > scene.animation_index {
            let animation = animations.animations.get(&scene.animation_index).unwrap();
            let set = get_animation_set::<usize, Transform>(controls, entity).unwrap();
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

    let app_root = application_root_dir()?;

    let display_config_path = app_root.join("examples/gltf/config/display.ron");
    let assets_dir = app_root.join("examples/assets/");

    let game_data = GameDataBuilder::default()
        .with(AutoFovSystem::default(), "auto_fov", &[])
        .with_system_desc(
            PrefabLoaderSystemDesc::<ScenePrefabData>::default(),
            "scene_loader",
            &[],
        )
        .with_system_desc(
            GltfSceneLoaderSystemDesc::default(),
            "gltf_loader",
            &["scene_loader"], // This is important so that entity instantiation is performed in a single frame.
        )
        .with_bundle(
            AnimationBundle::<usize, Transform>::new("animation_control", "sampler_interpolation")
                .with_dep(&["gltf_loader"]),
        )?
        .with_bundle(
            FlyControlBundle::<StringBindings>::new(None, None, None)
                .with_sensitivity(0.1, 0.1)
                .with_speed(5.),
        )?
        .with_bundle(TransformBundle::new().with_dep(&[
            "animation_control",
            "sampler_interpolation",
            "fly_movement",
        ]))?
        .with_bundle(VertexSkinningBundle::new().with_dep(&[
            "transform_system",
            "animation_control",
            "sampler_interpolation",
        ]))?
        // `VisibilitySortingSystem` (part of `RenderPbr3D`) should depend on:
        // &["fly_movement", "transform_system", "auto_fov"]
        //
        // There is currently no way to pass the dependencies to that system. However, since that
        // system is thread local as part of rendering, it runs after all of the systems anyway.
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderToWindow::from_config_path(display_config_path))
                .with_plugin(RenderPbr3D::default().with_skinning())
                .with_plugin(RenderSkybox::default()),
        )?;

    let mut game = Application::build(assets_dir, Example::default())?.build(game_data)?;
    game.run();
    Ok(())
}
