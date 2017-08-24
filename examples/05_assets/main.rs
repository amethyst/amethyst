//! Demostrates loading custom assets using the Amethyst engine.
// TODO: Add asset loader directory store for the meshes.

extern crate amethyst;
extern crate cgmath;

use amethyst::{Application, Error, State, Trans};
use amethyst::config::Config;
use amethyst::ecs::World;
use amethyst::ecs::resources::input::InputHandler;
use amethyst::prelude::*;
use amethyst::renderer::{Camera, Config as DisplayConfig};
use amethyst::renderer::prelude::*;

struct AssetsExample;

impl State for AssetsExample {
    fn on_start(&mut self, engine: &mut Engine) {
        let input = InputHandler::new();
        engine.world.add_resource(input);

        initialise_camera(&mut engine.world.write_resource::<Camera>());
        initialise_lights(&mut engine.world);

        // TODO: Load colours, textures and meshes.
        // Meshes to load: teapot.obj, lid.obj, cube.obj, sphere.obj
        // Textures to load: crate.png, grass.bmp
    }

    fn handle_event(&mut self, engine: &mut Engine, event: Event) -> Trans {
        let mut input = engine.world.write_resource::<InputHandler>();
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::Closed |
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Escape), .. }, ..
                    } => {
                        // If the user pressed the escape key, or requested the window to be closed,
                        // quit the application.
                        Trans::Quit
                    }
                    _ => {
                        // If we didn't handle the event, forward it to the input handler.
                        input.update(&[event]);
                        Trans::None
                    }
                }
            }
            _ => Trans::None,
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Could not run the example!");
        eprintln!("{}", error);
    }
}

/// Wrapper around the main, so we can return errors easily.
fn run() -> Result<(), Error> {
    use amethyst::ecs::components::{Child, Init, LocalTransform, Transform};
    use amethyst::ecs::systems::TransformSystem;
    use std::env::set_var;

    // Set up an assets path by setting an environment variable. Note that
    // this would normally be done with something like this:
    //
    //     AMETHYST_ASSET_DIRS=/foo/bar cargo run
    let assets_path = format!(
        "{}/examples/05_assets/resources/textures",
        env!("CARGO_MANIFEST_DIR")
    );
    set_var("AMETHYST_ASSET_DIRS", assets_path);

    let path = format!(
        "{}/examples/05_assets/resources/config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let display_config = DisplayConfig::load(path);
    let pipeline_builder = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_model_pass(pass::DrawFlat::<PosNormTex>::new()),
    );

    let mut game = Application::build(AssetsExample)
        .register::<Child>()
        .register::<LocalTransform>()
        .register::<Transform>()
        .register::<Init>()
        .with::<TransformSystem>(TransformSystem::new(), "transform_system", &[])
        .with_renderer(pipeline_builder, display_config)?
        .build()?;

    game.run();
    Ok(())
}


/// Initialises the camera structure.
fn initialise_camera(camera: &mut Camera) {
    use cgmath::Deg;

    // TODO: Fix the aspect ratio.
    camera.proj = Projection::perspective(1.0, Deg(60.0)).into();
    camera.eye = [0.0, -20.0, 10.0].into();

    camera.forward = [0., 0., -1.0].into();
    camera.right = [1.0, 0.0, 0.0].into();
    camera.up = [0., 1.0, 0.].into();
}

/// Adds lights to the scene.
fn initialise_lights(world: &mut World) {
    use amethyst::ecs::components::LightComponent;
    use amethyst::renderer::light::PointLight;

    let light = PointLight {
        center: [5.0, -20.0, 15.0].into(),
        intensity: 10.0,
        radius: 100.0,
        ..Default::default()
    };

    world
        .create_entity()
        .with(LightComponent(light.into()))
        .build();
}
