//! Demostrates loading custom assets using the Amethyst engine.
// TODO: Add asset loader directory store for the meshes.

extern crate amethyst;
extern crate cgmath;
extern crate futures;
extern crate rayon;

use amethyst::{Application, Error, State, Trans};
use amethyst::assets::{AssetFuture, BoxedErr, Context, Format, Loader, NoError};
use amethyst::assets::formats::textures::{BmpFormat, PngFormat};
use amethyst::config::Config;
use amethyst::ecs::World;
use amethyst::ecs::input::InputHandler;
use amethyst::ecs::rendering::{Factory, MaterialComponent, MeshComponent, MeshContext,
                               TextureComponent, TextureContext};
use amethyst::ecs::transform::{LocalTransform, Transform};
use amethyst::prelude::*;
use amethyst::renderer::{Camera, Config as DisplayConfig, Rgba};
use amethyst::renderer::prelude::*;
use cgmath::{Deg, Euler, Quaternion};
use futures::Future;
use rayon::ThreadPool;

struct Custom;

impl Format for Custom {
    const EXTENSIONS: &'static [&'static str] = &["custom"];
    type Data = Vec<PosNormTex>;
    type Error = NoError;
    type Result = Result<Vec<PosNormTex>, NoError>;

    fn parse(&self, bytes: Vec<u8>, _: &ThreadPool) -> Self::Result {
        let data: String = String::from_utf8(bytes).unwrap();

        let trimmed: Vec<&str> = data.lines().filter(|line| line.len() >= 1).collect();

        let mut result = Vec::new();

        for line in trimmed {
            let nums: Vec<&str> = line.split_whitespace().collect();

            let vertex = [
                nums[0].parse::<f32>().unwrap(),
                nums[1].parse::<f32>().unwrap(),
                nums[2].parse::<f32>().unwrap(),
            ];

            let normal = [
                nums[3].parse::<f32>().unwrap(),
                nums[4].parse::<f32>().unwrap(),
                nums[5].parse::<f32>().unwrap(),
            ];

            result.push(PosNormTex {
                a_position: vertex,
                a_normal: normal,
                a_tex_coord: [0.0, 0.0],
            });
        }
        Ok(result)
    }
}


struct AssetsExample;

impl State for AssetsExample {
    fn on_start(&mut self, engine: &mut Engine) {
        use amethyst::assets::formats::meshes::ObjFormat;

        let input = InputHandler::new();
        engine.world.add_resource(input);
        engine.world.add_resource(0usize);

        initialise_camera(&mut engine.world.write_resource::<Camera>());
        initialise_lights(&mut engine.world);

        // Add teapot and lid to scene
        for mesh in vec!["lid", "teapot"].iter() {
            let mut trans = LocalTransform::default();
            trans.rotation = Quaternion::from(Euler::new(Deg(90.0), Deg(-90.0), Deg(0.0))).into();
            trans.translation = [5.0, 0.0, 5.0];
            let mesh = load_mesh(engine, mesh, ObjFormat);
            let mtl = make_material(engine, [0.0, 0.0, 1.0, 1.0]);
            engine
                .world
                .create_entity()
                .with(mesh)
                .with(mtl)
                .with(trans)
                .with(Transform::default())
                .build();
        }

        // Add custom cube object to scene
        let mesh = load_mesh(engine, "cuboid", Custom);
        let mtl = make_material(engine, [0.0, 0.0, 1.0, 1.0]);
        let mut trans = LocalTransform::default();
        trans.translation = [-5.0, 0.0, 0.0];
        trans.scale = [2.0, 2.0, 2.0];
        engine
            .world
            .create_entity()
            .with(mesh)
            .with(mtl)
            .with(trans)
            .with(Transform::default())
            .build();

        // Add cube to scene
        let mesh = load_mesh(engine, "cube", ObjFormat);
        let mtl = load_material(engine, "crate", PngFormat);
        let mut trans = LocalTransform::default();
        trans.translation = [5.0, 0.0, 0.0];
        trans.scale = [2.0, 2.0, 2.0];
        engine
            .world
            .create_entity()
            .with(mesh)
            .with(mtl)
            .with(trans)
            .with(Transform::default())
            .build();

        // Add sphere to scene
        let mesh = load_mesh(engine, "sphere", ObjFormat);
        let mtl = load_material(engine, "grass", BmpFormat);
        let mut trans = LocalTransform::default();
        trans.translation = [-5.0, 0.0, 7.5];
        trans.rotation = Quaternion::from(Euler::new(Deg(90.0), Deg(0.0), Deg(0.0))).into();
        trans.scale = [0.15, 0.15, 0.15];
        engine
            .world
            .create_entity()
            .with(mesh)
            .with(mtl)
            .with(trans)
            .with(Transform::default())
            .build();
    }

    fn handle_event(&mut self, _: &mut Engine, event: Event) -> Trans {
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::Closed |
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => {
                        // If the user pressed the escape key, or requested the window to be closed,
                        // quit the application.
                        Trans::Quit
                    }
                    _ => Trans::None,
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
        ::std::process::exit(1);
    }
}

/// Wrapper around the main, so we can return errors easily.
fn run() -> Result<(), Error> {
    use amethyst::assets::Directory;
    use amethyst::ecs::common::Errors;
    use amethyst::ecs::transform::{Child, Init, LocalTransform, TransformSystem};

    // Add our meshes directory to the asset loader.
    let resources_directory = format!(
        "{}/examples/05_assets/resources",
        env!("CARGO_MANIFEST_DIR")
    );

    let display_config_path = format!(
        "{}/examples/05_assets/resources/config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let display_config = DisplayConfig::load(display_config_path);
    let pipeline_builder = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_model_pass(pass::DrawShaded::<PosNormTex>::new()),
    );

    let mut game = Application::build(AssetsExample)
        .expect("Failed to build ApplicationBuilder for an unknown reason.")
        .register::<Child>()
        .register::<LocalTransform>()
        .register::<Init>()
        .with::<TransformSystem>(TransformSystem::new(), "transform_system", &[])
        .with_renderer(pipeline_builder, Some(display_config))?
        .add_store("resources", Directory::new(resources_directory))
        .add_resource(Errors::new())
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

    camera.forward = [0.0, 20.0, -5.0].into();
    camera.right = [1.0, 0.0, 0.0].into();
    camera.up = [0.0, 0.0, 1.0].into();
}

/// Adds lights to the scene.
fn initialise_lights(world: &mut World) {
    use amethyst::ecs::rendering::LightComponent;
    use amethyst::renderer::light::PointLight;

    let light = PointLight {
        center: [5.0, -20.0, 15.0].into(),
        intensity: 100.0,
        radius: 1.0,
        color: Rgba::white(),
        ..Default::default()
    };

    world
        .create_entity()
        .with(LightComponent(light.into()))
        .build();
}

fn load_material<F>(engine: &mut Engine, albedo: &str, format: F) -> AssetFuture<MaterialComponent>
where
    F: Format + 'static,
    F::Data: Into<<TextureContext as Context>::Data>,
{
    let future = {
        let factory = engine.world.read_resource::<Factory>();
        factory
            .create_material(MaterialBuilder::new())
            .map_err(BoxedErr::new)
    }.join({
        let loader = engine.world.read_resource::<Loader>();
        loader.load_from::<TextureComponent, _, _, _>(albedo, format, "resources")
    })
        .map(|(mut mtl, albedo)| {
            mtl.albedo = albedo.0.inner();
            MaterialComponent(mtl)
        });
    AssetFuture::from_future(future)
}

fn make_material(engine: &mut Engine, albedo: [f32; 4]) -> AssetFuture<MaterialComponent> {
    let future = {
        let factory = engine.world.read_resource::<Factory>();
        factory
            .create_material(
                MaterialBuilder::new().with_albedo(TextureBuilder::from_color_val(albedo)),
            )
            .map(MaterialComponent)
            .map_err(BoxedErr::new)
    };
    AssetFuture::from_future(future)
}

fn load_mesh<F>(engine: &mut Engine, name: &str, f: F) -> AssetFuture<MeshComponent>
where
    F: Format + 'static,
    F::Data: Into<<MeshContext as Context>::Data>,
{
    let future = {
        let loader = engine.world.read_resource::<Loader>();
        loader.load_from::<MeshComponent, _, _, _>(name, f, "resources")
    };
    future
}
