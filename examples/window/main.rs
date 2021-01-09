//! Opens an empty window.

use amethyst::{
    input::is_key_down, prelude::*, utils::application_root_dir, window::WindowBundle,
    winit::event::VirtualKeyCode,
};

struct ExampleState;

impl SimpleState for ExampleState {
    fn handle_event(&mut self, _: StateData<'_, GameData>, event: StateEvent) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("config/display.ron");

    let assets_dir = app_root.join("assets/");
    let mut builder = DispatcherBuilder::default();

    builder.add_bundle(WindowBundle::from_config_path(display_config_path)?);

    let game = Application::build(assets_dir, ExampleState)?.build(builder)?;
    game.run();

    Ok(())
}
