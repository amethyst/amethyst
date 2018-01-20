//! Displays a 2D GLTF scene

extern crate amethyst;
extern crate amethyst_animation;
extern crate amethyst_gltf;
#[macro_use]
extern crate log;

use amethyst::assets::{AssetStorage, Handle, Loader};
use amethyst::core::cgmath::{Deg, Quaternion, Rotation3, Vector3};
use amethyst::core::transform::{LocalTransform, Transform, TransformBundle};
use amethyst::ecs::Entity;
use amethyst::prelude::*;
use amethyst::renderer::*;
use amethyst_animation::{toggle_animation, AnimationBundle, AnimationSet, EndControl,
                         VertexSkinningBundle};
use amethyst_gltf::{GltfSceneAsset, GltfSceneFormat, GltfSceneLoaderSystem, GltfSceneOptions};

struct Example;

struct Scene {
    entity: Entity,
    animation_index: usize,
}

impl State for Example {
    fn on_start(&mut self, world: &mut World) {
        let gltf_scene = load_gltf_mesh(
            &world,
            &*world.read_resource(),
            "mesh/Monster.gltf",
            GltfSceneOptions {
                generate_tex_coords: Some((0.1, 0.1)),
                load_animations: true,
                flip_v_coord: true,
                move_to_origin: true,
            },
        );

        let entity = world
            .create_entity()
            .with(gltf_scene)
            .with(Transform::default())
            .build();

        world.add_resource(Scene {
            entity,
            animation_index: 0,
        });

        info!("Create lights");
        world
            .create_entity()
            .with(Light::from(PointLight {
                center: [6.0, 6.0, -6.0].into(),
                intensity: 6.0,
                color: [0.8, 0.0, 0.0].into(),
                ..PointLight::default()
            }))
            .build();

        world
            .create_entity()
            .with(Light::from(PointLight {
                center: [0.0, 4.0, 4.0].into(),
                intensity: 5.0,
                color: [0.0, 0.3, 0.7].into(),
                ..PointLight::default()
            }))
            .build();

        info!("Put camera");

        let mut camera_transform = LocalTransform::default();
        camera_transform.translation = Vector3::new(100.0, 20.0, 0.0);
        camera_transform.rotation = Quaternion::from_angle_y(Deg(90.));
        world
            .create_entity()
            .with(Camera::from(Projection::perspective(
                1024. / 768.,
                Deg(60.),
            )))
            .with(Transform::default())
            .with(camera_transform)
            .build();

        world.add_resource(AmbientColor(Rgba(0.2, 0.2, 0.2, 0.2)));
    }

    fn handle_event(&mut self, world: &mut World, event: Event) -> Trans {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                }
                | WindowEvent::Closed => Trans::Quit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Space),
                            state: ElementState::Released,
                            ..
                        },
                    ..
                } => {
                    let mut scene = world.write_resource::<Scene>();
                    let sets = world.read::<AnimationSet>();
                    let animations = sets.get(scene.entity).unwrap();
                    if animations.animations.len() > 0 {
                        if scene.animation_index >= animations.animations.len() {
                            scene.animation_index = 0;
                        }
                        let animation = &animations.animations[scene.animation_index];
                        scene.animation_index += 1;
                        toggle_animation(
                            &mut world.write(),
                            animation,
                            scene.entity,
                            EndControl::Normal,
                        );
                    }
                    Trans::None
                }
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }
}

fn run() -> Result<(), amethyst::Error> {
    let path = format!(
        "{}/examples/gltf/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let resources_directory = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));
    let config = DisplayConfig::load(&path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawShadedSeparate::new().with_vertex_skinning()),
    );

    let mut game = Application::build(resources_directory, Example)?
        .with(GltfSceneLoaderSystem::new(), "loader_system", &[])
        .with_bundle(RenderBundle::new())?
        .with_bundle(AnimationBundle::new().with_dep(&["loader_system"]))?
        .with_bundle(
            TransformBundle::new()
                .with_dep(&["animation_control_system", "sampler_interpolation_system"]),
        )?
        .with_bundle(VertexSkinningBundle::new().with_dep(&[
            "transform_system",
            "animation_control_system",
            "sampler_interpolation_system",
        ]))?
        .with_local(RenderSystem::build(pipe, Some(config))?)
        .with_resource(AssetStorage::<GltfSceneAsset>::new())
        .register::<Handle<GltfSceneAsset>>()
        .build()?;

    Ok(game.run())
}

fn main() {
    if let Err(e) = run() {
        error!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn load_gltf_mesh(
    world: &World,
    loader: &Loader,
    name: &str,
    options: GltfSceneOptions,
) -> Handle<GltfSceneAsset> {
    loader.load(name, GltfSceneFormat, options, (), &world.read_resource())
}
