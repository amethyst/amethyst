//! Demonstrates how to use the fly camera

extern crate amethyst;

use amethyst::assets::{PrefabLoader, PrefabLoaderSystem, RonFormat};
use amethyst::controls::{FlyControlBundle, FlyControlTag, HideCursor};
use amethyst::core::transform::TransformBundle;
use amethyst::core::WithNamed;
use amethyst::ecs::join::Join;
use amethyst::input::{is_key_down, InputBundle};
use amethyst::prelude::*;
use amethyst::renderer::{Camera, DrawShaded, PosNormTex, VirtualKeyCode};
use amethyst::utils::application_root_dir;
use amethyst::utils::scene::BasicScenePrefab;
use amethyst::Error;

type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

#[derive(Default)]
struct ExampleState {
    is_grabbed: bool,
}

impl<'a, 'b> SimpleState<'a, 'b> for ExampleState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let prefab_handle = data.world.exec(|loader: PrefabLoader<MyPrefabData>| {
            loader.load("prefab/fly_camera.ron", RonFormat, (), ())
        });
        data.world
            .create_entity()
            .named("Fly Camera Scene")
            .with(prefab_handle)
            .build();
        self.is_grabbed = true;
    }

    fn handle_event(
        &mut self,
        data: StateData<GameData>,
        event: StateEvent<()>,
    ) -> SimpleTrans<'a, 'b> {
        if let StateEvent::Window(event) = event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                if self.is_grabbed {
                    self.ungrab_camera(data.world);
                } else {
                    self.grab_camera(data.world);
                }
            }
        }

        Trans::None
    }
}

impl ExampleState {
    fn ungrab_camera(&mut self, world: &mut World) {
        for (entity, _) in (&*world.entities(), &world.read_storage::<Camera>()).join() {
            world.write_storage::<FlyControlTag>().remove(entity);
        }
        world.write_resource::<HideCursor>().hide = false;
        self.is_grabbed = false;
    }

    fn grab_camera(&mut self, world: &mut World) {
        for (entity, _) in (&*world.entities(), &world.read_storage::<Camera>()).join() {
            world
                .write_storage::<FlyControlTag>()
                .insert(entity, Default::default())
                .expect("unable to attach FlyControlTag to camera");
        }
        world.write_resource::<HideCursor>().hide = true;
        self.is_grabbed = true;
    }
}

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir();

    let resources_directory = format!("{}/examples/assets", app_root);

    let display_config_path = format!(
        "{}/examples/fly_camera/resources/display_config.ron",
        app_root
    );

    let key_bindings_path = format!("{}/examples/fly_camera/resources/input.ron", app_root);

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(
            FlyControlBundle::<String, String>::new(
                Some(String::from("move_x")),
                Some(String::from("move_y")),
                Some(String::from("move_z")),
            ).with_sensitivity(0.1, 0.1),
        )?.with_bundle(TransformBundle::new().with_dep(&["fly_movement"]))?
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path)?,
        )?.with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), false)?;

    Application::build(resources_directory, ExampleState::default())?
        .build(game_data)?
        .run();

    Ok(())
}
