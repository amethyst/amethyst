//! Displays a 2D GLTF scene

extern crate amethyst;
extern crate amethyst_animation;
extern crate amethyst_gltf;
#[macro_use]
extern crate log;

use amethyst::assets::{Handle, Loader};
use amethyst::core::cgmath::{Deg, Quaternion, Rotation3, Vector3};
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::ecs::prelude::Entity;
use amethyst::input::{is_close_requested, is_key};
use amethyst::prelude::*;
use amethyst::renderer::*;
use amethyst_animation::{get_animation_set, AnimationBundle, AnimationCommand, AnimationSet,
                         EndControl, VertexSkinningBundle};
use amethyst_gltf::{GltfSceneAsset, GltfSceneFormat, GltfSceneLoaderSystem, GltfSceneOptions};

struct Example;

struct Scene {
    entity: Entity,
    animation_index: usize,
}

impl<'a, 'b> State<GameData<'a, 'b>> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
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
            .with(GlobalTransform::default())
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

        let mut camera_transform = Transform::default();
        camera_transform.translation = Vector3::new(100.0, 20.0, 0.0);
        camera_transform.rotation = Quaternion::from_angle_y(Deg(90.));
        world
            .create_entity()
            .with(Camera::from(Projection::perspective(
                1024. / 768.,
                Deg(60.),
            )))
            .with(GlobalTransform::default())
            .with(camera_transform)
            .build();

        world.add_resource(AmbientColor(Rgba(0.2, 0.2, 0.2, 0.2)));
    }

    fn handle_event(&mut self, data: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        let StateData { world, .. } = data;
        if is_close_requested(&event) || is_key(&event, VirtualKeyCode::Escape) {
            Trans::Quit
        } else if is_key(&event, VirtualKeyCode::Space) {
            let mut scene = world.write_resource::<Scene>();
            let sets = world.read_storage::<AnimationSet<usize, Transform>>();
            let animations = sets.get(scene.entity).unwrap();
            if animations.animations.len() > 0 {
                let animation = animations.animations.get(&scene.animation_index).unwrap();
                let mut controls = world.write_storage();
                let set = get_animation_set::<usize, Transform>(&mut controls, scene.entity);
                if set.has_animation(scene.animation_index) {
                    set.toggle(scene.animation_index);
                } else {
                    set.add_animation(
                        scene.animation_index,
                        animation,
                        EndControl::Normal,
                        1.0,
                        AnimationCommand::Start,
                    );
                }
                scene.animation_index += 1;
                if scene.animation_index >= animations.animations.len() {
                    scene.animation_index = 0;
                }
            }
            Trans::None
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(&data.world);
        Trans::None
    }
}

fn main() -> amethyst::Result<()> {
    let path = format!(
        "{}/examples/gltf/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let resources_directory = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let game_data = GameDataBuilder::default()
        .with(GltfSceneLoaderSystem::new(), "loader_system", &[])
        .with_basic_renderer(
            path,
            DrawShadedSeparate::new().with_vertex_skinning(),
            false,
        )?
        .with_bundle(
            AnimationBundle::<usize, Transform>::new(
                "animation_control_system",
                "sampler_interpolation_system",
            ).with_dep(&["loader_system"]),
        )?
        .with_bundle(
            TransformBundle::new()
                .with_dep(&["animation_control_system", "sampler_interpolation_system"]),
        )?
        .with_bundle(VertexSkinningBundle::new().with_dep(&[
            "transform_system",
            "animation_control_system",
            "sampler_interpolation_system",
        ]))?;

    let mut game = Application::new(resources_directory, Example, game_data)?;
    game.run();
    Ok(())
}

fn load_gltf_mesh(
    world: &World,
    loader: &Loader,
    name: &str,
    options: GltfSceneOptions,
) -> Handle<GltfSceneAsset> {
    loader.load(name, GltfSceneFormat, options, (), &world.read_resource())
}
