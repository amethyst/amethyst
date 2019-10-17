//! Displays debug lines using an orthographic camera.

use amethyst::{
    core::{
        transform::{Transform, TransformBundle},
        SystemDesc, Time,
    },
    derive::SystemDesc,
    ecs::{Read, ReadExpect, System, SystemData, World, WorldExt, Write},
    prelude::*,
    renderer::{
        camera::Camera,
        debug_drawing::{DebugLines, DebugLinesComponent, DebugLinesParams},
        palette::Srgba,
        plugins::{RenderDebugLines, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
    window::ScreenDimensions,
};

#[derive(SystemDesc)]
struct ExampleLinesSystem;

impl ExampleLinesSystem {
    pub fn new() -> Self {
        Self
    }
}

impl<'s> System<'s> for ExampleLinesSystem {
    type SystemData = (
        ReadExpect<'s, ScreenDimensions>,
        Write<'s, DebugLines>,
        Read<'s, Time>,
    );

    fn run(&mut self, (screen_dimensions, mut debug_lines_resource, time): Self::SystemData) {
        let t = (time.absolute_time_seconds() as f32).cos() / 2.0 + 0.5;

        let screen_w = screen_dimensions.width();
        let screen_h = screen_dimensions.height();
        let y = t * screen_h;

        debug_lines_resource.draw_line(
            [0.0, y, 1.0].into(),
            [screen_w, y + 2.0, 1.0].into(),
            Srgba::new(0.3, 0.3, 1.0, 1.0),
        );
    }
}

struct ExampleState;

impl SimpleState for ExampleState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // Setup debug lines as a resource
        data.world.insert(DebugLines::new());
        // Configure width of lines. Optional step
        data.world.insert(DebugLinesParams { line_width: 2.0 });

        // Setup debug lines as a component and add lines to render axis&grid
        let mut debug_lines_component = DebugLinesComponent::new();

        let (screen_w, screen_h) = {
            let screen_dimensions = data.world.read_resource::<ScreenDimensions>();
            (screen_dimensions.width(), screen_dimensions.height())
        };

        for y in (0..(screen_h as u16)).step_by(50).map(f32::from) {
            debug_lines_component.add_line(
                [0.0, y, 1.0].into(),
                [screen_w, (y + 2.0), 1.0].into(),
                Srgba::new(0.3, 0.3, 0.3, 1.0),
            );
        }

        for x in (0..(screen_w as u16)).step_by(50).map(f32::from) {
            debug_lines_component.add_line(
                [x, 0.0, 1.0].into(),
                [x, screen_h, 1.0].into(),
                Srgba::new(0.3, 0.3, 0.3, 1.0),
            );
        }

        debug_lines_component.add_line(
            [20.0, 20.0, 1.0].into(),
            [780.0, 580.0, 1.0].into(),
            Srgba::new(1.0, 0.0, 0.2, 1.0), // Red
        );

        data.world
            .create_entity()
            .with(debug_lines_component)
            .build();

        // Setup camera
        let mut local_transform = Transform::default();
        local_transform.set_translation_xyz(screen_w / 2., screen_h / 2., 10.0);
        data.world
            .create_entity()
            .with(Camera::standard_2d(screen_w, screen_h))
            .with(local_transform)
            .build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let display_config_path = app_root.join("examples/debug_lines_ortho/config/display.ron");
    let assets_dir = app_root.join("examples/assets/");

    let game_data = GameDataBuilder::default()
        .with(ExampleLinesSystem::new(), "example_lines_system", &[])
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                .with_plugin(RenderDebugLines::default()),
        )?;

    let mut game = Application::new(assets_dir, ExampleState, game_data)?;
    game.run();
    Ok(())
}
