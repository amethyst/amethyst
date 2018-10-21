//! Demonstrates how to use the fly camera

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
    renderer::{DrawShaded, PosNormTex},
    utils::{application_root_dir, scene::BasicScenePrefab},
    Error,
};
use std::hash::Hash;

type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

struct ExampleState;

impl<'a, 'b> SimpleState<'a, 'b> for ExampleState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let prefab_handle = data.world.exec(|loader: PrefabLoader<MyPrefabData>| {
            loader.load("prefab/arc_ball_camera.ron", RonFormat, (), ())
        });
        data.world.create_entity().with(prefab_handle).build();
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

    let display_config_path = format!(
        "{}/examples/arc_ball_camera/resources/display_config.ron",
        app_root
    );

    let key_bindings_path = format!("{}/examples/arc_ball_camera/resources/input.ron", app_root);

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new().with_dep(&[]))?
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path)?,
        )?.with_bundle(ArcBallControlBundle::<String, String>::new())?
        .with(
            CameraDistanceSystem::<String>::new(),
            "camera_distance_system",
            &["input_system"],
        ).with_basic_renderer(
            display_config_path,
            DrawShaded::<PosNormTex>::new(),
            false,
        )?;
    let mut game = Application::build(resources_directory, ExampleState)?.build(game_data)?;
    game.run();
    Ok(())
}
