//! Displays a 2D GLTF scene

extern crate amethyst;
extern crate cgmath;

use amethyst::assets::{AssetStorage, Handle, Loader};
use amethyst::core::transform::{LocalTransform, Transform, TransformBundle};
use amethyst::gltf::{GltfSceneAsset, GltfSceneFormat, GltfSceneLoaderSystem, GltfSceneOptions};
use amethyst::prelude::*;
use amethyst::renderer::Config as DisplayConfig;
use amethyst::renderer::bundle::RenderBundle;
use amethyst::renderer::prelude::*;
use cgmath::Deg;

struct Example;

impl State for Example {
    fn on_start(&mut self, engine: &mut Engine) {
        let gltf_scene = load_gltf_mesh(
            engine,
            &*engine.world.read_resource(),
            "mesh/Box.gltf",
            GltfSceneOptions {
                generate_tex_coords: Some((0.1, 0.1)),
            },
        );

        engine
            .world
            .create_entity()
            .with(gltf_scene)
            .with(LocalTransform::default())
            .with(Transform::default())
            .build();

        println!("Create lights");
        engine
            .world
            .create_entity()
            .with(Light::from(PointLight {
                center: [6.0, 6.0, -6.0].into(),
                intensity: 6.0,
                color: [0.8, 0.0, 0.0].into(),
                ..PointLight::default()
            }))
            .build();

        engine
            .world
            .create_entity()
            .with(Light::from(PointLight {
                center: [0.0, 4.0, 4.0].into(),
                intensity: 5.0,
                color: [0.0, 0.3, 0.7].into(),
                ..PointLight::default()
            }))
            .build();

        println!("Put camera");

        engine.world.add_resource(Camera {
            eye: [-2.0, 2.0, 2.0].into(),
            // Make the arena fit perfectly in the view of the camera.
            proj: Projection::perspective(1024. / 768., Deg(60.)).into(),
            forward: [1.0, -1.0, -1.0].into(),
            right: [1.0, 0.0, 0.0].into(),
            up: [0., 1.0, 0.].into(),
        });

        engine
            .world
            .add_resource(AmbientColor(Rgba(0.2, 0.2, 0.2, 0.2)));
    }

    fn handle_event(&mut self, _: &mut Engine, event: Event) -> Trans {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } |
                WindowEvent::Closed => Trans::Quit,
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }
}

type DrawPass = pass::DrawShadedSeparate;

fn run() -> Result<(), amethyst::Error> {
    let path = format!(
        "{}/examples/06_material/resources/config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let resources_directory = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));
    let config = DisplayConfig::load(&path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawPass::new()),
    );

    let mut game = Application::build(resources_directory, Example)?
        .with_bundle(RenderBundle::new())?
        .with_bundle(TransformBundle::new())?
        .with_local(RenderSystem::build(pipe, Some(config))?)
        .with_resource(AssetStorage::<GltfSceneAsset>::new())
        .with(GltfSceneLoaderSystem::new(), "", &[])
        .register::<Handle<GltfSceneAsset>>()
        .build()?;

    Ok(game.run())
}

fn main() {
    if let Err(e) = run() {
        println!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn load_gltf_mesh(
    engine: &Engine,
    loader: &Loader,
    name: &str,
    options: GltfSceneOptions,
) -> Handle<GltfSceneAsset> {
    loader.load(
        name,
        GltfSceneFormat,
        options,
        (),
        &engine.world.read_resource(),
    )
}
