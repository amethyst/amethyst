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

struct EmptyState;

impl SimpleState for EmptyState {}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let assets_dir = app_root.join("assets/");
    let mut game_data = DispatcherBuilder::default();
    let game: CoreApplication<GameData>;

    if !cfg!(feature = "empty") {
        let display_config_path = app_root.join("config/display.ron");
        game_data.add_bundle(WindowBundle::from_config_path(display_config_path)?);
        game = Application::build(assets_dir, ExampleState)?.build(game_data)?;
    } else {
        game = Application::build(assets_dir, EmptyState)?.build(game_data)?;
    }

    game.run();

    Ok(())
}
