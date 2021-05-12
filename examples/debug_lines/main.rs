//! Displays dynamic and static debug lines via the resource and component
//! methods respectively. A pendulum of sorts is drawn using the resource
//! method, while a grid pattern is drawn on the floor using the component
//! method.

use amethyst::{
    assets::LoaderBundle,
    controls::{FlyControl, FlyControlBundle, HideCursor},
    core::{
        math::{Point3, Vector3},
        transform::{Transform, TransformBundle},
        Time,
    },
    input::{is_key_down, is_mouse_button_down, InputBundle, VirtualKeyCode},
    prelude::*,
    renderer::{
        camera::Camera,
        debug_drawing::{DebugLines, DebugLinesComponent, DebugLinesParams},
        palette::Srgba,
        plugins::{RenderDebugLines, RenderSkybox, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
    window::ScreenDimensions,
    winit::event::MouseButton,
};

struct ExampleLinesSystem;

impl System for ExampleLinesSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("ExampleLinesSystem")
                .read_resource::<Time>()
                .write_resource::<DebugLines>()
                .build(|_, _, (time, debug_lines_resource), _| {
                    // Drawing debug lines as a resource
                    let t = time.absolute_time().as_secs_f32().cos();

                    debug_lines_resource.draw_direction(
                        Point3::new(t, 0.0, 0.5),
                        Vector3::new(0.0, 0.3, 0.0),
                        Srgba::new(0.5, 0.05, 0.65, 1.0),
                    );

                    debug_lines_resource.draw_line(
                        Point3::new(t, 0.0, 0.5),
                        Point3::new(0.0, 0.0, 0.2),
                        Srgba::new(0.5, 0.05, 0.65, 1.0),
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

        // Setup debug lines as a component and add lines to render axes & grid
        let mut debug_lines_component = DebugLinesComponent::with_capacity(100);

        // X-axis (red)
        debug_lines_component.add_direction(
            Point3::new(0.0, 0.0001, 0.0),
            Vector3::new(0.2, 0.0, 0.0),
            Srgba::new(1.0, 0.0, 0.23, 1.0),
        );

        // Y-axis (yellowish-green)
        debug_lines_component.add_direction(
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.2, 0.0),
            Srgba::new(0.5, 0.85, 0.1, 1.0),
        );

        // Z-axis (blue)
        debug_lines_component.add_direction(
            Point3::new(0.0, 0.0001, 0.0),
            Vector3::new(0.0, 0.0, 0.2),
            Srgba::new(0.2, 0.75, 0.93, 1.0),
        );

        let width: u32 = 10;
        let depth: u32 = 10;
        let main_color = Srgba::new(0.4, 0.4, 0.4, 1.0);

        // Grid lines in X-axis
        for x in 0..=width {
            let (x, width, depth) = (x as f32, width as f32, depth as f32);

            let position = Point3::new(x - width / 2.0, 0.0, -depth / 2.0);
            let direction = Vector3::new(0.0, 0.0, depth);

            debug_lines_component.add_direction(position, direction, main_color);

            // Sub-grid lines
            if (x - width).abs() < f32::EPSILON {
                for sub_x in 1..10 {
                    let sub_offset = Vector3::new((1.0 / 10.0) * sub_x as f32, -0.001, 0.0);

                    debug_lines_component.add_direction(
                        position + sub_offset,
                        direction,
                        Srgba::new(0.1, 0.1, 0.1, 0.2),
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
            if (z - depth).abs() < f32::EPSILON {
                for sub_z in 1..10 {
                    let sub_offset = Vector3::new(0.0, -0.001, (1.0 / 10.0) * sub_z as f32);

                    debug_lines_component.add_direction(
                        position + sub_offset,
                        direction,
                        Srgba::new(0.1, 0.1, 0.1, 0.2),
                    );
                }
            }
        }

        world.push((debug_lines_component,));

        // Setup camera
        let (width, height) = {
            let dim = resources.get::<ScreenDimensions>().unwrap();
            (dim.width(), dim.height())
        };
        let mut local_transform = Transform::default();
        local_transform.set_translation_xyz(0.0, 0.5, 2.0);
        world.push((
            FlyControl,
            Camera::standard_3d(width, height),
            local_transform,
        ));
    }

    fn handle_event(&mut self, data: StateData<'_, GameData>, event: StateEvent) -> SimpleTrans {
        let StateData { resources, .. } = data;
        if let StateEvent::Window(event) = &event {
            let mut hide_cursor = resources.get_mut::<HideCursor>().unwrap();

            if is_key_down(&event, VirtualKeyCode::Escape) {
                hide_cursor.hide = false;
            } else if is_mouse_button_down(&event, MouseButton::Left) {
                hide_cursor.hide = true;
            }
        }
        Trans::None
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let display_config_path = app_root.join("config/display.ron");
    let key_bindings_path = app_root.join("config/input.ron");
    let assets_dir = app_root.join("assets/");

    let mut game_data = DispatcherBuilder::default();
    game_data
        .add_bundle(LoaderBundle)
        .add_bundle(InputBundle::new().with_bindings_from_file(&key_bindings_path)?)
        .add_system(ExampleLinesSystem)
        .add_bundle(
            FlyControlBundle::new(
                Some("move_x".into()),
                Some("move_y".into()),
                Some("move_z".into()),
            )
            .with_sensitivity(0.1, 0.1)
            .with_speed(5.0),
        )
        .add_bundle(TransformBundle::default())
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderToWindow::from_config_path(display_config_path)?)
                .with_plugin(RenderDebugLines::default())
                .with_plugin(RenderSkybox::default()),
        );

    let game = Application::new(assets_dir, ExampleState, game_data)?;
    game.run();
    Ok(())
}
