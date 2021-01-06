//! Demonstrates how to load and render sprites.
//!
//! Sprites are from <https://opengameart.org/content/bat-32x32>.

use amethyst::{
    animation::{
        get_animation_set, Animation, AnimationBundle, AnimationCommand, AnimationSet, EndControl,
        InterpolationFunction, Sampler, SpriteRenderPrimitive,
    },
    assets::{DefaultLoader, Handle, Loader, LoaderBundle, ProcessingQueue, ProgressCounter},
    core::transform::{Transform, TransformBundle},
    ecs::*,
    renderer::{
        camera::Camera,
        plugins::{RenderFlat2D, RenderToWindow},
        rendy::hal::command::ClearColor,
        sprite::SpriteRender,
        types::DefaultBackend,
        RenderingBundle, SpriteSheet,
    },
    utils::application_root_dir,
    window::ScreenDimensions,
    Application, GameData, SimpleState, SimpleTrans, StateData, Trans,
};
use serde::{Deserialize, Serialize};

/// Animation ids used in a AnimationSet
#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone, Deserialize, Serialize)]
enum AnimationId {
    Fly,
}

/// Loading data for one entity
// #[derive(Debug, Clone, Deserialize)]
// struct MyPrefabData {
//     /// Information for rendering a scene with sprites
//     sprite_scene: SpriteScenePrefab,
//     /// Аll animations that can be run on the entity
//     animation_set: AnimationSetPrefab<AnimationId, SpriteRender>,
// }

/// The main state
#[derive(Default)]
struct Example {
    /// A progress tracker to check that assets are loaded
    pub progress_counter: Option<ProgressCounter>,
    pub animated: bool,
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData {
            world, resources, ..
        } = data;

        self.progress_counter = Some(Default::default());

        {
            let loader = resources
                .get_mut::<DefaultLoader>()
                .expect("Missing loader");

            let texture = loader.load("texture/bat.32x32.png");
            let sprites = loader.load("sprites/bat.ron");

            let sheet: Handle<SpriteSheet> = loader.load_from_data(
                SpriteSheet { texture, sprites },
                self.progress_counter.as_mut().unwrap(),
                &resources.get().expect("processing queue for SpriteSheet"),
            );

            let anims = loader.load_from_data(
                Sampler::<SpriteRenderPrimitive> {
                    input: vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0],
                    output: vec![
                        SpriteRenderPrimitive::SpriteIndex(4),
                        SpriteRenderPrimitive::SpriteIndex(3),
                        SpriteRenderPrimitive::SpriteIndex(2),
                        SpriteRenderPrimitive::SpriteIndex(1),
                        SpriteRenderPrimitive::SpriteIndex(0),
                    ],
                    function: InterpolationFunction::Step,
                },
                self.progress_counter.as_mut().unwrap(),
                &resources.get().expect("queue for Sampler"),
            );

            let anim_handle: Handle<Animation<SpriteRender>> = loader.load_from_data(
                Animation::<SpriteRender>::new_single(
                    0,
                    amethyst::animation::SpriteRenderChannel::SpriteIndex,
                    anims,
                ),
                self.progress_counter.as_mut().unwrap(),
                &resources.get().expect("queue for Animation"),
            );

            let mut anim_set = AnimationSet::new();
            anim_set.insert(AnimationId::Fly, anim_handle);

            world.push((SpriteRender::new(sheet, 0), Transform::default(), anim_set));
        }

        initialise_camera(world, resources);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        let mut query = <(Entity, Read<AnimationSet<AnimationId, SpriteRender>>)>::query();
        let mut buffer = CommandBuffer::new(data.world);

        if let Some(ref progress_counter) = self.progress_counter {
            // Checks progress
            if progress_counter.is_complete() {
                let (query_world, mut subworld) = data.world.split_for_query(&query);
                for (entity, animation_set) in query.iter(&query_world) {
                    // Creates a new AnimationControlSet for the entity
                    if let Some(control_set) =
                        get_animation_set(&mut subworld, &mut buffer, *entity)
                    {
                        if control_set.is_empty() && !self.animated {
                            // Adds the `Fly` animation to AnimationControlSet and loops infinitely
                            control_set.add_animation(
                                AnimationId::Fly,
                                &animation_set.get(&AnimationId::Fly).unwrap(),
                                EndControl::Loop(None),
                                1.0,
                                AnimationCommand::Start,
                            );
                            self.animated = true;
                        }
                    }
                }
            }
        }
        buffer.flush(data.world);

        Trans::None
    }
}

fn initialise_camera(world: &mut World, resources: &mut Resources) {
    let (width, height) = {
        let dim = resources.get::<ScreenDimensions>().unwrap();
        (dim.width(), dim.height())
    };

    let mut camera_transform = Transform::default();
    camera_transform.set_translation_z(1.0);

    world.push((camera_transform, Camera::standard_2d(width, height)));
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let assets_dir = app_root.join("examples/sprite_animation/assets/");
    let display_config_path = app_root.join("examples/sprite_animation/config/display.ron");

    let mut game_data = DispatcherBuilder::default();
    game_data
        .add_bundle(LoaderBundle)
        .add_bundle(AnimationBundle::<AnimationId, SpriteRender>::default())
        .add_bundle(TransformBundle::default())
        .flush() // to ensure that animation changes are flushed before rendering
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                        float32: [0.34, 0.36, 0.52, 1.0],
                    }),
                )
                .with_plugin(RenderFlat2D::default()),
        );

    let game = Application::build(assets_dir, Example::default())?.build(game_data)?;
    game.run();

    Ok(())
}
