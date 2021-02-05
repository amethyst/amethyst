//! Demonstrates how to load and render sprites.
//!
//! Sprites are from <https://opengameart.org/content/bat-32x32>.

use amethyst::{
    animation::{
        get_animation_set, Animation, AnimationBundle, AnimationCommand, AnimationSet, EndControl,
        InterpolationFunction, Sampler, SpriteRenderPrimitive,
    },
    assets::{DefaultLoader, Handle, Loader, LoaderBundle, ProgressCounter},
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

/// a list of possible animations to play for a Bat sprite
#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone, Deserialize, Serialize)]
enum BatAnimations {
    Fly,
}

#[derive(Default)]
struct ExampleState {
    pub progress_counter: Option<ProgressCounter>,
}

impl SimpleState for ExampleState {
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
            anim_set.insert(BatAnimations::Fly, anim_handle);

            world.push((SpriteRender::new(sheet, 0), Transform::default(), anim_set));
        }

        initialize_camera(world, resources);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        let mut query = <(Entity, Read<AnimationSet<BatAnimations, SpriteRender>>)>::query();
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
                        if control_set.is_empty() {
                            // Adds the `Fly` animation to AnimationControlSet and loops infinitely
                            control_set.add_animation(
                                BatAnimations::Fly,
                                &animation_set.get(&BatAnimations::Fly).unwrap(),
                                EndControl::Loop(None),
                                1.0,
                                AnimationCommand::Start,
                            );
                            self.progress_counter = None;
                        }
                    }
                }
            }
        }
        buffer.flush(data.world);

        Trans::None
    }
}

fn initialize_camera(world: &mut World, resources: &mut Resources) {
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
    let assets_dir = app_root.join("assets/");
    let display_config_path = app_root.join("config/display.ron");

    let mut game_data = DispatcherBuilder::default();
    game_data
        .add_bundle(LoaderBundle)
        .add_bundle(AnimationBundle::<BatAnimations, SpriteRender>::default())
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

    let game = Application::build(assets_dir, ExampleState::default())?.build(game_data)?;
    game.run();

    Ok(())
}
