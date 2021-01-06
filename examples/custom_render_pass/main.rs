//! Custom Render Pass example

mod custom_pass;

use amethyst::{
    assets::LoaderBundle,
    input::{
        is_close_requested, is_key_down, InputBundle, InputEvent, ScrollDirection, VirtualKeyCode,
    },
    prelude::*,
    renderer::{
        plugins::RenderToWindow, rendy::hal::command::ClearColor, types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
};

use crate::custom_pass::{CustomUniformArgs, RenderCustom, Triangle};

pub struct CustomShaderState;

impl SimpleState for CustomShaderState {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let world = data.world;

        // Add some triangles
        world.push((Triangle {
            points: [[0., 0.], [0., 1.], [1., 0.0]],
            colors: [[1., 0., 0., 1.], [0., 1., 0., 1.], [0., 0., 1., 1.]],
        },));
        world.push((Triangle {
            points: [[-1., -1.], [0., -1.], [-1., 1.0]],
            colors: [[1., 1., 0., 1.], [0., 1., 1., 1.], [1., 0., 1., 1.]],
        },));
        world.push((Triangle {
            points: [[0.2, -0.7], [0.4, -0.1], [0.8, -1.5]],
            colors: [[1., 0., 0., 1.], [0., 0., 0., 1.], [1., 1., 1., 1.]],
        },));
        world.push((Triangle {
            points: [[-0.2, 0.7], [-0.4, 0.1], [-0.8, 0.5]],
            colors: [
                [0.337, 0.176, 0.835, 1.],
                [0.337, 0.176, 0.835, 1.],
                [0.337, 0.176, 0.835, 1.],
            ],
        },));
    }

    fn handle_event(&mut self, data: StateData<'_, GameData>, event: StateEvent) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    Trans::Quit
                } else {
                    Trans::None
                }
            }
            // Using the Mouse Wheel to control the scale
            StateEvent::Input(input) => {
                if let InputEvent::MouseWheelMoved(dir) = input {
                    let mut scale = data.resources.get_mut::<CustomUniformArgs>().unwrap();
                    match dir {
                        ScrollDirection::ScrollUp => (*scale).scale *= 1.1,
                        ScrollDirection::ScrollDown => (*scale).scale /= 1.1,
                        _ => {}
                    }
                }
                Trans::None
            }
            _ => Trans::None,
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("config/display.ron");
    let assets_dir = app_root.join("assets/");

    let mut game_data = DispatcherBuilder::default();
    game_data
        .add_bundle(LoaderBundle)
        .add_bundle(InputBundle::new())
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                        float32: [1.0, 1.0, 1.0, 1.0],
                    }),
                )
                // Add our custom render plugin to the rendering bundle.
                .with_plugin(RenderCustom::default()),
        );

    let game = Application::build(assets_dir, CustomShaderState)?.build(game_data)?;

    game.run();
    Ok(())
}
