//! Custom Render Pass example

mod custom_pass;

use crate::custom_pass::{CustomUniformArgs, RenderCustom, Triangle};
use amethyst::{
    input::{
        is_close_requested, is_key_down, InputBundle, InputEvent, ScrollDirection, StringBindings,
    },
    prelude::*,
    renderer::{plugins::RenderToWindow, types::DefaultBackend, RenderingBundle},
    utils::application_root_dir,
    window::{DisplayConfig, EventLoop},
    winit::event::VirtualKeyCode,
};
use amethyst_rendy::rendy;

const CLEAR_COLOR: rendy::hal::command::ClearColor = rendy::hal::command::ClearColor {
    float32: [0.34, 0.36, 0.52, 1.0],
};

pub struct CustomShaderState;

impl SimpleState for CustomShaderState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // Add some triangles
        world
            .create_entity()
            .with(Triangle {
                points: [[0., 0.], [0., 1.], [1., 0.0]],
                colors: [[1., 0., 0., 1.], [0., 1., 0., 1.], [0., 0., 1., 1.]],
            })
            .build();
        world
            .create_entity()
            .with(Triangle {
                points: [[-1., -1.], [0., -1.], [-1., 1.0]],
                colors: [[1., 1., 0., 1.], [0., 1., 1., 1.], [1., 0., 1., 1.]],
            })
            .build();
        world
            .create_entity()
            .with(Triangle {
                points: [[0.2, -0.7], [0.4, -0.1], [0.8, -1.5]],
                colors: [[1., 0., 0., 1.], [0., 0., 0., 1.], [1., 1., 1., 1.]],
            })
            .build();

        world
            .create_entity()
            .with(Triangle {
                points: [[-0.2, 0.7], [-0.4, 0.1], [-0.8, 0.5]],
                colors: [
                    [0.337, 0.176, 0.835, 1.],
                    [0.337, 0.176, 0.835, 1.],
                    [0.337, 0.176, 0.835, 1.],
                ],
            })
            .build();
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
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
                    let mut scale = data.world.write_resource::<CustomUniformArgs>();
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
    let display_config_path = app_root.join("examples/custom_render_pass/config/display.ron");
    let assets_dir = app_root.join("examples/assets/");

    let event_loop = EventLoop::new();
    let display_config = DisplayConfig::load(display_config_path)?;
    let game_data = GameDataBuilder::default()
        .with_bundle(InputBundle::<StringBindings>::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new(display_config, &event_loop)
                .with_plugin(RenderToWindow::new().with_clear(CLEAR_COLOR))
                // Add our custom render plugin to the rendering bundle.
                .with_plugin(RenderCustom::default()),
        )?;

    let game = Application::new(assets_dir, CustomShaderState, game_data)?;
    game.run_winit_loop(event_loop);
}
