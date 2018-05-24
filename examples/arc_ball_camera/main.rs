//! Demonstrates how to use the fly camera

extern crate amethyst;

use amethyst::assets::Loader;
use amethyst::config::Config;
use amethyst::controls::{ArcBallControlTag, ArcBallMovementSystem, FlyControlTag,
                         FreeRotationSystem, MouseCenterLockSystem, MouseFocusUpdateSystem};
use amethyst::core::cgmath::{Deg, Vector3};
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::ecs::prelude::{Entity, World};
use amethyst::input::InputBundle;
use amethyst::renderer::{AmbientColor, Camera, DisplayConfig, DrawShaded, ElementState, Event,
                         KeyboardInput, Material, MaterialDefaults, MeshHandle, ObjFormat,
                         Pipeline, PosNormTex, Projection, RenderBundle, Rgba, Stage,
                         VirtualKeyCode, WindowEvent};
use amethyst::{Application, Error, GameData, GameDataBuilder, State, StateData, Trans};

struct ExampleState;

impl<'a, 'b> State<GameData<'a, 'b>> for ExampleState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        let assets = load_assets(&world);

        // Add cube to scene
        let mut trans = Transform::default();
        trans.translation = Vector3::new(-5.0, 0.0, -10.0);
        world
            .create_entity()
            .with(assets.cube.clone())
            .with(assets.red.clone())
            .with(trans)
            .with(GlobalTransform::default())
            .build();

        let mut trans = Transform::default();
        trans.translation = Vector3::new(0.0, 0.0, -5.0);

        let entity = world
            .create_entity()
            .with(assets.cube.clone())
            .with(assets.red.clone())
            .with(trans)
            .with(GlobalTransform::default())
            .build();

        initialise_camera(world, entity);

        world.add_resource(AmbientColor(Rgba::from([0.1; 3])));
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode,
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match virtual_keycode {
                    Some(VirtualKeyCode::Escape) => return Trans::Quit,
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }
        Trans::None
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

fn main() {
    if let Err(error) = run() {
        eprintln!("Could not run the example!");
        eprintln!("{}", error);
        ::std::process::exit(1);
    }
}

/// Wrapper around the main, so we can return errors easily.
fn run() -> Result<(), Error> {
    let resources_directory = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

    let display_config_path = format!(
        "{}/examples/arc_ball_camera/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let display_config = DisplayConfig::load(display_config_path);

    let key_bindings_path = format!(
        "{}/examples/arc_ball_camera/resources/input.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let pipeline_builder = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawShaded::<PosNormTex>::new()),
    );
    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new().with_dep(&[]))?
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path),
        )?
        .with(MouseFocusUpdateSystem::new(), "mouse_focus", &[])
        .with(MouseCenterLockSystem, "mouse_lock", &["mouse_focus"])
        // This system will keep the camera focussing the target while conserving the orientation
        // of the camera and its distance to the target
        .with(ArcBallMovementSystem {}, "arc_ball_movement_system", &[])
        // This system manage the orientation of camera in accord to mouse input
        .with(
            FreeRotationSystem::<String, String>::new(1., 1.),
            "free_rotation_system",
            &[],
        )
        .with_bundle(RenderBundle::new(pipeline_builder, Some(display_config)))?;
    let mut game = Application::build(resources_directory, ExampleState)?
        .build(game_data)?;
    game.run();
    Ok(())
}

fn initialise_camera(world: &mut World, entity: Entity) {
    let local = Transform::default();

    world
        .create_entity()
        .with(Camera::from(Projection::perspective(1.3, Deg(60.0))))
        .with(local)
        .with(GlobalTransform::default())
        // Will allow the FreeRotationSystem to catch this entity
        .with(FlyControlTag)
        // Will allow the ArcBallMovementSystem to catch this entity
        .with(ArcBallControlTag {
            target: entity,
            distance: 10.,
        })
        .build();
}
