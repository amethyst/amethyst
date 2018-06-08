//! Demonstrates how to use the fly camera

extern crate amethyst;

use amethyst::assets::Loader;
use amethyst::controls::{FlyControlBundle, FlyControlTag};
use amethyst::core::cgmath::{Deg, Vector3};
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::ecs::prelude::World;
use amethyst::input::{is_close_requested, is_key, InputBundle};
use amethyst::renderer::{AmbientColor, Camera, DrawShaded, Event, Material, MaterialDefaults,
                         MeshHandle, ObjFormat, PosNormTex, Projection, Rgba, VirtualKeyCode};
use amethyst::{Application, Error, GameData, GameDataBuilder, State, StateData, Trans};

struct ExampleState;

impl<'a, 'b> State<GameData<'a, 'b>> for ExampleState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        initialise_camera(world);

        let assets = load_assets(&world);

        // Add cube to scene
        let mut trans = Transform::default();
        trans.translation = Vector3::new(0.0, 0.0, -5.0);
        world
            .create_entity()
            .with(assets.cube.clone())
            .with(assets.red.clone())
            .with(trans)
            .with(GlobalTransform::default())
            .build();

        world.add_resource(AmbientColor(Rgba::from([0.1; 3])));
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        if is_close_requested(&event) || is_key(&event, VirtualKeyCode::Escape) {
            Trans::Quit
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(&data.world);
        Trans::None
    }
}

struct Assets {
    cube: MeshHandle,
    red: Material,
}

fn load_assets(world: &World) -> Assets {
    let mesh_storage = world.read_resource();
    let tex_storage = world.read_resource();
    let mat_defaults = world.read_resource::<MaterialDefaults>();
    let loader = world.read_resource::<Loader>();

    let red = loader.load_from_data([1.0, 0.0, 0.0, 1.0].into(), (), &tex_storage);
    let red = Material {
        albedo: red,
        ..mat_defaults.0.clone()
    };

    let cube = loader.load("mesh/cube.obj", ObjFormat, (), (), &mesh_storage);

    Assets { cube, red }
}

fn main() -> Result<(), Error> {
    let resources_directory = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

    let display_config_path = format!(
        "{}/examples/fly_camera/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let key_bindings_path = format!(
        "{}/examples/fly_camera/resources/input.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let game_data = GameDataBuilder::default()
        .with_bundle(
            FlyControlBundle::<String, String>::new(
                Some(String::from("move_x")),
                Some(String::from("move_y")),
                Some(String::from("move_z")),
            ).with_sensitivity(0.1, 0.1),
        )?
        .with_bundle(TransformBundle::new().with_dep(&["fly_movement"]))?
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path),
        )?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), false)?;
    let mut game = Application::build(resources_directory, ExampleState)?.build(game_data)?;
    game.run();
    Ok(())
}

fn initialise_camera(world: &mut World) {
    let local = Transform::default();

    world
        .create_entity()
        .with(Camera::from(Projection::perspective(1.3, Deg(60.0))))
        .with(local)
        .with(GlobalTransform::default())
        .with(FlyControlTag)
        .build();
}
