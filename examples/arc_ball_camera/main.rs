//! Demonstrates how to use the fly camera

#[macro_use]
extern crate amethyst;

use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystem, RonFormat},
    controls::{ArcBallControlBundle, ArcBallControlTag},
    core::{
        shrev::{EventChannel, ReaderId},
        transform::{Transform, TransformBundle},
    },
    ecs::prelude::{Join, Read, ReadStorage, Resources, System, SystemData, WriteStorage},
    input::{InputBundle, InputEvent, ScrollDirection},
    prelude::*,
    renderer::{DisplayConfig, DrawShaded, DrawSkybox, Pipeline, PosNormTex, RenderBundle, Stage},
    utils::{application_root_dir, scene::BasicScenePrefab},
    Error,
};
use std::hash::Hash;

type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

#[derive(State, Debug, Clone)]
enum State {
    Example,
}

struct ExampleState;

impl<S, E> StateCallback<S, E> for ExampleState {
    fn on_start(&mut self, world: &mut World) {
        let prefab_handle = world.exec(|loader: PrefabLoader<MyPrefabData>| {
            loader.load("prefab/arc_ball_camera.ron", RonFormat, (), ())
        });
        world.create_entity().with(prefab_handle).build();
    }
}

struct CameraDistanceSystem<AC>
where
    AC: Hash + Eq + 'static,
{
    event_reader: Option<ReaderId<InputEvent<AC>>>,
}

impl<AC> CameraDistanceSystem<AC>
where
    AC: Hash + Eq + 'static,
{
    pub fn new() -> Self {
        CameraDistanceSystem { event_reader: None }
    }
}

impl<'a, AC> System<'a> for CameraDistanceSystem<AC>
where
    AC: Hash + Eq + Clone + Send + Sync + 'static,
{
    type SystemData = (
        Read<'a, EventChannel<InputEvent<AC>>>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, ArcBallControlTag>,
    );

    fn run(&mut self, (events, transforms, mut tags): Self::SystemData) {
        for event in events.read(&mut self.event_reader.as_mut().unwrap()) {
            match *event {
                InputEvent::MouseWheelMoved(direction) => match direction {
                    ScrollDirection::ScrollUp => {
                        for (_, tag) in (&transforms, &mut tags).join() {
                            tag.distance *= 0.9;
                        }
                    }
                    ScrollDirection::ScrollDown => {
                        for (_, tag) in (&transforms, &mut tags).join() {
                            tag.distance *= 1.1;
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        self.event_reader = Some(
            res.fetch_mut::<EventChannel<InputEvent<AC>>>()
                .register_reader(),
        );
    }
}

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir();

    let resources_directory = format!("{}/examples/assets", app_root);

    let key_bindings_path = format!("{}/examples/arc_ball_camera/resources/input.ron", app_root);

    let render_bundle = {
        let display_config = {
            let path = format!(
                "{}/examples/arc_ball_camera/resources/display_config.ron",
                app_root
            );
            DisplayConfig::load(&path)
        };
        let pipe = Pipeline::build().with_stage(
            Stage::with_backbuffer()
                .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
                .with_pass(DrawShaded::<PosNormTex>::new())
                .with_pass(DrawSkybox::new()),
        );
        RenderBundle::new(pipe, Some(display_config))
    };

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new().with_dep(&[]))?
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path)?,
        )?.with_bundle(ArcBallControlBundle::<String, String>::new())?
        .with_bundle(render_bundle)?
        .with(
            CameraDistanceSystem::<String>::new(),
            "camera_distance_system",
            &["input_system"],
        );

    let mut game = Application::build(resources_directory)?
        .with_defaults()
        .with_state(State::Example, ExampleState)?
        .build(game_data)?;

    game.run();
    Ok(())
}
