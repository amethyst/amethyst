//! Displays several lines with both methods.

use amethyst::{
    controls::{FlyControlBundle, FlyControlTag},
    core::{
        nalgebra::{Point3, Vector3},
        transform::{Transform, TransformBundle},
        Time,
    },
    ecs::{Read, System, Write},
    input::InputBundle,
    prelude::*,
    renderer::*,
    utils::application_root_dir,
};

struct ExampleLinesSystem;
impl<'s> System<'s> for ExampleLinesSystem {
    type SystemData = (
        Write<'s, DebugLines>, // Request DebugLines resource
        Read<'s, Time>,
    );

    fn run(&mut self, (mut debug_lines_resource, time): Self::SystemData) {
        // Drawing debug lines, as a resource
        let t = (time.absolute_time_seconds() as f32).cos();

        debug_lines_resource.draw_direction(
            [t, 0.0, 0.5].into(),
            [0.0, 0.3, 0.0].into(),
            [0.5, 0.05, 0.65, 1.0].into(),
        );

        debug_lines_resource.draw_line(
            [t, 0.0, 0.5].into(),
            [0.0, 0.0, 0.2].into(),
            [0.5, 0.05, 0.65, 1.0].into(),
        );
    }
}

struct ExampleState;
impl SimpleState for ExampleState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // Setup debug lines as a resource
        data.world
            .add_resource(DebugLines::new().with_capacity(100));
        // Configure width of lines. Optional step
        data.world.add_resource(DebugLinesParams {
            line_width: 1.0 / 400.0,
        });

        // Setup debug lines as a component and add lines to render axis&grid
        let mut debug_lines_component = DebugLinesComponent::new().with_capacity(100);
        debug_lines_component.add_direction(
            [0.0, 0.0001, 0.0].into(),
            [0.2, 0.0, 0.0].into(),
            [1.0, 0.0, 0.23, 1.0].into(),
        );
        debug_lines_component.add_direction(
            [0.0, 0.0, 0.0].into(),
            [0.0, 0.2, 0.0].into(),
            [0.5, 0.85, 0.1, 1.0].into(),
        );
        debug_lines_component.add_direction(
            [0.0, 0.0001, 0.0].into(),
            [0.0, 0.0, 0.2].into(),
            [0.2, 0.75, 0.93, 1.0].into(),
        );

        let width: u32 = 10;
        let depth: u32 = 10;
        let main_color = [0.4, 0.4, 0.4, 1.0].into();

        // Grid lines in X-axis
        for x in 0..=width {
            let (x, width, depth) = (x as f32, width as f32, depth as f32);

            let position = Point3::new(x - width / 2.0, 0.0, -depth / 2.0);
            let direction = Vector3::new(0.0, 0.0, depth);

            debug_lines_component.add_direction(position, direction, main_color);

            // Sub-grid lines
            if x != width {
                for sub_x in 1..10 {
                    let sub_offset = Vector3::new((1.0 / 10.0) * sub_x as f32, -0.001, 0.0);

                    debug_lines_component.add_direction(
                        position + sub_offset,
                        direction,
                        [0.1, 0.1, 0.1, 0.1].into(),
                    );
                }
            }
        }

        // Grid lines in Z-axis
        for z in 0..=depth {
            let (z, width, depth) = (z as f32, width as f32, depth as f32);

            let position = Point3::new(-width / 2.0, 0.0, z - depth / 2.0);
            let direction = Vector3::new(width, 0.0, 0.0);

            debug_lines_component.add_direction(position, direction, main_color);

            // Sub-grid lines
            if z != depth {
                for sub_z in 1..10 {
                    let sub_offset = Vector3::new(0.0, -0.001, (1.0 / 10.0) * sub_z as f32);

                    debug_lines_component.add_direction(
                        position + sub_offset,
                        direction,
                        [0.1, 0.1, 0.1, 0.0].into(),
                    );
                }
            }
        }
        data.world.register::<DebugLinesComponent>();
        data.world
            .create_entity()
            .with(debug_lines_component)
            .build();

        // Setup camera
        let mut local_transform = Transform::default();
        local_transform.set_position([0.0, 0.5, 2.0].into());
        data.world
            .create_entity()
            .with(FlyControlTag)
            .with(Camera::from(Projection::perspective(
                1.33333,
                std::f32::consts::FRAC_PI_2,
            )))
            .with(local_transform)
            .build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let display_config_path = app_root.join("examples/debug_lines/resources/display.ron");
    let key_bindings_path = app_root.join("examples/debug_lines/resources/input.ron");
    let resources = app_root.join("examples/assets/");

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
    )
    .with_sensitivity(0.1, 0.1);

    let game_data = GameDataBuilder::default()
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path)?,
        )?
        .with(ExampleLinesSystem, "example_lines_system", &[])
        .with_bundle(fly_control_bundle)?
        .with_bundle(TransformBundle::new().with_dep(&["fly_movement"]))?
        .with_bundle(RenderBundle::new(pipe, Some(config)))?;

    let mut game = Application::new(resources, ExampleState, game_data)?;
    game.run();
    Ok(())
}
