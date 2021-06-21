//! Displays debug lines using an orthographic camera.

use amethyst::{
    assets::LoaderBundle,
    core::{
        transform::{Transform, TransformBundle},
        Time,
    },
    prelude::*,
    renderer::{
        camera::Camera,
        debug_drawing::{DebugLines, DebugLinesComponent, DebugLinesParams},
        palette::Srgba,
        plugins::{RenderDebugLines, RenderToWindow},
        rendy::hal::command::ClearColor,
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
    window::ScreenDimensions,
};

struct ExampleLinesSystem;

impl System for ExampleLinesSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("ExampleLinesSystem")
                .read_resource::<ScreenDimensions>()
                .read_resource::<Time>()
                .write_resource::<DebugLines>()
                .build(|_, _, (screen_dimensions, time, debug_lines_resource), _| {
                    let t = time.absolute_time().as_secs_f32().cos() / 2.0 + 0.5;

                    let screen_w = screen_dimensions.width();
                    let screen_h = screen_dimensions.height();
                    let y = t * screen_h;

                    debug_lines_resource.draw_line(
                        [0.0, y, 1.0].into(),
                        [screen_w, y + 2.0, 1.0].into(),
                        Srgba::new(0.3, 0.3, 1.0, 1.0),
                    );
                }),
        )
    }
}

struct ExampleState;

impl SimpleState for ExampleState {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData {
            world, resources, ..
        } = data;

        // Setup debug lines as a resource
        resources.insert(DebugLines::new());
        // Configure width of lines. Optional step
        resources.insert(DebugLinesParams { line_width: 2.0 });

        // Setup debug lines as a component and add lines to render axis&grid
        let mut debug_lines_component = DebugLinesComponent::new();

        let (screen_w, screen_h) = {
            let dim = resources.get::<ScreenDimensions>().unwrap();
            (dim.width(), dim.height())
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

        world.push((debug_lines_component,));

        // Setup camera
        let mut local_transform = Transform::default();
        local_transform.set_translation_xyz(screen_w / 2., screen_h / 2., 10.0);
        world.push((Camera::standard_2d(screen_w, screen_h), local_transform));
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let display_config_path = app_root.join("config/display.ron");
    let assets_dir = app_root.join("assets/");

    let mut game_data = DispatcherBuilder::default();
    game_data
        .add_system(ExampleLinesSystem)
        .add_bundle(TransformBundle::default())
        .add_bundle(LoaderBundle)
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    }),
                )
                .with_plugin(RenderDebugLines::default()),
        );

    let game = Application::build(assets_dir, ExampleState)?.build(game_data)?;
    game.run();
    Ok(())
}
