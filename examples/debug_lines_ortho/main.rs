//! Displays several lines with both methods.

use amethyst::{
    core::{
        math::Orthographic3,
        transform::{Transform, TransformBundle},
        Time,
    },
    ecs::{Read, ReadExpect, System, Write},
    prelude::*,
    renderer::*,
    utils::application_root_dir,
};

struct ExampleLinesSystem;
impl<'s> System<'s> for ExampleLinesSystem {
    type SystemData = (
        ReadExpect<'s, ScreenDimensions>,
        Write<'s, DebugLines>,
        Read<'s, Time>,
    );

    fn run(&mut self, (screen_dimensions, mut debug_lines_resource, time): Self::SystemData) {
        let t = (time.absolute_time_seconds() as f32).cos();

        let screen_w = screen_dimensions.width();
        let screen_h = screen_dimensions.height();
        let y = t * screen_h;

        debug_lines_resource.draw_line(
            [0.0, y, 1.0].into(),
            [screen_w, y + 2.0, 1.0].into(),
            [0.3, 0.3, 1.0, 1.0].into(),
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
        data.world
            .add_resource(DebugLinesParams { line_width: 10.0 });

        // Setup debug lines as a component and add lines to render axis&grid
        let mut debug_lines_component = DebugLinesComponent::new().with_capacity(100);

        let (screen_w, screen_h) = {
            let screen_dimensions = data.world.read_resource::<ScreenDimensions>();
            (screen_dimensions.width(), screen_dimensions.height())
        };

        (0..(screen_h as u16))
            .step_by(50)
            .map(f32::from)
            .for_each(|y| {
                debug_lines_component.add_line(
                    [0.0, y, 1.0].into(),
                    [screen_w, (y + 2.0), 1.0].into(),
                    [0.3, 0.3, 0.3, 1.0].into(),
                );
            });

        (0..(screen_w as u16))
            .step_by(50)
            .map(f32::from)
            .for_each(|x| {
                debug_lines_component.add_line(
                    [x, 0.0, 1.0].into(),
                    [x, screen_h, 1.0].into(),
                    [0.3, 0.3, 0.3, 1.0].into(),
                );
            });

        debug_lines_component.add_line(
            [20.0, 20.0, 1.0].into(),
            [780.0, 580.0, 1.0].into(),
            [1.0, 0.0, 0.2, 1.0].into(), // Red
        );

        data.world.register::<DebugLinesComponent>();
        data.world
            .create_entity()
            .with(debug_lines_component)
            .build();

        // Setup camera
        let mut local_transform = Transform::default();
        local_transform.set_translation_xyz(0.0, 0.0, 10.0);
        let left = 0.0;
        let right = screen_w;
        let bottom = 0.0;
        let top = screen_h;
        let znear = 0.0;
        let zfar = 100.0;
        data.world
            .create_entity()
            .with(Camera::from(Projection::Orthographic(Orthographic3::new(
                left, right, bottom, top, znear, zfar,
            ))))
            .with(local_transform)
            .build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let display_config_path = app_root.join("examples/debug_lines_ortho/resources/display.ron");
    // let key_bindings_path = app_root.join("examples/debug_lines/resources/input.ron");
    let resources = app_root.join("examples/assets/");

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.001, 0.005, 0.005, 1.0], 1.0)
            .with_pass(DrawDebugLines::<PosColorNorm>::new()),
    );

    let config = DisplayConfig::load(display_config_path);

    let game_data = GameDataBuilder::default()
        .with(ExampleLinesSystem, "example_lines_system", &[])
        .with_bundle(TransformBundle::new())?
        .with_bundle(RenderBundle::new(pipe, Some(config)))?;

    let mut game = Application::new(resources, ExampleState, game_data)?;
    game.run();
    Ok(())
}
