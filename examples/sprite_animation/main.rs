//! Demonstrates how to load and render sprites.
//!
//! Sprites are from <https://opengameart.org/content/bat-32x32>.

use amethyst::{
    animation::{
        get_animation_set, AnimationBundle, AnimationCommand, AnimationControlSet, AnimationSet,
        AnimationSetPrefab, EndControl,
    },
    assets::{PrefabData, PrefabLoader, PrefabLoaderSystem, ProgressCounter, RonFormat},
    core::transform::{Transform, TransformBundle},
    derive::PrefabData,
    ecs::{prelude::Entity, Entities, Join, ReadStorage, WriteStorage},
    error::Error,
    prelude::{Builder, World},
    renderer::{
        camera::Camera,
        plugins::{RenderFlat2D, RenderToWindow},
        sprite::{prefab::SpriteScenePrefab, SpriteRender},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
    window::ScreenDimensions,
    Application, GameData, GameDataBuilder, SimpleState, SimpleTrans, StateData, Trans,
};
use serde::{Deserialize, Serialize};

/// Animation ids used in a AnimationSet
#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone, Deserialize, Serialize)]
enum AnimationId {
    Fly,
}

/// Loading data for one entity
#[derive(Debug, Clone, Deserialize, PrefabData)]
struct MyPrefabData {
    /// Information for rendering a scene with sprites
    sprite_scene: SpriteScenePrefab,
    /// Аll animations that can be run on the entity
    animation_set: AnimationSetPrefab<AnimationId, SpriteRender>,
}

/// The main state
#[derive(Default)]
struct Example {
    /// A progress tracker to check that assets are loaded
    pub progress_counter: Option<ProgressCounter>,
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;
        // Crates new progress counter
        self.progress_counter = Some(Default::default());
        // Starts asset loading
        let bat_prefab = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load(
                "prefab/sprite_animation.ron",
                RonFormat,
                self.progress_counter.as_mut().unwrap(),
            )
        });
        let arrow_test_prefab = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load(
                "prefab/sprite_animation_test.ron",
                RonFormat,
                self.progress_counter.as_mut().unwrap(),
            )
        });
        // Creates new entities with components from MyPrefabData
        world.create_entity().with(bat_prefab).build();
        world.create_entity().with(arrow_test_prefab).build();
        // Creates a new camera
        initialise_camera(world);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // Checks if we are still loading data

        if let Some(ref progress_counter) = self.progress_counter {
            // Checks progress
            if progress_counter.is_complete() {
                let StateData { world, .. } = data;
                // Execute a pass similar to a system
                world.exec(
                    |(entities, animation_sets, mut control_sets): (
                        Entities,
                        ReadStorage<AnimationSet<AnimationId, SpriteRender>>,
                        WriteStorage<AnimationControlSet<AnimationId, SpriteRender>>,
                    )| {
                        // For each entity that has AnimationSet
                        for (entity, animation_set) in (&entities, &animation_sets).join() {
                            // Creates a new AnimationControlSet for the entity
                            let control_set = get_animation_set(&mut control_sets, entity).unwrap();
                            // Adds the `Fly` animation to AnimationControlSet and loops infinitely
                            control_set.add_animation(
                                AnimationId::Fly,
                                &animation_set.get(&AnimationId::Fly).unwrap(),
                                EndControl::Loop(None),
                                1.0,
                                AnimationCommand::Start,
                            );
                        }
                    },
                );
                // All data loaded
                self.progress_counter = None;
            }
        }
        Trans::None
    }
}

fn initialise_camera(world: &mut World) {
    let (width, height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
    };
    //println!("Init camera with dimensions: {}x{}", width, height);

    let mut camera_transform = Transform::default();
    camera_transform.set_translation_z(1.0);

    world
        .create_entity()
        .with(camera_transform)
        .with(Camera::standard_2d(width, height))
        .build();
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let assets_directory = app_root.join("examples/assets/");
    let display_config_path = app_root.join("examples/sprite_animation/config/display.ron");

    let game_data = GameDataBuilder::default()
        .with(
            PrefabLoaderSystem::<MyPrefabData>::default(),
            "scene_loader",
            &[],
        )
        .with_bundle(AnimationBundle::<AnimationId, SpriteRender>::new(
            "sprite_animation_control",
            "sprite_sampler_interpolation",
        ))?
        .with_bundle(
            TransformBundle::new()
                .with_dep(&["sprite_animation_control", "sprite_sampler_interpolation"]),
        )?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderFlat2D::default()),
        )?;

    let mut game = Application::new(assets_directory, Example::default(), game_data)?;
    game.run();

    Ok(())
}
