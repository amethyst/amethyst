//! Custom UI example

mod custom_pass;

use crate::custom_pass::CustomUniformArgs;
use crate::custom_pass::RenderCustom;
use crate::custom_pass::Triangle;
use amethyst::{
    input::{
        is_close_requested, is_key_down, InputBundle, InputEvent, ScrollDirection, StringBindings,
    },
    prelude::*,
    renderer::{plugins::RenderToWindow, types::DefaultBackend, RenderingBundle},
    utils::application_root_dir,
    winit::VirtualKeyCode,
};

pub struct CustomShaderState;

impl SimpleState for CustomShaderState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // add some triangles
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
            //Using the Mouse Wheel to control the scale
            StateEvent::Input(input) => {
                match input {
                    InputEvent::MouseWheelMoved(dir) => {
                        let mut scale = data.world.write_resource::<CustomUniformArgs>();
                        match dir {
                            ScrollDirection::ScrollUp => (*scale).scale *= 1.1,
                            ScrollDirection::ScrollDown => (*scale).scale /= 1.1,
                            _ => {}
                        }
                    }
                    _ => {}
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
    let display_config_path = app_root.join("examples/custom_shader/config/display.ron");

    // This line is not mentioned in the pong tutorial as it is specific to the context
    // of the git repository. It only is a different location to load the assets from.
    let assets_dir = app_root.join("examples/assets/");

    let game_data = GameDataBuilder::default()
        .with_bundle(InputBundle::<StringBindings>::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                // The RenderToWindow plugin provides all the scaffolding for opening a window and
                // drawing on it
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear([1.0, 1.0, 1.0, 1.0]),
                )
                // RenderFlat2D plugin is used to render entities with `SpriteRender` component.
                .with_plugin(RenderCustom::default()),
        )?;

    let mut game = Application::new(assets_dir, CustomShaderState, game_data)?;

    game.run();
    Ok(())
}
