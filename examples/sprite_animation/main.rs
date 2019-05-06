//! Demonstrates how to load and render sprites.
//!
//! Sprites are from <https://opengameart.org/content/bat-32x32>.

use amethyst::{
    animation::{
        get_animation_set, AnimationBundle, AnimationCommand, AnimationControlSet, AnimationSet,
        AnimationSetPrefab, EndControl,
    },
    assets::{PrefabData, PrefabLoader, PrefabLoaderSystem, ProgressCounter, RonFormat},
    config::Config,
    core::transform::{Transform, TransformBundle},
    derive::PrefabData,
    ecs::{prelude::Entity, Entities, Join, ReadStorage, WriteStorage},
    error::Error,
    prelude::{Builder, World},
    renderer::{
        Camera, DisplayConfig, DrawFlat2D, Pipeline, Projection, RenderBundle, ScreenDimensions,
        SpriteRender, SpriteScenePrefab, Stage,
    },
    utils::application_root_dir,
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
    sprite_scene: SpriteScenePrefab<f32>,
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
        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load(
                "prefab/sprite_animation.ron",
                RonFormat,
                (),
                self.progress_counter.as_mut().unwrap(),
            )
        });
        // Creates new entities with components from MyPrefabData
        world.create_entity().with(prefab_handle).build();
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

    let mut camera_transform = Transform::<f32>::default();
    camera_transform.set_translation_z(1.0);

    world
        .create_entity()
        .with(camera_transform)
        .with(Camera::from(Projection::orthographic(
            0.0, width, 0.0, height,
        )))
        .build();
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let assets_directory = app_root.join("examples/assets/");
    let display_conf_path = app_root.join("examples/sprite_animation/resources/display_config.ron");
    let display_config = DisplayConfig::load(display_conf_path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawFlat2D::<f32>::new()),
    );

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::<f32>::new())?
        .with_bundle(AnimationBundle::<AnimationId, SpriteRender>::new(
            "animation_control_system",
            "sampler_interpolation_system",
        ))?
        .with_bundle(
            RenderBundle::<_, _, f32>::new(pipe, Some(display_config))
                .with_sprite_sheet_processor(),
        )?;

    let mut game = Application::new(assets_directory, Example::default(), game_data)?;
    game.run();

    Ok(())
}
