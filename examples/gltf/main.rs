//! Displays a 2D GLTF scene

extern crate amethyst;
extern crate amethyst_gltf;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

use amethyst::animation::{get_animation_set, AnimationBundle, AnimationCommand,
                          AnimationControlSet, AnimationSet, EndControl, VertexSkinningBundle};
use amethyst::assets::{AssetPrefab, PrefabLoader, PrefabLoaderSystem, RonFormat};
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::ecs::prelude::{Entity, ReadStorage, WriteStorage};
use amethyst::prelude::*;
use amethyst::renderer::*;
use amethyst::utils::tag::{Tag, TagFinder};
use amethyst_gltf::{GltfSceneAsset, GltfSceneFormat, GltfSceneLoaderSystem};

#[derive(Default)]
struct Example {
    entity: Option<Entity>,
}

#[derive(Clone, Serialize, Deserialize)]
struct AnimationMarker;

#[derive(Default)]
struct Scene {
    animation_index: usize,
}

type ScenePrefabData = (
    Option<Transform>,
    Option<AssetPrefab<GltfSceneAsset, GltfSceneFormat>>,
    Option<CameraPrefab>,
    Option<LightPrefab>,
    Option<Tag<AnimationMarker>>,
);

impl<'a, 'b> State<GameData<'a, 'b>> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;

        let scene = world.exec(|loader: PrefabLoader<ScenePrefabData>| {
            loader.load("gltf_scene.ron", RonFormat, (), ())
        });

        world
            .create_entity()
            .with(scene)
            .with(GlobalTransform::default())
            .build();

        world.add_resource(Scene::default());
    }

    fn handle_event(&mut self, data: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        let StateData { world, .. } = data;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => Trans::Quit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Space),
                            state: ElementState::Released,
                            ..
                        },
                    ..
                } => {
                    toggle_or_cycle_animation(
                        self.entity,
                        &mut world.write_resource(),
                        &world.read_storage(),
                        &mut world.write_storage(),
                    );
                    Trans::None
                }
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        if self.entity.is_none() {
            if let Some(entity) = data.world
                .exec(|finder: TagFinder<AnimationMarker>| finder.find())
            {
                self.entity = Some(entity);
            }
        }
        data.data.update(&data.world);
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
            let mut set = get_animation_set::<usize, Transform>(controls, entity);
            if set.has_animation(scene.animation_index) {
                set.toggle(scene.animation_index);
            } else {
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

fn run() -> Result<(), amethyst::Error> {
    let path = format!(
        "{}/examples/gltf/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let resources_directory = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));
    let config = DisplayConfig::load(&path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawPbmSeparate::new()),
    );

    let game_data = GameDataBuilder::default()
        .with(
            PrefabLoaderSystem::<ScenePrefabData>::default(),
            "scene_loader",
            &[],
        )
        .with(
            GltfSceneLoaderSystem::default(),
            "loader_system",
            &["scene_loader"],
        )
        .with_bundle(
            AnimationBundle::<usize, Transform>::new(
                "animation_control_system",
                "sampler_interpolation_system",
            ).with_dep(&["loader_system"]),
        )?
        .with_bundle(
            TransformBundle::new()
                .with_dep(&["animation_control_system", "sampler_interpolation_system"]),
        )?
        .with_bundle(VertexSkinningBundle::new().with_dep(&[
            "transform_system",
            "animation_control_system",
            "sampler_interpolation_system",
        ]))?
        .with_bundle(
            RenderBundle::new(pipe, Some(config)).with_visibility_sorting(&["transform_system"]),
        )?;

    let mut game = Application::new(resources_directory, Example::default(), game_data)?;
    game.run();
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        error!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}
