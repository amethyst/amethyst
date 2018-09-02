//! Displays a shaded sphere to the user.

extern crate amethyst;

use amethyst::assets::{Loader, ProgressCounter};
use amethyst::controls::FlyControlBundle;
use amethyst::controls::FlyControlTag;
use amethyst::core::cgmath::{Deg, Matrix4};
use amethyst::core::transform::TransformBundle;
use amethyst::core::transform::{GlobalTransform, Transform};
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::renderer::*;

// use amethyst::utils::application_root_dir;

struct Example;

impl<'a, 'b> SimpleState<'a, 'b> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let mut progress = ProgressCounter::default();

        let mesh: MeshHandle = {
            let mesh_storage = data.world.read_resource();
            let loader = data.world.read_resource::<Loader>();
            let mut vertices = vec![
                PosColorNorm {
                    position: [0.5, -0.5, 0.5],
                    color: [0.5, 0.05, 0.65, 1.0],
                    normal: [-0.2, 0.6, 0.0],
                },
                PosColorNorm {
                    position: [0.0, 0.5, 0.5],
                    color: [0.5, 0.05, 0.65, 1.0],
                    normal: [0.0, 0.3, 0.0],
                },
                PosColorNorm {
                    position: [-0.5, -0.5, 0.5],
                    color: [0.5, 0.05, 0.65, 1.0],
                    normal: [0.1, 1.0, 0.0],
                },
                PosColorNorm {
                    position: [0.0, 0.001, 0.0],
                    color: [1.0, 0.0, 0.23, 1.0],
                    normal: [1.0, 0.0, 0.0],
                },
                PosColorNorm {
                    position: [0.0, 0.0, 0.0],
                    color: [0.666, 0.85, 0.0, 1.0],
                    normal: [0.0, 1.0, 0.0],
                },
                PosColorNorm {
                    position: [0.0, 0.001, 0.0],
                    color: [0.2, 0.75, 0.93, 1.0],
                    normal: [0.0, 0.0, 1.0],
                },
            ];

            let main_color = [0.4, 0.4, 0.4, 1.0];

            let width: u32 = 10;
            let depth: u32 = 10;

            for x in 0..=width {
                let (x, width, depth) = (x as f32, width as f32, depth as f32);

                let position = [x - width / 2.0, 0.0, -depth / 2.0];
                let normal = [0.0, 0.0, depth];

                let vertex = PosColorNorm {
                    position: position,
                    color: main_color,
                    normal: normal,
                };
                vertices.push(vertex);
            }
            for z in 0..=depth {
                let (z, width, depth) = (z as f32, width as f32, depth as f32);

                let position = [-width / 2.0, 0.0, z - depth / 2.0];
                let normal = [width, 0.0, 0.0];

                let vertex = PosColorNorm {
                    position: position,
                    color: main_color,
                    normal: normal,
                };
                vertices.push(vertex);
            }
            loader.load_from_data(MeshData::from(vertices), &mut progress, &mesh_storage)
        };

        data.world
            .create_entity()
            .with(mesh)
            .with(FlyControlTag)
            .build();

        let mut local_transform = Transform::default();
        local_transform.set_position([0.0, 0.5, 2.0].into());

        data.world
            .create_entity()
            .with(FlyControlTag)
            .with(Camera::from(Projection::perspective(1.33333, Deg(90.0))))
            .with(GlobalTransform::default())
            .with(local_transform)
            .build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = env!("CARGO_MANIFEST_DIR");

    let display_config_path = format!("{}/examples/debug_lines/resources/display.ron", app_root);
    let key_bindings_path = format!("{}/examples/fly_camera/resources/input.ron", app_root);

    let resources = format!("{}/examples/assets/", app_root);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.001, 0.005, 0.005, 1.0], 1.0)
            .with_pass(DrawDebugLines::<PosColorNorm>::new()),
    );

    let config = DisplayConfig::load(display_config_path);

    let fly_control_bundle = FlyControlBundle::<String, String>::new(
        Some(String::from("move_x")),
        Some(String::from("move_y")),
        Some(String::from("move_z")),
    ).with_sensitivity(0.1, 0.1);

    let game_data = GameDataBuilder::default()
        .with_bundle(fly_control_bundle)?
        .with_bundle(TransformBundle::new().with_dep(&["fly_movement"]))?
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path)?,
        )?
        .with_bundle(RenderBundle::new(pipe, Some(config)))?;

    let mut game = Application::new(resources, Example, game_data)?;
    game.run();
    Ok(())
}
