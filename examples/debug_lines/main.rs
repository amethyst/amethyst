//! Displays several lines with both methods.

extern crate amethyst;

use amethyst::controls::FlyControlBundle;
use amethyst::controls::FlyControlTag;
use amethyst::core::cgmath::Deg;
use amethyst::core::transform::TransformBundle;
use amethyst::core::transform::{GlobalTransform, Transform};
use amethyst::core::Time;
use amethyst::ecs::{Component, Join, NullStorage, Read, ReadStorage, System, Write, WriteStorage};
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::renderer::*;

use amethyst::utils::application_root_dir;

#[derive(Default)]
struct ExampleLinesTag;
impl Component for ExampleLinesTag {
    type Storage = NullStorage<Self>;
}

struct ExampleLinesSystem;
impl<'s> System<'s> for ExampleLinesSystem {
    type SystemData = (
        WriteStorage<'s, DebugLines>, // Request DebugLines components
        Write<'s, DebugLines>,        // Request DebugLines resource
        Read<'s, Time>,
        ReadStorage<'s, ExampleLinesTag>,
    );

    fn run(&mut self, (mut debug_lines, mut debug_lines_resource, time, tags): Self::SystemData) {
        // Drawing debug lines, as a resource
        {
            let t = (time.absolute_time_seconds() as f32).cos();

            debug_lines_resource.add_as_direction(
                [t, 0.0, 0.5],
                [0.0, 0.3, 0.0],
                [0.5, 0.05, 0.65, 1.0].into(),
            );

            debug_lines_resource.add_as_line(
                [t, 0.0, 0.5],
                [0.0, 0.0, 0.2],
                [0.5, 0.05, 0.65, 1.0].into(),
            );
        }

        // Drawing debug lines, as a component
        for (mut debug_lines, _) in (&mut debug_lines, &tags).join() {
            // Axis lines
            debug_lines.add_as_direction(
                [0.0, 0.0001, 0.0],
                [0.2, 0.0, 0.0],
                [1.0, 0.0, 0.23, 1.0].into(),
            );
            debug_lines.add_as_direction(
                [0.0, 0.0, 0.0],
                [0.0, 0.2, 0.0],
                [0.5, 0.85, 0.1, 1.0].into(),
            );
            debug_lines.add_as_direction(
                [0.0, 0.0001, 0.0],
                [0.0, 0.0, 0.2],
                [0.2, 0.75, 0.93, 1.0].into(),
            );

            let width: u32 = 10;
            let depth: u32 = 10;
            let main_color = [0.4, 0.4, 0.4, 1.0].into();

            // Grid lines in X-axis
            for x in 0..=width {
                let (x, width, depth) = (x as f32, width as f32, depth as f32);

                let position = [x - width / 2.0, 0.0, -depth / 2.0];
                let normal = [0.0, 0.0, depth];

                debug_lines.add_as_direction(position, normal, main_color);

                // Sub-grid lines
                if x != width {
                    for sub_x in 1..10 {
                        let position = [
                            position[0] + (1.0 / 10.0) * sub_x as f32,
                            -0.001,
                            position[2],
                        ];

                        debug_lines.add_as_direction(position, normal, [0.1, 0.1, 0.1, 0.1].into());
                    }
                }
            }

            // Grid lines in Z-axis
            for z in 0..=depth {
                let (z, width, depth) = (z as f32, width as f32, depth as f32);

                let position = [-width / 2.0, 0.0, z - depth / 2.0];
                let normal = [width, 0.0, 0.0];

                debug_lines.add_as_direction(position, normal, main_color);

                // Sub-grid lines
                if z != depth {
                    for sub_z in 1..10 {
                        let position = [
                            position[0],
                            -0.001,
                            position[2] + (1.0 / 10.0) * sub_z as f32,
                        ];

                        debug_lines.add_as_direction(position, normal, [0.1, 0.1, 0.1, 0.0].into());
                    }
                }
            }
        }
    }
}

struct Example;
impl<'a, 'b> SimpleState<'a, 'b> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        data.world.register::<DebugLines>();

        // Setup debug lines as a component
        data.world
            .create_entity()
            .with(DebugLines::new().with_capacity(100))
            .with(ExampleLinesTag)
            .build();

        // Setup debug lines as a resource
        data.world
            .add_resource(DebugLines::new().with_capacity(100));

        // Setup camera
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

    let app_root = application_root_dir();

    let display_config_path = format!("{}/examples/debug_lines/resources/display.ron", app_root);
    let key_bindings_path = format!("{}/examples/debug_lines/resources/input.ron", app_root);
    let resources = format!("{}/examples/assets/", app_root);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.001, 0.005, 0.005, 1.0], 1.0)
            .with_pass(DrawDebugLines::<PosColorNorm>::new().with_transparency(
                ColorMask::all(),
                ALPHA,
                Some(DepthMode::LessEqualWrite),
            )),
    );

    let config = DisplayConfig::load(display_config_path);

    let fly_control_bundle = FlyControlBundle::<String, String>::new(
        Some(String::from("move_x")),
        Some(String::from("move_y")),
        Some(String::from("move_z")),
    ).with_sensitivity(0.1, 0.1);

    let game_data = GameDataBuilder::default()
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path)?,
        )?
        .with(ExampleLinesSystem, "example_lines_system", &[])
        .with_bundle(fly_control_bundle)?
        .with_bundle(TransformBundle::new().with_dep(&["fly_movement"]))?
        .with_bundle(RenderBundle::new(pipe, Some(config)))?;

    let mut game = Application::new(resources, Example, game_data)?;
    game.run();
    Ok(())
}
